//! Light probe system for dynamic global illumination.
//!
//! Light probes capture lighting information at specific points in space and use
//! spherical harmonics to represent diffuse irradiance. This provides efficient
//! real-time global illumination by interpolating between probes.
//!
//! # Architecture
//!
//! - **Light Probes**: Sample points capturing spherical lighting information
//! - **Probe Grid**: 3D grid of probes for spatial interpolation
//! - **Spherical Harmonics**: Compact representation of diffuse lighting (9 coefficients)
//! - **Trilinear Interpolation**: Smooth blending between nearby probes
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{LightProbeManager, LightProbeGrid};
//! use praxis_math::Vec3;
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create a probe grid covering a room
//! let grid = LightProbeGrid::new(
//!     Vec3::new(-10.0, 0.0, -10.0),  // Min bounds
//!     Vec3::new(10.0, 5.0, 10.0),     // Max bounds
//!     [5, 3, 5],                      // Grid dimensions
//! );
//!
//! // let mut manager = LightProbeManager::new(device, allocator)?;
//! // manager.add_grid("room", grid)?;
//! # Ok(())
//! # }
//! ```

use bytemuck::Zeroable;
use praxis_math::{Vec3, Vec4};
use praxis_utils::{eyre, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Maximum number of light probes supported in the shader.
pub const MAX_LIGHT_PROBES: usize = 64;

/// Number of spherical harmonic coefficients per probe (L2 SH = 9 coefficients, RGB = 27 total).
pub const PROBE_IRRADIANCE_COEFFS: usize = 27;

/// Light probe capturing spherical lighting information at a point.
#[derive(Debug, Clone, Copy)]
pub struct LightProbe {
    pub position: Vec3,
    pub sh_coefficients: [Vec4; 9],
    pub intensity: f32,
    pub radius: f32,
}

impl LightProbe {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            sh_coefficients: [Vec4::ZERO; 9],
            intensity: 1.0,
            radius: 10.0,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }
}

impl Default for LightProbe {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}

/// Light probe data for GPU upload (std140 layout).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightProbeData {
    pub position: [f32; 4],
    pub sh_r: [f32; 4],
    pub sh_g: [f32; 4],
    pub sh_b: [f32; 4],
    pub sh_r2: [f32; 4],
    pub sh_g2: [f32; 4],
    pub sh_b2: [f32; 4],
    pub sh_r3: [f32; 4],
    pub sh_g3: [f32; 4],
    pub sh_b3: [f32; 4],
    pub intensity: f32,
    pub radius: f32,
    pub _padding: [f32; 2],
}

impl From<&LightProbe> for LightProbeData {
    fn from(probe: &LightProbe) -> Self {
        let mut data = Self {
            position: [probe.position.x, probe.position.y, probe.position.z, 0.0],
            sh_r: [0.0; 4],
            sh_g: [0.0; 4],
            sh_b: [0.0; 4],
            sh_r2: [0.0; 4],
            sh_g2: [0.0; 4],
            sh_b2: [0.0; 4],
            sh_r3: [0.0; 4],
            sh_g3: [0.0; 4],
            sh_b3: [0.0; 4],
            intensity: probe.intensity,
            radius: probe.radius,
            _padding: [0.0; 2],
        };

        for i in 0..3 {
            data.sh_r[i] = probe.sh_coefficients[i].x;
            data.sh_g[i] = probe.sh_coefficients[i].y;
            data.sh_b[i] = probe.sh_coefficients[i].z;
        }
        for i in 0..3 {
            data.sh_r2[i] = probe.sh_coefficients[i + 3].x;
            data.sh_g2[i] = probe.sh_coefficients[i + 3].y;
            data.sh_b2[i] = probe.sh_coefficients[i + 3].z;
        }
        for i in 0..3 {
            data.sh_r3[i] = probe.sh_coefficients[i + 6].x;
            data.sh_g3[i] = probe.sh_coefficients[i + 6].y;
            data.sh_b3[i] = probe.sh_coefficients[i + 6].z;
        }

        data
    }
}

