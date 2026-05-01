pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MaterialHandle(pub u32);
