//! Common components for the Praxis ECS.
//!
//! This module provides frequently-used components that are common across
//! most game projects. These components are designed to work together
//! to form the building blocks of game entities.
//!
//! # Lighting Components
//!
//! The ECS provides two types of light components for scene lighting:
//!
//! - **`DirectionalLight`**: Simulates distant light sources like the sun,
//!   where all light rays are parallel. The light's direction is specified
//!   but position is irrelevant.
//!
//! - **`PointLight`**: Simulates omnidirectional light sources like light bulbs,
//!   emitting light in all directions from a point. Uses the entity's Transform
//!   position and has distance-based attenuation.
//!
//! ## Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, DirectionalLight, PointLight, Transform};
//! use praxis_math::Vec3;
//!
//! let mut world = World::new();
//!
//! // Add a directional light (sun)
//! world.spawn(DirectionalLight::new(
//!     Vec3::new(0.5, -1.0, 0.3).normalize(),
//!     Vec3::new(1.0, 0.95, 0.8),
//!     1.0,
//! ));
//!
//! // Add a point light
//! world.spawn((
//!     Transform::from_xyz(0.0, 5.0, 0.0),
//!     PointLight::new(Vec3::new(1.0, 0.8, 0.6), 10.0, 20.0),
//! ));
//! ```

use bevy_ecs::component::Component;
use bevy_ecs::system::Resource;
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
    ///
    /// This method modifies the transform's rotation to face the target position.
    /// It returns `Self` to allow method chaining in builder patterns.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::Transform;
    /// use praxis_math::Vec3;
    ///
    /// // Builder pattern usage
    /// let transform = Transform::from_xyz(0.0, 10.0, 20.0)
    ///     .look_at(Vec3::ZERO, Vec3::Y);
    ///
    /// // Mutation usage
    /// let mut transform = Transform::from_xyz(5.0, 5.0, 5.0);
    /// transform.look_at(Vec3::new(10.0, 0.0, 0.0), Vec3::Y);
    /// ```
    pub fn look_at(mut self, target: Vec3, up: Vec3) -> Self {
        let forward = (target - self.translation).normalize();
        self.rotation = Quat::from_mat4(&Mat4::look_to_rh(self.translation, forward, up));
        self
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

    /// Creates a global transform from a translation vector.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            matrix: Mat4::from_translation(translation),
        }
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

/// Handle to a texture asset stored in the graphics system.
///
/// This component references a texture by its unique identifier. The actual
/// texture data is managed by the graphics system.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, TextureHandle, Transform};
///
/// let mut world = World::new();
///
/// // Spawn an entity with a texture
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     TextureHandle::new("brick"),
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureHandle {
    /// Unique identifier for the texture.
    pub id: String,
}

impl TextureHandle {
    /// Creates a new texture handle with the given identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Gets the texture identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<&str> for TextureHandle {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for TextureHandle {
    fn from(id: String) -> Self {
        Self { id }
    }
}

/// Handle to a material asset stored in the graphics system.
///
/// This component references a material by its unique identifier. The actual
/// material data (textures, properties, descriptor sets) is managed by the graphics system.
///
/// Materials define the visual appearance of surfaces, including textures, colors,
/// and physical properties like metallic and roughness values.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, MaterialHandle, MeshHandle, Transform};
///
/// let mut world = World::new();
///
/// // Spawn an entity with a mesh and material
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     MeshHandle::new("cube"),
///     MaterialHandle::new("brick"),
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialHandle {
    /// Unique identifier for the material.
    pub id: String,
}

impl MaterialHandle {
    /// Creates a new material handle with the given identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Gets the material identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<&str> for MaterialHandle {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for MaterialHandle {
    fn from(id: String) -> Self {
        Self { id }
    }
}

/// Material properties component for PBR-style rendering.
///
/// This component wraps `MaterialProperties` from the graphics system and allows
/// attaching material properties directly to entities. This is useful for objects
/// that need custom material properties without creating a named material asset.
///
/// # PBR Properties
///
/// - **Base Color**: RGBA tint multiplied with texture color
/// - **Metallic**: How metal-like the surface is (0.0 = dielectric, 1.0 = metal)
/// - **Roughness**: Surface smoothness (0.0 = mirror, 1.0 = matte)
/// - **Emissive Strength**: Self-illumination intensity
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, MaterialPropertiesComponent, MeshHandle, Transform};
///
/// let mut world = World::new();
///
/// // Spawn a shiny metallic cube
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     MeshHandle::new("cube"),
///     MaterialPropertiesComponent::default()
///         .with_metallic(0.9)
///         .with_roughness(0.1),
/// ));
/// ```
///
/// Note: This type is a newtype wrapper to avoid circular dependencies between
/// praxis_ecs and praxis_graphics. The actual MaterialProperties type is defined
/// in praxis_graphics and must be imported for the builder methods to work.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MaterialPropertiesComponent(pub praxis_graphics::MaterialProperties);

impl MaterialPropertiesComponent {
    /// Creates material properties with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base color tint.
    pub fn with_base_color(mut self, color: [f32; 4]) -> Self {
        self.0 = self.0.with_base_color(color);
        self
    }