/// Probe blending mode for interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeBlendMode {
    Nearest,
    Trilinear,
    Tetrahedral,
}

/// 3D grid of light probes for spatial interpolation.
#[derive(Debug, Clone)]
pub struct LightProbeGrid {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub dimensions: [usize; 3],
    pub probes: Vec<LightProbe>,
    pub blend_mode: ProbeBlendMode,
}

impl LightProbeGrid {
    pub fn new(min_bounds: Vec3, max_bounds: Vec3, dimensions: [usize; 3]) -> Self {
        let total_probes = dimensions[0] * dimensions[1] * dimensions[2];
        let mut probes = Vec::with_capacity(total_probes);

        let cell_size = Vec3::new(
            (max_bounds.x - min_bounds.x) / (dimensions[0] - 1).max(1) as f32,
            (max_bounds.y - min_bounds.y) / (dimensions[1] - 1).max(1) as f32,
            (max_bounds.z - min_bounds.z) / (dimensions[2] - 1).max(1) as f32,
        );

        for z in 0..dimensions[2] {
            for y in 0..dimensions[1] {
                for x in 0..dimensions[0] {
                    let position = Vec3::new(
                        min_bounds.x + x as f32 * cell_size.x,
                        min_bounds.y + y as f32 * cell_size.y,
                        min_bounds.z + z as f32 * cell_size.z,
                    );
                    probes.push(LightProbe::new(position));
                }
            }
        }

        Self {
            min_bounds,
            max_bounds,
            dimensions,
            probes,
            blend_mode: ProbeBlendMode::Trilinear,
        }
    }

    pub fn probe_at(&self, x: usize, y: usize, z: usize) -> Option<&LightProbe> {
        if x >= self.dimensions[0] || y >= self.dimensions[1] || z >= self.dimensions[2] {
            return None;
        }
        let index = x + y * self.dimensions[0] + z * self.dimensions[0] * self.dimensions[1];
        self.probes.get(index)
    }

    pub fn probe_at_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut LightProbe> {
        if x >= self.dimensions[0] || y >= self.dimensions[1] || z >= self.dimensions[2] {
            return None;
        }
        let index = x + y * self.dimensions[0] + z * self.dimensions[0] * self.dimensions[1];
        self.probes.get_mut(index)
    }

    pub fn interpolate_at(&self, position: Vec3) -> Option<LightProbe> {
        if position.x < self.min_bounds.x
            || position.x > self.max_bounds.x
            || position.y < self.min_bounds.y
            || position.y > self.max_bounds.y
            || position.z < self.min_bounds.z
            || position.z > self.max_bounds.z
        {
            return None;
        }

        let cell_size = Vec3::new(
            (self.max_bounds.x - self.min_bounds.x) / (self.dimensions[0] - 1).max(1) as f32,
            (self.max_bounds.y - self.min_bounds.y) / (self.dimensions[1] - 1).max(1) as f32,
            (self.max_bounds.z - self.min_bounds.z) / (self.dimensions[2] - 1).max(1) as f32,
        );

        let local_pos = position - self.min_bounds;
        let fx = local_pos.x / cell_size.x;
        let fy = local_pos.y / cell_size.y;
        let fz = local_pos.z / cell_size.z;

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let z0 = fz.floor() as usize;

        match self.blend_mode {
            ProbeBlendMode::Nearest => {
                let x = fx.round() as usize;
                let y = fy.round() as usize;
                let z = fz.round() as usize;
                self.probe_at(x, y, z).cloned()
            }
            ProbeBlendMode::Trilinear => {
                let x1 = (x0 + 1).min(self.dimensions[0] - 1);
                let y1 = (y0 + 1).min(self.dimensions[1] - 1);
                let z1 = (z0 + 1).min(self.dimensions[2] - 1);

                let tx = fx - x0 as f32;
                let ty = fy - y0 as f32;
                let tz = fz - z0 as f32;

                let c000 = self.probe_at(x0, y0, z0)?;
                let c001 = self.probe_at(x0, y0, z1)?;
                let c010 = self.probe_at(x0, y1, z0)?;
                let c011 = self.probe_at(x0, y1, z1)?;
                let c100 = self.probe_at(x1, y0, z0)?;
                let c101 = self.probe_at(x1, y0, z1)?;
                let c110 = self.probe_at(x1, y1, z0)?;
                let c111 = self.probe_at(x1, y1, z1)?;

                let mut result = LightProbe::new(position);

                for i in 0..9 {
                    let c00 = c000.sh_coefficients[i].lerp(c001.sh_coefficients[i], tz);
                    let c01 = c010.sh_coefficients[i].lerp(c011.sh_coefficients[i], tz);
                    let c10 = c100.sh_coefficients[i].lerp(c101.sh_coefficients[i], tz);
                    let c11 = c110.sh_coefficients[i].lerp(c111.sh_coefficients[i], tz);

                    let c0 = c00.lerp(c01, ty);
                    let c1 = c10.lerp(c11, ty);

                    result.sh_coefficients[i] = c0.lerp(c1, tx);
                }

                result.intensity = (1.0 - tx) * (1.0 - ty) * (1.0 - tz) * c000.intensity
                    + (1.0 - tx) * (1.0 - ty) * tz * c001.intensity
                    + (1.0 - tx) * ty * (1.0 - tz) * c010.intensity
                    + (1.0 - tx) * ty * tz * c011.intensity
                    + tx * (1.0 - ty) * (1.0 - tz) * c100.intensity
                    + tx * (1.0 - ty) * tz * c101.intensity
                    + tx * ty * (1.0 - tz) * c110.intensity
                    + tx * ty * tz * c111.intensity;

                Some(result)
            }
            ProbeBlendMode::Tetrahedral => self.probe_at(x0, y0, z0).cloned(),
        }
    }
}

