pub mod depth;
pub mod instance;
pub mod prepare;
pub mod renderer;
pub mod shaders;
pub mod staging;
pub mod uniforms;

#[cfg(test)]
mod shader_function_tests;
#[cfg(test)]
mod shader_tests;

pub use depth::{DEPTH_FORMAT, create_depth_texture};
pub use instance::{INSTANCE_CAPACITY, InstanceData};
pub use prepare::{FrameData, FramePreparer};
pub(crate) use renderer::MAIN_PASS;
pub use renderer::Renderer;
pub use staging::{FrameStaging, LoadStaging};
pub use uniforms::{CameraUniform, LightUniform};
