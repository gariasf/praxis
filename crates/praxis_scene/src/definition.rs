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
    #[must_use]
    pub const fn entity_count(&self) -> usize {
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
    #[must_use]
    pub const fn new() -> Self {
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
    #[must_use]
    pub const fn with_transform(mut self, transform: TransformDef) -> Self {
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
    #[must_use]
    pub fn with_child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    /// Gets the total number of entities including this one and all descendants.
    #[must_use]
    pub fn total_count(&self) -> usize {
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
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            translation: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        }
    }

    /// Creates a transform definition from translation.
    #[must_use]
    pub const fn from_translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: (x, y, z),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        }
    }

    /// Converts to ECS Transform component types.
    #[must_use]
    pub const fn to_components(&self) -> (Vec3, Quat, Vec3) {
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
    #[must_use]
    pub const fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
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
    #[must_use]
    pub const fn orthographic(
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
    #[must_use]
    pub const fn new(direction: (f32, f32, f32), color: (f32, f32, f32), intensity: f32) -> Self {
        Self {
            direction,
            color,
            intensity,
        }
    }

    /// Converts to ECS component types.
    #[must_use]
    pub const fn to_components(&self) -> (Vec3, Vec3, f32) {
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
    #[must_use]
    pub const fn new(color: (f32, f32, f32), intensity: f32, range: f32) -> Self {
        Self {
            color,
            intensity,
            range,
        }
    }

    /// Converts to ECS component types.
    #[must_use]
    pub const fn to_components(&self) -> (Vec3, f32, f32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_definition_new() {
        let scene = SceneDefinition::new("Test Scene");
        assert_eq!(scene.name, "Test Scene");
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_scene_definition_add_entity() {
        let mut scene = SceneDefinition::new("Test");
        let entity = EntityDefinition::new().with_name("Entity1");
        scene.add_entity(entity);
        assert_eq!(scene.entity_count(), 1);
    }

    #[test]
    fn test_scene_definition_total_entity_count() {
        let mut scene = SceneDefinition::new("Test");
        
        let child = EntityDefinition::new().with_name("Child");
        let parent = EntityDefinition::new().with_name("Parent").with_child(child);
        
        scene.add_entity(parent);
        assert_eq!(scene.entity_count(), 1);
        assert_eq!(scene.total_entity_count(), 2);
    }

    #[test]
    fn test_scene_definition_total_entity_count_complex() {
        let mut scene = SceneDefinition::new("Test");
        
        let grandchild1 = EntityDefinition::new().with_name("Grandchild1");
        let grandchild2 = EntityDefinition::new().with_name("Grandchild2");
        let child1 = EntityDefinition::new().with_name("Child1").with_child(grandchild1);
        let child2 = EntityDefinition::new().with_name("Child2").with_child(grandchild2);
        let parent = EntityDefinition::new().with_name("Parent")
            .with_child(child1)
            .with_child(child2);
        
        scene.add_entity(parent);
        assert_eq!(scene.entity_count(), 1);
        assert_eq!(scene.total_entity_count(), 5);
    }

    #[test]
    fn test_scene_metadata_default() {
        let metadata = SceneMetadata::default();
        assert!(metadata.description.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.version.is_none());
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_entity_definition_new() {
        let entity = EntityDefinition::new();
        assert!(entity.name.is_none());
        assert!(entity.transform.is_none());
        assert!(entity.mesh.is_none());
        assert_eq!(entity.children.len(), 0);
    }

    #[test]
    fn test_entity_definition_default() {
        let entity = EntityDefinition::default();
        assert!(entity.name.is_none());
    }

    #[test]
    fn test_entity_definition_with_name() {
        let entity = EntityDefinition::new().with_name("TestEntity");
        assert_eq!(entity.name.as_deref(), Some("TestEntity"));
    }

    #[test]
    fn test_entity_definition_with_transform() {
        let transform = TransformDef::from_translation(1.0, 2.0, 3.0);
        let entity = EntityDefinition::new().with_transform(transform);
        assert!(entity.transform.is_some());
    }

    #[test]
    fn test_entity_definition_with_mesh() {
        let entity = EntityDefinition::new().with_mesh("cube");
        assert_eq!(entity.mesh.as_deref(), Some("cube"));
    }

    #[test]
    fn test_entity_definition_with_child() {
        let child = EntityDefinition::new().with_name("Child");
        let parent = EntityDefinition::new().with_name("Parent").with_child(child);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].name.as_deref(), Some("Child"));
    }

    #[test]
    fn test_entity_definition_total_count() {
        let entity = EntityDefinition::new().with_name("Single");
        assert_eq!(entity.total_count(), 1);
    }

    #[test]
    fn test_entity_definition_total_count_with_children() {
        let child1 = EntityDefinition::new().with_name("Child1");
        let child2 = EntityDefinition::new().with_name("Child2");
        let parent = EntityDefinition::new()
            .with_name("Parent")
            .with_child(child1)
            .with_child(child2);
        assert_eq!(parent.total_count(), 3);
    }

    #[test]
    fn test_transform_def_identity() {
        let transform = TransformDef::identity();
        assert_eq!(transform.translation, (0.0, 0.0, 0.0));
        assert_eq!(transform.rotation, (0.0, 0.0, 0.0, 1.0));
        assert_eq!(transform.scale, (1.0, 1.0, 1.0));
    }

    #[test]
    fn test_transform_def_default() {
        let transform = TransformDef::default();
        assert_eq!(transform.translation, (0.0, 0.0, 0.0));
        assert_eq!(transform.rotation, (0.0, 0.0, 0.0, 1.0));
        assert_eq!(transform.scale, (1.0, 1.0, 1.0));
    }

    #[test]
    fn test_transform_def_from_translation() {
        let transform = TransformDef::from_translation(5.0, 10.0, 15.0);
        assert_eq!(transform.translation, (5.0, 10.0, 15.0));
        assert_eq!(transform.rotation, (0.0, 0.0, 0.0, 1.0));
        assert_eq!(transform.scale, (1.0, 1.0, 1.0));
    }

    #[test]
    fn test_transform_def_to_components() {
        let transform = TransformDef {
            translation: (1.0, 2.0, 3.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (2.0, 2.0, 2.0),
        };
        let (translation, rotation, scale) = transform.to_components();
        assert_eq!(translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(rotation, Quat::from_xyzw(0.0, 0.0, 0.0, 1.0));
        assert_eq!(scale, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_camera_def_perspective() {
        let camera = CameraDef::perspective(1.22, 1.77, 0.1, 1000.0);
        assert_eq!(camera.camera_type, CameraType::Perspective);
        assert_eq!(camera.fov, Some(1.22));
        assert_eq!(camera.aspect_ratio, Some(1.77));
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
        assert!(camera.is_active);
    }

    #[test]
    fn test_camera_def_orthographic() {
        let camera = CameraDef::orthographic(-10.0, 10.0, -7.5, 7.5, 0.1, 100.0);
        assert_eq!(camera.camera_type, CameraType::Orthographic);
        assert_eq!(camera.left, Some(-10.0));
        assert_eq!(camera.right, Some(10.0));
        assert_eq!(camera.bottom, Some(-7.5));
        assert_eq!(camera.top, Some(7.5));
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 100.0);
    }

    #[test]
    fn test_camera_type_equality() {
        assert_eq!(CameraType::Perspective, CameraType::Perspective);
        assert_eq!(CameraType::Orthographic, CameraType::Orthographic);
        assert_ne!(CameraType::Perspective, CameraType::Orthographic);
    }

    #[test]
    fn test_directional_light_def_new() {
        let light = DirectionalLightDef::new((0.0, -1.0, 0.0), (1.0, 0.8, 0.6), 1.5);
        assert_eq!(light.direction, (0.0, -1.0, 0.0));
        assert_eq!(light.color, (1.0, 0.8, 0.6));
        assert_eq!(light.intensity, 1.5);
    }

    #[test]
    fn test_directional_light_def_default() {
        let light = DirectionalLightDef::default();
        assert_eq!(light.direction, (0.0, -1.0, 0.0));
        assert_eq!(light.color, (1.0, 1.0, 1.0));
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_directional_light_def_to_components() {
        let light = DirectionalLightDef::new((1.0, 0.0, 0.0), (1.0, 0.5, 0.25), 2.0);
        let (direction, color, intensity) = light.to_components();
        assert_eq!(direction, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(color, Vec3::new(1.0, 0.5, 0.25));
        assert_eq!(intensity, 2.0);
    }

    #[test]
    fn test_point_light_def_new() {
        let light = PointLightDef::new((1.0, 0.8, 0.6), 2.5, 15.0);
        assert_eq!(light.color, (1.0, 0.8, 0.6));
        assert_eq!(light.intensity, 2.5);
        assert_eq!(light.range, 15.0);
    }

    #[test]
    fn test_point_light_def_default() {
        let light = PointLightDef::default();
        assert_eq!(light.color, (1.0, 1.0, 1.0));
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.range, 10.0);
    }

    #[test]
    fn test_point_light_def_to_components() {
        let light = PointLightDef::new((0.5, 0.7, 0.9), 3.0, 20.0);
        let (color, intensity, range) = light.to_components();
        assert_eq!(color, Vec3::new(0.5, 0.7, 0.9));
        assert_eq!(intensity, 3.0);
        assert_eq!(range, 20.0);
    }

    #[test]
    fn test_entity_definition_perspective_camera() {
        let camera = EntityDefinition::perspective_camera("MainCamera", (0.0, 5.0, 10.0), 1.22, 1.77);
        assert_eq!(camera.name.as_deref(), Some("MainCamera"));
        assert!(camera.transform.is_some());
        assert!(camera.camera.is_some());
        
        let camera_def = camera.camera.unwrap();
        assert_eq!(camera_def.camera_type, CameraType::Perspective);
    }

    #[test]
    fn test_entity_definition_orthographic_camera() {
        let camera = EntityDefinition::orthographic_camera("OrthoCamera", (0.0, 0.0, 10.0), (20.0, 15.0));
        assert_eq!(camera.name.as_deref(), Some("OrthoCamera"));
        assert!(camera.transform.is_some());
        assert!(camera.camera.is_some());
        
        let camera_def = camera.camera.unwrap();
        assert_eq!(camera_def.camera_type, CameraType::Orthographic);
    }

    #[test]
    fn test_entity_definition_directional_light() {
        let light = EntityDefinition::directional_light("Sun", (0.0, -1.0, 0.0), (1.0, 1.0, 0.9), 1.5);
        assert_eq!(light.name.as_deref(), Some("Sun"));
        assert!(light.directional_light.is_some());
        
        let light_def = light.directional_light.unwrap();
        assert_eq!(light_def.direction, (0.0, -1.0, 0.0));
    }

    #[test]
    fn test_entity_definition_point_light() {
        let light = EntityDefinition::point_light("Lamp", (0.0, 2.0, 0.0), (1.0, 0.8, 0.6), 2.0, 10.0);
        assert_eq!(light.name.as_deref(), Some("Lamp"));
        assert!(light.transform.is_some());
        assert!(light.point_light.is_some());
        
        let light_def = light.point_light.unwrap();
        assert_eq!(light_def.intensity, 2.0);
        assert_eq!(light_def.range, 10.0);
    }

    #[test]
    fn test_entity_definition_mesh_entity() {
        let entity = EntityDefinition::mesh_entity("Cube", (1.0, 2.0, 3.0), "cube_mesh");
        assert_eq!(entity.name.as_deref(), Some("Cube"));
        assert!(entity.transform.is_some());
        assert_eq!(entity.mesh.as_deref(), Some("cube_mesh"));
        assert!(entity.texture.is_none());
    }

    #[test]
    fn test_entity_definition_textured_mesh_entity() {
        let entity = EntityDefinition::textured_mesh_entity(
            "TexturedCube",
            (1.0, 2.0, 3.0),
            "cube_mesh",
            "cube_texture",
        );
        assert_eq!(entity.name.as_deref(), Some("TexturedCube"));
        assert!(entity.transform.is_some());
        assert_eq!(entity.mesh.as_deref(), Some("cube_mesh"));
        assert_eq!(entity.texture.as_deref(), Some("cube_texture"));
    }

    #[test]
    fn test_entity_definition_builder_pattern() {
        let entity = EntityDefinition::new()
            .with_name("TestEntity")
            .with_transform(TransformDef::from_translation(1.0, 2.0, 3.0))
            .with_mesh("test_mesh")
            .with_child(EntityDefinition::new().with_name("Child1"))
            .with_child(EntityDefinition::new().with_name("Child2"));
        
        assert_eq!(entity.name.as_deref(), Some("TestEntity"));
        assert!(entity.transform.is_some());
        assert_eq!(entity.mesh.as_deref(), Some("test_mesh"));
        assert_eq!(entity.children.len(), 2);
    }

    #[test]
    fn test_scene_definition_clone() {
        let mut scene1 = SceneDefinition::new("Test");
        scene1.add_entity(EntityDefinition::new().with_name("Entity1"));
        
        let scene2 = scene1.clone();
        assert_eq!(scene1.name, scene2.name);
        assert_eq!(scene1.entity_count(), scene2.entity_count());
    }

    #[test]
    fn test_transform_def_clone() {
        let transform1 = TransformDef::from_translation(1.0, 2.0, 3.0);
        let transform2 = transform1;
        assert_eq!(transform1.translation, transform2.translation);
    }

    #[test]
    fn test_camera_def_clone() {
        let camera1 = CameraDef::perspective(1.22, 1.77, 0.1, 1000.0);
        let camera2 = camera1;
        assert_eq!(camera1.fov, camera2.fov);
    }
}
