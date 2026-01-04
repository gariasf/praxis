//! Post-processing system for screen-space effects.
//!
//! This module provides a flexible framework for implementing post-processing effects
//! that operate on rendered images. Post-processing effects are applied after the main
//! 3D scene rendering and before presenting to the screen.
//!
//! # Architecture
//!
//! The post-processing system consists of several key components:
//!
//! - **`PostProcessPass`**: Trait defining a single post-processing effect
//! - **`RenderTarget`**: Offscreen framebuffer for render-to-texture operations
//! - **`RenderTargetPool`**: Manages reusable render targets to reduce allocations
//! - **`FullScreenQuad`**: Renders a full-screen textured quad for applying effects
//! - **`PostProcessChain`**: Chains multiple post-processing passes together
//!
//! # Render-to-Texture Flow
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │   Main      │────▶│    Pass 1   │────▶│    Pass 2   │────▶ Swapchain
//! │   Render    │     │  (Texture)  │     │  (Texture)  │
//! └─────────────┘     └─────────────┘     └─────────────┘
//!    Render to          Apply effect        Apply effect
//!    texture A          A → B               B → screen
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{PostProcessChain, RenderTargetPool, FullScreenQuad};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create a render target pool for texture reuse
//! // let mut pool = RenderTargetPool::new(...);
//!
//! // Create a post-processing chain
//! // let mut chain = PostProcessChain::new();
//!
//! // Add post-processing passes
//! // chain.add_pass(Box::new(GrayscalePass::new(...)?));
//! // chain.add_pass(Box::new(BlurPass::new(...)?));
//!
//! // In render loop:
//! // 1. Render scene to texture
//! // let scene_texture = pool.acquire(...)?;
//! // render_scene_to_texture(scene_texture);
//!
//! // 2. Apply post-processing chain
//! // let final_texture = chain.process(scene_texture, &mut pool)?;
//!
//! // 3. Blit final texture to swapchain
//! // blit_to_screen(final_texture);
//! # Ok(())
//! # }
//! ```

pub mod bloom;
mod chain;
pub mod cinematic;
mod full_screen_quad;
mod pass;
pub mod passes;
mod render_target;
#[cfg(test)]
mod tests;

pub use bloom::{
    BloomConfig, BloomEffect, BrightnessExtractionPass, GaussianBlurHorizontalPass,
    GaussianBlurVerticalPass, ToneMapPass,
};
pub use chain::PostProcessChain;
pub use cinematic::{
    ChromaticAberrationConfig, ChromaticAberrationPass, DepthOfFieldPass, DofConfig,
    FilmGrainConfig, FilmGrainPass, MotionBlurConfig, MotionBlurPass, VelocityUniforms,
    VignetteConfig, VignettePass,
};
pub use full_screen_quad::{FullScreenQuad, QuadVertex};
pub use pass::{PostProcessContext, PostProcessPass};
pub use passes::{CopyPass, GrayscalePass};
pub use render_target::{RenderTarget, RenderTargetPool};
