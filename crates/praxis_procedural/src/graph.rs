//! Texture graph system for composing procedural textures.
//!
//! This module provides a node-based system for building complex textures
//! from simple operations. Nodes can generate noise, transform coordinates,
//! blend textures, and apply various effects.

use praxis_math::Vec2;
use std::collections::HashMap;

/// Unique identifier for a texture node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureNodeId(pub u32);

/// Type of noise to generate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoiseType {
    /// Perlin noise - smooth gradient noise
    Perlin,
    /// Simplex noise - improved Perlin with better isotropy
    Simplex,
    /// Worley noise - cellular/voronoi patterns
    Worley,
}

/// Blend mode for combining two textures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    /// Add two textures together
    Add,
    /// Multiply two textures
    Multiply,
    /// Take the minimum value
    Min,
    /// Take the maximum value
    Max,
    /// Linear interpolation based on alpha
    Mix,
    /// Screen blending (inverted multiply)
    Screen,
    /// Overlay blending
    Overlay,
    /// Subtract second from first
    Subtract,
}

/// Parameters for coordinate transformation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformParams {
    /// Translation offset
    pub offset: Vec2,
    /// Rotation angle in radians
    pub rotation: f32,
    /// Scale factor
    pub scale: Vec2,
}

impl Default for TransformParams {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

/// A color stop in a color ramp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    /// Position along the ramp [0, 1]
    pub position: f32,
    /// Color at this position (RGBA)
    pub color: [f32; 4],
}

/// A color ramp for mapping values to colors.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorRamp {
    /// Sorted list of color stops
    pub stops: Vec<ColorStop>,
}

impl ColorRamp {
    /// Creates a new color ramp with the given stops.
    ///
    /// Stops will be sorted by position automatically.
    pub fn new(mut stops: Vec<ColorStop>) -> Self {
        stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        Self { stops }
    }

    /// Creates a grayscale ramp from black to white.
    pub fn grayscale() -> Self {
        Self::new(vec![
            ColorStop {
                position: 0.0,
                color: [0.0, 0.0, 0.0, 1.0],
            },
            ColorStop {
                position: 1.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ])
    }

    /// Evaluates the color ramp at a given position.
    pub fn evaluate(&self, t: f32) -> [f32; 4] {
        if self.stops.is_empty() {
            return [0.0, 0.0, 0.0, 1.0];
        }

        if t <= self.stops[0].position {
            return self.stops[0].color;
        }

        if t >= self.stops[self.stops.len() - 1].position {
            return self.stops[self.stops.len() - 1].color;
        }

        for i in 0..self.stops.len() - 1 {
            let stop1 = &self.stops[i];
            let stop2 = &self.stops[i + 1];

            if t >= stop1.position && t <= stop2.position {
                let range = stop2.position - stop1.position;
                let factor = (t - stop1.position) / range;

                return [
                    stop1.color[0] + (stop2.color[0] - stop1.color[0]) * factor,
                    stop1.color[1] + (stop2.color[1] - stop1.color[1]) * factor,
                    stop1.color[2] + (stop2.color[2] - stop1.color[2]) * factor,
                    stop1.color[3] + (stop2.color[3] - stop1.color[3]) * factor,
                ];
            }
        }

        [0.0, 0.0, 0.0, 1.0]
    }
}

/// A node in the texture generation graph.
#[derive(Debug, Clone, PartialEq)]
pub enum TextureNode {
    /// Generate noise
    Noise {
        /// Type of noise to generate
        noise_type: NoiseType,
        /// Scale of the noise pattern
        scale: f32,
        /// Number of octaves for fractal noise
        octaves: u32,
        /// Amplitude decay per octave
        persistence: f32,
        /// Frequency multiplier per octave
        lacunarity: f32,
    },

    /// Constant color value
    Constant {
        /// RGBA color value
        color: [f32; 4],
    },

    /// Transform texture coordinates
    Transform {
        /// Input node
        input: TextureNodeId,
        /// Transformation parameters
        params: TransformParams,
    },

    /// Blend two textures together
    Blend {
        /// First input texture
        input_a: TextureNodeId,
        /// Second input texture
        input_b: TextureNodeId,
        /// Blend mode to use
        mode: BlendMode,
        /// Blend factor [0, 1]
        factor: f32,
    },

    /// Apply a color ramp to a grayscale input
    ColorRamp {
        /// Input node (uses red channel as value)
        input: TextureNodeId,
        /// Color ramp to apply
        ramp: ColorRamp,
    },

    /// Invert the input values
    Invert {
        /// Input node
        input: TextureNodeId,
    },

    /// Clamp values to a range
    Clamp {
        /// Input node
        input: TextureNodeId,
        /// Minimum value
        min: f32,
        /// Maximum value
        max: f32,
    },

    /// Apply power function to values
    Power {
        /// Input node
        input: TextureNodeId,
        /// Exponent
        exponent: f32,
    },

    /// Mix between black and white based on threshold
    Threshold {
        /// Input node
        input: TextureNodeId,
        /// Threshold value
        threshold: f32,
    },

    /// Apply contrast adjustment
    Contrast {
        /// Input node
        input: TextureNodeId,
        /// Contrast amount (-1 to 1)
        amount: f32,
    },

