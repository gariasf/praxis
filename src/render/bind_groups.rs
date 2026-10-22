//! Bind groups, each defined once: its index and, per binding, its WGSL name
//! and type, its layout entry and who writes it. A definition builds its
//! layout and bind groups, gives the index pipeline layouts and passes use,
//! and declares its bindings in every shader module assembled with it, so no
//! shader part declares one. Groups are packed by how often they change,
//! within the Capability floor's four (ADR 0021): 0 the frame's, 1 the
//! Materials', 2 the scene geometry's, 3 a pass's own.

use std::num::NonZeroU32;

use crate::assets::material::TEXTURES_PER_CHANNEL;
use crate::render::targets::DEPTH_PYRAMID_FORMAT;

/// Group 0, written every frame: the camera and the lights.
pub const FRAME: BindGroupDefinition = BindGroupDefinition {
    index: 0,
    label: "Frame Bind Group",
    bindings: &[
        uniform(
            "camera",
            "CameraUniform",
            wgpu::ShaderStages::VERTEX
                .union(wgpu::ShaderStages::COMPUTE)
                .union(wgpu::ShaderStages::FRAGMENT),
        ),
        uniform("light", "LightUniform", wgpu::ShaderStages::FRAGMENT),
    ],
};

/// Group 1, which the Material pool rebuilds when loads add Textures: the
/// Materials (ADR 0001), each Channel's Textures as a binding array indexed
/// by Layer, in `MaterialChannel` order (ADR 0017), and their sampler.
pub const MATERIALS: BindGroupDefinition = BindGroupDefinition {
    index: 1,
    label: "Material Bind Group",
    bindings: &[
        read_only_storage(
            "materials",
            "array<MaterialData>",
            wgpu::ShaderStages::FRAGMENT,
        ),
        channel("albedo_textures"),
        channel("normal_textures"),
        channel("metallic_roughness_textures"),
        channel("occlusion_textures"),
        channel("emissive_textures"),
        Binding {
            name: "pbr_sampler",
            wgsl_type: "sampler",
            binding_type: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            stages: wgpu::ShaderStages::FRAGMENT,
            count: None,
            writer: Writer::Cpu,
        },
    ],
};

/// Group 2, the scene geometry, built once since its buffers never grow: the
/// Instances, the vertex Arena the vertex shader pulls from (ADR 0019), and
/// the Mesh pool's tables of Meshes and Meshlets.
pub const SCENE_GEOMETRY: BindGroupDefinition = BindGroupDefinition {
    index: 2,
    label: "Scene Geometry Bind Group",
    bindings: &[
        read_only_storage(
            "instances",
            "array<InstanceData>",
            wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
        ),
        read_only_storage(
            "vertices",
            "array<CookedVertex>",
            wgpu::ShaderStages::VERTEX,
        ),
        read_only_storage("meshes", "array<MeshData>", wgpu::ShaderStages::COMPUTE),
        read_only_storage(
            "meshlets",
            "array<MeshletData>",
            wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
        ),
        read_only_storage("meshlet_vertices", "array<u32>", wgpu::ShaderStages::VERTEX),
        read_only_storage(
            "meshlet_triangles",
            "array<u32>",
            wgpu::ShaderStages::COMPUTE,
        ),
    ],
};

/// Group 3 of the culling pass: the Instance slots in use, which the CPU
/// writes when they change, and what the pass writes: each slot's offsets in
/// the frame's draw, and the draw, its Meshlet records, its packed indices,
/// and its indirect arguments and counts, which the CPU reads back; each
/// slot's visibility, which phase two writes for the next frame's phase one;
/// and the depth pyramid, which phase two tests Instances against.
pub const FRAME_DRAW: BindGroupDefinition = BindGroupDefinition {
    index: 3,
    label: "Frame Draw Bind Group",
    bindings: &[
        uniform("instance_slot_count", "u32", wgpu::ShaderStages::COMPUTE),
        storage("instance_offsets", "array<vec2<u32>>"),
        storage("meshlet_records", "array<MeshletRecord>"),
        storage("packed_indices", "array<u32>"),
        storage("frame_draw", "FrameDraw"),
        storage("instance_visibility", "array<u32>"),
        loaded_texture(
            "depth_pyramid",
            "texture_2d<f32>",
            wgpu::TextureSampleType::Float { filterable: false },
            false,
            wgpu::ShaderStages::COMPUTE,
        ),
    ],
};

