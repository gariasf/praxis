use bevy_ecs::prelude::*;

/// All channel arrays are this square dimension. glTF assets are scale-padded
/// to it on load (the default SciFiHelmet is already 2048², so the pad is a
/// no-op and not yet implemented — see `MaterialPool::upload`).
pub const TEXTURE_DIM: u32 = 2048;

/// Layers pre-allocated per channel array. Eagerly sized; array-layer growth
/// (realloc + re-copy) is deferred until an asset needs more than this.
const LAYER_CAPACITY: u32 = 8;

/// Materials the SSBO holds. Materials are few and never deleted in Phase 6,
/// so a fixed cap avoids growth + bind-group rebuilds for now.
const MATERIAL_CAPACITY: u64 = 256;

/// Every channel array is RGBA8.
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

/// One texture channel of a material. Each channel has its own
/// `texture_2d_array`; a material stores one layer index per channel.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MaterialChannel {
    Albedo,
    Normal,
    MetallicRoughness,
    Occlusion,
    Emissive,
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

/// A `texture_2d_array` for one channel plus how many layers hold data.
struct ChannelArray {
    texture: wgpu::Texture,
    layers_used: u32,
}

impl ChannelArray {
    fn new(device: &wgpu::Device, label: &str, format: wgpu::TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: TEXTURE_DIM,
                height: TEXTURE_DIM,
                depth_or_array_layers: LAYER_CAPACITY,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            texture,
            layers_used: 0,
        }
    }

    fn view(&self) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }

    /// Writes one `TEXTURE_DIM`-square RGBA8 image into the next free layer and
    /// returns that layer, or `MaterialPool::DEFAULT_LAYER` when the array is full.
    fn push_layer(&mut self, queue: &wgpu::Queue, channel: MaterialChannel, pixels: &[u8]) -> u32 {
        if self.layers_used >= LAYER_CAPACITY {
            tracing::error!(
                ?channel,
                capacity = LAYER_CAPACITY,
                "channel array full; using the neutral default layer"
            );
            return MaterialPool::DEFAULT_LAYER;
        }
        let layer = self.layers_used;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(BYTES_PER_PIXEL * TEXTURE_DIM),
                rows_per_image: Some(TEXTURE_DIM),
            },
            wgpu::Extent3d {
                width: TEXTURE_DIM,
                height: TEXTURE_DIM,
                depth_or_array_layers: 1,
            },
        );
        self.layers_used += 1;
        layer
    }
}

/// Material-as-ID: one SSBO of `MaterialData` indexed by `MaterialHandle`, plus
/// one `texture_2d_array` per channel. A single bind group serves every
/// material; the fragment shader looks up `materials[instance.material_id]` and
/// samples each array by the layer index stored in `texture_indices`.
#[derive(Resource)]
pub struct MaterialPool {
    materials: Vec<MaterialData>,
    materials_buffer: wgpu::Buffer,
    albedo: ChannelArray,
    normal: ChannelArray,
    metallic_roughness: ChannelArray,
    occlusion: ChannelArray,
    emissive: ChannelArray,
    // Referenced by the bind group, which keeps it alive; never read again.
    #[allow(dead_code)]
    sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl MaterialPool {
    /// Layer 0 of every channel array holds a neutral texture: white for
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
        let mut albedo =
            ChannelArray::new(device, "albedo_array", wgpu::TextureFormat::Rgba8UnormSrgb);
        let mut normal = ChannelArray::new(device, "normal_array", wgpu::TextureFormat::Rgba8Unorm);
        let mut metallic_roughness = ChannelArray::new(
            device,
            "metallic_roughness_array",
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let mut occlusion =
            ChannelArray::new(device, "occlusion_array", wgpu::TextureFormat::Rgba8Unorm);
        let mut emissive = ChannelArray::new(
            device,
            "emissive_array",
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        // Neutral defaults into layer 0 of every array (see DEFAULT_LAYER).
        let pixel_count = (TEXTURE_DIM * TEXTURE_DIM) as usize;
        let white = vec![255u8; pixel_count * BYTES_PER_PIXEL as usize];
        let flat_normal = [128u8, 128, 255, 255].repeat(pixel_count);
        albedo.push_layer(queue, MaterialChannel::Albedo, &white);
        normal.push_layer(queue, MaterialChannel::Normal, &flat_normal);
        metallic_roughness.push_layer(queue, MaterialChannel::MetallicRoughness, &white);
        occlusion.push_layer(queue, MaterialChannel::Occlusion, &white);
        emissive.push_layer(queue, MaterialChannel::Emissive, &white);

        let albedo_view = albedo.view();
        let normal_view = normal.view();
        let metallic_roughness_view = metallic_roughness.view();
        let occlusion_view = occlusion.view();
        let emissive_view = emissive.view();

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
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: materials_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&metallic_roughness_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&occlusion_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&emissive_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            materials: Vec::new(),
            materials_buffer,
            albedo,
            normal,
            metallic_roughness,
            occlusion,
            emissive,
            sampler,
            bind_group_layout,
            bind_group,
        }
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

    /// Uploads RGBA8 pixels into the next free layer of `channel`, returning
    /// the layer index to store in `MaterialData`. Phase 6 requires
    /// `TEXTURE_DIM`-square images (SciFiHelmet is 2048²); scale-padding for
    /// off-size assets is deferred until one needs it. Any rejected upload
    /// falls back to `DEFAULT_LAYER` so the material still renders.
    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        channel: MaterialChannel,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> u32 {
        if width != TEXTURE_DIM || height != TEXTURE_DIM {
            tracing::error!(
                ?channel,
                width,
                height,
                expected = TEXTURE_DIM,
                "texture is not TEXTURE_DIM-square; using the neutral default layer"
            );
            return Self::DEFAULT_LAYER;
        }
        let expected_bytes = (BYTES_PER_PIXEL * TEXTURE_DIM * TEXTURE_DIM) as usize;
        if pixels.len() != expected_bytes {
            tracing::error!(
                ?channel,
                bytes = pixels.len(),
                expected_bytes,
                "texture byte length does not match RGBA8 layout; using the neutral default layer"
            );
            return Self::DEFAULT_LAYER;
        }
        let channel_array = match channel {
            MaterialChannel::Albedo => &mut self.albedo,
            MaterialChannel::Normal => &mut self.normal,
            MaterialChannel::MetallicRoughness => &mut self.metallic_roughness,
            MaterialChannel::Occlusion => &mut self.occlusion,
            MaterialChannel::Emissive => &mut self.emissive,
        };
        let layer = channel_array.push_layer(queue, channel, pixels);
        tracing::debug!(?channel, layer, "texture uploaded");
        layer
    }
}
