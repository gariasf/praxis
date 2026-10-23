pub mod acceleration_structures;
pub mod bind_groups;
pub mod culling_pass;
pub mod depth_pyramid_pass;
pub mod froxel_pass;
pub mod instance;
pub mod main_pass;
pub mod prepare;
pub mod readback;
pub mod renderer;
pub mod shaders;
pub mod staging;
pub mod targets;
pub mod tonemapping_pass;
pub mod uniforms;

#[cfg(test)]
mod shader_function_tests;
#[cfg(test)]
mod shader_tests;

pub use acceleration_structures::{
    AccelerationStructureBytes, DYNAMIC_TLAS_CAPACITY, DynamicTlas, STATIC_TLAS_CAPACITY,
    StaticTlas,
};
pub use instance::{INSTANCE_CAPACITY, InstanceData, NO_MATERIAL, NO_MESH};
pub use prepare::{FrameData, FramePreparer};
pub(crate) use renderer::TIMED_PASSES;
pub use renderer::{RenderSwitches, Renderer};
pub use staging::{FrameStaging, LoadStaging};
pub use targets::{DEPTH_FORMAT, HDR_FORMAT};
pub use uniforms::{CameraUniform, LIGHT_CAPACITY, LightData, LightUniform};

/// Frames submitted but not finished on the GPU when the next one starts, at
/// most: the window's swapchain latency and capture's queue. Per-frame
/// staging and the GPU stats readback are sized by it.
pub const FRAMES_IN_FLIGHT: usize = 2;
