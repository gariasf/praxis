//! This module has been moved to `utilities::light_probe`.
//!
//! Please update your imports to use:
//! ```
//! use praxis_graphics::utilities::light_probe;
//! // or
//! use praxis_graphics::{LightProbe, LightProbeManager, LightProbeGrid};
//! ```

// Re-export everything from the new location for backwards compatibility
pub use crate::utilities::light_probe::*;
