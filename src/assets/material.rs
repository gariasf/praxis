use std::collections::HashMap;
use std::time::Instant;

use anyhow::Context;

use crate::assets::pools::Generations;
use crate::cooked::{AssetId, CookedCache, CookedTexture};
use crate::gpu::{FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE, FLOOR_TEXTURE_ARRAY_LAYERS};
use crate::render::LoadStaging;
use crate::render::bind_groups::MATERIALS;
use crate::stats::{Counted, GpuMemory, GpuMemoryKind};

/// Texture classes each Channel's binding array can hold: the Channels split
/// the Capability floor's binding-array elements per stage (ADR 0021),
/// 13,107 each.
pub(crate) const CLASSES_PER_CHANNEL: u32 =
    FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE / MaterialChannel::ALL.len() as u32;

/// The most slices a Texture class holds, the floor's array layers, and the
/// Layers each class spans: a Layer is its class times this plus its slice.
pub(crate) const SLICES_PER_CLASS: u32 = FLOOR_TEXTURE_ARRAY_LAYERS;

/// Materials the SSBO holds: a fixed cap avoids growth and bind-group
/// rebuilds.
const MATERIAL_CAPACITY: u32 = 256;

/// The colour Channels' anisotropy (albedo, emissive) and the data
/// Channels' (normal, metallic-roughness, occlusion), the owner's pick from
/// the candidates measured in `docs/performance.md`: 4x on the data
/// Channels shortens minification's lit pass and drops the sparkle
/// of highlights whose normals came from finer mips than Toksvig widened
/// their roughness for.
const COLOUR_ANISOTROPY: u16 = 16;
const DATA_ANISOTROPY: u16 = 4;

/// A Material's index in the Material SSBO, which the GPU sees, and the
/// generation of that index it was loaded in, which no later Material there
/// shares.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MaterialHandle {
    pub index: u32,
    pub generation: u32,
}

impl MaterialHandle {
    /// Index into the material SSBO.
    pub fn index(self) -> u32 {
        self.index
    }
}

/// One texture channel of a material. Each channel has its own binding array;
/// a material stores one Layer per channel. The discriminant is the
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
    pub emissive: [f32; 4],           // xyz=color, w=pad (unread)
    pub layers: [u32; 4],             // x=albedo, y=normal, z=MR, w=AO
    pub extra: [u32; 4],              // x=emissive Layer, yzw=pad
}

/// What every Texture of one Texture class shares.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct TextureShape {
    pub width: u32,
    pub height: u32,
    pub mip_level_count: u32,
    pub format: wgpu::TextureFormat,
}

impl TextureShape {
    pub(crate) fn of(texture: &CookedTexture) -> Self {
        Self {
            width: texture.width,
            height: texture.height,
            mip_level_count: texture.levels.len() as u32,
            format: texture.format,
        }
    }
}

/// A Texture's place in its Channel: its Texture class, which indexes the
/// Channel's binding array, and its slice of that class's texture array.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassAndSlice {
    pub class: u32,
    pub slice: u32,
}

impl ClassAndSlice {
    /// The Layer that names it, which WGSL's `layer_class_and_slice` decodes.
    pub(crate) fn layer(self) -> u32 {
        self.class * SLICES_PER_CLASS + self.slice
    }
}

/// One Channel's Texture classes, in the order they opened.
#[derive(Default)]
pub(crate) struct ClassTable {
    classes: Vec<ClassSlices>,
}

/// A Texture class's shape, its slice count, fixed when it opens, and the
/// slices taken.
struct ClassSlices {
    shape: TextureShape,
    slices: u32,
    taken: u32,
}

