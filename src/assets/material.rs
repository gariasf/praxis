use bevy_ecs::prelude::*;

/// All channel arrays are this square dimension. glTF assets are scale-padded
/// to it on load (Phase 6 ships DamagedHelmet, already 2048², so the pad is a
/// no-op and not yet implemented — see `upload_albedo`).
pub const TEXTURE_DIM: u32 = 2048;

/// Layers pre-allocated per channel array. Eagerly sized; array-layer growth
/// (realloc + re-copy) is deferred until an asset needs more than this.
const LAYER_CAPACITY: u32 = 8;

/// Materials the SSBO holds. Materials are few and never deleted in Phase 6,
/// so a fixed cap avoids growth + bind-group rebuilds for now.
const MATERIAL_CAPACITY: u64 = 256;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MaterialHandle(pub u32);

impl MaterialHandle {
    /// Index into the material SSBO. Reads better than `.0` at call sites that
    /// unwrap a wrapping newtype (e.g. `material_ref.0.index()`).
    pub fn index(self) -> u32 {
        self.0
    }
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

// Layout guard: must match the WGSL `MaterialData` struct (5 × vec4 = 80 bytes).
// Catches a silent Rust/WGSL mismatch at compile time if a field is added or
// the vec3-pads are dropped.
const _: () = assert!(
    std::mem::size_of::<MaterialData>() == 80,
    "MaterialData must stay 80 bytes to match the WGSL layout"
);

/// Material-as-ID: one SSBO of `MaterialData` indexed by `MaterialHandle`, plus
/// one `texture_2d_array` per channel. A single bind group serves every
/// material; the fragment shader looks up `materials[instance.material_id]` and
/// samples each array by the layer index stored in `texture_indices`.
#[derive(Resource)]
pub struct MaterialPool {
    materials: Vec<MaterialData>,
    materials_buffer: wgpu::Buffer,
    albedo_array: wgpu::Texture,
    // The bind group already references these channels' views + the sampler, so
    // they keep the GPU resources alive. The texture handles are read again only
    // when Step 4 uploads normal / MR / AO / emissive pixels into them.
    #[allow(dead_code)]
    normal_array: wgpu::Texture,
    #[allow(dead_code)]
    metallic_roughness_array: wgpu::Texture,
    #[allow(dead_code)]
    ao_array: wgpu::Texture,
    #[allow(dead_code)]
    emissive_array: wgpu::Texture,
    #[allow(dead_code)]
    sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    albedo_layers_used: u32,
}

fn create_channel_array(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
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
    })
}

impl MaterialPool {
    pub fn new(device: &wgpu::Device) -> Self {
        let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("materials_buffer"),
            size: MATERIAL_CAPACITY * std::mem::size_of::<MaterialData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Color channels are sRGB; data channels (vectors / scalars) are linear.
        let albedo_array =
            create_channel_array(device, "albedo_array", wgpu::TextureFormat::Rgba8UnormSrgb);
        let emissive_array = create_channel_array(
            device,
            "emissive_array",
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let normal_array =
            create_channel_array(device, "normal_array", wgpu::TextureFormat::Rgba8Unorm);
        let metallic_roughness_array = create_channel_array(
            device,
            "metallic_roughness_array",
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let ao_array = create_channel_array(device, "ao_array", wgpu::TextureFormat::Rgba8Unorm);

        let view = |t: &wgpu::Texture| {
            t.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            })
        };
        let albedo_view = view(&albedo_array);
        let normal_view = view(&normal_array);
        let mr_view = view(&metallic_roughness_array);
        let ao_view = view(&ao_array);
        let emissive_view = view(&emissive_array);

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
                    resource: wgpu::BindingResource::TextureView(&mr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&ao_view),
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
            albedo_array,
            normal_array,
            metallic_roughness_array,
            ao_array,
            emissive_array,
            sampler,
            bind_group_layout,
            bind_group,
            albedo_layers_used: 0,
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

    /// Uploads RGBA pixels into the next free albedo layer, returning its index.
    /// Phase 6 requires `TEXTURE_DIM`-square images (DamagedHelmet is 2048²);
    /// scale-padding for off-size assets is deferred until one needs it.
    pub fn upload_albedo(
        &mut self,
        queue: &wgpu::Queue,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> u32 {
        if width != TEXTURE_DIM || height != TEXTURE_DIM {
            tracing::error!(
                width,
                height,
                expected = TEXTURE_DIM,
                "albedo not square-2048; upload skipped"
            );
            return 0;
        }
        if self.albedo_layers_used >= LAYER_CAPACITY {
            tracing::error!(
                capacity = LAYER_CAPACITY,
                "albedo array full; reusing layer 0"
            );
            return 0;
        }
        let layer = self.albedo_layers_used;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.albedo_array,
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
                bytes_per_row: Some(4 * TEXTURE_DIM),
                rows_per_image: Some(TEXTURE_DIM),
            },
            wgpu::Extent3d {
                width: TEXTURE_DIM,
                height: TEXTURE_DIM,
                depth_or_array_layers: 1,
            },
        );
        self.albedo_layers_used += 1;
        layer
    }
}
