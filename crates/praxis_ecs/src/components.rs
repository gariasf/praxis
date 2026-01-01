//! Common components for the Praxis ECS.
//!
//! This module provides frequently-used components that are common across
//! most game projects. These components are designed to work together
//! to form the building blocks of game entities.

use bevy_ecs::component::Component;
use praxis_math::{Mat4, Quat, Vec3};

/// A name component for debugging and identification.
///
/// This component is useful for debugging, logging, and editor tools.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Name};
///
/// let mut world = World::new();
/// world.spawn(Name("Player".to_string()));
/// ```
#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

impl Name {
    /// Creates a new Name component.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Gets the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl From<String> for Name {
    fn from(name: String) -> Self {
        Self(name)
    }
}

/// Transform component representing position, rotation, and scale in 3D space.
///
/// This is one of the most fundamental components in a 3D game engine.
/// It uses separate fields for position, rotation, and scale to make
/// manipulation easier and more efficient than using a full 4x4 matrix.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Transform};
/// use praxis_math::{Vec3, Quat};
///
/// let mut world = World::new();
///
/// // Spawn an entity at position (10, 0, 5)
/// world.spawn(Transform::from_xyz(10.0, 0.0, 5.0));
///
/// // Spawn with custom rotation and scale
/// world.spawn(Transform {
///     translation: Vec3::new(0.0, 5.0, 0.0),
///     rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
///     scale: Vec3::new(2.0, 2.0, 2.0),
/// });
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Transform {
    /// The position in world space.
    pub translation: Vec3,

    /// The rotation as a quaternion.
    pub rotation: Quat,

    /// The scale factor for each axis.
    pub scale: Vec3,
}

impl Transform {
    /// Creates a new transform with identity values (no translation, rotation, or scale).
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Creates a transform from just a translation.
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: Vec3::new(x, y, z),
            ..Self::IDENTITY
        }
    }

    /// Creates a transform from a translation vector.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Creates a transform from a rotation quaternion.
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    /// Creates a transform from a scale vector.
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    /// Computes the transformation matrix.
    ///
    /// This matrix combines translation, rotation, and scale into a single
    /// 4x4 matrix that can be used for rendering.
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Computes the inverse transformation matrix.
    ///
    /// This is useful for converting from world space to local space.
    pub fn compute_inverse_matrix(&self) -> Mat4 {
        self.compute_matrix().inverse()
    }

    /// Transforms a point from local space to world space.
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.rotation * (self.scale * point) + self.translation
    }

    /// Transforms a direction vector (ignoring translation).
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * (self.scale * direction)
    }

    /// Looks at a target position with the given up vector.
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.translation).normalize();
        self.rotation = Quat::from_mat4(&Mat4::look_to_rh(self.translation, forward, up));
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Global transform component representing world-space transformation.
///
/// This component is typically computed from the local Transform and
/// the parent's GlobalTransform in a hierarchical scene graph.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Transform, GlobalTransform};
///
/// let mut world = World::new();
///
/// // Usually computed automatically by transform propagation systems
/// world.spawn((
///     Transform::from_xyz(5.0, 0.0, 0.0),
///     GlobalTransform::default(),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct GlobalTransform {
    /// The world-space transformation matrix.
    pub matrix: Mat4,
}

impl GlobalTransform {
    /// Creates a new global transform from a matrix.
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }

    /// Creates a global transform from translation, rotation, and scale.
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Self {
        Self {
            matrix: Mat4::from_scale_rotation_translation(scale, rotation, translation),
        }
    }

    /// Extracts the translation component.
    pub fn translation(&self) -> Vec3 {
        self.matrix.col(3).truncate()
    }

    /// Extracts the scale component (approximate).
    pub fn scale(&self) -> Vec3 {
        Vec3::new(
            self.matrix.col(0).truncate().length(),
            self.matrix.col(1).truncate().length(),
            self.matrix.col(2).truncate().length(),
        )
    }

    /// Transforms a point from local space to world space.
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix.transform_point3(point)
    }

    /// Transforms a direction vector (ignoring translation).
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.matrix.transform_vector3(direction)
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }
}

impl From<Transform> for GlobalTransform {
    fn from(transform: Transform) -> Self {
        Self {
            matrix: transform.compute_matrix(),
        }
    }
}

/// Parent component for hierarchical relationships.
///
/// Entities with this component are children of the referenced parent entity.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Parent, Transform};
///
/// let mut world = World::new();
///
/// let parent = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0));
/// let child = world.spawn((
///     Transform::from_xyz(5.0, 0.0, 0.0), // 5 units to the right of parent
///     Parent(parent),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Parent(pub bevy_ecs::entity::Entity);

