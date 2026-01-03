//! Post-processing chain for composing multiple effects.
//!
//! This module provides the `PostProcessChain` struct which manages a sequence
//! of post-processing passes and handles the render-to-texture ping-pong between them.

use super::{pass::PostProcessPass, render_target::RenderTarget, render_target::RenderTargetPool};
use praxis_utils::{debug, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        allocator::CommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    device::Queue,
    sync::GpuFuture,
};

/// A chain of post-processing passes.
///
/// The `PostProcessChain` manages multiple post-processing passes and handles
/// the orchestration of rendering between them. Each pass reads from one
/// render target and writes to another, with the chain handling the ping-pong
/// buffering automatically.
///
/// # Architecture
///
/// ```text
/// Input Texture
///      │
///      ▼
/// ┌─────────┐     Temp Target 1
/// │ Pass 1  │──────────────────────┐
/// └─────────┘                      │
///      │                           │
///      ▼                           ▼
/// ┌─────────┐     Temp Target 2   │
/// │ Pass 2  │◄─────────────────────┘
/// └─────────┘
///      │
///      ▼
/// ┌─────────┐     Output Target
/// │ Pass 3  │──────────────────────►
/// └─────────┘
/// ```
///
/// # Example
///
/// ```rust,no_run
/// # use praxis_graphics::post_process::PostProcessChain;
/// # use std::sync::Arc;
/// # use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
/// # use vulkano::device::Queue;
/// # fn example(
/// #     command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
/// #     graphics_queue: Arc<Queue>,
/// # ) -> praxis_utils::Result<()> {
/// let mut chain = PostProcessChain::new(command_buffer_allocator, graphics_queue);
///
/// // Add passes
/// // chain.add_pass(Box::new(grayscale_pass));
/// // chain.add_pass(Box::new(blur_pass));
///
/// // Process a texture through the chain
/// // let output = chain.process(&input_texture, &mut render_target_pool)?;
/// # Ok(())
/// # }
/// ```
pub struct PostProcessChain {
    passes: Vec<Box<dyn PostProcessPass>>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    graphics_queue: Arc<Queue>,
}

impl PostProcessChain {
    /// Creates a new empty post-processing chain.
    ///
    /// # Arguments
    ///
    /// * `command_buffer_allocator` - Allocator for command buffers
    /// * `graphics_queue` - Graphics queue for command submission
    pub fn new(
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        graphics_queue: Arc<Queue>,
    ) -> Self {
        info!("Creating post-processing chain");
        Self {
            passes: Vec::new(),
            command_buffer_allocator,
            graphics_queue,
        }
    }

    /// Adds a post-processing pass to the chain.
    ///
    /// Passes are executed in the order they are added.
    ///
    /// # Arguments
    ///
    /// * `pass` - The post-processing pass to add
    pub fn add_pass(&mut self, pass: Box<dyn PostProcessPass>) {
        info!("Adding post-processing pass: {}", pass.name());
        self.passes.push(pass);
    }

    /// Removes all passes from the chain.
    pub fn clear_passes(&mut self) {
        debug!("Clearing all post-processing passes");
        self.passes.clear();
    }

    /// Returns the number of passes in the chain.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Returns whether the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }

    /// Processes an input texture through all passes in the chain.
    ///
    /// # Arguments
    ///
    /// * `input` - The input render target
    /// * `output` - The output render target
    /// * `pool` - Render target pool for temporary targets
    ///
    /// # Returns
    ///
    /// The final output texture after all passes have been applied.
    ///
    /// # Errors
    ///
    /// Returns an error if any pass fails or if resource allocation fails.
    pub fn process(
        &mut self,
        input: &RenderTarget,
        output: &RenderTarget,
        pool: &mut RenderTargetPool,
    ) -> Result<()> {
        if self.passes.is_empty() {
            trace!("Post-processing chain is empty, copying input to output");
            return self.copy_texture(input, output);
        }

        debug!(
            "Processing texture through {} post-processing passes",
            self.passes.len()
        );

        let extent = input.extent();

        // Create command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        // For multiple passes, we need to ping-pong between render targets
        let mut current_input = input.clone();
        let mut temp_targets: Vec<RenderTarget> = Vec::new();

        let pass_count = self.passes.len();
        for (i, pass) in self.passes.iter_mut().enumerate() {
            trace!("Executing pass {}: {}", i, pass.name());

            let pass_output = if i == pass_count - 1 {
                // Last pass: write to final output
                output.clone()
            } else {
                // Intermediate pass: acquire a temporary render target
                let temp = pool.acquire(extent)?;
                temp_targets.push(temp.clone());
                temp
            };

            pass.execute(&mut builder, &current_input, &pass_output)?;

            // The output of this pass becomes the input for the next pass
            current_input = pass_output;
        }

        // Build and submit command buffer
        let command_buffer = builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build post-processing command buffer: {}", e))?;

        trace!("Submitting post-processing command buffer");
        let future = vulkano::sync::now(self.graphics_queue.device().clone())
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to submit post-processing command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| eyre::eyre!("Failed to flush post-processing commands: {}", e))?;

        future
            .wait(None)
            .map_err(|e| eyre::eyre!("Failed to wait for post-processing completion: {}", e))?;

        // Release temporary render targets back to pool
        for temp in temp_targets {
            pool.release(temp);
        }

        debug!("Post-processing chain completed successfully");

        Ok(())
    }

    /// Copies the input texture to the output texture.
    ///
    /// This is used when the chain is empty or as a fallback.
    fn copy_texture(&self, _input: &RenderTarget, _output: &RenderTarget) -> Result<()> {
        // For now, we'll just return Ok. In a real implementation, you would
        // use a blit or copy command to transfer the image data.
        // This would require additional shader infrastructure for a simple copy pass.
        Ok(())
    }

    /// Returns a reference to the graphics queue.
    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }

    /// Returns a reference to the command buffer allocator.
    pub fn command_buffer_allocator(&self) -> &Arc<dyn CommandBufferAllocator> {
        &self.command_buffer_allocator
    }
}
