//! HDR (High Dynamic Range) rendering system.
//!
//! This module provides a complete HDR rendering pipeline with:
//! - Floating-point render targets for HDR scene rendering
//! - Automatic and manual exposure calculation
//! - Multiple tone mapping operators (ACES, Reinhard, Uncharted 2)
//! - Luminance histogram for automatic exposure
//!
//! # Architecture
//!
//! The HDR system works in several stages:
//!
//! 1. **HDR Scene Rendering**: Scene is rendered to floating-point render target (R16G16B16A16_SFLOAT)
//! 2. **Luminance Calculation**: Average scene luminance is calculated for automatic exposure
//! 3. **Tone Mapping**: HDR values are mapped to LDR [0,1] range using selected operator
//! 4. **Gamma Correction**: Final gamma correction is applied
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::hdr::{HdrRenderTarget, ToneMapper, ToneMappingOperator, ExposureMode};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create HDR render target
//! // let hdr_target = HdrRenderTarget::new(...)?;
//!
//! // Create tone mapper with ACES operator
//! // let mut tone_mapper = ToneMapper::new(..., ToneMappingOperator::ACES)?;
//!
//! // Set exposure mode
//! // tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });
//!
//! // In render loop:
//! // 1. Render scene to HDR target
//! // render_scene_to_hdr(&hdr_target);
//!
//! // 2. Apply tone mapping
//! // tone_mapper.apply(builder, &hdr_target, &output_target, delta_time)?;
//! # Ok(())
//! # }
//! ```

mod exposure;
mod render_target;
mod tone_mapper;

pub use exposure::{calculate_luminance, ExposureCalculator, ExposureMode};
pub use render_target::HdrRenderTarget;
pub use tone_mapper::{ToneMapPass, ToneMapper, ToneMappingOperator};
