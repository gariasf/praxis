pub struct Texture {
    pub view: wgpu::TextureView,
    pub texture: wgpu::Texture,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TextureHandle(pub u32);
