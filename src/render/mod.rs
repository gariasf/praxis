pub mod bind_groups;
pub mod culling_pass;
pub mod instance;
pub mod main_pass;
pub mod prepare;
pub mod renderer;
pub mod shaders;
pub mod staging;
pub mod targets;
pub mod uniforms;

#[cfg(test)]
mod shader_function_tests;
#[cfg(test)]
mod shader_tests;

pub use instance::{INSTANCE_CAPACITY, InstanceData, NO_MATERIAL, NO_MESH};
pub(crate) use main_pass::MAIN_PASS;
pub use prepare::{FrameData, FramePreparer};
pub use renderer::{RenderSwitches, Renderer};
pub use staging::{FrameStaging, LoadStaging};
pub use targets::DEPTH_FORMAT;
pub use uniforms::{CameraUniform, LightUniform};
