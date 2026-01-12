//! Terrain editing tools for the editor.

use crate::heightmap::TerrainHeightmap;
use crate::splatmap::SplatMap;
use crate::vegetation::{VegetationInstance, VegetationLayer};
use praxis_math::{Quat, Vec3};
use praxis_utils::Result;

/// Types of terrain editing operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainEditOperation {
    /// Raise terrain height.
    Raise,
    /// Lower terrain height.
    Lower,
    /// Smooth terrain.
    Smooth,
    /// Flatten terrain to a specific height.
    Flatten,
    /// Set terrain to exact height.
    SetHeight,
}

/// Brush shape for terrain editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushShape {
    /// Circular brush.
    Circle,
    /// Square brush.
    Square,
}

/// Falloff curve for brush strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushFalloff {
    /// Linear falloff from center to edge.
    Linear,
    /// Smooth (cosine) falloff.
    Smooth,
    /// Constant strength (no falloff).
    Constant,
}

/// Heightmap editing brush.
pub struct HeightmapBrush {
    /// Brush radius in world units.
    pub radius: f32,

    /// Brush strength (0.0 to 1.0).
    pub strength: f32,

    /// Brush shape.
    pub shape: BrushShape,

    /// Falloff curve.
    pub falloff: BrushFalloff,

    /// Target height for flatten/set operations.
    pub target_height: f32,
}

impl HeightmapBrush {
    /// Creates a new heightmap brush.
    pub fn new(radius: f32, strength: f32) -> Self {
        Self {
            radius,
            strength,
            shape: BrushShape::Circle,
            falloff: BrushFalloff::Smooth,
            target_height: 0.0,
        }
    }

    /// Sets the brush shape.
    pub fn with_shape(mut self, shape: BrushShape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the falloff curve.
    pub fn with_falloff(mut self, falloff: BrushFalloff) -> Self {
        self.falloff = falloff;
        self
    }

    /// Sets the target height for flatten/set operations.
    pub fn with_target_height(mut self, height: f32) -> Self {
        self.target_height = height;
        self
    }

    /// Applies the brush to the heightmap at the specified position.
    pub fn apply(
        &self,
        heightmap: &mut TerrainHeightmap,
        world_x: f32,
        world_z: f32,
        world_size: f32,
        operation: TerrainEditOperation,
        delta_time: f32,
    ) -> Result<()> {
        if world_size <= 0.0 || heightmap.width == 0 || heightmap.height == 0 {
            return Err(praxis_utils::eyre::eyre!("Invalid heightmap or world_size"));
        }

        let grid_x =
            (world_x / world_size * heightmap.width as f32).clamp(0.0, heightmap.width as f32);
        let grid_z =
            (world_z / world_size * heightmap.height as f32).clamp(0.0, heightmap.height as f32);
        let grid_radius = (self.radius / world_size * heightmap.width as f32).max(1.0);

        let min_x = ((grid_x - grid_radius).floor() as i32)
            .max(0)
            .min(heightmap.width as i32 - 1) as u32;
        let max_x = ((grid_x + grid_radius).ceil() as i32)
            .max(0)
            .min(heightmap.width as i32 - 1) as u32;
        let min_z = ((grid_z - grid_radius).floor() as i32)
            .max(0)
            .min(heightmap.height as i32 - 1) as u32;
        let max_z = ((grid_z + grid_radius).ceil() as i32)
            .max(0)
            .min(heightmap.height as i32 - 1) as u32;

        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let dx = x as f32 - grid_x;
                let dz = z as f32 - grid_z;

                let in_brush = match self.shape {
                    BrushShape::Circle => (dx * dx + dz * dz).sqrt() <= grid_radius,
                    BrushShape::Square => dx.abs() <= grid_radius && dz.abs() <= grid_radius,
                };

                if !in_brush {
                    continue;
                }

                let dist = (dx * dx + dz * dz).sqrt();
                let falloff_strength = match self.falloff {
                    BrushFalloff::Linear => (1.0 - dist / grid_radius).max(0.0),
                    BrushFalloff::Smooth => {
                        let t = (dist / grid_radius).min(1.0);
                        ((1.0 - t) * std::f32::consts::PI / 2.0).cos()
                    }
                    BrushFalloff::Constant => 1.0,
                };

                let final_strength = self.strength * falloff_strength * delta_time;
                let current_height = heightmap.get_height(x, z);

                let new_height = match operation {
                    TerrainEditOperation::Raise => current_height + final_strength * 10.0,
                    TerrainEditOperation::Lower => current_height - final_strength * 10.0,
                    TerrainEditOperation::Flatten => {
                        current_height + (self.target_height - current_height) * final_strength
                    }
                    TerrainEditOperation::SetHeight => {
                        current_height + (self.target_height - current_height) * final_strength
                    }
                    TerrainEditOperation::Smooth => {
                        let sum = heightmap.get_height(x.saturating_sub(1), z)
                            + heightmap.get_height(x + 1, z)
                            + heightmap.get_height(x, z.saturating_sub(1))
                            + heightmap.get_height(x, z + 1)
                            + current_height;
                        let avg = sum / 5.0;
                        current_height + (avg - current_height) * final_strength
                    }
                };

                heightmap.set_height(x, z, new_height);
            }
        }

