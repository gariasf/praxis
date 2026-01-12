//! Vegetation system with GPU instancing for grass, trees, and other foliage.

use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;

/// A single vegetation instance.
#[derive(Debug, Clone, Copy)]
pub struct VegetationInstance {
    /// World position of the vegetation instance.
    pub position: Vec3,

    /// Rotation quaternion.
    pub rotation: Quat,

    /// Scale factor (uniform scale).
    pub scale: f32,

    /// Color variation (RGB multiplier).
    pub color_variation: Vec3,
}

impl VegetationInstance {
    /// Creates a new vegetation instance.
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            color_variation: Vec3::ONE,
        }
    }

    /// Sets the rotation.
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale.
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the color variation.
    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color_variation = color;
        self
    }

    /// Computes the model matrix for this instance.
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(Vec3::splat(self.scale), self.rotation, self.position)
    }
}

/// Vegetation layer configuration.
#[derive(Debug, Clone)]
pub struct VegetationLayer {
    /// Name identifier for this layer.
    pub name: String,

    /// Mesh name to use for rendering.
    pub mesh_name: String,

    /// Material/texture name to use.
    pub material_name: String,

    /// Density (instances per square unit).
    pub density: f32,

    /// Minimum height for placement.
    pub min_height: f32,

    /// Maximum height for placement.
    pub max_height: f32,

    /// Minimum slope angle (degrees) for placement.
    pub min_slope: f32,

    /// Maximum slope angle (degrees) for placement.
    pub max_slope: f32,

    /// Minimum scale factor for random variation.
    pub scale_min: f32,

    /// Maximum scale factor for random variation.
    pub scale_max: f32,

    /// Whether to randomly rotate instances around Y axis.
    pub random_rotation: bool,

    /// Color variation range.
    pub color_variation: f32,

    /// Wind strength affecting this layer.
    pub wind_strength: f32,

    /// All instances for this layer.
    pub instances: Vec<VegetationInstance>,
}

impl VegetationLayer {
    /// Creates a new vegetation layer.
    pub fn new(
        name: impl Into<String>,
        mesh_name: impl Into<String>,
        material_name: impl Into<String>,
        density: f32,
    ) -> Self {
        Self {
            name: name.into(),
            mesh_name: mesh_name.into(),
            material_name: material_name.into(),
            density,
            min_height: 0.0,
            max_height: f32::MAX,
            min_slope: 0.0,
            max_slope: 45.0,
            scale_min: 0.8,
            scale_max: 1.2,
            random_rotation: true,
            color_variation: 0.1,
            wind_strength: 1.0,
            instances: Vec::new(),
        }
    }

    /// Sets the height range for placement.
    pub fn with_height_range(mut self, min: f32, max: f32) -> Self {
        self.min_height = min;
        self.max_height = max;
        self
    }

    /// Sets the slope range for placement.
    pub fn with_slope_range(mut self, min: f32, max: f32) -> Self {
        self.min_slope = min;
        self.max_slope = max;
        self
    }

    /// Sets the scale variation range.
    pub fn with_scale_range(mut self, min: f32, max: f32) -> Self {
        self.scale_min = min;
        self.scale_max = max;
        self
    }

    /// Sets whether to use random rotation.
    pub fn with_random_rotation(mut self, enabled: bool) -> Self {
        self.random_rotation = enabled;
        self
    }

    /// Sets the color variation amount.
    pub fn with_color_variation(mut self, variation: f32) -> Self {
        self.color_variation = variation;
        self
    }

    /// Sets the wind strength.
    pub fn with_wind_strength(mut self, strength: f32) -> Self {
        self.wind_strength = strength;
        self
    }

    /// Checks if a position is valid for this vegetation layer.
    pub fn is_valid_position(&self, height: f32, slope_degrees: f32) -> bool {
        height >= self.min_height
            && height <= self.max_height
            && slope_degrees >= self.min_slope
            && slope_degrees <= self.max_slope
    }

