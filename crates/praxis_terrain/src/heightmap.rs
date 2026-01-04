//! Heightmap data structure and utilities.

use praxis_math::Vec3;
use praxis_utils::{eyre, Result};

/// Heightmap representing terrain elevation data.
///
/// The heightmap stores height values in a 2D grid. Heights are stored as
/// floating-point values representing elevation above the base level.
#[derive(Clone, Debug)]
pub struct TerrainHeightmap {
    /// Width of the heightmap in samples.
    pub width: u32,
    /// Height of the heightmap in samples.
    pub height: u32,
    /// Height values stored in row-major order.
    heights: Vec<f32>,
    /// Maximum height value for normalization.
    pub max_height: f32,
}

impl TerrainHeightmap {
    /// Creates a new flat heightmap with the given dimensions.
    pub fn new(width: u32, height: u32, max_height: f32) -> Self {
        Self {
            width,
            height,
            heights: vec![0.0; (width * height) as usize],
            max_height,
        }
    }

    /// Creates a heightmap from a grayscale image file.
    ///
    /// The image brightness is interpreted as height, with black (0) being
    /// the lowest point and white (255) being the highest point.
    pub fn from_file(path: impl AsRef<std::path::Path>, max_height: f32) -> Result<Self> {
        let img = image::open(path)
            .map_err(|e| eyre::eyre!("Failed to load heightmap image: {}", e))?
            .to_luma8();

        let (width, height) = img.dimensions();
        let mut heights = Vec::with_capacity((width * height) as usize);

        for pixel in img.pixels() {
            let normalized = pixel.0[0] as f32 / 255.0;
            heights.push(normalized * max_height);
        }

        Ok(Self {
            width,
            height,
            heights,
            max_height,
        })
    }

    /// Creates a heightmap from raw height data.
    pub fn from_heights(width: u32, height: u32, heights: Vec<f32>, max_height: f32) -> Self {
        assert_eq!(heights.len(), (width * height) as usize);
        Self {
            width,
            height,
            heights,
            max_height,
        }
    }

    /// Creates a heightmap using procedural noise.
    pub fn from_noise(
        width: u32,
        height: u32,
        max_height: f32,
        scale: f64,
        octaves: u32,
    ) -> Self {
        use noise::{NoiseFn, Perlin};

        let perlin = Perlin::new(42);
        let mut heights = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64 * scale;
                let ny = y as f64 / height as f64 * scale;

                let mut value = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = 1.0;
                let mut max_value = 0.0;

                for _ in 0..octaves {
                    value += perlin.get([nx * frequency, ny * frequency]) * amplitude;
                    max_value += amplitude;
                    amplitude *= 0.5;
                    frequency *= 2.0;
                }

                value /= max_value;
                let height = ((value + 1.0) / 2.0) as f32 * max_height;
                heights.push(height.max(0.0));
            }
        }

        Self {
            width,
            height,
            heights,
            max_height,
        }
    }

    /// Gets the height at the specified grid coordinates.
    #[inline]
    pub fn get_height(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.heights[(y * self.width + x) as usize]
    }

    /// Sets the height at the specified grid coordinates.
    #[inline]
    pub fn set_height(&mut self, x: u32, y: u32, height: f32) {
        if x < self.width && y < self.height {
            self.heights[(y * self.width + x) as usize] = height.clamp(0.0, self.max_height);
        }
    }

    /// Gets interpolated height at world position (bilinear interpolation).
    pub fn get_height_at(&self, world_x: f32, world_z: f32, world_size: f32) -> f32 {
        let grid_x = (world_x / world_size * self.width as f32).clamp(0.0, self.width as f32 - 1.0);
        let grid_z = (world_z / world_size * self.height as f32).clamp(0.0, self.height as f32 - 1.0);

        let x0 = grid_x.floor() as u32;
        let z0 = grid_z.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.height - 1);

        let fx = grid_x - x0 as f32;
        let fz = grid_z - z0 as f32;

        let h00 = self.get_height(x0, z0);
        let h10 = self.get_height(x1, z0);
        let h01 = self.get_height(x0, z1);
        let h11 = self.get_height(x1, z1);

        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;

        h0 * (1.0 - fz) + h1 * fz
    }

    /// Calculates the normal vector at the specified grid coordinates.
    pub fn calculate_normal(&self, x: u32, y: u32, world_scale: f32) -> Vec3 {
        let h_l = self.get_height(x.saturating_sub(1), y);
        let h_r = self.get_height((x + 1).min(self.width - 1), y);
        let h_d = self.get_height(x, y.saturating_sub(1));
        let h_u = self.get_height(x, (y + 1).min(self.height - 1));

        let dx = (h_r - h_l) / (2.0 * world_scale);
        let dz = (h_u - h_d) / (2.0 * world_scale);

        Vec3::new(-dx, 1.0, -dz).normalize()
    }

    /// Applies a smoothing filter to the heightmap.
    pub fn smooth(&mut self, iterations: u32) {
        for _ in 0..iterations {
            let mut new_heights = self.heights.clone();

            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let sum = self.get_height(x - 1, y - 1)
                        + self.get_height(x, y - 1)
                        + self.get_height(x + 1, y - 1)
                        + self.get_height(x - 1, y)
                        + self.get_height(x, y)
                        + self.get_height(x + 1, y)
                        + self.get_height(x - 1, y + 1)
                        + self.get_height(x, y + 1)
                        + self.get_height(x + 1, y + 1);

                    new_heights[(y * self.width + x) as usize] = sum / 9.0;
                }
            }

            self.heights = new_heights;
        }
    }

    /// Gets a reference to the raw height data.
    pub fn heights(&self) -> &[f32] {
        &self.heights
    }

    /// Gets a mutable reference to the raw height data.
    pub fn heights_mut(&mut self) -> &mut [f32] {
        &mut self.heights
    }
}