        Ok(())
    }
}

/// Paint brush for splat map editing.
pub struct PaintBrush {
    /// Brush radius in world units.
    pub radius: f32,

    /// Brush strength (0.0 to 1.0).
    pub strength: f32,

    /// Brush shape.
    pub shape: BrushShape,

    /// Falloff curve.
    pub falloff: BrushFalloff,

    /// Layer index to paint (0-3 for first splat map, 4-7 for second, etc.).
    pub layer_index: usize,
}

impl PaintBrush {
    /// Creates a new paint brush.
    pub fn new(radius: f32, strength: f32, layer_index: usize) -> Self {
        Self {
            radius,
            strength,
            shape: BrushShape::Circle,
            falloff: BrushFalloff::Smooth,
            layer_index,
        }
    }

    /// Applies the paint brush to the splat map at the specified position.
    pub fn apply(
        &self,
        splatmap: &mut SplatMap,
        world_x: f32,
        world_z: f32,
        world_size: f32,
        delta_time: f32,
    ) -> Result<()> {
        let final_strength = self.strength * delta_time;
        splatmap.paint_circle(
            world_x,
            world_z,
            self.radius,
            self.layer_index,
            final_strength,
            world_size,
        );
        Ok(())
    }
}

/// Vegetation painting tool.
pub struct VegetationPainter {
    /// Brush radius in world units.
    pub radius: f32,

    /// Density of vegetation to place (instances per square unit).
    pub density: f32,
}

impl VegetationPainter {
    /// Creates a new vegetation painter.
    pub fn new(radius: f32, density: f32) -> Self {
        Self { radius, density }
    }

    /// Places vegetation instances in the brush area.
    pub fn paint(
        &self,
        layer: &mut VegetationLayer,
        world_x: f32,
        world_z: f32,
        height_fn: impl Fn(f32, f32) -> f32,
        normal_fn: impl Fn(f32, f32) -> Vec3,
    ) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let area = std::f32::consts::PI * self.radius * self.radius;
        let target_count = (area * self.density) as usize;

        for _ in 0..target_count {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..self.radius);

            let x = world_x + angle.cos() * distance;
            let z = world_z + angle.sin() * distance;
            let y = height_fn(x, z);
            let normal = normal_fn(x, z);

            let slope = normal.angle_between(Vec3::Y).to_degrees();

            if !layer.is_valid_position(y, slope) {
                continue;
            }

            let scale = rng.gen_range(layer.scale_min..layer.scale_max);

            let rotation = if layer.random_rotation {
                Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::TAU))
            } else {
                Quat::IDENTITY
            };

            let color_r = 1.0 + rng.gen_range(-layer.color_variation..layer.color_variation);
            let color_g = 1.0 + rng.gen_range(-layer.color_variation..layer.color_variation);
            let color_b = 1.0 + rng.gen_range(-layer.color_variation..layer.color_variation);

            let instance = VegetationInstance::new(Vec3::new(x, y, z))
                .with_scale(scale)
                .with_rotation(rotation)
                .with_color(Vec3::new(color_r, color_g, color_b));

            layer.add_instance(instance);
        }

        Ok(())
    }

    /// Erases vegetation instances in the brush area.
    pub fn erase(&self, layer: &mut VegetationLayer, world_x: f32, world_z: f32) -> Result<()> {
        layer.remove_instances(Vec3::new(world_x, 0.0, world_z), self.radius);
        Ok(())
    }
}

/// Terrain editing tool for the editor.
pub struct TerrainEditTool {
    /// Active editing operation.
    pub operation: TerrainEditOperation,

    /// Heightmap brush.
    pub heightmap_brush: HeightmapBrush,

    /// Paint brush for materials.
    pub paint_brush: PaintBrush,

    /// Vegetation painter.
    pub vegetation_painter: VegetationPainter,

    /// Whether the tool is currently active.
    pub is_active: bool,
}

impl TerrainEditTool {
    /// Creates a new terrain edit tool.
    pub fn new() -> Self {
        Self {
            operation: TerrainEditOperation::Raise,
            heightmap_brush: HeightmapBrush::new(5.0, 0.5),
            paint_brush: PaintBrush::new(5.0, 0.5, 0),
            vegetation_painter: VegetationPainter::new(5.0, 2.0),
            is_active: false,
        }
    }

    /// Sets the active operation.
    pub fn set_operation(&mut self, operation: TerrainEditOperation) {
        self.operation = operation;
    }

    /// Sets the brush radius for all brushes.
    pub fn set_radius(&mut self, radius: f32) {
        self.heightmap_brush.radius = radius;
        self.paint_brush.radius = radius;
        self.vegetation_painter.radius = radius;
    }

    /// Sets the brush strength.
    pub fn set_strength(&mut self, strength: f32) {
        self.heightmap_brush.strength = strength;
        self.paint_brush.strength = strength;
    }

    /// Activates the tool.
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// Deactivates the tool.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

impl Default for TerrainEditTool {
    fn default() -> Self {
        Self::new()
    }
}
