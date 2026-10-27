use std::collections::HashMap;
use std::time::Instant;

use anyhow::Context;

use crate::cooked::{AssetId, CookedCache, CookedTexture};
use crate::gpu::FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE;
use crate::render::LoadStaging;
use crate::render::bind_groups::MATERIALS;
use crate::stats::texture_bytes;

/// Textures each Channel's binding array can hold: the Channels split the
/// Capability floor's binding-array elements per stage (ADR 0021), 13,107
/// each, room for texture-wall's 10,000 and Layer 0.
pub(crate) const TEXTURES_PER_CHANNEL: u32 =
    FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE / MaterialChannel::ALL.len() as u32;

/// Materials the SSBO holds. Materials are few and never deleted yet,
/// so a fixed cap avoids growth + bind-group rebuilds for now.
const MATERIAL_CAPACITY: u64 = 256;

/// The colour Channels' anisotropy (albedo, emissive) and the data
/// Channels' (normal, metallic-roughness, occlusion), the owner's pick from
/// the candidates measured in `docs/performance.md`: 4x on the data
/// Channels saves 126 µs of minification's lit pass and drops the sparkle
/// of highlights whose normals came from finer mips than Toksvig widened
/// their roughness for.
const COLOUR_ANISOTROPY: u16 = 16;
const DATA_ANISOTROPY: u16 = 4;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MaterialHandle(pub u32);

impl MaterialHandle {
    /// Index into the material SSBO. Reads better than `.0` at call sites that
    /// unwrap a wrapping newtype (e.g. `material_ref.0.index()`).
    pub fn index(self) -> u32 {
        self.0
    }
}

/// One texture channel of a material. Each channel has its own binding array;
/// a material stores one Layer index per channel. The discriminant is the
/// Channel's position in `MaterialPool::channels` and part of every
/// Texture's Asset ID: never renumber.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MaterialChannel {
    Albedo = 0,
    Normal = 1,
    MetallicRoughness = 2,
    Occlusion = 3,
    Emissive = 4,
}

impl MaterialChannel {
    pub const ALL: [Self; 5] = [
        Self::Albedo,
        Self::Normal,
        Self::MetallicRoughness,
        Self::Occlusion,
        Self::Emissive,
    ];
}

/// One entry in the material SSBO. All-`vec4` layout: WGSL pads `vec3<f32>` to
/// 16 bytes silently, so every field is 16-byte sized to match the WGSL struct.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialData {
    pub base_color: [f32; 4],
    pub metallic_roughness: [f32; 4], // x=metallic, y=roughness, z=normal scale, w=pad
    pub emissive: [f32; 4],           // xyz=color, w=strength
    pub layers: [u32; 4],             // x=albedo, y=normal, z=MR, w=AO
    pub extra: [u32; 4],              // x=emissive Layer, yzw=pad
}

/// One Channel's Textures, each sized to its source, bound as one partially
/// bound binding array (ADR 0017). A Layer is an index into `views`.
struct ChannelTextures {
    /// Its binding's name in `MATERIALS`, which labels its Textures too.
    label: &'static str,
    textures: Vec<wgpu::Texture>,
    views: Vec<wgpu::TextureView>,
}

impl ChannelTextures {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            textures: Vec::new(),
            views: Vec::new(),
        }
    }

    /// Creates `texture` with its full mip chain, written through `staging`,
    /// and returns its Layer, or `None` when the Channel is full.
    fn push(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        texture: &CookedTexture,
    ) -> Option<u32> {
        if self.views.len() as u32 >= TEXTURES_PER_CHANNEL {
            tracing::error!(
                channel = self.label,
                capacity = TEXTURES_PER_CHANNEL,
                "channel full; using the neutral default layer"
            );
            return None;
        }
        let layer = self.views.len() as u32;
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(self.label),
            size,
            mip_level_count: texture.levels.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, bytes) in texture.levels.iter().enumerate() {
            staging.write_texture(&gpu_texture, level as u32, bytes);
        }
        self.views
            .push(gpu_texture.create_view(&Default::default()));
        self.textures.push(gpu_texture);
        Some(layer)
    }
}

/// Material-as-ID: one SSBO of `MaterialData` indexed by `MaterialHandle`, plus
/// one texture binding array per Channel. A single bind group serves every
/// material; the fragment shader looks up `materials[instance.material_id]` and
/// samples each Channel's array at the Layer stored in `layers`.
pub struct MaterialPool {
    materials: Vec<MaterialData>,
    materials_buffer: wgpu::Buffer,
    channels: [ChannelTextures; 5],
    /// Each loaded Texture's Layer in its Channel, which its Asset ID names.
    texture_layers: HashMap<AssetId, u32>,
    colour_sampler: wgpu::Sampler,
    data_sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// Set when a load adds a Texture the bind group lacks.
    bind_group_is_stale: bool,
}

