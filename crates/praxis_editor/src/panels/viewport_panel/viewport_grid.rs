//! Grid floor rendering for the viewport.

use praxis_graphics::{MeshData, RenderContext};
use praxis_math::Mat4;
use praxis_utils::Result;

/// Creates a grid mesh for the viewport floor.
///
/// # Arguments
///
/// * `size` - Size of the grid in world units (grid extends from -size to +size)
/// * `divisions` - Number of divisions along each axis
/// * `y_position` - Y coordinate of the grid plane (typically 0.0 for floor)
///
/// # Returns
///
/// MeshData containing grid lines as indexed line segments.
pub fn create_grid_mesh(size: f32, divisions: u32, y_position: f32) -> MeshData {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let step = (size * 2.0) / divisions as f32;
    let half_size = size;

    // Create grid lines parallel to X axis (running along Z)
    for i in 0..=divisions {
        let z = -half_size + i as f32 * step;

        // Determine color based on position (highlight center lines)
        let color = if i == divisions / 2 {
            [0.5, 0.5, 0.5] // Center line brighter
        } else if i % 5 == 0 {
            [0.3, 0.3, 0.3] // Every 5th line slightly brighter
        } else {
            [0.2, 0.2, 0.2] // Regular grid lines
        };

        let idx = positions.len() as u16;
        positions.push([-half_size, y_position, z]);
        positions.push([half_size, y_position, z]);
        colors.push(color);
        colors.push(color);
        indices.push(idx);
        indices.push(idx + 1);
    }

    // Create grid lines parallel to Z axis (running along X)
    for i in 0..=divisions {
        let x = -half_size + i as f32 * step;

        // Determine color based on position (highlight center lines)
        let color = if i == divisions / 2 {
            [0.5, 0.5, 0.5] // Center line brighter
        } else if i % 5 == 0 {
            [0.3, 0.3, 0.3] // Every 5th line slightly brighter
        } else {
            [0.2, 0.2, 0.2] // Regular grid lines
        };

        let idx = positions.len() as u16;
        positions.push([x, y_position, -half_size]);
        positions.push([x, y_position, half_size]);
        colors.push(color);
        colors.push(color);
        indices.push(idx);
        indices.push(idx + 1);
    }

    // Add axis lines (X = red, Z = blue)
    let center_idx = positions.len() as u16;

    // X axis (red)
    positions.push([-half_size, y_position, 0.0]);
    positions.push([half_size, y_position, 0.0]);
    colors.push([1.0, 0.0, 0.0]);
    colors.push([1.0, 0.0, 0.0]);
    indices.push(center_idx);
    indices.push(center_idx + 1);

    // Z axis (blue)
    positions.push([0.0, y_position, -half_size]);
    positions.push([0.0, y_position, half_size]);
    colors.push([0.0, 0.0, 1.0]);
    colors.push([0.0, 0.0, 1.0]);
    indices.push(center_idx + 2);
    indices.push(center_idx + 3);

    // Generate normals (all pointing up for a horizontal grid)
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        tangents: None,
        indices,
    }
}

/// Manages the grid mesh for viewport rendering.
pub struct GridRenderer {
    mesh_id: String,
    grid_size: f32,
    grid_divisions: u32,
    y_position: f32,
}

impl GridRenderer {
    /// Creates a new grid renderer.
    pub fn new() -> Self {
        Self {
            mesh_id: "_viewport_grid".to_string(),
            grid_size: 50.0,
            grid_divisions: 50,
            y_position: 0.0,
        }
    }

    /// Initializes the grid mesh in the render context.
    pub fn initialize(&self, render_context: &mut RenderContext) -> Result<()> {
        let mesh_data = create_grid_mesh(self.grid_size, self.grid_divisions, self.y_position);
        render_context
            .mesh_manager_mut()
            .load_mesh(&self.mesh_id, mesh_data)?;
        Ok(())
    }

    /// Returns the mesh ID for the grid.
    pub fn mesh_id(&self) -> &str {
        &self.mesh_id
    }

    /// Returns the model matrix for the grid (identity since it's at origin).
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::IDENTITY
    }

    /// Sets the grid size.
    #[allow(dead_code)] // Public API for future use
    pub fn set_grid_size(&mut self, size: f32, render_context: &mut RenderContext) -> Result<()> {
        self.grid_size = size;
        self.reinitialize(render_context)
    }

    /// Sets the grid divisions.
    #[allow(dead_code)] // Public API for future use
    pub fn set_grid_divisions(
        &mut self,
        divisions: u32,
        render_context: &mut RenderContext,
    ) -> Result<()> {
        self.grid_divisions = divisions;
        self.reinitialize(render_context)
    }

    /// Sets the Y position of the grid.
    #[allow(dead_code)] // Public API for future use
    pub fn set_y_position(&mut self, y: f32, render_context: &mut RenderContext) -> Result<()> {
        self.y_position = y;
        self.reinitialize(render_context)
    }

    /// Reinitializes the grid mesh with current parameters.
    #[allow(dead_code)] // Used by public setter methods
    fn reinitialize(&self, render_context: &mut RenderContext) -> Result<()> {
        let mesh_data = create_grid_mesh(self.grid_size, self.grid_divisions, self.y_position);
        render_context
            .mesh_manager_mut()
            .load_mesh(&self.mesh_id, mesh_data)?;
        Ok(())
    }
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_grid_mesh() {
        let mesh = create_grid_mesh(10.0, 10, 0.0);

        // Should have vertices for grid lines and axis lines
        assert!(!mesh.positions.is_empty());
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());

        // Indices should come in pairs (line segments)
        assert_eq!(mesh.indices.len() % 2, 0);
    }

    #[test]
    fn test_grid_renderer_creation() {
        let renderer = GridRenderer::new();
        assert_eq!(renderer.grid_size, 50.0);
        assert_eq!(renderer.grid_divisions, 50);
        assert_eq!(renderer.y_position, 0.0);
    }

    #[test]
    fn test_grid_model_matrix() {
        let renderer = GridRenderer::new();
        assert_eq!(renderer.model_matrix(), Mat4::IDENTITY);
    }
}