/// Children component containing a list of child entities.
///
/// This component is typically managed automatically when Parent components
/// are added or removed.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Children, Transform};
///
/// let mut world = World::new();
///
/// let parent = world.spawn((
///     Transform::default(),
///     Children::default(),
/// ));
/// ```
#[derive(Component, Debug, Clone, Default)]
pub struct Children(pub Vec<bevy_ecs::entity::Entity>);

impl Children {
    /// Creates a new empty Children component.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates Children with the given child entities.
    pub fn with_children(children: Vec<bevy_ecs::entity::Entity>) -> Self {
        Self(children)
    }

    /// Returns the number of children.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no children.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the children.
    pub fn iter(&self) -> std::slice::Iter<bevy_ecs::entity::Entity> {
        self.0.iter()
    }

    /// Adds a child entity.
    pub fn push(&mut self, child: bevy_ecs::entity::Entity) {
        self.0.push(child);
    }

    /// Removes a child entity.
    pub fn remove(&mut self, child: bevy_ecs::entity::Entity) -> bool {
        if let Some(pos) = self.0.iter().position(|&c| c == child) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Visibility component controlling whether an entity should be rendered.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Visibility};
///
/// let mut world = World::new();
///
/// // Spawn a visible entity
/// world.spawn(Visibility::Visible);
///
/// // Spawn a hidden entity
/// world.spawn(Visibility::Hidden);
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// The entity is visible and should be rendered.
    Visible,

    /// The entity is hidden and should not be rendered.
    Hidden,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

impl Visibility {
    /// Returns true if the entity is visible.
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Visible)
    }

    /// Returns true if the entity is hidden.
    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden)
    }

    /// Sets the visibility to visible.
    pub fn show(&mut self) {
        *self = Self::Visible;
    }

    /// Sets the visibility to hidden.
    pub fn hide(&mut self) {
        *self = Self::Hidden;
    }
}

/// A marker component indicating that an entity is active/enabled.
///
/// This is useful for temporarily disabling entities without removing them.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Active};
///
/// let mut world = World::new();
///
/// // Spawn an active entity
/// let entity = world.spawn(Active);
///
/// // Disable it by removing the Active component
/// world.remove_component::<Active>(entity);
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Active;

/// A marker component for entities that should not be saved/serialized.
///
/// Useful for temporary entities like particles, debug visualizations, etc.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NoSave;

/// A marker component for entities that are managed by the engine.
///
/// These entities should not be directly manipulated by game code.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct EngineManaged;

/// Handle to a mesh asset stored in the graphics system.
///
/// This component references a mesh by its unique identifier. The actual
/// mesh data (vertices, indices) is managed by the graphics system.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, MeshHandle, Transform};
///
/// let mut world = World::new();
///
/// // Spawn an entity with a mesh
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     MeshHandle::new("cube"),
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshHandle {
    /// Unique identifier for the mesh.
    pub id: String,
}

impl MeshHandle {
    /// Creates a new mesh handle with the given identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Gets the mesh identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<&str> for MeshHandle {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for MeshHandle {
    fn from(id: String) -> Self {
        Self { id }
    }
}

/// Mesh component containing vertex and index data.
///
/// This component stores the actual geometry data for rendering.
/// It's typically used with procedurally generated meshes or when
/// the mesh data needs to be stored directly on the entity.
///
/// For asset-based meshes, use `MeshHandle` instead to reference
/// shared mesh data managed by the graphics system.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Mesh, Transform};
///
/// let mut world = World::new();
///
/// // Create a simple triangle mesh
/// let vertices = vec![
///     [0.0, 1.0, 0.0],   // Top
///     [-1.0, -1.0, 0.0], // Bottom-left
///     [1.0, -1.0, 0.0],  // Bottom-right
/// ];
/// let indices = vec![0, 1, 2];
///
/// world.spawn((
///     Transform::default(),
///     Mesh::new(vertices, indices),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct Mesh {
    /// Vertex positions in local space.
    pub vertices: Vec<[f32; 3]>,
    
    /// Indices defining triangles (triplets of vertex indices).
    pub indices: Vec<u16>,
    
    /// Optional vertex colors (RGB).
    pub colors: Option<Vec<[f32; 3]>>,
    
    /// Optional vertex normals.
    pub normals: Option<Vec<[f32; 3]>>,
    
    /// Optional texture coordinates (UV).
    pub uvs: Option<Vec<[f32; 2]>>,
}