    /// Sets the metallic factor [0.0, 1.0].
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.0 = self.0.with_metallic(metallic);
        self
    }

    /// Sets the roughness factor [0.0, 1.0].
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.0 = self.0.with_roughness(roughness);
        self
    }

    /// Sets the emissive strength.
    pub fn with_emissive_strength(mut self, strength: f32) -> Self {
        self.0 = self.0.with_emissive_strength(strength);
        self
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
    pub fn with_colors(vertices: Vec<[f32; 3]>, colors: Vec<[f32; 3]>, indices: Vec<u16>) -> Self {
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

/// Camera component marking an entity as a camera.
///
/// This component, combined with a projection component, enables an entity to act as a camera
/// for rendering. The camera uses its Transform to compute the view matrix.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Camera, PerspectiveProjection, Transform};
///
/// let mut world = World::new();
///
/// // Create a perspective camera
/// world.spawn((
///     Transform::from_xyz(0.0, 5.0, 10.0),
///     Camera::default(),
///     PerspectiveProjection::default(),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct Camera {
    /// Whether this camera is currently active.
    pub is_active: bool,

    /// The rendering priority order. Higher priority cameras render last.
    pub priority: i32,
}

impl Camera {
    /// Creates a new active camera with default priority.
    pub fn new() -> Self {
        Self {
            is_active: true,
            priority: 0,
        }
    }

    /// Creates a new camera with the specified priority.
    pub fn with_priority(priority: i32) -> Self {
        Self {
            is_active: true,
            priority,
        }
    }

    /// Returns true if the camera is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Activates the camera.
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// Deactivates the camera.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Perspective projection component for cameras.
///
/// Defines a perspective projection with field of view, aspect ratio, and near/far planes.
/// This is the most common projection type for 3D games.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Camera, PerspectiveProjection, Transform};
///
/// let mut world = World::new();
///
/// world.spawn((
///     Transform::from_xyz(0.0, 5.0, 10.0),
///     Camera::default(),
///     PerspectiveProjection {
///         fov: 60.0_f32.to_radians(),
///         aspect_ratio: 16.0 / 9.0,
///         near: 0.1,
///         far: 1000.0,
///     },
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct PerspectiveProjection {
    /// Vertical field of view in radians.
    pub fov: f32,

    /// Aspect ratio (width / height).
    pub aspect_ratio: f32,

    /// Near clipping plane distance.
    pub near: f32,

    /// Far clipping plane distance.
    pub far: f32,
}

impl PerspectiveProjection {
    /// Creates a new perspective projection.
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            aspect_ratio,
            near,
            far,
        }
    }

    /// Computes the projection matrix.
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    /// Updates the aspect ratio (e.g., when window is resized).
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Orthographic projection component for cameras.
///
/// Defines an orthographic projection with a defined volume. Objects maintain their size
/// regardless of distance from the camera. Common for 2D games, UI, and certain 3D scenarios
/// like strategy games.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Camera, OrthographicProjection, Transform};
///
/// let mut world = World::new();
///
/// world.spawn((
///     Transform::from_xyz(0.0, 10.0, 0.0),
///     Camera::default(),
///     OrthographicProjection::new(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct OrthographicProjection {
    /// Left edge of the view volume.
    pub left: f32,

    /// Right edge of the view volume.
    pub right: f32,

    /// Bottom edge of the view volume.
    pub bottom: f32,

    /// Top edge of the view volume.
    pub top: f32,

    /// Near clipping plane distance.
    pub near: f32,

    /// Far clipping plane distance.
    pub far: f32,
}

impl OrthographicProjection {
    /// Creates a new orthographic projection.
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    /// Creates an orthographic projection centered at origin with given dimensions.
    pub fn from_size(width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            left: -width / 2.0,
            right: width / 2.0,
            bottom: -height / 2.0,
            top: height / 2.0,
            near,
            far,
        }
    }

    /// Computes the projection matrix.
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }

    /// Updates the projection bounds (e.g., when window is resized).
    pub fn set_bounds(&mut self, left: f32, right: f32, bottom: f32, top: f32) {
        self.left = left;
        self.right = right;
        self.bottom = bottom;
        self.top = top;
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Component storing computed camera matrices.
///
/// This component is automatically updated by the camera system based on the Transform
/// and projection components. It stores the view matrix, projection matrix, and their
/// combined view-projection matrix for efficient rendering.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Query, CameraMatrices};
///
/// fn rendering_system(cameras: Query<&CameraMatrices>) {
///     for matrices in cameras.iter() {
///         // Use matrices.view_projection for rendering
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct CameraMatrices {
    /// The view matrix (transforms from world space to view space).
    pub view: Mat4,

    /// The projection matrix (transforms from view space to clip space).
    pub projection: Mat4,

    /// The combined view-projection matrix.
    pub view_projection: Mat4,
}

impl CameraMatrices {
    /// Creates camera matrices from view and projection matrices.
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        Self {
            view,
            projection,
            view_projection: projection * view,
        }
    }

    /// Updates the matrices.
    pub fn update(&mut self, view: Mat4, projection: Mat4) {
        self.view = view;
        self.projection = projection;
        self.view_projection = projection * view;
    }
}

impl Default for CameraMatrices {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
            view_projection: Mat4::IDENTITY,
        }
    }
}

/// Directional light component representing a light source at infinite distance.
///
/// Directional lights simulate distant light sources like the sun, where all
/// light rays are parallel. The position of the entity is ignored; only the
/// direction matters.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, DirectionalLight, Transform};
/// use praxis_math::{Vec3, Quat};
///
/// let mut world = World::new();
///
/// // Create a directional light pointing down and forward (like afternoon sun)
/// let direction = Vec3::new(0.5, -1.0, 0.5).normalize();
/// world.spawn((
///     DirectionalLight {
///         direction,
///         color: Vec3::new(1.0, 0.95, 0.8),
///         intensity: 1.0,
///     },
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct DirectionalLight {
    /// Direction the light is shining (should be normalized).
    pub direction: Vec3,

    /// RGB color of the light.
    pub color: Vec3,

    /// Intensity multiplier for the light.
    pub intensity: f32,
}

