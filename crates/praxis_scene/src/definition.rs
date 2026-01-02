//! Scene definition structures for loading from RON format.

use praxis_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A complete scene definition loaded from RON format.
///
/// This structure represents a scene that can be spawned into the ECS world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDefinition {
    /// The name of the scene.
    pub name: String,

    /// Root entities in the scene.
    pub entities: Vec<EntityDefinition>,

    /// Optional metadata for the scene.
    #[serde(default)]
    pub metadata: SceneMetadata,
}

impl SceneDefinition {
    /// Creates a new empty scene definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
            metadata: SceneMetadata::default(),
        }
    }

    /// Adds an entity to the scene.
    pub fn add_entity(&mut self, entity: EntityDefinition) {
        self.entities.push(entity);
    }

    /// Gets the number of root entities in the scene.
    #[must_use] pub const fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Gets the total number of entities including children.
    pub fn total_entity_count(&self) -> usize {
        fn count_recursive(entity: &EntityDefinition) -> usize {
            1 + entity.children.iter().map(count_recursive).sum::<usize>()
        }

        self.entities.iter().map(count_recursive).sum()
    }
}

/// Metadata associated with a scene.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneMetadata {
    /// Optional description of the scene.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional author information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Optional version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Optional tags for categorization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Definition of an entity within a scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDefinition {
    /// Optional name for the entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional transform component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformDef>,

    /// Optional mesh handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<String>,

    /// Optional texture handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,

    /// Optional camera configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<CameraDef>,

    /// Optional directional light configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directional_light: Option<DirectionalLightDef>,

    /// Optional point light configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point_light: Option<PointLightDef>,

    /// Optional visibility setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    /// Optional active state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Child entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<EntityDefinition>,
}

impl EntityDefinition {
    /// Creates a new empty entity definition.
    #[must_use] pub const fn new() -> Self {
        Self {
            name: None,
            transform: None,
            mesh: None,
            texture: None,
            camera: None,
            directional_light: None,
            point_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }

    /// Sets the name of the entity.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the transform of the entity.
    #[must_use] pub const fn with_transform(mut self, transform: TransformDef) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Sets the mesh handle of the entity.
    #[must_use]
    pub fn with_mesh(mut self, mesh: impl Into<String>) -> Self {
        self.mesh = Some(mesh.into());
        self
    }

    /// Adds a child entity.
    #[must_use] pub fn with_child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    /// Gets the total number of entities including this one and all descendants.
    #[must_use] pub fn total_count(&self) -> usize {
        1 + self.children.iter().map(Self::total_count).sum::<usize>()
    }
}

impl Default for EntityDefinition {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform definition for scene entities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransformDef {
    /// Position in 3D space (x, y, z).
    pub translation: (f32, f32, f32),

    /// Rotation as a quaternion (x, y, z, w).
    pub rotation: (f32, f32, f32, f32),

    /// Scale factors (x, y, z).
    pub scale: (f32, f32, f32),
}

impl TransformDef {
    /// Creates a new transform definition with identity values.
    #[must_use] pub const fn identity() -> Self {
        Self {
            translation: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        }
    }

    /// Creates a transform definition from translation.
    #[must_use] pub const fn from_translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: (x, y, z),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        }
    }

    /// Converts to ECS Transform component types.
    #[must_use] pub const fn to_components(&self) -> (Vec3, Quat, Vec3) {
        let translation = Vec3::new(self.translation.0, self.translation.1, self.translation.2);
        let rotation = Quat::from_xyzw(
            self.rotation.0,
            self.rotation.1,
            self.rotation.2,
            self.rotation.3,
        );
        let scale = Vec3::new(self.scale.0, self.scale.1, self.scale.2);
        (translation, rotation, scale)
    }
}

impl Default for TransformDef {
    fn default() -> Self {
        Self::identity()
    }
}

/// Camera definition for scene entities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CameraDef {
    /// Type of camera projection.
    pub camera_type: CameraType,

    /// Field of view in radians (for perspective cameras).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fov: Option<f32>,

    /// Aspect ratio (for perspective cameras).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<f32>,

    /// Left bound (for orthographic cameras).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<f32>,

    /// Right bound (for orthographic cameras).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<f32>,

    /// Bottom bound (for orthographic cameras).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<f32>,

    /// Top bound (for orthographic cameras).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<f32>,

    /// Near clipping plane.
    pub near: f32,

    /// Far clipping plane.
    pub far: f32,

    /// Whether the camera is active.
    #[serde(default = "default_camera_active")]
    pub is_active: bool,

    /// Camera priority.
    #[serde(default)]
    pub priority: i32,
}

const fn default_camera_active() -> bool {
    true
}

impl CameraDef {
    /// Creates a perspective camera definition.
    #[must_use] pub const fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            camera_type: CameraType::Perspective,
            fov: Some(fov),
            aspect_ratio: Some(aspect_ratio),
            left: None,
            right: None,
            bottom: None,
            top: None,
            near,
            far,
            is_active: true,
            priority: 0,
        }
    }

    /// Creates an orthographic camera definition.
    #[must_use] pub const fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            camera_type: CameraType::Orthographic,
            fov: None,
            aspect_ratio: None,
            left: Some(left),
            right: Some(right),
            bottom: Some(bottom),
            top: Some(top),
            near,
            far,
            is_active: true,
            priority: 0,
        }
    }
}

