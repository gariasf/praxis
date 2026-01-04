//! GPU compute shader-based texture generation.
//!
//! This module provides high-performance texture generation using GPU compute shaders.
//! The generator evaluates texture graphs and generates texture data directly on the GPU.

use crate::graph::{BlendMode, NoiseType, TextureGraph, TextureNode, TextureNodeId};
use praxis_utils::{eyre, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::allocator::CommandBufferAllocator,
    descriptor_set::allocator::DescriptorSetAllocator,
    device::{Device, Queue},
    memory::allocator::MemoryAllocator,
};

/// Parameters for texture generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureGenerationParams {
    /// Width of the texture in pixels
    pub width: u32,
    /// Height of the texture in pixels
    pub height: u32,
    /// Random seed for noise generation
    pub seed: u32,
}

impl Default for TextureGenerationParams {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            seed: 0,
        }
    }
}

/// CPU-based procedural texture generator.
///
/// This generator evaluates texture graphs on the CPU to generate texture data.
pub struct ProceduralTextureGenerator {
    #[allow(dead_code)]
    device: Arc<Device>,
    #[allow(dead_code)]
    queue: Arc<Queue>,
    #[allow(dead_code)]
    memory_allocator: Arc<dyn MemoryAllocator>,
    #[allow(dead_code)]
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    #[allow(dead_code)]
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
}

