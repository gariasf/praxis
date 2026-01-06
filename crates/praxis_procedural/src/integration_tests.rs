//! Integration tests for the procedural texture system.

#[cfg(test)]
mod tests {
    use crate::{
        cache::{ProceduralTextureCache, TextureCacheKey},
        generator::TextureGenerationParams,
        graph::{BlendMode, ColorRamp, ColorStop, NoiseType, TextureGraph, TextureNode},
    };

    // Note: GPU tests are in a separate module that requires Vulkan initialization

    #[test]
    fn test_simple_perlin_graph() {
        let mut graph = TextureGraph::new();
        let noise_id = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 8.0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        });
        graph.set_output(noise_id);

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_complex_graph_with_blend() {
        let mut graph = TextureGraph::new();

        let noise1 = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 4.0,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        });

        let noise2 = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Simplex,
            scale: 8.0,
            octaves: 2,
            persistence: 0.5,
            lacunarity: 2.0,
        });

        let blend = graph.add_node(TextureNode::Blend {
            input_a: noise1,
            input_b: noise2,
            mode: BlendMode::Add,
            factor: 0.5,
        });

        graph.set_output(blend);

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_graph_with_color_ramp() {
        let mut graph = TextureGraph::new();

        let noise = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Worley,
            scale: 10.0,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
        });

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

        let ramp_node = graph.add_node(TextureNode::ColorRamp { input: noise, ramp });

        graph.set_output(ramp_node);

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_cache_key_consistency() {
        let graph1 = {
            let mut g = TextureGraph::new();
            let n = g.add_node(TextureNode::Noise {
                noise_type: NoiseType::Perlin,
                scale: 5.0,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            });
            g.set_output(n);
            g
        };

        let graph2 = {
            let mut g = TextureGraph::new();
            let n = g.add_node(TextureNode::Noise {
                noise_type: NoiseType::Perlin,
                scale: 5.0,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            });
            g.set_output(n);
            g
        };

        let params = TextureGenerationParams {
            width: 256,
            height: 256,
            seed: 42,
        };

        let key1 = TextureCacheKey::new(&graph1, params);
        let key2 = TextureCacheKey::new(&graph2, params);

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_changes_with_params() {
        let mut graph = TextureGraph::new();
        let n = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 5.0,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        });
        graph.set_output(n);

        let params1 = TextureGenerationParams {
            width: 256,
            height: 256,
            seed: 0,
        };

        let params2 = TextureGenerationParams {
            width: 512,
            height: 512,
            seed: 0,
        };

        let params3 = TextureGenerationParams {
            width: 256,
            height: 256,
            seed: 1,
        };

        let key1 = TextureCacheKey::new(&graph, params1);
        let key2 = TextureCacheKey::new(&graph, params2);
        let key3 = TextureCacheKey::new(&graph, params3);

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = ProceduralTextureCache::new(10, 1024 * 1024);

        let mut graph = TextureGraph::new();
        let n = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 5.0,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        });
        graph.set_output(n);

        let params = TextureGenerationParams {
            width: 64,
            height: 64,
            seed: 0,
        };

        let key = TextureCacheKey::new(&graph, params);

        assert!(cache.get(&key).is_none());

        let data = vec![0u8; 64 * 64 * 4];
        cache.insert(key.clone(), data.clone(), 64, 64);

        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);

        let stats = cache.statistics();
        assert_eq!(stats.total_lookups, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_invalid_graph_detection() {
        let mut graph = TextureGraph::new();
        let noise = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 5.0,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        });

        assert!(graph.validate().is_err());

        graph.set_output(noise);
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_all_blend_modes() {
        let blend_modes = [
            BlendMode::Add,
            BlendMode::Multiply,
            BlendMode::Min,
            BlendMode::Max,
            BlendMode::Mix,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::Subtract,
        ];

        for mode in &blend_modes {
            let mut graph = TextureGraph::new();

            let n1 = graph.add_node(TextureNode::Noise {
                noise_type: NoiseType::Perlin,
                scale: 5.0,
                octaves: 2,
                persistence: 0.5,
                lacunarity: 2.0,
            });

            let n2 = graph.add_node(TextureNode::Noise {
                noise_type: NoiseType::Simplex,
                scale: 5.0,
                octaves: 2,
                persistence: 0.5,
                lacunarity: 2.0,
            });

            let blend = graph.add_node(TextureNode::Blend {
                input_a: n1,
                input_b: n2,
                mode: *mode,
                factor: 0.5,
            });

            graph.set_output(blend);

            assert!(
                graph.validate().is_ok(),
                "Failed to validate graph with blend mode {mode:?}"
            );
        }
    }

    #[test]
    fn test_all_noise_types() {
        let noise_types = [NoiseType::Perlin, NoiseType::Simplex, NoiseType::Worley];

        for noise_type in &noise_types {
            let mut graph = TextureGraph::new();

            let noise = graph.add_node(TextureNode::Noise {
                noise_type: *noise_type,
                scale: 8.0,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            });

            graph.set_output(noise);

            assert!(
                graph.validate().is_ok(),
                "Failed to validate graph with noise type {noise_type:?}"
            );
        }
    }

    #[test]
    fn test_deep_graph_hierarchy() {
        let mut graph = TextureGraph::new();

        let noise = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 8.0,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        });

        let power = graph.add_node(TextureNode::Power {
            input: noise,
            exponent: 2.0,
        });

        let contrast = graph.add_node(TextureNode::Contrast {
            input: power,
            amount: 0.3,
        });

        let brightness = graph.add_node(TextureNode::Brightness {
            input: contrast,
            amount: 0.1,
        });

        let clamp = graph.add_node(TextureNode::Clamp {
            input: brightness,
            min: 0.0,
            max: 1.0,
        });

        graph.set_output(clamp);

        assert!(graph.validate().is_ok());
    }
}
