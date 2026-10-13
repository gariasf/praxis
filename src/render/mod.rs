pub mod depth;
pub mod instance;
pub mod prepare;
pub mod staging;
pub mod uniforms;
pub mod vertex;

#[cfg(test)]
mod shader_function_tests;
#[cfg(test)]
mod shader_tests;

pub use depth::{DEPTH_FORMAT, create_depth_texture};
pub use instance::{INSTANCE_BUFFER_INITIAL_CAPACITY, InstanceData};
pub use prepare::prepare_renderables;
pub use staging::FrameStaging;
pub use uniforms::{CameraUniform, LightUniform};
pub use vertex::Vertex;
