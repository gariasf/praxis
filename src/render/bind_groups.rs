//! Bind groups, each defined once: its index and, per binding, its WGSL name
//! and type and its layout entry. A definition builds its layout and bind
//! groups, gives the index pipeline layouts and passes use, and declares its
//! bindings in every shader module assembled with it, so no shader part
//! declares one. Groups are packed by how often they change, within the
//! Capability floor's four (ADR 0021): 0 the frame's, 1 the Materials', 2 the
//! scene geometry's, 3 a pass's own.

use std::num::NonZeroU32;

use crate::assets::material::TEXTURES_PER_CHANNEL;

/// Group 0, written every frame: the camera and the lights.
pub const FRAME: BindGroupDefinition = BindGroupDefinition {
    index: 0,
    label: "Frame Bind Group",
    bindings: &[
        uniform(
            "camera",
            "CameraUniform",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            stages: wgpu::ShaderStages::FRAGMENT,
            count: None,
        },
    ],
};

/// Group 2, the scene geometry, built once since its buffers never grow: the
/// Instances, and the vertex Arena the vertex shader pulls from (ADR 0019).
pub const SCENE: BindGroupDefinition = BindGroupDefinition {
    index: 2,
    label: "Scene Bind Group",
    bindings: &[
        read_only_storage(
            "instances",
            "array<InstanceData>",
            wgpu::ShaderStages::VERTEX,
        ),
        read_only_storage(
            "vertices",
            "array<CookedVertex>",
            wgpu::ShaderStages::VERTEX,
        ),
    ],
};

/// A bind group: its index and its bindings, each numbered by its place.
pub struct BindGroupDefinition {
    /// Where pipeline layouts hold the group and passes set it.
    pub index: u32,
    pub label: &'static str,
    pub bindings: &'static [Binding],
}

/// One binding: its WGSL variable and its layout entry.
pub struct Binding {
    pub name: &'static str,
    /// The variable's WGSL type; with a `count`, its elements'.
    pub wgsl_type: &'static str,
    pub ty: wgpu::BindingType,
    pub stages: wgpu::ShaderStages,
    /// The length of its binding array; `None` for one resource.
    pub count: Option<NonZeroU32>,
}

impl BindGroupDefinition {
    pub fn layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..)
            .zip(self.bindings)
            .map(|(binding, definition)| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: definition.stages,
                ty: definition.ty,
                count: definition.count,
            })
            .collect();
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(self.label),
            entries: &entries,
        })
    }

    /// A bind group of `layout`, this group's, holding `resources` in
    /// binding order.
    pub fn bind_group<'a>(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: impl IntoIterator<Item = wgpu::BindingResource<'a>>,
    ) -> wgpu::BindGroup {
        let entries: Vec<wgpu::BindGroupEntry> = (0..)
            .zip(resources)
            .map(|(binding, resource)| wgpu::BindGroupEntry { binding, resource })
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
                let address_space = match binding.ty {
                    wgpu::BindingType::Buffer { ty, .. } => match ty {
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

/// A uniform buffer.
const fn uniform(
    name: &'static str,
    wgsl_type: &'static str,
    stages: wgpu::ShaderStages,
) -> Binding {
    buffer(name, wgsl_type, wgpu::BufferBindingType::Uniform, stages)
}

/// A storage buffer the shaders only read.
const fn read_only_storage(
    name: &'static str,
    wgsl_type: &'static str,
    stages: wgpu::ShaderStages,
) -> Binding {
    let ty = wgpu::BufferBindingType::Storage { read_only: true };
    buffer(name, wgsl_type, ty, stages)
}

const fn buffer(
    name: &'static str,
    wgsl_type: &'static str,
    ty: wgpu::BufferBindingType,
    stages: wgpu::ShaderStages,
) -> Binding {
    Binding {
        name,
        wgsl_type,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        stages,
        count: None,
    }
}

/// One Channel's Textures, which the fragment shader samples.
const fn channel(name: &'static str) -> Binding {
    Binding {
        name,
        wgsl_type: "texture_2d<f32>",
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        stages: wgpu::ShaderStages::FRAGMENT,
        count: NonZeroU32::new(TEXTURES_PER_CHANNEL),
    }
}