impl DirectionalLight {
    /// Creates a new directional light.
    ///
    /// # Arguments
    ///
    /// * `direction` - Direction the light is shining (will be normalized)
    /// * `color` - RGB color of the light
    /// * `intensity` - Intensity multiplier
    pub fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
        }
    }

    /// Creates a default white directional light pointing down.
    pub fn white() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
        }
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::white()
    }
}

/// Point light component representing an omnidirectional light source.
///
/// Point lights emit light in all directions from a single point in space,
/// like a light bulb. The light intensity falls off with distance.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, PointLight, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Create a warm point light at position (0, 5, 0)
/// world.spawn((
///     Transform::from_xyz(0.0, 5.0, 0.0),
///     PointLight {
///         color: Vec3::new(1.0, 0.8, 0.6),
///         intensity: 10.0,
///         range: 20.0,
///     },
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct PointLight {
    /// RGB color of the light.
    pub color: Vec3,

    /// Intensity of the light at the source.
    pub intensity: f32,

    /// Maximum range of the light. Beyond this distance, light contribution is zero.
    pub range: f32,
}

impl PointLight {
    /// Creates a new point light.
    ///
    /// # Arguments
    ///
    /// * `color` - RGB color of the light
    /// * `intensity` - Intensity of the light
    /// * `range` - Maximum effective range
    pub fn new(color: Vec3, intensity: f32, range: f32) -> Self {
        Self {
            color,
            intensity,
            range,
        }
    }

    /// Creates a default white point light.
    pub fn white() -> Self {
        Self {
            color: Vec3::ONE,
            intensity: 1.0,
            range: 10.0,
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self::white()
    }
}

/// Lighting data collected from light components in the scene.
///
/// This resource stores lighting information gathered from DirectionalLight and PointLight
/// components by the `gather_lighting_system`. It provides an intermediate representation
/// that can be consumed by the render system to update lighting uniforms.
///
/// The data includes both the light properties (color, intensity, etc.) and their
/// world-space transforms, which are computed by combining the light entities' Transform
/// or GlobalTransform components.
///
/// # Usage
///
/// 1. Add this resource to your ECS world:
///    ```rust,no_run
///    use praxis_ecs::{World, LightingData};
///    let mut world = World::new();
///    world.insert_resource(LightingData::default());
///    ```
///
/// 2. Run `gather_lighting_system` each frame to update the resource:
///    ```rust,no_run
///    use praxis_ecs::{Schedule, IntoSystemConfigs};
///    use praxis_ecs::systems::gather_lighting_system;
///    
///    let mut schedule = Schedule::default();
///    schedule.add_systems(gather_lighting_system);
///    ```
///
/// 3. Access the resource in your render system:
///    ```rust,no_run
///    use praxis_ecs::{Res, LightingData};
///    
///    fn render_system(lighting_data: Res<LightingData>) {
///        for light in &lighting_data.directional_lights {
///            // Use light.direction, light.color, light.intensity
///        }
///        for light in &lighting_data.point_lights {
///            // Use light.position, light.color, light.intensity, light.range
///        }
///    }
///    ```
#[derive(Resource, Debug, Clone)]
pub struct LightingData {
    /// Collected directional light data with world-space directions.
    ///
    /// Each entry contains the light's direction (from the DirectionalLight component,
    /// potentially transformed by rotation if the entity has a Transform), color, and intensity.
    pub directional_lights: Vec<DirectionalLightInfo>,

    /// Collected point light data with world-space positions.
    ///
    /// Each entry contains the light's position (from the entity's Transform or GlobalTransform),
    /// color, intensity, and range.
    pub point_lights: Vec<PointLightInfo>,

    /// Global ambient light color.
    ///
    /// This is a constant base illumination applied to all objects to prevent them from
    /// being completely black in shadow. Default is a soft gray (0.1, 0.1, 0.1).
    pub ambient_color: Vec3,
}

impl Default for LightingData {
    fn default() -> Self {
        Self::new()
    }
}

impl LightingData {
    /// Creates a new empty lighting data collection with default ambient lighting.
    pub fn new() -> Self {
        Self {
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            ambient_color: Vec3::new(0.1, 0.1, 0.1),
        }
    }

    /// Clears all collected lighting data.
    ///
    /// This should be called at the beginning of each frame before gathering new lights.
    pub fn clear(&mut self) {
        self.directional_lights.clear();
        self.point_lights.clear();
    }

    /// Returns the number of directional lights.
    pub fn directional_light_count(&self) -> usize {
        self.directional_lights.len()
    }

    /// Returns the number of point lights.
    pub fn point_light_count(&self) -> usize {
        self.point_lights.len()
    }
}

/// Information about a directional light collected from the ECS.
///
/// Contains the light properties along with its world-space direction.
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLightInfo {
    /// Direction the light is shining in world space (normalized).
    pub direction: Vec3,

    /// RGB color of the light.
    pub color: Vec3,

    /// Intensity multiplier for the light.
    pub intensity: f32,
}

/// Information about a point light collected from the ECS.
///
/// Contains the light properties along with its world-space position.
#[derive(Debug, Clone, Copy)]
pub struct PointLightInfo {
    /// World-space position of the light.
    pub position: Vec3,

    /// RGB color of the light.
    pub color: Vec3,

    /// Intensity of the light at the source.
    pub intensity: f32,

    /// Maximum range of the light in world units.
    pub range: f32,
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

        let _matrix = transform.compute_matrix();

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
        let vertices = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
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
        let vertices = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
        let colors = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
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

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new();
        assert!(camera.is_active());
        assert_eq!(camera.priority, 0);

        let camera2 = Camera::with_priority(5);
        assert!(camera2.is_active());
        assert_eq!(camera2.priority, 5);
    }