/// Group 3 of the main pass: the Meshlet records of the frame's draw, which
/// the culling pass writes.
pub const MESHLET_RECORDS: BindGroupDefinition = BindGroupDefinition {
    index: 3,
    label: "Meshlet Records Bind Group",
    bindings: &[Binding {
        writer: Writer::Gpu,
        ..read_only_storage(
            "meshlet_records",
            "array<MeshletRecord>",
            wgpu::ShaderStages::VERTEX,
        )
    }],
};

/// Group 3 of the depth pyramid pass, one bind group per level of the
/// pyramid (`RenderTargets`): the level written, and what it reduces, the
/// depth target for level 0, the level below for the others.
pub const DEPTH_PYRAMID_LEVEL: BindGroupDefinition = BindGroupDefinition {
    index: 3,
    label: "Depth Pyramid Level Bind Group",
    bindings: &[
        Binding {
            name: "level",
            wgsl_type: "texture_storage_2d<r32float, write>",
            binding_type: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: DEPTH_PYRAMID_FORMAT,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            stages: wgpu::ShaderStages::COMPUTE,
            count: None,
            writer: Writer::Gpu,
        },
        loaded_texture(
            "depth",
            "texture_depth_multisampled_2d",
            wgpu::TextureSampleType::Depth,
            true,
            wgpu::ShaderStages::COMPUTE,
        ),
        loaded_texture(
            "level_below",
            "texture_2d<f32>",
            wgpu::TextureSampleType::Float { filterable: false },
            false,
            wgpu::ShaderStages::COMPUTE,
        ),
    ],
};

/// Group 3 of the tonemapping pass: the HDR target, which the lit pass
/// resolves into, and the dither's mask, which the CPU loads.
pub const TONEMAPPING_INPUT: BindGroupDefinition = BindGroupDefinition {
    index: 3,
    label: "Tonemapping Input Bind Group",
    bindings: &[
        loaded_texture(
            "hdr_target",
            "texture_2d<f32>",
            wgpu::TextureSampleType::Float { filterable: false },
            false,
            wgpu::ShaderStages::FRAGMENT,
        ),
        Binding {
            writer: Writer::Cpu,
            ..loaded_texture(
                "dither_mask",
                "texture_2d<f32>",
                wgpu::TextureSampleType::Float { filterable: false },
                false,
                wgpu::ShaderStages::FRAGMENT,
            )
        },
    ],
};

/// A bind group: its index and its bindings, each numbered by its place.
pub struct BindGroupDefinition {
    /// Where pipeline layouts hold the group and passes set it.
    pub index: u32,
    pub label: &'static str,
    pub bindings: &'static [Binding],
}

/// One binding: its WGSL variable, its layout entry and who writes it.
pub struct Binding {
    pub name: &'static str,
    /// The variable's WGSL type; with a `count`, its elements'.
    pub wgsl_type: &'static str,
    pub binding_type: wgpu::BindingType,
    /// Exactly the stages whose entry points use it, in every module that
    /// declares its group (`shader_tests`).
    pub stages: wgpu::ShaderStages,
    /// The length of its binding array; `None` for one resource.
    pub count: Option<NonZeroU32>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub writer: Writer,
}

/// Who writes a binding's contents. A struct the CPU writes has a Rust twin
/// whose layout `shader_tests` compares with it; one only the GPU writes
/// needs none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Writer {
    Cpu,
    Gpu,
}

impl BindGroupDefinition {
    /// Whether `stage` sees any of the group's bindings.
    pub fn visible_to(&self, stage: wgpu::ShaderStages) -> bool {
        self.bindings
            .iter()
            .any(|binding| binding.stages.contains(stage))
    }