impl MaterialPool {
    /// Layer 0 of every Channel holds a 1×1 neutral texture: white for
    /// albedo / metallic-roughness / occlusion / emissive, the flat normal,
    /// x and y at 0.5, for normal. A material without a channel points here,
    /// samples a no-op, and its factors pass through unchanged.
    pub const DEFAULT_LAYER: u32 = 0;

    pub fn new(device: &wgpu::Device, staging: &mut LoadStaging) -> Self {
        let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("materials_buffer"),
            size: MATERIAL_CAPACITY * std::mem::size_of::<MaterialData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Ordered as `MaterialChannel`.
        let mut channels = [
            "albedo_textures",
            "normal_textures",
            "metallic_roughness_textures",
            "occlusion_textures",
            "emissive_textures",
        ]
        .map(ChannelTextures::new);

        // Neutral defaults into Layer 0 of every Channel (see DEFAULT_LAYER):
        // colour Channels are sRGB, data Channels linear.
        for channel in MaterialChannel::ALL {
            let (format, pixel) = match channel {
                MaterialChannel::Albedo | MaterialChannel::Emissive => {
                    (wgpu::TextureFormat::Rgba8UnormSrgb, [255; 4])
                }
                MaterialChannel::Normal => (wgpu::TextureFormat::Rgba8Unorm, [128, 128, 255, 255]),
                MaterialChannel::MetallicRoughness | MaterialChannel::Occlusion => {
                    (wgpu::TextureFormat::Rgba8Unorm, [255; 4])
                }
            };
            let neutral = CookedTexture {
                format,
                width: 1,
                height: 1,
                levels: vec![pixel.to_vec()],
            };
            channels[channel as usize].push(device, staging, &neutral);
        }

        let colour_sampler = trilinear_sampler(device, "Colour Channel Sampler", COLOUR_ANISOTROPY);
        let data_sampler = trilinear_sampler(device, "Data Channel Sampler", DATA_ANISOTROPY);

        let bind_group_layout = MATERIALS.layout(device);
        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            &materials_buffer,
            &channels,
            &colour_sampler,
            &data_sampler,
        );

        Self {
            materials: Vec::new(),
            materials_buffer,
            channels,
            texture_layers: HashMap::new(),
            colour_sampler,
            data_sampler,
            bind_group_layout,
            bind_group,
            bind_group_is_stale: false,
        }
    }

    /// Bytes of the SSBO and every Channel's Textures.
    pub fn gpu_memory_bytes(&self, transient_saves_memory: bool) -> u64 {
        let texture_bytes: u64 = self
            .channels
            .iter()
            .flat_map(|channel| &channel.textures)
            .map(|texture| texture_bytes(texture, transient_saves_memory))
            .sum();
        self.materials_buffer.size() + texture_bytes
    }

    /// Registers a material in the SSBO through `staging`, returning its
    /// handle.
    pub fn insert(&mut self, staging: &mut LoadStaging, data: MaterialData) -> MaterialHandle {
        if self.materials.len() as u64 >= MATERIAL_CAPACITY {
            tracing::error!(
                capacity = MATERIAL_CAPACITY,
                "material SSBO full; reusing material 0 (raise MATERIAL_CAPACITY)"
            );
            return MaterialHandle(0);
        }
        let handle = MaterialHandle(self.materials.len() as u32);
        let offset = handle.0 as u64 * std::mem::size_of::<MaterialData>() as u64;
        staging.write(&self.materials_buffer, offset, bytemuck::bytes_of(&data));
        self.materials.push(data);
        tracing::debug!(handle = handle.0, "material registered");
        handle
    }

    /// Loads the Material `id` from `cache`, and each Texture it samples once
    /// per Asset ID, written through `staging`, and returns its Handle. A
    /// Channel the Material lacks samples Layer 0, as does one already full.
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        cache: &CookedCache,
        id: AssetId,
    ) -> anyhow::Result<MaterialHandle> {
        let material = cache.material(id)?;
        let mut layers = [Self::DEFAULT_LAYER; 5];
        for (channel, texture_id) in MaterialChannel::ALL.into_iter().zip(material.textures) {
            if let Some(texture_id) = texture_id {
                layers[channel as usize] = self
                    .load_texture(device, staging, cache, channel, texture_id)
                    .with_context(|| format!("loading Material {id}"))?;
            }
        }
        let [red, green, blue] = material.emissive;
        let data = MaterialData {
            base_color: material.base_color,
            metallic_roughness: [
                material.metallic,
                material.roughness,
                material.normal_scale,
                0.0,
            ],
            emissive: [red, green, blue, 1.0],
            layers: [layers[0], layers[1], layers[2], layers[3]],
            extra: [layers[4], 0, 0, 0],
        };
        tracing::debug!(material = %id, ?layers, "Material loaded");
        Ok(self.insert(staging, data))
    }

    /// Loads the Texture `id` from `cache` into `channel` once per Asset ID,
    /// written through `staging`, and returns its Layer: the next free one,
    /// or Layer 0 when the Channel is full.
    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        cache: &CookedCache,
        channel: MaterialChannel,
        id: AssetId,
    ) -> anyhow::Result<u32> {
        if let Some(&layer) = self.texture_layers.get(&id) {
            return Ok(layer);
        }
        let texture = cache.texture(id)?;
        let Some(layer) = self.channels[channel as usize].push(device, staging, &texture) else {
            return Ok(Self::DEFAULT_LAYER);
        };
        self.texture_layers.insert(id, layer);
        self.bind_group_is_stale = true;
        Ok(layer)
    }

    /// Rebuilds the bind group when loads since the last call added
    /// Textures, so a scene's loads build it once: bind groups are
    /// snapshots.
    pub fn refresh_bind_group(&mut self, device: &wgpu::Device) {
        if std::mem::take(&mut self.bind_group_is_stale) {
            let started = Instant::now();
            self.bind_group = create_bind_group(
                device,
                &self.bind_group_layout,
                &self.materials_buffer,
                &self.channels,
                &self.colour_sampler,
                &self.data_sampler,
            );
            // ADR 0017's gate: the build's cost at the bound Textures' count.
            tracing::info!(
                textures = self
                    .channels
                    .iter()
                    .map(|channel| channel.views.len())
                    .sum::<usize>(),
                microseconds = started.elapsed().as_secs_f64() * 1e6,
                "material bind group built"
            );
        }
    }
}