    #[test]
    fn test_camera_activation() {
        let mut camera = Camera::new();
        assert!(camera.is_active());

        camera.deactivate();
        assert!(!camera.is_active());

        camera.activate();
        assert!(camera.is_active());
    }

    #[test]
    fn test_perspective_projection() {
        let proj = PerspectiveProjection::new(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);

        assert_eq!(proj.fov, 60.0_f32.to_radians());
        assert_eq!(proj.aspect_ratio, 16.0 / 9.0);
        assert_eq!(proj.near, 0.1);
        assert_eq!(proj.far, 1000.0);

        let matrix = proj.compute_matrix();
        assert_ne!(matrix, Mat4::IDENTITY);
    }

    #[test]
    fn test_perspective_projection_aspect_ratio() {
        let mut proj = PerspectiveProjection::default();
        assert_eq!(proj.aspect_ratio, 16.0 / 9.0);

        proj.set_aspect_ratio(4.0 / 3.0);
        assert_eq!(proj.aspect_ratio, 4.0 / 3.0);
    }

    #[test]
    fn test_orthographic_projection() {
        let proj = OrthographicProjection::new(-10.0, 10.0, -5.0, 5.0, 0.1, 100.0);

        assert_eq!(proj.left, -10.0);
        assert_eq!(proj.right, 10.0);
        assert_eq!(proj.bottom, -5.0);
        assert_eq!(proj.top, 5.0);
        assert_eq!(proj.near, 0.1);
        assert_eq!(proj.far, 100.0);

        let matrix = proj.compute_matrix();
        assert_ne!(matrix, Mat4::IDENTITY);
    }

    #[test]
    fn test_orthographic_projection_from_size() {
        let proj = OrthographicProjection::from_size(20.0, 10.0, 0.1, 100.0);

        assert_eq!(proj.left, -10.0);
        assert_eq!(proj.right, 10.0);
        assert_eq!(proj.bottom, -5.0);
        assert_eq!(proj.top, 5.0);
    }

    #[test]
    fn test_orthographic_projection_set_bounds() {
        let mut proj = OrthographicProjection::default();

        proj.set_bounds(-20.0, 20.0, -15.0, 15.0);

        assert_eq!(proj.left, -20.0);
        assert_eq!(proj.right, 20.0);
        assert_eq!(proj.bottom, -15.0);
        assert_eq!(proj.top, 15.0);
    }

    #[test]
    fn test_camera_matrices() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let projection = Mat4::perspective_rh(70.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);

        let matrices = CameraMatrices::new(view, projection);