    /// Apply brightness adjustment
    Brightness {
        /// Input node
        input: TextureNodeId,
        /// Brightness amount (-1 to 1)
        amount: f32,
    },
}

/// A graph of texture generation nodes.
///
/// The graph is a directed acyclic graph (DAG) where each node represents
/// an operation on texture data. The graph is evaluated to produce a final texture.
pub struct TextureGraph {
    nodes: HashMap<TextureNodeId, TextureNode>,
    next_id: u32,
    output_node: Option<TextureNodeId>,
    seed: u32,
}

impl TextureGraph {
    /// Creates a new empty texture graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 0,
            output_node: None,
            seed: 0,
        }
    }

    /// Sets the random seed for noise generation.
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    /// Gets the current random seed.
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Adds a node to the graph and returns its ID.
    pub fn add_node(&mut self, node: TextureNode) -> TextureNodeId {
        let id = TextureNodeId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    /// Removes a node from the graph.
    pub fn remove_node(&mut self, id: TextureNodeId) -> Option<TextureNode> {
        if self.output_node == Some(id) {
            self.output_node = None;
        }
        self.nodes.remove(&id)
    }

    /// Gets a reference to a node.
    pub fn get_node(&self, id: TextureNodeId) -> Option<&TextureNode> {
        self.nodes.get(&id)
    }

    /// Gets a mutable reference to a node.
    pub fn get_node_mut(&mut self, id: TextureNodeId) -> Option<&mut TextureNode> {
        self.nodes.get_mut(&id)
    }

    /// Sets the output node of the graph.
    pub fn set_output(&mut self, id: TextureNodeId) {
        self.output_node = Some(id);
    }

    /// Gets the output node ID.
    pub fn output(&self) -> Option<TextureNodeId> {
        self.output_node
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Validates that the graph is well-formed.
    ///
    /// Checks for:
    /// - Output node exists and is in the graph
    /// - All referenced input nodes exist
    /// - No cycles in the graph
    pub fn validate(&self) -> Result<(), String> {
        let output_id = self
            .output_node
            .ok_or_else(|| "No output node set".to_string())?;

        if !self.nodes.contains_key(&output_id) {
            return Err("Output node not found in graph".to_string());
        }

        for (id, node) in &self.nodes {
            let inputs = self.get_node_inputs(node);
            for input in inputs {
                if !self.nodes.contains_key(&input) {
                    return Err(format!(
                        "Node {id:?} references non-existent input {input:?}"
                    ));
                }
            }
        }

        self.check_cycles()?;

        Ok(())
    }

    fn get_node_inputs(&self, node: &TextureNode) -> Vec<TextureNodeId> {
        match node {
            TextureNode::Transform { input, .. }
            | TextureNode::ColorRamp { input, .. }
            | TextureNode::Invert { input }
            | TextureNode::Clamp { input, .. }
            | TextureNode::Power { input, .. }
            | TextureNode::Threshold { input, .. }
            | TextureNode::Contrast { input, .. }
            | TextureNode::Brightness { input, .. } => vec![*input],
            TextureNode::Blend {
                input_a, input_b, ..
            } => vec![*input_a, *input_b],
            _ => vec![],
        }
    }

    fn check_cycles(&self) -> Result<(), String> {
        if let Some(output) = self.output_node {
            let mut visited = HashMap::new();
            let mut stack = HashMap::new();
            self.visit_node(output, &mut visited, &mut stack)?;
        }
        Ok(())
    }

    fn visit_node(
        &self,
        node_id: TextureNodeId,
        visited: &mut HashMap<TextureNodeId, bool>,
        stack: &mut HashMap<TextureNodeId, bool>,
    ) -> Result<(), String> {
        if stack.contains_key(&node_id) {
            return Err(format!("Cycle detected at node {node_id:?}"));
        }

        if visited.contains_key(&node_id) {
            return Ok(());
        }

        stack.insert(node_id, true);

        if let Some(node) = self.nodes.get(&node_id) {
            for input in self.get_node_inputs(node) {
                self.visit_node(input, visited, stack)?;
            }
        }

        stack.remove(&node_id);
        visited.insert(node_id, true);

        Ok(())
    }

    /// Returns an iterator over all nodes in the graph.
    pub fn nodes(&self) -> impl Iterator<Item = (TextureNodeId, &TextureNode)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }
}

impl Default for TextureGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = TextureGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert!(graph.output().is_none());
    }

    #[test]
    fn test_add_node() {
        let mut graph = TextureGraph::new();
        let id = graph.add_node(TextureNode::Constant {
            color: [1.0, 0.0, 0.0, 1.0],
        });
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(id).is_some());
    }

    #[test]
    fn test_validate_simple_graph() {
        let mut graph = TextureGraph::new();
        let noise_id = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 1.0,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
        });
        graph.set_output(noise_id);
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_output() {
        let graph = TextureGraph::new();
        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_validate_missing_input() {
        let mut graph = TextureGraph::new();
        let transform_id = graph.add_node(TextureNode::Transform {
            input: TextureNodeId(999),
            params: TransformParams::default(),
        });
        graph.set_output(transform_id);
        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_color_ramp_evaluation() {
        let ramp = ColorRamp::new(vec![
            ColorStop {
                position: 0.0,
                color: [0.0, 0.0, 0.0, 1.0],
            },
            ColorStop {
                position: 1.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]);

        let black = ramp.evaluate(0.0);
        assert_eq!(black, [0.0, 0.0, 0.0, 1.0]);

        let white = ramp.evaluate(1.0);
        assert_eq!(white, [1.0, 1.0, 1.0, 1.0]);

        let gray = ramp.evaluate(0.5);
        assert_eq!(gray, [0.5, 0.5, 0.5, 1.0]);
    }
}