/// A trilinear, repeating sampler at `anisotropy`, which the
/// Capability floor's anisotropic filtering keeps wgpu from clamping to 1
/// (ADR 0021).
fn trilinear_sampler(device: &wgpu::Device, label: &str, anisotropy: u16) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: anisotropy,
        ..Default::default()
    })
}

/// The material bind group (`MATERIALS`): the SSBO, each Channel's
/// Textures, and the colour and data Channels' samplers. Bind groups are
/// snapshots, so a new Texture needs a new one.
fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    materials_buffer: &wgpu::Buffer,
    channels: &[ChannelTextures; 5],
    colour_sampler: &wgpu::Sampler,
    data_sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let views: Vec<Vec<&wgpu::TextureView>> = channels
        .iter()
        .map(|channel| channel.views.iter().collect())
        .collect();
    let channel_resources = channels.iter().zip(&views).map(|(channel, channel_views)| {
        let resource = wgpu::BindingResource::TextureViewArray(channel_views);
        (channel.label, resource)
    });
    let samplers = [
        ("colour_sampler", colour_sampler),
        ("data_sampler", data_sampler),
    ]
    .map(|(name, sampler)| (name, wgpu::BindingResource::Sampler(sampler)));
    let resources = std::iter::once(("materials", materials_buffer.as_entire_binding()))
        .chain(channel_resources)
        .chain(samplers);
    MATERIALS.bind_group(device, layout, resources)
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::cooked::CookedMaterial;
    use crate::gpu::GpuContext;

    /// Two Materials sampling one Texture load it once, and a Material
    /// without that Channel samples Layer 0; each carries its normal scale.
    #[test]
    fn a_texture_two_materials_share_loads_once() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-material-pool-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        // 4 × 4 BC4: one 8-byte block at each of its three mips.
        let occlusion = CookedTexture {
            format: wgpu::TextureFormat::Bc4RUnorm,
            width: 4,
            height: 4,
            levels: vec![vec![255; 8]; 3],
        };
        cache
            .write(AssetId(10), "key", &occlusion.to_blob(1).unwrap())
            .unwrap();
        let shared = [None, None, None, Some(AssetId(10)), None];
        for (id, textures) in [(1, shared), (2, shared), (3, [None; 5])] {
            let material = CookedMaterial {
                base_color: [1.0; 4],
                metallic: 0.0,
                roughness: 1.0,
                emissive: [0.0; 3],
                normal_scale: 0.5,
                textures,
            };
            cache
                .write(AssetId(id), "key", &material.to_blob())
                .unwrap();
        }

        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MaterialPool::new(&context.device, &mut staging);
        let neutral_bytes = pool.gpu_memory_bytes(false);
        let occlusion_layers: Vec<u32> = (1..=3)
            .map(|id| {
                let handle = pool
                    .load(&context.device, &mut staging, &cache, AssetId(id))
                    .unwrap();
                pool.materials[handle.index() as usize].layers[3]
            })
            .collect();
        assert_eq!(pool.gpu_memory_bytes(false) - neutral_bytes, 3 * 8);
        assert_eq!(occlusion_layers, [1, 1, MaterialPool::DEFAULT_LAYER]);
        assert!(
            pool.materials
                .iter()
                .all(|material| material.metallic_roughness[2] == 0.5)
        );
        std::fs::remove_dir_all(&directory).ok();
    }
}