/// Type of camera projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraType {
    /// Perspective projection.
    Perspective,
    /// Orthographic projection.
    Orthographic,
}

/// Directional light definition for scene entities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DirectionalLightDef {
    /// Direction the light is shining (x, y, z).
    pub direction: (f32, f32, f32),

    /// RGB color of the light.
    pub color: (f32, f32, f32),

    /// Intensity multiplier.
    pub intensity: f32,
}

impl DirectionalLightDef {
    /// Creates a new directional light definition.
    #[must_use] pub const fn new(direction: (f32, f32, f32), color: (f32, f32, f32), intensity: f32) -> Self {
        Self {
            direction,
            color,
            intensity,
        }
    }

    /// Converts to ECS component types.
    #[must_use] pub const fn to_components(&self) -> (Vec3, Vec3, f32) {
        let direction = Vec3::new(self.direction.0, self.direction.1, self.direction.2);
        let color = Vec3::new(self.color.0, self.color.1, self.color.2);
        (direction, color, self.intensity)
    }
}

impl Default for DirectionalLightDef {
    fn default() -> Self {
        Self {
            direction: (0.0, -1.0, 0.0),
            color: (1.0, 1.0, 1.0),
            intensity: 1.0,
        }
    }
}

/// Point light definition for scene entities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointLightDef {
    /// RGB color of the light.
    pub color: (f32, f32, f32),

    /// Intensity of the light.
    pub intensity: f32,

    /// Maximum range of the light.
    pub range: f32,
}

impl PointLightDef {
    /// Creates a new point light definition.
    #[must_use] pub const fn new(color: (f32, f32, f32), intensity: f32, range: f32) -> Self {
        Self {
            color,
            intensity,
            range,
        }
    }

    /// Converts to ECS component types.
    #[must_use] pub const fn to_components(&self) -> (Vec3, f32, f32) {
        let color = Vec3::new(self.color.0, self.color.1, self.color.2);
        (color, self.intensity, self.range)
    }
}

impl Default for PointLightDef {
    fn default() -> Self {
        Self {
            color: (1.0, 1.0, 1.0),
            intensity: 1.0,
            range: 10.0,
        }
    }
}

/// Helper methods for creating common entity definitions.
impl EntityDefinition {
    /// Creates a perspective camera entity.
    pub fn perspective_camera(
        name: impl Into<String>,
        position: (f32, f32, f32),
        fov: f32,
        aspect_ratio: f32,
    ) -> Self {
        Self {
            name: Some(name.into()),
            transform: Some(TransformDef {
                translation: position,
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            }),
            camera: Some(CameraDef::perspective(fov, aspect_ratio, 0.1, 1000.0)),
            mesh: None,
            texture: None,
            directional_light: None,
            point_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }

    /// Creates an orthographic camera entity.
    pub fn orthographic_camera(
        name: impl Into<String>,
        position: (f32, f32, f32),
        size: (f32, f32),
    ) -> Self {
        let (width, height) = size;
        Self {
            name: Some(name.into()),
            transform: Some(TransformDef {
                translation: position,
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            }),
            camera: Some(CameraDef::orthographic(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                0.1,
                1000.0,
            )),
            mesh: None,
            texture: None,
            directional_light: None,
            point_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }

    /// Creates a directional light entity (e.g., sun).
    pub fn directional_light(
        name: impl Into<String>,
        direction: (f32, f32, f32),
        color: (f32, f32, f32),
        intensity: f32,
    ) -> Self {
        Self {
            name: Some(name.into()),
            directional_light: Some(DirectionalLightDef {
                direction,
                color,
                intensity,
            }),
            transform: None,
            mesh: None,
            texture: None,
            camera: None,
            point_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }

    /// Creates a point light entity.
    pub fn point_light(
        name: impl Into<String>,
        position: (f32, f32, f32),
        color: (f32, f32, f32),
        intensity: f32,
        range: f32,
    ) -> Self {
        Self {
            name: Some(name.into()),
            transform: Some(TransformDef {
                translation: position,
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            }),
            point_light: Some(PointLightDef {
                color,
                intensity,
                range,
            }),
            mesh: None,
            texture: None,
            camera: None,
            directional_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }

    /// Creates a mesh entity.
    pub fn mesh_entity(
        name: impl Into<String>,
        position: (f32, f32, f32),
        mesh_id: impl Into<String>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            transform: Some(TransformDef {
                translation: position,
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            }),
            mesh: Some(mesh_id.into()),
            texture: None,
            camera: None,
            directional_light: None,
            point_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }

    /// Creates a mesh entity with texture.
    pub fn textured_mesh_entity(
        name: impl Into<String>,
        position: (f32, f32, f32),
        mesh_id: impl Into<String>,
        texture_id: impl Into<String>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            transform: Some(TransformDef {
                translation: position,
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            }),
            mesh: Some(mesh_id.into()),
            texture: Some(texture_id.into()),
            camera: None,
            directional_light: None,
            point_light: None,
            visible: None,
            active: None,
            children: Vec::new(),
        }
    }
}