        assert_eq!(matrices.view, view);
        assert_eq!(matrices.projection, projection);
        assert_eq!(matrices.view_projection, projection * view);
    }

    #[test]
    fn test_camera_matrices_update() {
        let mut matrices = CameraMatrices::default();

        let view = Mat4::look_at_rh(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y);
        let projection = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);

        matrices.update(view, projection);

        assert_eq!(matrices.view, view);
        assert_eq!(matrices.projection, projection);
        assert_eq!(matrices.view_projection, projection * view);
    }

    #[test]
    fn test_directional_light_creation() {
        let light =
            DirectionalLight::new(Vec3::new(0.5, -1.0, 0.5), Vec3::new(1.0, 0.95, 0.8), 1.0);

        // Direction should be normalized
        assert!((light.direction.length() - 1.0).abs() < 0.001);
        assert_eq!(light.color, Vec3::new(1.0, 0.95, 0.8));
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_directional_light_default() {
        let light = DirectionalLight::default();
        assert_eq!(light.direction, Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(light.color, Vec3::ONE);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_point_light_creation() {
        let light = PointLight::new(Vec3::new(1.0, 0.8, 0.6), 10.0, 20.0);

        assert_eq!(light.color, Vec3::new(1.0, 0.8, 0.6));
        assert_eq!(light.intensity, 10.0);
        assert_eq!(light.range, 20.0);
    }

    #[test]
    fn test_point_light_default() {
        let light = PointLight::default();
        assert_eq!(light.color, Vec3::ONE);
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.range, 10.0);
    }

    #[test]
    fn test_material_handle_creation() {
        let handle = MaterialHandle::new("brick");
        assert_eq!(handle.id(), "brick");

        let handle2: MaterialHandle = "metal".into();
        assert_eq!(handle2.id(), "metal");

        let handle3: MaterialHandle = "wood".to_string().into();
        assert_eq!(handle3.id(), "wood");
    }

    #[test]
    fn test_material_handle_equality() {
        let handle1 = MaterialHandle::new("brick");
        let handle2 = MaterialHandle::new("brick");
        let handle3 = MaterialHandle::new("metal");

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_name_component() {
        let name1 = Name::new("Player");
        assert_eq!(name1.as_str(), "Player");

        let name2: Name = "Enemy".into();
        assert_eq!(name2.as_str(), "Enemy");

        let name3: Name = "Boss".to_string().into();
        assert_eq!(name3.as_str(), "Boss");
    }

    #[test]
    fn test_transform_from_translation() {
        let transform = Transform::from_translation(Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(transform.translation, Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_from_rotation() {
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let transform = Transform::from_rotation(rotation);
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, rotation);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_from_scale() {
        let scale = Vec3::new(2.0, 3.0, 4.0);
        let transform = Transform::from_scale(scale);
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, scale);
    }

    #[test]
    fn test_transform_direction() {
        let transform = Transform {
            translation: Vec3::new(10.0, 0.0, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            scale: Vec3::new(2.0, 1.0, 1.0),
        };

        let direction = Vec3::new(1.0, 0.0, 0.0);
        let transformed = transform.transform_direction(direction);

        assert!(transformed.x.abs() < 0.001);
        assert!(transformed.y.abs() < 0.001);
        assert!((transformed.z.abs() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_look_at_builder_pattern() {
        // Test builder pattern chaining
        let transform = Transform::from_xyz(0.0, 5.0, 10.0)
            .look_at(Vec3::ZERO, Vec3::Y);
        
        // The camera should be looking at the origin from above
        let forward = transform.rotation * Vec3::NEG_Z;
        let expected_forward = (Vec3::ZERO - transform.translation).normalize();
        
        // Check that forward direction is approximately correct
        assert!((forward.dot(expected_forward) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_transform_look_at_mutation() {
        // Test mutation pattern
        let mut transform = Transform::from_xyz(5.0, 0.0, 0.0);
        transform = transform.look_at(Vec3::new(10.0, 0.0, 0.0), Vec3::Y);
        
        // The camera should be looking at (10, 0, 0) from (5, 0, 0)
        let forward = transform.rotation * Vec3::NEG_Z;
        let expected_forward = Vec3::new(1.0, 0.0, 0.0);
        
        // Check that forward direction is approximately correct
        assert!((forward.dot(expected_forward) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_global_transform_from_transform() {
        let transform = Transform::from_xyz(5.0, 10.0, 15.0);
        let global_transform = GlobalTransform::from(transform);

        let translation = global_transform.translation();
        assert_eq!(translation, Vec3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_global_transform_from_translation() {
        let translation = Vec3::new(5.0, 10.0, 15.0);
        let global_transform = GlobalTransform::from_translation(translation);

        assert_eq!(global_transform.translation(), translation);
        let scale = global_transform.scale();
        assert!((scale.x - 1.0).abs() < 0.001);
        assert!((scale.y - 1.0).abs() < 0.001);
        assert!((scale.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_global_transform_from_scale_rotation_translation() {
        let scale = Vec3::new(2.0, 2.0, 2.0);
        let rotation = Quat::IDENTITY;
        let translation = Vec3::new(10.0, 20.0, 30.0);

        let global_transform =
            GlobalTransform::from_scale_rotation_translation(scale, rotation, translation);

        assert_eq!(global_transform.translation(), translation);
        let extracted_scale = global_transform.scale();
        assert!((extracted_scale.x - 2.0).abs() < 0.001);
        assert!((extracted_scale.y - 2.0).abs() < 0.001);
        assert!((extracted_scale.z - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_global_transform_transform_direction() {
        let matrix = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let global_transform = GlobalTransform::from_matrix(matrix);

        let direction = Vec3::new(1.0, 0.0, 0.0);
        let transformed = global_transform.transform_direction(direction);

        assert!(transformed.x.abs() < 0.001);
        assert!(transformed.y.abs() < 0.001);
        assert!(transformed.z.abs() > 0.99);
    }

    #[test]
    fn test_parent_component() {
        use bevy_ecs::entity::Entity;
        let parent_entity = Entity::from_raw(42);
        let parent = Parent(parent_entity);
        assert_eq!(parent.0, parent_entity);
    }

    #[test]
    fn test_children_component_operations() {
        use bevy_ecs::entity::Entity;

        let mut children = Children::new();
        assert!(children.is_empty());
        assert_eq!(children.len(), 0);

        let child1 = Entity::from_raw(1);
        let child2 = Entity::from_raw(2);
        let child3 = Entity::from_raw(3);

        children.push(child1);
        children.push(child2);
        children.push(child3);

        assert_eq!(children.len(), 3);
        assert!(!children.is_empty());

        let mut iter = children.iter();
        assert_eq!(iter.next(), Some(&child1));
        assert_eq!(iter.next(), Some(&child2));
        assert_eq!(iter.next(), Some(&child3));
        assert_eq!(iter.next(), None);

        assert!(children.remove(child2));
        assert_eq!(children.len(), 2);
        assert!(!children.remove(child2));
    }

    #[test]
    fn test_children_with_children() {
        use bevy_ecs::entity::Entity;

        let child1 = Entity::from_raw(1);
        let child2 = Entity::from_raw(2);

        let children = Children::with_children(vec![child1, child2]);
        assert_eq!(children.len(), 2);
        assert_eq!(children.0[0], child1);
        assert_eq!(children.0[1], child2);
    }

    #[test]
    fn test_active_component() {
        let active = Active;
        assert_eq!(std::mem::size_of_val(&active), 0);
    }

    #[test]
    fn test_texture_handle() {
        let handle = TextureHandle::new("brick_texture");
        assert_eq!(handle.id(), "brick_texture");

        let handle2: TextureHandle = "wood_texture".into();
        assert_eq!(handle2.id(), "wood_texture");

        let handle3: TextureHandle = "metal_texture".to_string().into();
        assert_eq!(handle3.id(), "metal_texture");
    }

    #[test]
    fn test_texture_handle_equality() {
        let handle1 = TextureHandle::new("brick");
        let handle2 = TextureHandle::new("brick");
        let handle3 = TextureHandle::new("wood");

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_material_properties_component() {
        let props = MaterialPropertiesComponent::new()
            .with_base_color([1.0, 0.5, 0.0, 1.0])
            .with_metallic(0.8)
            .with_roughness(0.2)
            .with_emissive_strength(1.5);

        assert_eq!(props.0.base_color, [1.0, 0.5, 0.0, 1.0]);
        assert_eq!(props.0.metallic, 0.8);
        assert_eq!(props.0.roughness, 0.2);
        assert_eq!(props.0.emissive_strength, 1.5);
    }

    #[test]
    fn test_lighting_data_operations() {
        let mut lighting_data = LightingData::new();

        assert_eq!(lighting_data.directional_light_count(), 0);
        assert_eq!(lighting_data.point_light_count(), 0);
        assert_eq!(lighting_data.ambient_color, Vec3::new(0.1, 0.1, 0.1));

        lighting_data.directional_lights.push(DirectionalLightInfo {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
        });

        lighting_data.point_lights.push(PointLightInfo {
            position: Vec3::ZERO,
            color: Vec3::ONE,
            intensity: 1.0,
            range: 10.0,
        });

        assert_eq!(lighting_data.directional_light_count(), 1);
        assert_eq!(lighting_data.point_light_count(), 1);

        lighting_data.clear();

        assert_eq!(lighting_data.directional_light_count(), 0);
        assert_eq!(lighting_data.point_light_count(), 0);
    }

    #[test]
    fn test_directional_light_white() {
        let light = DirectionalLight::white();
        assert_eq!(light.direction, Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(light.color, Vec3::ONE);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_point_light_white() {
        let light = PointLight::white();
        assert_eq!(light.color, Vec3::ONE);
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.range, 10.0);
    }

    #[test]
    fn test_skybox_component() {
        let skybox = Skybox::new("sky_cubemap");
        assert_eq!(skybox.cubemap_id(), "sky_cubemap");

        let skybox2: Skybox = "night_sky".into();
        assert_eq!(skybox2.cubemap_id(), "night_sky");
    }

    #[test]
    fn test_particle_emitter_creation() {
        let emitter = ParticleEmitter::new("fire");
        assert_eq!(emitter.id(), "fire");

        let emitter2: ParticleEmitter = "smoke".into();
        assert_eq!(emitter2.id(), "smoke");

        let emitter3: ParticleEmitter = "explosion".to_string().into();
        assert_eq!(emitter3.id(), "explosion");
    }

    #[test]
    fn test_particle_emitter_equality() {
        let emitter1 = ParticleEmitter::new("fire");
        let emitter2 = ParticleEmitter::new("fire");
        let emitter3 = ParticleEmitter::new("smoke");

        assert_eq!(emitter1, emitter2);
        assert_ne!(emitter1, emitter3);
    }

    #[test]
    fn test_environment_probe_creation() {
        let probe = EnvironmentProbe::new("test_probe");
        assert_eq!(probe.id(), "test_probe");
        assert_eq!(probe.resolution, 256);
        assert_eq!(probe.near_clip, 0.1);
        assert_eq!(probe.far_clip, 100.0);
        assert!(probe.is_enabled());
        assert_eq!(probe.influence_radius, 50.0);
        assert_eq!(probe.intensity, 1.0);
    }

    #[test]
    fn test_environment_probe_builder() {
        let probe = EnvironmentProbe::new("custom_probe")
            .with_resolution(512)
            .with_near_clip(0.5)
            .with_far_clip(200.0)
            .with_influence_radius(100.0)
            .with_intensity(1.5)
            .with_update_every_n_frames(30);

        assert_eq!(probe.resolution, 512);
        assert_eq!(probe.near_clip, 0.5);
        assert_eq!(probe.far_clip, 200.0);
        assert_eq!(probe.influence_radius, 100.0);
        assert_eq!(probe.intensity, 1.5);
        assert_eq!(
            probe.update_mode,
            EnvironmentProbeUpdateMode::EveryNFrames(30)
        );
    }

    #[test]
    fn test_environment_probe_enable_disable() {
        let mut probe = EnvironmentProbe::new("toggle_probe");
        assert!(probe.is_enabled());

        probe.disable();
        assert!(!probe.is_enabled());

        probe.enable();
        assert!(probe.is_enabled());
    }

    #[test]
    fn test_environment_probe_update_modes() {
        let probe_once = EnvironmentProbe::new("once").with_update_once();
        assert_eq!(probe_once.update_mode, EnvironmentProbeUpdateMode::Once);

        let probe_every_n = EnvironmentProbe::new("every_n").with_update_every_n_frames(60);
        assert_eq!(
            probe_every_n.update_mode,
            EnvironmentProbeUpdateMode::EveryNFrames(60)
        );

        let probe_manual = EnvironmentProbe::new("manual").with_update_manual();
        assert_eq!(probe_manual.update_mode, EnvironmentProbeUpdateMode::Manual);

        let probe_continuous = EnvironmentProbe::new("continuous").with_update_continuous();
        assert_eq!(
            probe_continuous.update_mode,
            EnvironmentProbeUpdateMode::Continuous
        );
    }

    #[test]
    fn test_environment_probe_default() {
        let probe = EnvironmentProbe::default();
        assert_eq!(probe.id(), "probe");
        assert_eq!(probe.resolution, 256);
        assert!(probe.is_enabled());
    }
}

/// Skybox component for rendering a background skybox.
///
/// A skybox is a large cube textured with a cubemap that surrounds the entire scene,
/// creating the illusion of a distant environment (sky, space, etc.). It's rendered
/// with reversed depth to ensure it always appears behind all other geometry.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Skybox};
///
/// let mut world = World::new();
///
/// // Spawn a skybox entity
/// world.spawn(Skybox::new("day_sky"));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Skybox {
    /// Unique identifier for the cubemap texture.
    pub cubemap_id: String,
}

impl Skybox {
    /// Creates a new skybox with the given cubemap identifier.
    pub fn new(cubemap_id: impl Into<String>) -> Self {
        Self {
            cubemap_id: cubemap_id.into(),
        }
    }

    /// Gets the cubemap identifier.
    pub fn cubemap_id(&self) -> &str {
        &self.cubemap_id
    }
}

impl From<&str> for Skybox {
    fn from(cubemap_id: &str) -> Self {
        Self::new(cubemap_id)
    }
}

impl From<String> for Skybox {
    fn from(cubemap_id: String) -> Self {
        Self { cubemap_id }
    }
}

/// Light probe component for capturing irradiance at a point.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LightProbeComponent {
    pub id: String,
}

impl LightProbeComponent {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<&str> for LightProbeComponent {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for LightProbeComponent {
    fn from(id: String) -> Self {
        Self { id }
    }
}

/// Area light component for polygon lights.
#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct AreaLightComponent {
    pub light_type: AreaLightType,
    pub color: Vec3,
    pub intensity: f32,
    pub two_sided: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum AreaLightType {
    Rectangle { width: f32, height: f32 },
    Disk { radius: f32 },
    Sphere { radius: f32 },
}

#[allow(dead_code)]
impl AreaLightComponent {
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self {
            light_type: AreaLightType::Rectangle { width, height },
            color: Vec3::ONE,
            intensity: 1.0,
            two_sided: false,
        }
    }

    pub fn disk(radius: f32) -> Self {
        Self {
            light_type: AreaLightType::Disk { radius },
            color: Vec3::ONE,
            intensity: 1.0,
            two_sided: false,
        }
    }

    pub fn sphere(radius: f32) -> Self {
        Self {
            light_type: AreaLightType::Sphere { radius },
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

    pub fn with_two_sided(mut self, two_sided: bool) -> Self {
        self.two_sided = two_sided;
        self
    }
}

impl Default for AreaLightComponent {
    fn default() -> Self {
        Self::rectangle(1.0, 1.0)
    }
}

/// Bounding volume component for spatial optimization.
///
/// Stores an axis-aligned bounding box (AABB) used for frustum culling,
/// occlusion culling, and spatial queries.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, BoundingBox, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Spawn an entity with a bounding box
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     BoundingBox::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
pub struct BoundingBox {
    /// Minimum corner of the box.
    pub min: Vec3,
    /// Maximum corner of the box.
    pub max: Vec3,
}

impl BoundingBox {
    /// Creates a new bounding box from minimum and maximum points.
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates a bounding box from center and half-extents.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Returns the center of the bounding box.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-extents of the bounding box.
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Returns the size of the bounding box.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::from_center_half_extents(Vec3::ZERO, Vec3::ONE)
    }
}

/// LOD (Level of Detail) component.
///
/// Specifies which LOD group this entity belongs to for distance-based mesh switching.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, LodComponent, MeshHandle, Transform};
///
/// let mut world = World::new();
///
/// // Spawn an entity that uses LOD
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     MeshHandle::new("tree_high"),
///     LodComponent::new("tree"),
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LodComponent {
    /// Name of the LOD group this entity belongs to.
    pub group_name: String,
}

impl LodComponent {
    /// Creates a new LOD component.
    pub fn new(group_name: impl Into<String>) -> Self {
        Self {
            group_name: group_name.into(),
        }
    }

    /// Gets the LOD group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }
}

impl From<&str> for LodComponent {
    fn from(group_name: &str) -> Self {
        Self::new(group_name)
    }
}

impl From<String> for LodComponent {
    fn from(group_name: String) -> Self {
        Self { group_name }
    }
}

/// LOD (Level of Detail) group component for managing mesh variants at different detail levels.
///
/// This component wraps the `LodGroup` from the graphics system and allows attaching
/// LOD management directly to entities. The LOD system automatically selects the appropriate
/// mesh variant based on distance from the camera.
///
/// # LOD System Benefits
///
/// - **Performance**: Reduces triangle count for distant objects
/// - **Visual Quality**: Maintains high detail where it matters (near camera)
/// - **Smooth Transitions**: Alpha-blended transitions between LOD levels prevent popping
/// - **Flexible Configuration**: Per-entity LOD settings and global LOD bias
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Transform, LodGroupComponent};
/// use praxis_graphics::lod::{LodLevel, LodGroup};
///
/// let mut world = World::new();
///
/// // Create LOD group with 3 detail levels
/// let lod_group = LodGroup::new(vec![
///     LodLevel::new("tree_high", 0.0, 20.0),    // High detail: 0-20 units
///     LodLevel::new("tree_medium", 20.0, 50.0), // Medium: 20-50 units
///     LodLevel::new("tree_low", 50.0, 100.0),   // Low detail: 50-100 units
/// ]);
///
/// // Spawn entity with LOD
/// world.spawn((
///     Transform::from_xyz(10.0, 0.0, 10.0),
///     LodGroupComponent(lod_group),
/// ));
/// ```
///
/// Note: This type is a newtype wrapper to avoid circular dependencies between
/// praxis_ecs and praxis_graphics. The actual LodGroup type is defined in
/// praxis_graphics and must be imported for construction.
#[derive(Component, Debug, Clone)]
pub struct LodGroupComponent(pub praxis_graphics::lod::LodGroup);

impl LodGroupComponent {
    /// Creates a new LOD group component from a graphics LOD group.
    pub fn new(lod_group: praxis_graphics::lod::LodGroup) -> Self {
        Self(lod_group)
    }

    /// Gets a reference to the underlying LOD group.
    pub fn lod_group(&self) -> &praxis_graphics::lod::LodGroup {
        &self.0
    }

    /// Gets a mutable reference to the underlying LOD group.
    pub fn lod_group_mut(&mut self) -> &mut praxis_graphics::lod::LodGroup {
        &mut self.0
    }

    /// Gets the mesh ID that should currently be rendered.
    pub fn current_mesh_id(&self) -> &str {
        self.0.current_mesh_id()
    }

    /// Gets all meshes that should be rendered (including transitions).
    pub fn get_render_meshes(&self) -> Vec<(&str, f32)> {
        self.0.get_render_meshes()
    }

    /// Checks if the LOD group is currently transitioning between levels.
    pub fn is_transitioning(&self) -> bool {
        self.0.is_transitioning()
    }

    /// Gets the current LOD level index.
    pub fn current_level(&self) -> usize {
        self.0.current_level()
    }

    /// Gets the number of LOD levels.
    pub fn level_count(&self) -> usize {
        self.0.level_count()
    }
}

/// Particle emitter component for spawning and managing particles.
///
/// This component marks an entity as a particle emitter, which spawns and manages
/// particles according to its configuration. The particle system in the graphics
/// module handles the actual particle simulation and rendering.
///
/// # Features
///
/// - **Multiple Emitter Shapes**: Point, sphere, box, circle, cone
/// - **Particle Properties**: Lifetime, velocity, color, size, rotation
/// - **Physical Forces**: Gravity, wind, attraction, radial forces, drag
/// - **Texture Atlases**: Support for sprite sheet animations
/// - **GPU Instancing**: Efficient rendering of thousands of particles
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, ParticleEmitter, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Create a fire particle emitter
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     ParticleEmitter::new("fire_emitter"),
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleEmitter {
    /// Unique identifier for the particle emitter configuration.
    pub id: String,
}

impl ParticleEmitter {
    /// Creates a new particle emitter with the given identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Gets the emitter identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<&str> for ParticleEmitter {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for ParticleEmitter {
    fn from(id: String) -> Self {
        Self { id }
    }
}

/// Environment probe component for image-based lighting.
///
/// Environment probes capture the surrounding environment as a cubemap and provide
/// lighting data for realistic reflections and ambient lighting. They are essential
/// for physically-based rendering with metallic and glossy surfaces.
///
/// # Features
///
/// - **Cubemap Capture**: Captures environment from 6 camera angles
/// - **Diffuse Irradiance**: Precomputed ambient lighting
/// - **Specular Reflection**: Prefiltered reflections for varying roughness
/// - **Real-time Updates**: Can update dynamically for moving objects
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, EnvironmentProbe, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// // Spawn an environment probe at the scene center
/// world.spawn((
///     Transform::from_xyz(0.0, 2.0, 0.0),
///     EnvironmentProbe::new("main_probe")
///         .with_resolution(512)
///         .with_update_every_n_frames(60),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct EnvironmentProbe {
    /// Unique identifier for this probe.
    pub id: String,

    /// Resolution of each cubemap face (e.g., 256, 512, 1024).
    pub resolution: u32,

    /// Near clipping plane for capture.
    pub near_clip: f32,

    /// Far clipping plane for capture.
    pub far_clip: f32,

    /// Update mode for this probe.
    pub update_mode: EnvironmentProbeUpdateMode,

    /// Whether this probe is currently enabled.
    pub enabled: bool,

    /// Influence radius - objects within this distance use this probe.
    pub influence_radius: f32,

    /// Intensity multiplier for this probe's contribution.
    pub intensity: f32,
}

impl EnvironmentProbe {
    /// Creates a new environment probe with the given identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            resolution: 256,
            near_clip: 0.1,
            far_clip: 100.0,
            update_mode: EnvironmentProbeUpdateMode::Once,
            enabled: true,
            influence_radius: 50.0,
            intensity: 1.0,
        }
    }

    /// Sets the cubemap resolution per face.
    pub fn with_resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }

    /// Sets the near clipping plane.
    pub fn with_near_clip(mut self, near_clip: f32) -> Self {
        self.near_clip = near_clip;
        self
    }

    /// Sets the far clipping plane.
    pub fn with_far_clip(mut self, far_clip: f32) -> Self {
        self.far_clip = far_clip;
        self
    }

    /// Sets the probe to update once when created.
    pub fn with_update_once(mut self) -> Self {
        self.update_mode = EnvironmentProbeUpdateMode::Once;
        self
    }

    /// Sets the probe to update every N frames.
    pub fn with_update_every_n_frames(mut self, n: u32) -> Self {
        self.update_mode = EnvironmentProbeUpdateMode::EveryNFrames(n);
        self
    }

    /// Sets the probe to update only when manually requested.
    pub fn with_update_manual(mut self) -> Self {
        self.update_mode = EnvironmentProbeUpdateMode::Manual;
        self
    }

    /// Sets the probe to update continuously every frame.
    pub fn with_update_continuous(mut self) -> Self {
        self.update_mode = EnvironmentProbeUpdateMode::Continuous;
        self
    }

    /// Sets the influence radius.
    pub fn with_influence_radius(mut self, radius: f32) -> Self {
        self.influence_radius = radius;
        self
    }

    /// Sets the intensity multiplier.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Gets the probe identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Checks if the probe is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables the probe.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables the probe.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for EnvironmentProbe {
    fn default() -> Self {
        Self::new("probe")
    }
}

/// Update mode for environment probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentProbeUpdateMode {
    /// Capture once when created, never update.
    Once,

    /// Update every N frames.
    EveryNFrames(u32),

    /// Update manually when requested.
    Manual,

    /// Update continuously every frame (expensive).
    Continuous,
}

/// Component marking that an entity is currently visible to the camera after frustum culling.
///
/// This component is added/updated by the frustum culling system to entities that
/// pass visibility tests. Renderers can query for this component to only process visible entities.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Query, Visible, MeshHandle};
///
/// fn render_system(visible_entities: Query<(&MeshHandle, &Visible)>) {
///     for (mesh, _) in visible_entities.iter() {
///         // Entity is visible, render it
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Visible;

/// Component marking that an entity has been culled and is not visible to the camera.
///
/// This component is added by the frustum culling system to entities that
/// fail visibility tests. It can be used for debugging, analytics, or to avoid
/// updating non-visible entities.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Query, Culled};
///
/// fn update_system(culled_entities: Query<&Culled>) {
///     // Skip expensive updates for culled entities
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Culled;