impl ClassTable {
    /// The next slice of the newest class of `shape`, or slice 0 of a new
    /// class when that one is full or there is none: the first of a shape
    /// has one slice, each later one twice the one before's, up to
    /// `SLICES_PER_CLASS`. `None` when the Channel holds
    /// `CLASSES_PER_CHANNEL` classes and none of `shape` has room.
    pub(crate) fn place(&mut self, shape: TextureShape) -> Option<ClassAndSlice> {
        let newest = self.classes.iter().rposition(|class| class.shape == shape);
        if let Some(class) = newest
            && let newest_class = &mut self.classes[class]
            && newest_class.taken < newest_class.slices
        {
            newest_class.taken += 1;
            return Some(ClassAndSlice {
                class: class as u32,
                slice: newest_class.taken - 1,
            });
        }
        if self.classes.len() as u32 >= CLASSES_PER_CHANNEL {
            return None;
        }
        let slices = newest.map_or(1, |class| {
            (self.classes[class].slices * 2).min(SLICES_PER_CLASS)
        });
        self.classes.push(ClassSlices {
            shape,
            slices,
            taken: 1,
        });
        Some(ClassAndSlice {
            class: self.classes.len() as u32 - 1,
            slice: 0,
        })
    }

    /// The slices `class` was opened with.
    fn slices(&self, class: u32) -> u32 {
        self.classes[class as usize].slices
    }
}

/// One Channel's Textures, a texture array per Texture class, bound as one
/// partially bound binding array (ADR 0017).
struct ChannelTextures {
    /// Its binding's name in `MATERIALS`, which labels its arrays too.
    label: &'static str,
    memory: GpuMemory,
    table: ClassTable,
    /// Each Texture class's array and its view, at the class's index.
    textures: Vec<Counted<wgpu::Texture>>,
    views: Vec<wgpu::TextureView>,
}

impl ChannelTextures {
    fn new(label: &'static str, memory: &GpuMemory) -> Self {
        Self {
            label,
            memory: memory.clone(),
            table: ClassTable::default(),
            textures: Vec::new(),
            views: Vec::new(),
        }
    }

    /// Writes `texture` with its full mip chain through `staging` into the
    /// slice its class has free, creating the class's array when the class
    /// is new, and returns its Layer, or `None` when the Channel is full.
    fn push(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        texture: &CookedTexture,
    ) -> Option<u32> {
        let shape = TextureShape::of(texture);
        let Some(placement) = self.table.place(shape) else {
            tracing::error!(
                channel = self.label,
                capacity = CLASSES_PER_CHANNEL,
                "channel has no room for another Texture class; using the neutral default layer"
            );
            return None;
        };
        if placement.slice == 0 {
            let array = self.memory.create_material_texture(
                device,
                &wgpu::TextureDescriptor {
                    label: Some(self.label),
                    size: wgpu::Extent3d {
                        width: shape.width,
                        height: shape.height,
                        depth_or_array_layers: self.table.slices(placement.class),
                    },
                    mip_level_count: shape.mip_level_count,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: shape.format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
            );
            // An array of one slice would otherwise get a 2D view.
            self.views
                .push(array.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                }));
            self.textures.push(array);
        }
        let array = &self.textures[placement.class as usize];
        for (level, bytes) in texture.levels.iter().enumerate() {
            staging.write_texture(array, level as u32, placement.slice, bytes);
        }
        Some(placement.layer())
    }
}

/// Material-as-ID: one SSBO of `MaterialData` indexed by `MaterialHandle`, plus
/// one binding array of Texture classes per Channel. A single bind group
/// serves every material; the fragment shader looks up the Material ID its
/// Meshlet record carries, the Instance's override or else the Meshlet's own,
/// and samples each Channel at the class and slice of the Layer stored in
/// `layers`.
pub struct MaterialPool {
    generations: Generations,
    /// Each Material's data, at its Handle's index; a freed one's stays
    /// until a later Material takes its index.
    materials: Vec<MaterialData>,
    materials_buffer: Counted<wgpu::Buffer>,
    channels: [ChannelTextures; 5],
    /// Each loaded Texture's Layer in its Channel, which its Asset ID names.
    texture_layers: HashMap<AssetId, u32>,
    /// Each loaded Material's Handle, which its Asset ID names.
    material_handles: HashMap<AssetId, MaterialHandle>,
    colour_sampler: wgpu::Sampler,
    data_sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// Set when a load opens a Texture class the bind group lacks.
    bind_group_is_stale: bool,
}

impl MaterialPool {
    /// Layer 0 of every Channel holds a 1×1 neutral texture: white for
    /// albedo / metallic-roughness / occlusion / emissive, the flat normal,
    /// x and y at 0.5, for normal. A material without a channel points here,
    /// samples a no-op, and its factors pass through unchanged.
    pub const DEFAULT_LAYER: u32 = 0;