    pub fn layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let names: Vec<&str> = self.bindings.iter().map(|binding| binding.name).collect();
        self.layout_of(device, &names)
    }

    /// The layout of the bindings `names` names alone, each at its index,
    /// for pipelines whose entry points use no others.
    pub fn layout_of(&self, device: &wgpu::Device, names: &[&str]) -> wgpu::BindGroupLayout {
        for name in names {
            assert!(
                self.bindings.iter().any(|binding| binding.name == *name),
                "{} has no binding `{name}`",
                self.label
            );
        }
        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..)
            .zip(self.bindings)
            .filter(|(_, binding)| names.contains(&binding.name))
            .map(|(index, binding)| wgpu::BindGroupLayoutEntry {
                binding: index,
                visibility: binding.stages,
                ty: binding.binding_type,
                count: binding.count,
            })
            .collect();
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(self.label),
            entries: &entries,
        })
    }

    /// A bind group of `layout`, this group's, holding each resource at the
    /// binding it names, so no caller depends on the bindings' order.
    pub fn bind_group<'a>(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: impl IntoIterator<Item = (&'static str, wgpu::BindingResource<'a>)>,
    ) -> wgpu::BindGroup {
        let entries: Vec<wgpu::BindGroupEntry> = resources
            .into_iter()
            .map(|(name, resource)| {
                let Some(index) = self
                    .bindings
                    .iter()
                    .position(|binding| binding.name == name)
                else {
                    panic!("{} has no binding `{name}`", self.label);
                };
                wgpu::BindGroupEntry {
                    binding: index as u32,
                    resource,
                }
            })
            .collect();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(self.label),
            layout,
            entries: &entries,
        })
    }

    /// The WGSL that declares each binding, one per line.
    pub fn declarations(&self) -> String {
        (0..)
            .zip(self.bindings)
            .map(|(binding_index, binding)| {
                let address_space = match binding.binding_type {
                    wgpu::BindingType::Buffer {
                        ty: buffer_type, ..
                    } => match buffer_type {
                        wgpu::BufferBindingType::Uniform => "<uniform>",
                        wgpu::BufferBindingType::Storage { read_only: true } => "<storage, read>",
                        wgpu::BufferBindingType::Storage { read_only: false } => {
                            "<storage, read_write>"
                        }
                    },
                    _ => "",
                };
                let wgsl_type = match binding.count {
                    Some(_) => format!("binding_array<{}>", binding.wgsl_type),
                    None => binding.wgsl_type.to_string(),
                };
                format!(
                    "@group({}) @binding({binding_index}) var{address_space} {}: {wgsl_type};\n",
                    self.index, binding.name
                )
            })
            .collect()
    }
}

/// A pipeline layout holding each of `groups` at its definition's index,
/// with any index below the highest left empty.
pub fn pipeline_layout(
    device: &wgpu::Device,
    label: &str,
    groups: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
) -> wgpu::PipelineLayout {
    let mut layouts: Vec<Option<&wgpu::BindGroupLayout>> = Vec::new();
    for &(definition, layout) in groups {
        let index = definition.index as usize;
        layouts.resize(layouts.len().max(index + 1), None);
        layouts[index] = Some(layout);
    }
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &layouts,
        immediate_size: 0,
    })
}

/// A uniform buffer, which the CPU writes.
pub(crate) const fn uniform(
    name: &'static str,
    wgsl_type: &'static str,
    stages: wgpu::ShaderStages,
) -> Binding {
    buffer(name, wgsl_type, wgpu::BufferBindingType::Uniform, stages)
}

/// A storage buffer the shaders only read, which the CPU writes.
pub(crate) const fn read_only_storage(
    name: &'static str,
    wgsl_type: &'static str,
    stages: wgpu::ShaderStages,
) -> Binding {
    let buffer_type = wgpu::BufferBindingType::Storage { read_only: true };
    buffer(name, wgsl_type, buffer_type, stages)
}

/// A storage buffer the compute stage reads and writes, which only the GPU
/// writes.
const fn storage(name: &'static str, wgsl_type: &'static str) -> Binding {
    let buffer_type = wgpu::BufferBindingType::Storage { read_only: false };
    Binding {
        writer: Writer::Gpu,
        ..buffer(name, wgsl_type, buffer_type, wgpu::ShaderStages::COMPUTE)
    }
}

const fn buffer(
    name: &'static str,
    wgsl_type: &'static str,
    buffer_type: wgpu::BufferBindingType,
    stages: wgpu::ShaderStages,
) -> Binding {
    Binding {
        name,
        wgsl_type,
        binding_type: wgpu::BindingType::Buffer {
            ty: buffer_type,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        stages,
        count: None,
        writer: Writer::Cpu,
    }
}

/// A texture `stages` load texels from, which only the GPU writes.
const fn loaded_texture(
    name: &'static str,
    wgsl_type: &'static str,
    sample_type: wgpu::TextureSampleType,
    multisampled: bool,
    stages: wgpu::ShaderStages,
) -> Binding {
    Binding {
        name,
        wgsl_type,
        binding_type: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled,
        },
        stages,
        count: None,
        writer: Writer::Gpu,
    }
}

/// One Channel's Textures, which the fragment shader samples.
const fn channel(name: &'static str) -> Binding {
    Binding {
        name,
        wgsl_type: "texture_2d<f32>",
        binding_type: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        stages: wgpu::ShaderStages::FRAGMENT,
        count: NonZeroU32::new(TEXTURES_PER_CHANNEL),
        writer: Writer::Cpu,
    }
}