impl Mesh {
    /// Creates a new mesh with positions and indices.
    pub fn new(vertices: Vec<[f32; 3]>, indices: Vec<u16>) -> Self {
        Self {
            vertices,
            indices,
            colors: None,
            normals: None,
            uvs: None,
        }
    }

    /// Creates a new mesh with positions, colors, and indices.
    pub fn with_colors(
        vertices: Vec<[f32; 3]>,
        colors: Vec<[f32; 3]>,
        indices: Vec<u16>,
    ) -> Self {
        Self {
            vertices,
            indices,
            colors: Some(colors),
            normals: None,
            uvs: None,
        }
    }

    /// Sets the vertex colors.
    pub fn set_colors(&mut self, colors: Vec<[f32; 3]>) {
        self.colors = Some(colors);
    }

    /// Sets the vertex normals.
    pub fn set_normals(&mut self, normals: Vec<[f32; 3]>) {
        self.normals = Some(normals);
    }

    /// Sets the texture coordinates.
    pub fn set_uvs(&mut self, uvs: Vec<[f32; 2]>) {
        self.uvs = Some(uvs);
    }

    /// Returns the number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of indices in the mesh.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Returns the number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_identity() {
        let transform = Transform::IDENTITY;
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);

        let matrix = transform.compute_matrix();
        assert_eq!(matrix, Mat4::IDENTITY);
    }

    #[test]
    fn test_transform_from_xyz() {
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_matrix() {
        let transform = Transform {
            translation: Vec3::new(10.0, 0.0, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };

        let matrix = transform.compute_matrix();

        // Test that a point transforms correctly
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_point(point);

        // After scaling by 2, rotating 90 degrees around Y, and translating by (10,0,0)
        // The point (1,0,0) should end up near (10,0,-2)
        assert!((transformed.x - 10.0).abs() < 0.001);
        assert!(transformed.y.abs() < 0.001);
        assert!((transformed.z - -2.0).abs() < 0.001);
    }

    #[test]
    fn test_visibility() {
        let mut vis = Visibility::default();
        assert!(vis.is_visible());

        vis.hide();
        assert!(vis.is_hidden());

        vis.show();
        assert!(vis.is_visible());
    }

    #[test]
    fn test_children() {
        use bevy_ecs::entity::Entity;

        let mut children = Children::new();
        assert!(children.is_empty());

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        children.push(entity1);
        children.push(entity2);

        assert_eq!(children.len(), 2);

        assert!(children.remove(entity1));
        assert_eq!(children.len(), 1);

        assert!(!children.remove(entity1)); // Already removed
    }

    #[test]
    fn test_mesh_handle_creation() {
        let handle = MeshHandle::new("cube");
        assert_eq!(handle.id(), "cube");
        
        let handle2: MeshHandle = "pyramid".into();
        assert_eq!(handle2.id(), "pyramid");
        
        let handle3: MeshHandle = "sphere".to_string().into();
        assert_eq!(handle3.id(), "sphere");
    }

    #[test]
    fn test_mesh_handle_equality() {
        let handle1 = MeshHandle::new("cube");
        let handle2 = MeshHandle::new("cube");
        let handle3 = MeshHandle::new("sphere");
        
        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_mesh_creation() {
        let vertices = vec![
            [0.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
        ];
        let indices = vec![0, 1, 2];
        
        let mesh = Mesh::new(vertices.clone(), indices.clone());
        
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.index_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
        assert!(mesh.colors.is_none());
        assert!(mesh.normals.is_none());
        assert!(mesh.uvs.is_none());
    }

    #[test]
    fn test_mesh_with_colors() {
        let vertices = vec![
            [0.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
        ];
        let colors = vec![
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let indices = vec![0, 1, 2];
        
        let mesh = Mesh::with_colors(vertices, colors, indices);
        
        assert_eq!(mesh.vertex_count(), 3);
        assert!(mesh.colors.is_some());
        assert_eq!(mesh.colors.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_mesh_attribute_setters() {
        let vertices = vec![[0.0, 0.0, 0.0]];
        let indices = vec![0];
        let mut mesh = Mesh::new(vertices, indices);
        
        // Test color setter
        mesh.set_colors(vec![[1.0, 0.0, 0.0]]);
        assert!(mesh.colors.is_some());
        
        // Test normal setter
        mesh.set_normals(vec![[0.0, 1.0, 0.0]]);
        assert!(mesh.normals.is_some());
        
        // Test UV setter
        mesh.set_uvs(vec![[0.5, 0.5]]);
        assert!(mesh.uvs.is_some());
    }
}
