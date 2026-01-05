//! Splat map for blending terrain material layers.

use praxis_utils::{eyre, Result};

/// Splat map controlling material layer blending.
///
/// Each pixel stores blend weights for up to 4 material layers per channel (RGBA).
/// Multiple splat maps can be used to support more than 4 layers (up to 16 total).
#[derive(Clone, Debug)]
pub struct SplatMap {
    /// Width of the splat map.
    pub width: u32,

    /// Height of the splat map.
    pub height: u32,

    /// Splat data stored as RGBA pixels, each component is a blend weight [0, 1].
    /// Format: [R0, G0, B0, A0, R1, G1, B1, A1, ...] for each pixel
    data: Vec<f32>,

    /// Number of layers this splat map supports (1-4).
    pub layers_per_map: u32,
}

impl SplatMap {
    /// Creates a new splat map with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let mut data = vec![0.0; size];

        for i in (0..size).step_by(4) {
            data[i] = 1.0;
        }

        Self {
            width,
            height,
            data,
            layers_per_map: 4,
        }
    }

    /// Creates a splat map from an image file.
    ///
    /// The image should be RGBA, with each channel representing a layer weight.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let img = image::open(path)
            .map_err(|e| eyre::eyre!("Failed to load splat map: {}", e))?
            .to_rgba8();

        let (width, height) = img.dimensions();
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for pixel in img.pixels() {
            for &component in &pixel.0 {
                data.push(component as f32 / 255.0);
            }
        }

        Ok(Self {
            width,
            height,
            data,
            layers_per_map: 4,
        })
    }

    /// Gets the blend weights at the specified pixel coordinates.
    pub fn get_weights(&self, x: u32, y: u32) -> [f32; 4] {
        if x >= self.width || y >= self.height {
            return [1.0, 0.0, 0.0, 0.0];
        }

        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ]
    }

    /// Sets the blend weights at the specified pixel coordinates.
    pub fn set_weights(&mut self, x: u32, y: u32, weights: [f32; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let sum: f32 = weights.iter().sum();
        let normalized = if sum > 0.0 {
            [
                weights[0] / sum,
                weights[1] / sum,
                weights[2] / sum,
                weights[3] / sum,
            ]
        } else {
            [1.0, 0.0, 0.0, 0.0]
        };

        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = normalized[0];
        self.data[idx + 1] = normalized[1];
        self.data[idx + 2] = normalized[2];
        self.data[idx + 3] = normalized[3];
    }

    /// Gets interpolated weights at a world position.
    pub fn get_weights_at(&self, world_x: f32, world_z: f32, world_size: f32) -> [f32; 4] {
        let grid_x = (world_x / world_size * self.width as f32).clamp(0.0, self.width as f32 - 1.0);
        let grid_z =
            (world_z / world_size * self.height as f32).clamp(0.0, self.height as f32 - 1.0);

        let x0 = grid_x.floor() as u32;
        let z0 = grid_z.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.height - 1);

        let fx = grid_x - x0 as f32;
        let fz = grid_z - z0 as f32;

        let w00 = self.get_weights(x0, z0);
        let w10 = self.get_weights(x1, z0);
        let w01 = self.get_weights(x0, z1);
        let w11 = self.get_weights(x1, z1);

        let mut result = [0.0; 4];
        for i in 0..4 {
            let w0 = w00[i] * (1.0 - fx) + w10[i] * fx;
            let w1 = w01[i] * (1.0 - fx) + w11[i] * fx;
            result[i] = w0 * (1.0 - fz) + w1 * fz;
        }

        result
    }

    /// Paints blend weights in a circular area.
    pub fn paint_circle(
        &mut self,
        center_x: f32,
        center_z: f32,
        radius: f32,
        layer_index: usize,
        strength: f32,
        world_size: f32,
    ) {
        let grid_center_x = center_x / world_size * self.width as f32;
        let grid_center_z = center_z / world_size * self.height as f32;
        let grid_radius = radius / world_size * self.width as f32;

        let min_x = ((grid_center_x - grid_radius).floor() as u32).min(self.width - 1);
        let max_x = ((grid_center_x + grid_radius).ceil() as u32).min(self.width - 1);
        let min_z = ((grid_center_z - grid_radius).floor() as u32).min(self.height - 1);
        let max_z = ((grid_center_z + grid_radius).ceil() as u32).min(self.height - 1);

        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let dx = x as f32 - grid_center_x;
                let dz = z as f32 - grid_center_z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= grid_radius {
                    let falloff = 1.0 - (dist / grid_radius).powi(2);
                    let paint_strength = strength * falloff;

                    let mut weights = self.get_weights(x, z);
                    weights[layer_index] += paint_strength;
                    self.set_weights(x, z, weights);
                }
            }
        }
    }

    /// Exports the splat map to an RGBA image file.
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        use image::{ImageBuffer, Rgba};

        let mut img = ImageBuffer::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let weights = self.get_weights(x, y);
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        (weights[0] * 255.0) as u8,
                        (weights[1] * 255.0) as u8,
                        (weights[2] * 255.0) as u8,
                        (weights[3] * 255.0) as u8,
                    ]),
                );
            }
        }

        img.save(path)
            .map_err(|e| eyre::eyre!("Failed to save splat map: {}", e))?;

        Ok(())
    }

    /// Gets a reference to the raw splat data.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Gets a mutable reference to the raw splat data.
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }
}