    /// The normal Channel's Layer 0 texel, `Rgba16Float`: 0.5, 0.5, 1 and 1
    /// as binary16. 0.5 has no `Unorm8` code (128 decodes to 0.502, a tilt
    /// of 0.32°), so the shader's `encoded * 2.0 - 1.0` gives exactly 0 only
    /// from a float format.
    const FLAT_NORMAL_TEXEL: [u16; 4] = [0x3800, 0x3800, 0x3C00, 0x3C00];

    /// The neutral default `channel` holds in Layer 0 (see DEFAULT_LAYER):
    /// colour Channels are sRGB, data Channels linear.
    pub(crate) fn neutral_texture(channel: MaterialChannel) -> CookedTexture {
        let (format, texel) = match channel {
            MaterialChannel::Albedo | MaterialChannel::Emissive => {
                (wgpu::TextureFormat::Rgba8UnormSrgb, vec![255; 4])
            }
            MaterialChannel::Normal => (
                wgpu::TextureFormat::Rgba16Float,
                Self::FLAT_NORMAL_TEXEL
                    .iter()
                    .flat_map(|half| half.to_le_bytes())
                    .collect(),
            ),
            MaterialChannel::MetallicRoughness | MaterialChannel::Occlusion => {
                (wgpu::TextureFormat::Rgba8Unorm, vec![255; 4])
            }
        };
        CookedTexture {
            format,
            width: 1,
            height: 1,
            levels: vec![texel],
        }
    }

    pub fn new(device: &wgpu::Device, memory: &GpuMemory, staging: &mut LoadStaging) -> Self {
        let materials_buffer = memory.create_buffer(
            device,
            GpuMemoryKind::SceneAndPassBuffers,
            &wgpu::BufferDescriptor {
                label: Some("materials_buffer"),
                size: u64::from(MATERIAL_CAPACITY) * std::mem::size_of::<MaterialData>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        // Ordered as `MaterialChannel`.
        let mut channels = [
            "albedo_textures",
            "normal_textures",
            "metallic_roughness_textures",
            "occlusion_textures",
            "emissive_textures",
        ]
        .map(|label| ChannelTextures::new(label, memory));

        for channel in MaterialChannel::ALL {
            channels[channel as usize].push(device, staging, &Self::neutral_texture(channel));
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
            generations: Generations::default(),
            materials: Vec::new(),
            materials_buffer,
            channels,
            texture_layers: HashMap::new(),
            material_handles: HashMap::new(),
            colour_sampler,
            data_sampler,
            bind_group_layout,
            bind_group,
            bind_group_is_stale: false,
        }
    }

    /// Registers a material in the SSBO through `staging`, returning its
    /// handle, or `None`, logged, when the SSBO is full.
    pub fn insert(
        &mut self,
        staging: &mut LoadStaging,
        data: MaterialData,
    ) -> Option<MaterialHandle> {
        if self.generations.full(MATERIAL_CAPACITY) {
            tracing::warn!(
                capacity = MATERIAL_CAPACITY,
                "Material refused: the Material SSBO is full"
            );
            return None;
        }
        let (index, generation) = self.generations.take();
        let offset = u64::from(index) * std::mem::size_of::<MaterialData>() as u64;
        staging.write(&self.materials_buffer, offset, bytemuck::bytes_of(&data));
        if index as usize == self.materials.len() {
            self.materials.push(data);
        } else {
            self.materials[index as usize] = data;
        }
        tracing::debug!(handle = index, "material registered");
        Some(MaterialHandle { index, generation })
    }

    /// The data of `handle`'s Material, or None when it was freed.
    #[cfg_attr(not(all(test, feature = "cook")), allow(dead_code))]
    pub fn get(&self, handle: MaterialHandle) -> Option<&MaterialData> {
        self.generations
            .live(handle.index, handle.generation)
            .then(|| &self.materials[handle.index as usize])
    }

    /// Releases `handle`'s index, which the next Material inserted takes, so
    /// `handle` finds nothing again; its Textures stay loaded. A Mesh whose
    /// Primitive names it must be freed first. A Handle that finds no
    /// Material is warned about.
    #[cfg_attr(not(all(test, feature = "cook")), allow(dead_code))]
    pub fn free(&mut self, handle: MaterialHandle) {
        if !self.generations.free(handle.index, handle.generation) {
            tracing::warn!(?handle, "Material not freed: its Handle finds no Material");
            return;
        }
        self.material_handles
            .retain(|_, material_handle| *material_handle != handle);
        tracing::debug!(?handle, "Material freed");
    }

    /// Which Material Handles find a Material, and how many Materials were
    /// freed, as prepare reads them.
    pub(crate) fn generations(&self) -> &Generations {
        &self.generations
    }

    /// Loads the Material `id` from `cache`, and each Texture it samples, once
    /// per Asset ID, written through `staging`, and returns its Handle, or
    /// the SSBO's refusal when it is full. A Channel the Material lacks
    /// samples Layer 0, as does one already full.
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        cache: &CookedCache,
        id: AssetId,
    ) -> anyhow::Result<MaterialHandle> {
        if let Some(&handle) = self.material_handles.get(&id) {
            return Ok(handle);
        }
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
        let handle = self
            .insert(staging, data)
            .with_context(|| format!("the Material pool refused Material {id}"))?;
        self.material_handles.insert(id, handle);
        tracing::debug!(material = %id, ?layers, "Material loaded");
        Ok(handle)
    }

    /// Loads the Texture `id` from `cache` into `channel` once per Asset ID,
    /// written through `staging`, and returns its Layer: the next free slice
    /// of its shape's Texture class, or Layer 0 when the Channel is full.
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
        let channel_textures = &mut self.channels[channel as usize];
        let classes_before = channel_textures.views.len();
        let Some(layer) = channel_textures.push(device, staging, &texture) else {
            return Ok(Self::DEFAULT_LAYER);
        };
        self.bind_group_is_stale |= channel_textures.views.len() != classes_before;
        self.texture_layers.insert(id, layer);
        Ok(layer)
    }