    /// Adds an instance to this layer.
    pub fn add_instance(&mut self, instance: VegetationInstance) {
        self.instances.push(instance);
    }

    /// Removes instances within a radius of a point.
    pub fn remove_instances(&mut self, center: Vec3, radius: f32) {
        let radius_sq = radius * radius;
        self.instances.retain(|inst| {
            let dx = inst.position.x - center.x;
            let dz = inst.position.z - center.z;
            dx * dx + dz * dz > radius_sq
        });
    }

    /// Clears all instances.
    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }

    /// Gets the number of instances in this layer.
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

/// Vegetation distribution generator.
pub struct VegetationDistributor;

impl VegetationDistributor {
    /// Generates vegetation instances for a terrain area using Poisson disc sampling.
    pub fn distribute(
        layer: &VegetationLayer,
        terrain_bounds_min: Vec3,
        terrain_bounds_max: Vec3,
        height_fn: impl Fn(f32, f32) -> f32,
        normal_fn: impl Fn(f32, f32) -> Vec3,
    ) -> Result<Vec<VegetationInstance>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut instances = Vec::new();

        let width = (terrain_bounds_max.x - terrain_bounds_min.x).abs();
        let depth = (terrain_bounds_max.z - terrain_bounds_min.z).abs();

        if width <= 0.0 || depth <= 0.0 {
            return Ok(instances);
        }

        let area = width * depth;
        let density = layer.density.max(0.0);
        let target_count = ((area * density) as usize).min(100_000);

        if target_count == 0 {
            return Ok(instances);
        }

        let min_distance = if density > 0.0 {
            (1.0 / density.sqrt()).max(0.5)
        } else {
            1.0
        };

        let mut attempts = 0;
        let max_attempts = (target_count * 30).min(1_000_000);

        let x_min = terrain_bounds_min.x.min(terrain_bounds_max.x);
        let x_max = terrain_bounds_min.x.max(terrain_bounds_max.x);
        let z_min = terrain_bounds_min.z.min(terrain_bounds_max.z);
        let z_max = terrain_bounds_min.z.max(terrain_bounds_max.z);

        while instances.len() < target_count && attempts < max_attempts {
            attempts += 1;

            let x = rng.gen_range(x_min..x_max);
            let z = rng.gen_range(z_min..z_max);
            let y = height_fn(x, z);
            let normal = normal_fn(x, z);

            if !y.is_finite()
                || !normal.x.is_finite()
                || !normal.y.is_finite()
                || !normal.z.is_finite()
            {
                continue;
            }

            let slope = normal.angle_between(Vec3::Y).to_degrees();

            if !layer.is_valid_position(y, slope) {
                continue;
            }

            let position = Vec3::new(x, y, z);

            let too_close = instances.iter().any(|inst: &VegetationInstance| {
                let dx = inst.position.x - position.x;
                let dz = inst.position.z - position.z;
                (dx * dx + dz * dz) < min_distance * min_distance
            });

            if too_close {
                continue;
            }

            let scale = rng
                .gen_range(layer.scale_min..layer.scale_max)
                .clamp(0.01, 10.0);

            let rotation = if layer.random_rotation {
                Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::TAU))
            } else {
                Quat::IDENTITY
            };

            let color_var = layer.color_variation.clamp(0.0, 1.0);
            let color_r = (1.0 + rng.gen_range(-color_var..color_var)).clamp(0.0, 2.0);
            let color_g = (1.0 + rng.gen_range(-color_var..color_var)).clamp(0.0, 2.0);
            let color_b = (1.0 + rng.gen_range(-color_var..color_var)).clamp(0.0, 2.0);

            instances.push(
                VegetationInstance::new(position)
                    .with_scale(scale)
                    .with_rotation(rotation)
                    .with_color(Vec3::new(color_r, color_g, color_b)),
            );
        }

        Ok(instances)
    }
}
