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

/// Compiled fragment shader for brightness extraction (bloom).
pub mod post_process_brightness_extract_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_brightness_extract.frag"
    }
}

/// Compiled fragment shader for horizontal Gaussian blur (bloom).
pub mod post_process_gaussian_blur_h_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_gaussian_blur_h.frag"
    }
}

/// Compiled fragment shader for vertical Gaussian blur (bloom).
pub mod post_process_gaussian_blur_v_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_gaussian_blur_v.frag"
    }
}

/// Compiled fragment shader for tone mapping with bloom.
pub mod post_process_tone_map_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_tone_map.frag"
    }
}

/// Compiled vertex shader for skybox rendering.
pub mod skybox_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/skybox.vert"
    }
}

/// Compiled fragment shader for skybox rendering.
pub mod skybox_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/skybox.frag"
    }
}

/// Compiled vertex shader for deferred rendering geometry pass.
pub mod deferred_geometry_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/deferred_geometry.vert"
    }
}

/// Compiled fragment shader for deferred rendering geometry pass.
pub mod deferred_geometry_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/deferred_geometry.frag"
    }
}

/// Compiled vertex shader for deferred rendering lighting pass.
pub mod deferred_lighting_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/deferred_lighting.vert"
    }
}

/// Compiled fragment shader for deferred rendering lighting pass.
pub mod deferred_lighting_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/deferred_lighting.frag"
    }
}