impl ProceduralTextureGenerator {
    /// Creates a new procedural texture generator.
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        memory_allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    ) -> Self {
        Self {
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    /// Generates a texture from a texture graph.
    ///
    /// The graph is evaluated on the CPU and returns a RGBA8 image.
    pub fn generate(&self, graph: &TextureGraph, params: TextureGenerationParams) -> Result<Vec<u8>> {
        graph.validate().map_err(|e| eyre::eyre!("Invalid texture graph: {}", e))?;

        let output_id = graph.output().ok_or_else(|| eyre::eyre!("No output node"))?;
        
        let mut data = Vec::with_capacity((params.width * params.height * 4) as usize);
        
        for y in 0..params.height {
            for x in 0..params.width {
                let uv_x = x as f32 / params.width as f32;
                let uv_y = y as f32 / params.height as f32;
                
                let color = Self::evaluate_node(graph, output_id, uv_x, uv_y, params.seed)?;
                
                data.push((color[0].clamp(0.0, 1.0) * 255.0) as u8);
                data.push((color[1].clamp(0.0, 1.0) * 255.0) as u8);
                data.push((color[2].clamp(0.0, 1.0) * 255.0) as u8);
                data.push((color[3].clamp(0.0, 1.0) * 255.0) as u8);
            }
        }

        Ok(data)
    }

    fn evaluate_node(
        graph: &TextureGraph,
        node_id: TextureNodeId,
        uv_x: f32,
        uv_y: f32,
        seed: u32,
    ) -> Result<[f32; 4]> {
        use crate::noise::{fbm_noise, perlin_noise, simplex_noise, worley_noise};
        
        let node = graph.get_node(node_id).ok_or_else(|| eyre::eyre!("Node not found"))?;
        
        match node {
            TextureNode::Noise { noise_type, scale, octaves, persistence, lacunarity } => {
                let value = match noise_type {
                    NoiseType::Perlin => {
                        fbm_noise(uv_x * scale, uv_y * scale, seed, *octaves, *persistence, *lacunarity, perlin_noise)
                    }
                    NoiseType::Simplex => {
                        fbm_noise(uv_x * scale, uv_y * scale, seed, *octaves, *persistence, *lacunarity, simplex_noise)
                    }
                    NoiseType::Worley => {
                        fbm_noise(uv_x * scale, uv_y * scale, seed, *octaves, *persistence, *lacunarity, 
                            |x, y, s| worley_noise(x, y, s, 1.0))
                    }
                };
                let normalized = value * 0.5 + 0.5;
                Ok([normalized, normalized, normalized, 1.0])
            }
            TextureNode::Constant { color } => {
                Ok(*color)
            }
            TextureNode::Transform { input, params } => {
                let mut transformed_x = uv_x - 0.5;
                let mut transformed_y = uv_y - 0.5;
                
                let cos_r = params.rotation.cos();
                let sin_r = params.rotation.sin();
                let rotated_x = transformed_x * cos_r - transformed_y * sin_r;
                let rotated_y = transformed_x * sin_r + transformed_y * cos_r;
                
                transformed_x = rotated_x / params.scale.x;
                transformed_y = rotated_y / params.scale.y;
                
                transformed_x += 0.5 + params.offset.x;
                transformed_y += 0.5 + params.offset.y;
                
                Self::evaluate_node(graph, *input, transformed_x, transformed_y, seed)
            }
            TextureNode::Blend { input_a, input_b, mode, factor } => {
                let a = Self::evaluate_node(graph, *input_a, uv_x, uv_y, seed)?;
                let b = Self::evaluate_node(graph, *input_b, uv_x, uv_y, seed)?;
                
                let blended = match mode {
                    BlendMode::Add => [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]],
                    BlendMode::Multiply => [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]],
                    BlendMode::Min => [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]), a[3].min(b[3])],
                    BlendMode::Max => [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2]), a[3].max(b[3])],
                    BlendMode::Mix => [
                        a[0] + (b[0] - a[0]) * factor,
                        a[1] + (b[1] - a[1]) * factor,
                        a[2] + (b[2] - a[2]) * factor,
                        a[3] + (b[3] - a[3]) * factor,
                    ],
                    BlendMode::Screen => [
                        1.0 - (1.0 - a[0]) * (1.0 - b[0]),
                        1.0 - (1.0 - a[1]) * (1.0 - b[1]),
                        1.0 - (1.0 - a[2]) * (1.0 - b[2]),
                        1.0 - (1.0 - a[3]) * (1.0 - b[3]),
                    ],
                    BlendMode::Overlay => {
                        let overlay = |a: f32, b: f32| {
                            if a < 0.5 {
                                2.0 * a * b
                            } else {
                                1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                            }
                        };
                        [overlay(a[0], b[0]), overlay(a[1], b[1]), overlay(a[2], b[2]), overlay(a[3], b[3])]
                    },
                    BlendMode::Subtract => [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]],
                };
                
                Ok(blended)
            }
            TextureNode::ColorRamp { input, ramp } => {
                let value = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                Ok(ramp.evaluate(value[0]))
            }
            TextureNode::Invert { input } => {
                let color = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                Ok([1.0 - color[0], 1.0 - color[1], 1.0 - color[2], color[3]])
            }
            TextureNode::Clamp { input, min, max } => {
                let color = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                Ok([
                    color[0].clamp(*min, *max),
                    color[1].clamp(*min, *max),
                    color[2].clamp(*min, *max),
                    color[3].clamp(*min, *max),
                ])
            }
            TextureNode::Power { input, exponent } => {
                let color = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                Ok([
                    color[0].powf(*exponent),
                    color[1].powf(*exponent),
                    color[2].powf(*exponent),
                    color[3],
                ])
            }
            TextureNode::Threshold { input, threshold } => {
                let color = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                let value = if color[0] >= *threshold { 1.0 } else { 0.0 };
                Ok([value, value, value, color[3]])
            }
            TextureNode::Contrast { input, amount } => {
                let color = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                Ok([
                    ((color[0] - 0.5) * (1.0 + amount) + 0.5),
                    ((color[1] - 0.5) * (1.0 + amount) + 0.5),
                    ((color[2] - 0.5) * (1.0 + amount) + 0.5),
                    color[3],
                ])
            }
            TextureNode::Brightness { input, amount } => {
                let color = Self::evaluate_node(graph, *input, uv_x, uv_y, seed)?;
                Ok([
                    color[0] + amount,
                    color[1] + amount,
                    color[2] + amount,
                    color[3],
                ])
            }
        }
    }
    
    #[allow(dead_code)]
    fn compile_graph_to_shader(&self, graph: &TextureGraph, params: TextureGenerationParams) -> Result<String> {
        let output_id = graph.output().ok_or_else(|| eyre::eyre!("No output node"))?;

        let mut shader = String::new();
        shader.push_str("#version 450\n\n");
        shader.push_str("layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;\n\n");
        shader.push_str("layout(set = 0, binding = 0, rgba8) uniform writeonly image2D outputImage;\n\n");
        
        shader.push_str(&format!("const uint SEED = {}u;\n", params.seed));
        shader.push_str(&format!("const uint WIDTH = {}u;\n", params.width));
        shader.push_str(&format!("const uint HEIGHT = {}u;\n\n", params.height));

        shader.push_str(&self.generate_noise_functions());
        shader.push_str(&self.generate_utility_functions());

        let mut generated_nodes = std::collections::HashSet::new();
        self.generate_node_function(graph, output_id, &mut generated_nodes, &mut shader)?;

        shader.push_str("\nvoid main() {\n");
        shader.push_str("    ivec2 pixel = ivec2(gl_GlobalInvocationID.xy);\n");
        shader.push_str("    if (pixel.x >= WIDTH || pixel.y >= HEIGHT) return;\n\n");
        shader.push_str("    vec2 uv = vec2(pixel) / vec2(WIDTH, HEIGHT);\n");
        shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", output_id.0));
        shader.push_str("    imageStore(outputImage, pixel, color);\n");
        shader.push_str("}\n");

        Ok(shader)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn generate_node_function(
        &self,
        graph: &TextureGraph,
        node_id: TextureNodeId,
        generated: &mut std::collections::HashSet<TextureNodeId>,
        shader: &mut String,
    ) -> Result<()> {
        if generated.contains(&node_id) {
            return Ok(());
        }

        let node = graph.get_node(node_id).ok_or_else(|| eyre::eyre!("Node not found"))?;

        match node {
            TextureNode::Noise { noise_type, scale, octaves, persistence, lacunarity } => {
                shader.push_str(&format!(
                    "vec4 eval_node_{}(vec2 uv) {{\n",
                    node_id.0
                ));
                
                let noise_fn = match noise_type {
                    NoiseType::Perlin => "perlin_noise",
                    NoiseType::Simplex => "simplex_noise",
                    NoiseType::Worley => "worley_noise",
                };

                shader.push_str(&format!(
                    "    float value = fbm_{noise_fn}(uv * {scale}, SEED, {octaves}, {persistence}, {lacunarity});\n"
                ));
                shader.push_str("    value = value * 0.5 + 0.5;\n");
                shader.push_str("    return vec4(value, value, value, 1.0);\n");
                shader.push_str("}\n\n");
            }
            TextureNode::Constant { color } => {
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!(
                    "    return vec4({}, {}, {}, {});\n",
                    color[0], color[1], color[2], color[3]
                ));
                shader.push_str("}\n\n");
            }
            TextureNode::Transform { input, params } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!(
                    "    vec2 transformed = transform_uv(uv, vec2({}, {}), {}, vec2({}, {}));\n",
                    params.offset.x, params.offset.y, params.rotation, params.scale.x, params.scale.y
                ));
                shader.push_str(&format!("    return eval_node_{}(transformed);\n", input.0));
                shader.push_str("}\n\n");
            }
            TextureNode::Blend { input_a, input_b, mode, factor } => {
                self.generate_node_function(graph, *input_a, generated, shader)?;
                self.generate_node_function(graph, *input_b, generated, shader)?;
                
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 a = eval_node_{}(uv);\n", input_a.0));
                shader.push_str(&format!("    vec4 b = eval_node_{}(uv);\n", input_b.0));
                
                let blend_expr = match mode {
                    BlendMode::Add => "a + b",
                    BlendMode::Multiply => "a * b",
                    BlendMode::Min => "min(a, b)",
                    BlendMode::Max => "max(a, b)",
                    BlendMode::Mix => &format!("mix(a, b, {factor})"),
                    BlendMode::Screen => "1.0 - (1.0 - a) * (1.0 - b)",
                    BlendMode::Overlay => "mix(2.0 * a * b, 1.0 - 2.0 * (1.0 - a) * (1.0 - b), step(0.5, a))",
                    BlendMode::Subtract => "a - b",
                };
                
                shader.push_str(&format!("    return {blend_expr};\n"));
                shader.push_str("}\n\n");
            }
            TextureNode::ColorRamp { input, ramp } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    float t = eval_node_{}(uv).r;\n", input.0));
                
                if ramp.stops.len() >= 2 {
                    for i in 0..ramp.stops.len() - 1 {
                        let stop1 = &ramp.stops[i];
                        let stop2 = &ramp.stops[i + 1];
                        
                        let condition = if i == 0 {
                            format!("if (t <= {})", stop2.position)
                        } else {
                            format!("else if (t <= {})", stop2.position)
                        };
                        
                        shader.push_str(&format!("    {condition} {{\n"));
                        shader.push_str(&format!(
                            "        float factor = (t - {}) / ({} - {});\n",
                            stop1.position, stop2.position, stop1.position
                        ));
                        shader.push_str(&format!(
                            "        vec4 c1 = vec4({}, {}, {}, {});\n",
                            stop1.color[0], stop1.color[1], stop1.color[2], stop1.color[3]
                        ));
                        shader.push_str(&format!(
                            "        vec4 c2 = vec4({}, {}, {}, {});\n",
                            stop2.color[0], stop2.color[1], stop2.color[2], stop2.color[3]
                        ));
                        shader.push_str("        return mix(c1, c2, factor);\n");
                        shader.push_str("    }\n");
                    }
                    shader.push_str(&format!(
                        "    return vec4({}, {}, {}, {});\n",
                        ramp.stops.last().unwrap().color[0],
                        ramp.stops.last().unwrap().color[1],
                        ramp.stops.last().unwrap().color[2],
                        ramp.stops.last().unwrap().color[3]
                    ));
                } else {
                    shader.push_str("    return vec4(0.0, 0.0, 0.0, 1.0);\n");
                }
                
                shader.push_str("}\n\n");
            }
            TextureNode::Invert { input } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", input.0));
                shader.push_str("    return vec4(1.0 - color.rgb, color.a);\n");
                shader.push_str("}\n\n");
            }
            TextureNode::Clamp { input, min, max } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", input.0));
                shader.push_str(&format!("    return clamp(color, {min}, {max});\n"));
                shader.push_str("}\n\n");
            }
            TextureNode::Power { input, exponent } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", input.0));
                shader.push_str(&format!("    return vec4(pow(color.rgb, vec3({exponent})), color.a);\n"));
                shader.push_str("}\n\n");
            }
            TextureNode::Threshold { input, threshold } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", input.0));
                shader.push_str(&format!("    float value = step({threshold}, color.r);\n"));
                shader.push_str("    return vec4(value, value, value, color.a);\n");
                shader.push_str("}\n\n");
            }
            TextureNode::Contrast { input, amount } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", input.0));
                shader.push_str(&format!("    vec3 adjusted = (color.rgb - 0.5) * (1.0 + {amount}) + 0.5;\n"));
                shader.push_str("    return vec4(adjusted, color.a);\n");
                shader.push_str("}\n\n");
            }
            TextureNode::Brightness { input, amount } => {
                self.generate_node_function(graph, *input, generated, shader)?;
                shader.push_str(&format!("vec4 eval_node_{}(vec2 uv) {{\n", node_id.0));
                shader.push_str(&format!("    vec4 color = eval_node_{}(uv);\n", input.0));
                shader.push_str(&format!("    return vec4(color.rgb + {amount}, color.a);\n"));
                shader.push_str("}\n\n");
            }
        }

        generated.insert(node_id);
        Ok(())
    }

    #[allow(dead_code)]
    fn generate_noise_functions(&self) -> String {
        include_str!("shaders/noise_functions.glsl").to_string()
    }

    #[allow(dead_code)]
    fn generate_utility_functions(&self) -> String {
        r#"
vec2 transform_uv(vec2 uv, vec2 offset, float rotation, vec2 scale) {
    uv -= 0.5;
    float s = sin(rotation);
    float c = cos(rotation);
    uv = vec2(uv.x * c - uv.y * s, uv.x * s + uv.y * c);
    uv /= scale;
    uv += 0.5 + offset;
    return uv;
}

"#.to_string()
    }


}
