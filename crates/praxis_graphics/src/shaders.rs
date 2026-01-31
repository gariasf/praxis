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

/// Compiled vertex shader for forward PBR rendering.
pub mod forward_pbr_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/forward_pbr.vert"
    }
}

/// Compiled fragment shader for forward PBR rendering.
pub mod forward_pbr_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/forward_pbr.frag"
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

/// Compiled fragment shader for advanced materials with parallax and extended PBR.
pub mod advanced_material_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/advanced_material.frag"
    }
}

/// Compiled vertex shader for material layer blending.
pub mod material_layer_blend_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/material_layer_blend.vert"
    }
}

/// Compiled fragment shader for material layer blending.
pub mod material_layer_blend_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/material_layer_blend.frag"
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

/// Compiled vertex shader for SSAO pass.
pub mod ssao_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/ssao.vert"
    }
}

/// Compiled fragment shader for SSAO pass.
pub mod ssao_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/ssao.frag"
    }
}

/// Compiled vertex shader for SSAO blur pass.
pub mod ssao_blur_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/ssao_blur.vert"
    }
}

/// Compiled fragment shader for SSAO blur pass.
pub mod ssao_blur_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/ssao_blur.frag"
    }
}

/// Compiled fragment shader for HDR tone mapping with multiple operators.
pub mod hdr_tone_map_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/hdr_tone_map.frag"
    }
}

/// Compiled fragment shader for depth-of-field effect with bokeh blur.
pub mod post_process_dof_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_dof.frag"
    }
}

/// Compiled fragment shader for motion blur using velocity buffer.
pub mod post_process_motion_blur_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_motion_blur.frag"
    }
}

/// Compiled fragment shader for chromatic aberration lens distortion.
pub mod post_process_chromatic_aberration_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_chromatic_aberration.frag"
    }
}

/// Compiled fragment shader for vignette effect.
pub mod post_process_vignette_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_vignette.frag"
    }
}

/// Compiled fragment shader for film grain noise.
pub mod post_process_film_grain_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process_film_grain.frag"
    }
}

/// Compiled vertex shader for velocity buffer generation.
pub mod velocity_buffer_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/velocity_buffer.vert"
    }
}

/// Compiled fragment shader for velocity buffer generation.
pub mod velocity_buffer_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/velocity_buffer.frag"
    }
}

/// Compiled vertex shader for line rendering.
pub mod line_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/line.vert"
    }
}

/// Compiled fragment shader for line rendering.
pub mod line_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/line.frag"
    }
}

/// Compiled vertex shader for equirectangular to cubemap conversion.
pub mod equirect_to_cube_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/equirect_to_cube.vert"
    }
}

/// Compiled fragment shader for equirectangular to cubemap conversion.
pub mod equirect_to_cube_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/equirect_to_cube.frag"
    }
}

/// Compiled vertex shader for TAA (Temporal Anti-Aliasing).
pub mod taa_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/taa.vert"
    }
}

/// Compiled fragment shader for TAA (Temporal Anti-Aliasing).
pub mod taa_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/taa.frag"
    }
}

/// Compiled compute shader for GPU-driven culling.
pub mod gpu_culling_comp {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/gpu_culling.comp"
    }
}

/// Compiled compute shader for GPU-driven LOD selection.
pub mod lod_selection_comp {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/lod_selection.comp"
    }
}

/// Compiled compute shader for Hi-Z pyramid generation.
pub mod hiz_generate_comp {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/hiz_generate.comp"
    }
}

/// Compiled vertex shader for SSR ray marching pass.
pub mod ssr_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/ssr.vert"
    }
}

/// Compiled fragment shader for SSR ray marching pass.
pub mod ssr_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/ssr.frag"
    }
}

/// Compiled vertex shader for SSR blur pass.
pub mod ssr_blur_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/ssr_blur.vert"
    }
}

/// Compiled fragment shader for SSR blur pass.
pub mod ssr_blur_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/ssr_blur.frag"
    }
}

/// Compiled vertex shader for SSR composite pass.
pub mod ssr_composite_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/ssr_composite.vert"
    }
}

/// Compiled fragment shader for SSR composite pass.
pub mod ssr_composite_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/ssr_composite.frag"
    }
}

use std::sync::Arc;
use vulkano::device::Device;

/// Loads the GPU culling compute shader.
pub fn load_gpu_culling_comp(
    device: Arc<Device>,
) -> Result<Arc<vulkano::shader::ShaderModule>, vulkano::Validated<vulkano::VulkanError>> {
    gpu_culling_comp::load(device)
}

/// Loads the LOD selection compute shader.
pub fn load_lod_selection_comp(
    device: Arc<Device>,
) -> Result<Arc<vulkano::shader::ShaderModule>, vulkano::Validated<vulkano::VulkanError>> {
    lod_selection_comp::load(device)
}

/// Loads the Hi-Z pyramid generation compute shader.
pub fn load_hiz_generate_comp(
    device: Arc<Device>,
) -> Result<Arc<vulkano::shader::ShaderModule>, vulkano::Validated<vulkano::VulkanError>> {
    hiz_generate_comp::load(device)
}
