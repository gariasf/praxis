//! Post-processing pass trait and common implementations.
//!
//! This module defines the `PostProcessPass` trait that all post-processing
//! effects must implement. It also provides the infrastructure for creating
//! custom post-processing effects.

use super::render_target::RenderTarget;
use praxis_utils::Result;
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    device::Queue,
};

/// Trait for post-processing passes.
///
/// A post-processing pass takes an input texture, applies an effect,
/// and writes the result to an output texture.
///
/// # Implementation
///
/// Implementors must define:
/// - How to execute the pass given an input and output
/// - The name of the pass (for debugging)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_graphics::post_process::{PostProcessPass, RenderTarget};
/// use praxis_utils::Result;
/// use std::sync::Arc;
/// use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
///
/// struct GrayscalePass {
///     // Pass-specific state (pipeline, descriptor sets, etc.)
/// }
///
/// impl PostProcessPass for GrayscalePass {
///     fn execute(
///         &mut self,
///         builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
///         input: &RenderTarget,
///         output: &RenderTarget,
///     ) -> Result<()> {
///         // Record commands to apply grayscale effect
///         // - Begin render pass with output framebuffer
///         // - Bind pipeline
///         // - Bind descriptor set with input texture
///         // - Draw full-screen quad
///         // - End render pass
///         Ok(())
///     }
///
///     fn name(&self) -> &str {
///         "Grayscale"
///     }
/// }
/// ```
pub trait PostProcessPass: Send + Sync {
    /// Executes the post-processing pass.
    ///
    /// This method should record all necessary commands to the command buffer
    /// to apply the effect. The input texture should be sampled, the effect
    /// applied, and the result written to the output texture.
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder for recording commands
    /// * `input` - Input render target (source texture)
    /// * `output` - Output render target (destination texture)
    ///
    /// # Errors
    ///
    /// Returns an error if command recording fails.
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()>;

    /// Returns the name of this pass.
    ///
    /// Used for debugging and profiling.
    fn name(&self) -> &str;

    /// Returns whether this pass requires the input texture to be in a specific state.
    ///
    /// By default, returns false. Override if your pass has specific requirements.
    fn requires_depth(&self) -> bool {
        false
    }

    /// Returns whether this pass modifies the alpha channel.
    ///
    /// By default, returns false. Override if your pass modifies alpha.
    fn modifies_alpha(&self) -> bool {
        false
    }
}

/// Context for post-processing operations.
///
/// This struct provides the necessary Vulkan resources for executing
/// post-processing passes.
pub struct PostProcessContext {
    /// Graphics queue for command submission.
    pub graphics_queue: Arc<Queue>,
}

impl PostProcessContext {
    /// Creates a new post-processing context.
    ///
    /// # Arguments
    ///
    /// * `graphics_queue` - The graphics queue for command submission
    pub fn new(graphics_queue: Arc<Queue>) -> Self {
        Self { graphics_queue }
    }
}
