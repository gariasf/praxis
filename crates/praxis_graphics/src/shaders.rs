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

/// Compiled vertex shader for shadow map generation.
pub mod shadow_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/shadow.vert"
    }
}

/// Compiled fragment shader for shadow map generation.
pub mod shadow_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/shadow.frag"
    }
}

/// Compiled vertex shader for post-processing full-screen quad.
pub mod post_process_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/post_process.vert"
    }
}

/// Compiled fragment shader for post-processing copy/passthrough.
pub mod post_process_copy_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_copy.frag"
    }
}

/// Compiled fragment shader for post-processing grayscale effect.
pub mod post_process_grayscale_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_grayscale.frag"
    }
}

/// Compiled fragment shader for post-processing blur effect.
pub mod post_process_blur_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_blur.frag"
    }
}
