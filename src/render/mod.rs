pub mod acceleration_structures;
pub mod bend_pass;
pub mod bind_groups;
pub mod blas_refit;
pub mod culling_pass;
pub mod depth_pyramid_pass;
pub mod froxel_pass;
pub mod instance;
pub mod light_count_pass;
pub mod main_pass;
pub mod prepare;
pub mod readback;
pub mod renderer;
pub mod shaders;
pub mod shadow_mask_pass;
pub mod staging;
pub mod targets;
pub mod tonemapping_pass;
pub mod uniforms;
pub mod visibility_pass;

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
pub(crate) use renderer::{BLAS_REFIT, DYNAMIC_TLAS_RECORDING, DYNAMIC_TLAS_SUBMIT, TIMED_PASSES};
pub use renderer::{CrowdForm, RenderSwitches, Renderer, VisibilityPrepass};
pub use staging::{FrameStaging, LoadStaging};
pub use targets::{DEPTH_FORMAT, HDR_FORMAT};
pub use uniforms::{CameraUniform, LIGHT_CAPACITY, LightData, LightUniform};

/// Frames submitted but not finished on the GPU when the next one starts, at
/// most: the window's swapchain latency and capture's queue. Per-frame
/// staging and the GPU stats readback are sized by it.
pub const FRAMES_IN_FLIGHT: usize = 2;
