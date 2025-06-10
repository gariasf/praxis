//! Shader compilation and management for the graphics system.
//!
//! This module uses vulkano-shaders to compile GLSL shaders to SPIR-V at build time.

/// Compiled vertex shader for basic triangle rendering.
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/triangle.vert"
    }
}

/// Compiled fragment shader for basic triangle rendering.
pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/triangle.frag"
    }
}
