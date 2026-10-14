use crate::stats::texture_bytes;

/// Textures each Channel's binding array can hold. Five Channels of 8192 stay
/// under the Capability floor's 65,536 binding-array elements per stage
/// (ADR 0021).
const TEXTURES_PER_CHANNEL: u32 = 8192;

/// Materials the SSBO holds. Materials are few and never deleted in Phase 6,
/// so a fixed cap avoids growth + bind-group rebuilds for now.
const MATERIAL_CAPACITY: u64 = 256;

/// Every Texture is RGBA8 until cooked BC Textures arrive (#150).
const BYTES_PER_PIXEL: u32 = 4;

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
/// Channel's position in `MaterialPool::channels` and its binding minus one.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MaterialChannel {
    Albedo = 0,
    Normal = 1,
    MetallicRoughness = 2,
    Occlusion = 3,
    Emissive = 4,
}

impl MaterialChannel {
    const ALL: [Self; 5] = [
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
    pub metallic_roughness: [f32; 4], // x=metallic, y=roughness, zw=pad
    pub emissive: [f32; 4],           // xyz=color, w=strength
    pub texture_indices: [u32; 4],    // x=albedo, y=normal, z=MR, w=AO
    pub extra: [u32; 4],              // x=emissive_idx, yzw=pad
}

/// One Channel's Textures, each sized to its source, bound as one partially
/// bound binding array (ADR 0017). A Layer is an index into `views`.
struct ChannelTextures {
    label: &'static str,
    format: wgpu::TextureFormat,
    textures: Vec<wgpu::Texture>,
    views: Vec<wgpu::TextureView>,
}

impl ChannelTextures {
    fn new(label: &'static str, format: wgpu::TextureFormat) -> Self {
        Self {
            label,
            format,
            textures: Vec::new(),
            views: Vec::new(),
        }
    }

    /// Creates a `width` × `height` RGBA8 Texture from `pixels` and returns
    /// its Layer, or `None` when the Channel is full.
    fn push(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pixels: &[u8],
        width: u32,
        height: u32,
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
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(self.label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(BYTES_PER_PIXEL * width),
                rows_per_image: Some(height),
            },
            size,
        );
        self.views.push(texture.create_view(&Default::default()));
        self.textures.push(texture);
        Some(layer)
    }
}

/// Material-as-ID: one SSBO of `MaterialData` indexed by `MaterialHandle`, plus
/// one texture binding array per Channel. A single bind group serves every
/// material; the fragment shader looks up `materials[instance.material_id]` and
/// samples each Channel's array at the Layer stored in `texture_indices`.
pub struct MaterialPool {
    materials: Vec<MaterialData>,
    materials_buffer: wgpu::Buffer,
    channels: [ChannelTextures; 5],
    sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl MaterialPool {
    /// Layer 0 of every Channel holds a 1×1 neutral texture: white for
    /// albedo / metallic-roughness / occlusion / emissive, the flat normal
    /// `(0.5, 0.5, 1.0)` for normal. A material without a channel points here,
    /// samples a no-op, and its factors pass through unchanged.
    pub const DEFAULT_LAYER: u32 = 0;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("materials_buffer"),
            size: MATERIAL_CAPACITY * std::mem::size_of::<MaterialData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Color channels are sRGB; data channels (vectors / scalars) are linear.
        // Ordered as `MaterialChannel`.
        let mut channels = [
            ChannelTextures::new("albedo", wgpu::TextureFormat::Rgba8UnormSrgb),
            ChannelTextures::new("normal", wgpu::TextureFormat::Rgba8Unorm),
            ChannelTextures::new("metallic_roughness", wgpu::TextureFormat::Rgba8Unorm),
            ChannelTextures::new("occlusion", wgpu::TextureFormat::Rgba8Unorm),
            ChannelTextures::new("emissive", wgpu::TextureFormat::Rgba8UnormSrgb),
        ];

        // Neutral defaults into Layer 0 of every Channel (see DEFAULT_LAYER).
        let white = [255u8; 4];
        let flat_normal = [128u8, 128, 255, 255];
        for channel in MaterialChannel::ALL {
            let pixel = if channel == MaterialChannel::Normal {
                &flat_normal
            } else {
                &white
            };
            channels[channel as usize].push(device, queue, pixel, 1, 1);
        }

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: std::num::NonZeroU32::new(TEXTURES_PER_CHANNEL),
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture_entry(1),
                texture_entry(2),
                texture_entry(3),
                texture_entry(4),
                texture_entry(5),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            &materials_buffer,
            &channels,
            &sampler,
        );