    /// Rebuilds the bind group when loads since the last call opened Texture
    /// classes, so a scene's loads build it once: bind groups are snapshots.
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
            // ADR 0017's gate: the build's cost at the bound arrays' count.
            tracing::info!(
                texture_classes = self
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
/// Texture classes, and the colour and data Channels' samplers. Bind groups
/// are snapshots, so a new Texture class needs a new one.
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
    use crate::assets::pools::tests::{gpu_context, logged};
    use crate::cooked::CookedMaterial;
    use crate::gpu::GpuContext;
    use crate::stats::gpu_memory::tests::gpu_memory;

    /// A shape's first class has one slice and each later one twice the one
    /// before's, up to `SLICES_PER_CLASS`; another shape opens classes of its
    /// own between them; and a Channel holding `CLASSES_PER_CHANNEL` classes
    /// refuses a Texture only when its shape's newest class is full.
    #[test]
    fn a_shapes_classes_double_to_the_floors_slices_and_a_full_channel_refuses() {
        let shape = |width| TextureShape {
            width,
            height: 4,
            mip_level_count: 3,
            format: wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        };
        let (wall, other) = (shape(4), shape(8));
        let mut table = ClassTable::default();
        let shapes = std::iter::repeat_n(wall, 7).chain([other, wall]);
        let placed: Vec<(u32, u32)> = shapes
            .map(|shape| table.place(shape).unwrap())
            .map(|placement| (placement.class, placement.slice))
            .collect();
        assert_eq!(
            placed,
            [
                (0, 0),
                (1, 0),
                (1, 1),
                (2, 0),
                (2, 1),
                (2, 2),
                (2, 3),
                (3, 0),
                (4, 0)
            ]
        );
        assert_eq!(table.slices(4), 8);
        for _ in 0..2000 {
            table.place(wall).unwrap();
        }
        let wall_slices: Vec<u32> = table
            .classes
            .iter()
            .filter(|class| class.shape == wall)
            .map(|class| class.slices)
            .collect();
        assert_eq!(wall_slices[..10], [1, 2, 4, 8, 16, 32, 64, 128, 256, 256]);
        assert!(wall_slices.iter().all(|&slices| slices <= SLICES_PER_CLASS));

        let mut width = 16;
        while table.classes.len() < CLASSES_PER_CHANNEL as usize {
            table.place(shape(width)).unwrap();
            width += 4;
        }
        assert_eq!(table.place(shape(width)), None);
        assert_eq!(table.place(other), None);
        assert!(table.place(wall).is_some());
        assert!(table.place(shape(16)).is_none());
    }

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
        let memory = gpu_memory(&context);
        let texture_bytes = || {
            let bytes = memory.bytes_by_kind();
            bytes[GpuMemoryKind::TexturePool as usize]
                + bytes[GpuMemoryKind::ResidentTails as usize]
        };
        let mut pool = MaterialPool::new(&context.device, &memory, &mut staging);
        let neutral_bytes = texture_bytes();
        let occlusion_layers: Vec<u32> = (1..=3)
            .map(|id| {
                let handle = pool
                    .load(&context.device, &mut staging, &cache, AssetId(id))
                    .unwrap();
                pool.materials[handle.index() as usize].layers[3]
            })
            .collect();
        assert_eq!(texture_bytes() - neutral_bytes, 3 * 8);
        assert_eq!(
            occlusion_layers,
            [
                SLICES_PER_CLASS,
                SLICES_PER_CLASS,
                MaterialPool::DEFAULT_LAYER
            ]
        );
        assert!(
            pool.materials
                .iter()
                .all(|material| material.metallic_roughness[2] == 0.5)
        );
        std::fs::remove_dir_all(&directory).ok();
    }