/// Manager for light probe system.
pub struct LightProbeManager {
    #[allow(dead_code)]
    device: Arc<Device>,
    #[allow(dead_code)]
    memory_allocator: Arc<StandardMemoryAllocator>,
    probe_buffer: Subbuffer<[LightProbeData; MAX_LIGHT_PROBES]>,
    grids: Vec<LightProbeGrid>,
    active_probes: Vec<LightProbe>,
}

impl LightProbeManager {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        let probe_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            [LightProbeData::zeroed(); MAX_LIGHT_PROBES],
        )
        .map_err(|e| eyre::eyre!("Failed to create probe buffer: {}", e))?;

        Ok(Self {
            device,
            memory_allocator,
            probe_buffer,
            grids: Vec::new(),
            active_probes: Vec::new(),
        })
    }

    pub fn add_grid(&mut self, grid: LightProbeGrid) {
        self.grids.push(grid);
    }

    pub fn update_probe(&mut self, index: usize, probe: LightProbe) -> Result<()> {
        if index >= MAX_LIGHT_PROBES {
            return Err(eyre::eyre!("Probe index out of bounds"));
        }

        if index >= self.active_probes.len() {
            self.active_probes.resize(index + 1, LightProbe::default());
        }
        self.active_probes[index] = probe;

        let mut write_lock = self
            .probe_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock probe buffer: {}", e))?;

        write_lock[index] = LightProbeData::from(&probe);

        Ok(())
    }

    pub fn update_all_probes(&mut self) -> Result<()> {
        let mut write_lock = self
            .probe_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock probe buffer: {}", e))?;

        for (i, probe) in self.active_probes.iter().enumerate().take(MAX_LIGHT_PROBES) {
            write_lock[i] = LightProbeData::from(probe);
        }

        Ok(())
    }

    pub fn query_at_position(&self, position: Vec3) -> Option<LightProbe> {
        for grid in &self.grids {
            if let Some(probe) = grid.interpolate_at(position) {
                return Some(probe);
            }
        }
        None
    }

    pub fn buffer(&self) -> &Subbuffer<[LightProbeData; MAX_LIGHT_PROBES]> {
        &self.probe_buffer
    }

    pub fn active_probe_count(&self) -> usize {
        self.active_probes.len().min(MAX_LIGHT_PROBES)
    }
}