        Self {
            materials: Vec::new(),
            materials_buffer,
            channels,
            sampler,
            bind_group_layout,
            bind_group,
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

    /// Registers a material in the SSBO, returning its handle.
    pub fn insert(&mut self, queue: &wgpu::Queue, data: MaterialData) -> MaterialHandle {
        if self.materials.len() as u64 >= MATERIAL_CAPACITY {
            tracing::error!(
                capacity = MATERIAL_CAPACITY,
                "material SSBO full; reusing material 0 (raise MATERIAL_CAPACITY or add growth)"
            );
            return MaterialHandle(0);
        }
        let handle = MaterialHandle(self.materials.len() as u32);
        let offset = handle.0 as u64 * std::mem::size_of::<MaterialData>() as u64;
        queue.write_buffer(&self.materials_buffer, offset, bytemuck::bytes_of(&data));
        self.materials.push(data);
        tracing::debug!(handle = handle.0, "material registered");
        handle
    }

    /// Uploads a `width` × `height` RGBA8 image as a new Texture of `channel`
    /// and rebuilds the bind group, returning the Layer to store in
    /// `MaterialData`. Any rejected upload falls back to `DEFAULT_LAYER` so the
    /// material still renders.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        channel: MaterialChannel,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> u32 {
        let max_dimension = device.limits().max_texture_dimension_2d;
        if width == 0 || height == 0 || width > max_dimension || height > max_dimension {
            tracing::error!(
                ?channel,
                width,
                height,
                max_dimension,
                "texture size is zero or above the device limit; using the neutral default layer"
            );
            return Self::DEFAULT_LAYER;
        }
        let expected_bytes = BYTES_PER_PIXEL as usize * width as usize * height as usize;
        if pixels.len() != expected_bytes {
            tracing::error!(
                ?channel,
                bytes = pixels.len(),
                expected_bytes,
                "texture byte length does not match RGBA8 layout; using the neutral default layer"
            );
            return Self::DEFAULT_LAYER;
        }
        let Some(layer) =
            self.channels[channel as usize].push(device, queue, pixels, width, height)
        else {
            return Self::DEFAULT_LAYER;
        };
        // One rebuild per upload is fine at load; streaming batches residency
        // changes into one rebuild per frame (ADR 0017).
        self.bind_group = create_bind_group(
            device,
            &self.bind_group_layout,
            &self.materials_buffer,
            &self.channels,
            &self.sampler,
        );
        tracing::debug!(?channel, layer, width, height, "texture uploaded");
        layer
    }
}

/// The material bind group: the SSBO, each Channel's Textures as bindings 1
/// to 5, and the sampler. Bind groups are snapshots, so a new Texture needs a
/// new one.
fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    materials_buffer: &wgpu::Buffer,
    channels: &[ChannelTextures; 5],
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let views: Vec<Vec<&wgpu::TextureView>> = channels
        .iter()
        .map(|channel| channel.views.iter().collect())
        .collect();
    let mut entries = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: materials_buffer.as_entire_binding(),
    }];
    entries.extend(
        views
            .iter()
            .zip(1..)
            .map(|(channel_views, binding)| wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureViewArray(channel_views),
            }),
    );
    entries.push(wgpu::BindGroupEntry {
        binding: 6,
        resource: wgpu::BindingResource::Sampler(sampler),
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Material Bind Group"),
        layout,
        entries: &entries,
    })
}
