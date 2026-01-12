//! Area light system with Linearly Transformed Cosines (LTC).
//!
//! This module provides realistic area light rendering using the LTC technique,
//! which allows real-time shading of polygon lights (rectangles, disks) with
//! accurate specular reflections and soft shadows.
//!
//! # Architecture
//!
//! - **LTC Matrices**: Pre-computed lookup tables for BRDF approximation
//! - **Polygon Clipping**: Efficient area light integration
//! - **Soft Shadows**: Natural penumbra from area lights
//! - **Multiple Shapes**: Rectangle, disk, sphere, and tube lights
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{AreaLight, AreaLightType, AreaLightManager};
//! use praxis_math::{Vec3, Vec2};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! let light = AreaLight {
//!     light_type: AreaLightType::Rectangle { width: 2.0, height: 1.0 },
//!     position: Vec3::new(0.0, 5.0, 0.0),
//!     direction: Vec3::new(0.0, -1.0, 0.0),
//!     color: Vec3::new(1.0, 0.9, 0.8),
//!     intensity: 10.0,
//!     ..Default::default()
//! };
//!
//! // let mut manager = AreaLightManager::new(device, allocator)?;
//! // manager.add_light(light)?;
//! # Ok(())
//! # }
//! ```

use bytemuck::Zeroable;
use praxis_math::{Mat4, Vec3};
use praxis_utils::{eyre, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    device::Device,
    image::view::ImageView,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Maximum number of area lights supported.
pub const MAX_AREA_LIGHTS: usize = 16;

/// Type of area light shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaLightType {
    Rectangle { width: f32, height: f32 },
    Disk { radius: f32 },
    Sphere { radius: f32 },
    Tube { length: f32, radius: f32 },
}

impl Default for AreaLightType {
    fn default() -> Self {
        Self::Rectangle {
            width: 1.0,
            height: 1.0,
        }
    }
}

/// Area light with shape and transform.
#[derive(Debug, Clone, Copy)]
pub struct AreaLight {
    pub light_type: AreaLightType,
    pub position: Vec3,
    pub direction: Vec3,
    pub up: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub two_sided: bool,
}

impl AreaLight {
    pub fn new_rectangle(position: Vec3, width: f32, height: f32) -> Self {
        Self {
            light_type: AreaLightType::Rectangle { width, height },
            position,
            direction: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
            color: Vec3::ONE,
            intensity: 1.0,
            two_sided: false,
        }
    }

    pub fn new_disk(position: Vec3, radius: f32) -> Self {
        Self {
            light_type: AreaLightType::Disk { radius },
            position,
            direction: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
            color: Vec3::ONE,
            intensity: 1.0,
            two_sided: false,
        }
    }

    pub fn new_sphere(position: Vec3, radius: f32) -> Self {
        Self {
            light_type: AreaLightType::Sphere { radius },
            position,
            direction: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
            color: Vec3::ONE,
            intensity: 1.0,
            two_sided: true,
        }
    }

    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color = color;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction = direction.normalize();
        self
    }

    pub fn compute_transform(&self) -> Mat4 {
        let right = self.up.cross(self.direction).normalize();
        let up = self.direction.cross(right).normalize();

        Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            self.direction.extend(0.0),
            self.position.extend(1.0),
        )
    }
}

impl Default for AreaLight {
    fn default() -> Self {
        Self {
            light_type: AreaLightType::default(),
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
            color: Vec3::ONE,
            intensity: 1.0,
            two_sided: false,
        }
    }
}

/// Area light data for GPU (std140 layout).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AreaLightData {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub intensity: f32,
    pub light_type: u32,
    pub param1: f32,
    pub param2: f32,
    pub two_sided: u32,
    pub _padding: [u32; 3],
}

impl From<&AreaLight> for AreaLightData {
    fn from(light: &AreaLight) -> Self {
        let transform = light.compute_transform();
        let (light_type, param1, param2) = match light.light_type {
            AreaLightType::Rectangle { width, height } => (0, width, height),
            AreaLightType::Disk { radius } => (1, radius, 0.0),
            AreaLightType::Sphere { radius } => (2, radius, 0.0),
            AreaLightType::Tube { length, radius } => (3, length, radius),
        };

        Self {
            transform: transform.to_cols_array_2d(),
            color: [light.color.x, light.color.y, light.color.z, 1.0],
            intensity: light.intensity,
            light_type,
            param1,
            param2,
            two_sided: if light.two_sided { 1 } else { 0 },
            _padding: [0; 3],
        }
    }
}

/// LTC matrix lookup table data.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LtcMatrixData {
    pub m: [[f32; 4]; 4],
    pub amplitude: f32,
    pub fresnel: f32,
    pub _padding: [f32; 2],
}

impl Default for LtcMatrixData {
    fn default() -> Self {
        Self {
            m: Mat4::IDENTITY.to_cols_array_2d(),
            amplitude: 1.0,
            fresnel: 0.04,
            _padding: [0.0; 2],
        }
    }
}

/// Manager for area light system.
#[deprecated(
    since = "0.1.0",
    note = "AreaLightManager is an experimental stub with no implementation. \
            See tracking issue: https://github.com/praxis-engine/praxis/issues/TBD"
)]
#[allow(dead_code)]
pub struct AreaLightManager {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    light_buffer: Subbuffer<[AreaLightData; MAX_AREA_LIGHTS]>,
    lights: Vec<AreaLight>,
    ltc_matrix_1: Option<Arc<ImageView>>,
    ltc_matrix_2: Option<Arc<ImageView>>,
}

#[allow(deprecated)]
impl AreaLightManager {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        let light_buffer = Buffer::from_data(
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
            [AreaLightData::zeroed(); MAX_AREA_LIGHTS],
        )
        .map_err(|e| eyre::eyre!("Failed to create area light buffer: {}", e))?;

        Ok(Self {
            device,
            memory_allocator,
            light_buffer,
            lights: Vec::new(),
            ltc_matrix_1: None,
            ltc_matrix_2: None,
        })
    }

    pub fn add_light(&mut self, light: AreaLight) -> Result<()> {
        if self.lights.len() >= MAX_AREA_LIGHTS {
            return Err(eyre::eyre!("Maximum number of area lights reached"));
        }
        self.lights.push(light);
        self.update_buffer()
    }

    pub fn update_light(&mut self, index: usize, light: AreaLight) -> Result<()> {
        if index >= self.lights.len() {
            return Err(eyre::eyre!("Light index out of bounds"));
        }
        self.lights[index] = light;
        self.update_buffer()
    }

    pub fn remove_light(&mut self, index: usize) -> Result<()> {
        if index >= self.lights.len() {
            return Err(eyre::eyre!("Light index out of bounds"));
        }
        self.lights.remove(index);
        self.update_buffer()
    }

    pub fn clear_lights(&mut self) -> Result<()> {
        self.lights.clear();
        self.update_buffer()
    }

    fn update_buffer(&mut self) -> Result<()> {
        let mut write_lock = self
            .light_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock area light buffer: {}", e))?;

        for (i, light) in self.lights.iter().enumerate().take(MAX_AREA_LIGHTS) {
            write_lock[i] = AreaLightData::from(light);
        }

        for i in self.lights.len()..MAX_AREA_LIGHTS {
            write_lock[i] = AreaLightData::zeroed();
        }

        Ok(())
    }

    pub fn buffer(&self) -> &Subbuffer<[AreaLightData; MAX_AREA_LIGHTS]> {
        &self.light_buffer
    }

    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    pub fn lights(&self) -> &[AreaLight] {
        &self.lights
    }
}
