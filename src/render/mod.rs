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

#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod shader_function_tests;
#[cfg(test)]
mod shader_tests;

pub use acceleration_structures::{
    DYNAMIC_TLAS_CAPACITY, DynamicTlas, StaticTlas, blas_bytes, compacted_blas_fraction, tlas_bytes,
};
pub use instance::{InstanceData, NO_MATERIAL, NO_MESH};
pub use prepare::{FrameData, FramePreparer};
pub(crate) use renderer::{BLAS_REFIT, DYNAMIC_TLAS_RECORDING, DYNAMIC_TLAS_SUBMIT, TIMED_PASSES};
pub use renderer::{CrowdForm, RenderCapacities, RenderSwitches, Renderer};
pub use staging::{FrameStaging, LoadStaging, mips_written_bytes, texture_written_bytes};
pub use targets::{HDR_FORMAT, MULTISAMPLED_FORMATS};
pub use uniforms::{CameraUniform, LightData, LightUniform};

/// Frames submitted but not finished on the GPU when the next one starts, at
/// most: the window's swapchain latency and capture's queue. Per-frame
/// staging and the GPU stats readback are sized by it.
pub const FRAMES_IN_FLIGHT: usize = 2;