    /// A freed Material's index takes the next Material inserted, in a new
    /// generation, which the freed Handle does not find; its Asset ID loads
    /// it again.
    #[test]
    fn a_freed_materials_index_takes_the_next_material_and_refuses_the_old_handle() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-material-free-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        cache.write(AssetId(1), "key", &material.to_blob()).unwrap();
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MaterialPool::new(&context.device, &gpu_memory(&context), &mut staging);
        let kept = pool
            .insert(&mut staging, bytemuck::Zeroable::zeroed())
            .unwrap();
        let freed = pool
            .load(&context.device, &mut staging, &cache, AssetId(1))
            .unwrap();

        pool.free(freed);
        assert!(pool.get(freed).is_none());
        assert!(pool.get(kept).is_some());
        assert_eq!(pool.generations().frees(), 1);
        let (_, log) = logged(|| pool.free(freed));
        assert!(log.contains("Material not freed"), "not logged: {log:?}");

        let mut replacement: MaterialData = bytemuck::Zeroable::zeroed();
        replacement.base_color = [0.5; 4];
        let inserted = pool.insert(&mut staging, replacement).unwrap();
        assert_eq!(inserted.index, freed.index);
        assert!(pool.get(freed).is_none());
        assert_eq!(
            pool.get(inserted).map(|data| data.base_color),
            Some([0.5; 4])
        );
        let reloaded = pool
            .load(&context.device, &mut staging, &cache, AssetId(1))
            .unwrap();
        assert!(reloaded != freed && reloaded != inserted);
        std::fs::remove_dir_all(&directory).ok();
    }

    /// The SSBO holds `MATERIAL_CAPACITY` Materials; the next gets no
    /// Handle, so nothing draws with another Material in its place, and the
    /// refusal is logged.
    #[test]
    fn a_material_past_capacity_is_refused_and_logged() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MaterialPool::new(&context.device, &gpu_memory(&context), &mut staging);
        let material = MaterialData {
            base_color: [1.0; 4],
            metallic_roughness: [0.0, 1.0, 1.0, 0.0],
            emissive: [0.0, 0.0, 0.0, 1.0],
            layers: [MaterialPool::DEFAULT_LAYER; 4],
            extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
        };
        for index in 0..MATERIAL_CAPACITY {
            let handle = pool.insert(&mut staging, material);
            assert!(handle.is_some_and(|handle| handle.index() == index));
        }
        let (handle, log) = logged(|| pool.insert(&mut staging, material));
        assert!(handle.is_none(), "a Material past capacity got a Handle");
        assert!(log.contains("WARN"), "not a warning: {log:?}");
        assert!(log.contains("Material refused"), "not logged: {log:?}");
        assert_eq!(pool.materials.len() as u32, MATERIAL_CAPACITY);
    }
}
