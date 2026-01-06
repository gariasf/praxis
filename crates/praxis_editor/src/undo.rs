//! Command pattern implementation for editor operations with undo/redo support.
//!
//! This module provides a comprehensive command-based undo/redo system for editor operations.
//! Commands can be executed, undone, redone, and serialized/deserialized for save/load functionality.
//!
//! # Architecture
//!
//! - **`EditorCommand`**: Trait defining execute, undo, and redo operations
//! - **`CommandHistory`**: System managing command execution and undo/redo stacks
//! - **Concrete Commands**: Specific implementations for various editor operations
//!   - `TransformEditCommand`: Edit entity transforms
//!   - `CreateEntityCommand`: Create new entities
//!   - `DeleteEntityCommand`: Delete entities
//!   - `AddComponentCommand`: Add components to entities
//!   - `RemoveComponentCommand`: Remove components from entities
//!   - `SetParentCommand`: Change entity parent relationships
//!   - `CompositeCommand`: Group multiple commands together
//!
//! # Serialization
//!
//! All commands implement `serde::Serialize` and `serde::Deserialize` via RON format,
//! allowing command history to be saved and loaded for features like:
//! - Session recovery
//! - Replay functionality
//! - Collaboration tools
//!
//! # Usage Examples
//!
//! ## Transform Editing
//!
//! ```rust,ignore
//! use praxis_editor::{CommandHistory, TransformEditCommand};
//! use bevy_ecs::world::World;
//! use praxis_ecs::Transform;
//!
//! let mut world = World::new();
//! let mut history = CommandHistory::new();
//!
//! let entity = world.spawn(Transform::default()).id();
//! let old_transform = Transform::default();
//! let new_transform = Transform::from_xyz(10.0, 0.0, 0.0);
//!
//! let command = TransformEditCommand::new(entity, old_transform, new_transform);
//! history.execute(&mut world, Box::new(command)).unwrap();
//!
//! history.undo(&mut world).unwrap();
//! history.redo(&mut world).unwrap();
//! ```
//!
//! ## Entity Creation
//!
//! ```rust,ignore
//! use praxis_editor::{CommandHistory, CreateEntityCommand, ComponentData};
//! use bevy_ecs::world::World;
//! use praxis_ecs::Transform;
//!
//! let mut world = World::new();
//! let mut history = CommandHistory::new();
//!
//! let command = CreateEntityCommand::with_transform(Transform::from_xyz(5.0, 0.0, 0.0));
//! history.execute(&mut world, Box::new(command)).unwrap();
//! ```
//!
//! ## Composite Commands
//!
//! ```rust,ignore
//! use praxis_editor::{CommandHistory, CompositeCommand, SerializableCommand, CreateEntityCommand};
//! use bevy_ecs::world::World;
//! use praxis_ecs::Transform;
//!
//! let mut world = World::new();
//! let mut history = CommandHistory::new();
//!
//! let mut composite = CompositeCommand::new("Create Multiple Entities".to_string());
//! for i in 0..5 {
//!     let cmd = CreateEntityCommand::with_transform(Transform::from_xyz(i as f32, 0.0, 0.0));
//!     composite.add_command(SerializableCommand::CreateEntity(cmd));
//! }
//!
//! history.execute(&mut world, Box::new(composite)).unwrap();
//! ```
//!
//! ## Serialization
//!
//! ```rust,ignore
//! use praxis_editor::{CommandHistory, TransformEditCommand};
//! use bevy_ecs::world::World;
//! use praxis_ecs::Transform;
//!
//! let mut history = CommandHistory::new();
//! let mut world = World::new();
//!
//! let entity = world.spawn(Transform::default()).id();
//! let command = TransformEditCommand::new(entity, Transform::default(), Transform::from_xyz(1.0, 2.0, 3.0));
//! history.execute(&mut world, Box::new(command)).unwrap();
//!
//! // Save history to RON
//! let ron_string = history.to_ron().unwrap();
//!
//! // Load history from RON
//! let mut new_history = CommandHistory::new();
//! new_history.from_ron(&ron_string).unwrap();
//! ```

use bevy_ecs::entity::Entity;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use praxis_ecs::{Children, Name, Parent, Transform};
use praxis_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

type Result<T> = std::result::Result<T, String>;

/// Maximum number of commands to keep in history.
const MAX_HISTORY_SIZE: usize = 100;

/// Trait for editor commands that can be executed, undone, and redone.
///
/// All commands must be serializable to support save/load functionality.
/// Commands are responsible for tracking their own state and maintaining
/// enough information to reverse their effects.
pub trait EditorCommand: Send + Sync {
    /// Executes the command, applying its changes to the world.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to modify
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err` if the command failed
    fn execute(&mut self, world: &mut World) -> Result<()>;

    /// Undoes the command, reverting its changes.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to modify
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err` if the undo failed
    fn undo(&mut self, world: &mut World) -> Result<()>;

    /// Redoes the command after it has been undone.
    ///
    /// Default implementation simply calls `execute()` again.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to modify
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err` if the redo failed
    fn redo(&mut self, world: &mut World) -> Result<()> {
        self.execute(world)
    }

    /// Returns a human-readable description of the command.
    ///
    /// Used for UI display in undo/redo menus.
    fn description(&self) -> String;

    /// Serializes the command to RON format.
    ///
    /// # Returns
    ///
    /// RON string representation of the command
    fn to_ron(&self) -> Result<String>;

    /// Returns a type identifier for this command.
    ///
    /// Used during deserialization to determine which concrete type to create.
    fn type_id(&self) -> &'static str;
}

/// Serializable command wrapper for RON serialization.
///
/// This enum contains all concrete command types and implements
/// serde's Serialize and Deserialize traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializableCommand {
    TransformEdit(TransformEditCommand),
    CreateEntity(CreateEntityCommand),
    DeleteEntity(DeleteEntityCommand),
    AddComponent(AddComponentCommand),
    RemoveComponent(RemoveComponentCommand),
    SetParent(SetParentCommand),
    Composite(CompositeCommand),
    CopyEntity(CopyEntityCommand),
    PasteEntity(PasteEntityCommand),
}

impl SerializableCommand {
    /// Creates a command from RON string.
    pub fn from_ron(ron: &str) -> Result<Self> {
        ron::from_str(ron).map_err(|e| format!("Failed to deserialize command: {e}"))
    }

    /// Converts the command to a boxed trait object.
    pub fn to_trait_object(self) -> Box<dyn EditorCommand> {
        match self {
            SerializableCommand::TransformEdit(cmd) => Box::new(cmd),
            SerializableCommand::CreateEntity(cmd) => Box::new(cmd),
            SerializableCommand::DeleteEntity(cmd) => Box::new(cmd),
            SerializableCommand::AddComponent(cmd) => Box::new(cmd),
            SerializableCommand::RemoveComponent(cmd) => Box::new(cmd),
            SerializableCommand::SetParent(cmd) => Box::new(cmd),
            SerializableCommand::Composite(cmd) => Box::new(cmd),
            SerializableCommand::CopyEntity(cmd) => Box::new(cmd),
            SerializableCommand::PasteEntity(cmd) => Box::new(cmd),
        }
    }
}

/// Command for editing entity transforms.
///
/// Stores the old and new transform states to enable undo/redo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformEditCommand {
    /// Entity whose transform is being edited.
    pub entity: SerializableEntity,
    /// Transform state before the edit.
    pub old_transform: SerializableTransform,
    /// Transform state after the edit.
    pub new_transform: SerializableTransform,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

/// Serializable entity representation.
///
/// Note: Entity IDs are not stable across sessions and may need remapping when loading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableEntity {
    index: u32,
    generation: u32,
}

impl From<Entity> for SerializableEntity {
    fn from(entity: Entity) -> Self {
        Self {
            index: entity.index(),
            generation: entity.generation(),
        }
    }
}

impl From<SerializableEntity> for Entity {
    fn from(se: SerializableEntity) -> Self {
        Entity::from_bits(((se.generation as u64) << 32) | (se.index as u64))
    }
}

/// Serializable transform representation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
        }
    }
}

impl From<SerializableTransform> for Transform {
    fn from(st: SerializableTransform) -> Self {
        Self {
            translation: Vec3::from_array(st.translation),
            rotation: Quat::from_xyzw(
                st.rotation[0],
                st.rotation[1],
                st.rotation[2],
                st.rotation[3],
            ),
            scale: Vec3::from_array(st.scale),
        }
    }
}

impl TransformEditCommand {
    /// Creates a new transform edit command.
    pub fn new(entity: Entity, old_transform: Transform, new_transform: Transform) -> Self {
        Self {
            entity: entity.into(),
            old_transform: old_transform.into(),
            new_transform: new_transform.into(),
            executed: false,
        }
    }
}

impl EditorCommand for TransformEditCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            if let Some(mut transform) = entity_mut.get_mut::<Transform>() {
                *transform = self.new_transform.into();
                self.executed = true;
                return Ok(());
            }
        }
        Err("Entity not found or missing Transform component".to_string())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            if let Some(mut transform) = entity_mut.get_mut::<Transform>() {
                *transform = self.old_transform.into();
                self.executed = false;
                return Ok(());
            }
        }
        Err("Entity not found or missing Transform component".to_string())
    }

    fn description(&self) -> String {
        let entity: Entity = self.entity.into();
        format!("Transform Entity {entity:?}")
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::TransformEdit(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "TransformEdit"
    }
}

/// Component data stored for entity creation/deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentData {
    Transform(SerializableTransform),
    Name(String),
    Parent(SerializableEntity),
    MeshHandle(String),
    MaterialHandle(String),
    MaterialProperties(SerializableMaterialProperties),
    RigidBody(SerializableRigidBody),
    Collider(SerializableCollider),
    PhysicsVelocity(SerializablePhysicsVelocity),
    Mass(SerializableMass),
    AudioSource(SerializableAudioSource),
    PerspectiveProjection(SerializablePerspectiveProjection),
}

/// Serializable material properties.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableMaterialProperties {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive_strength: f32,
}

/// Serializable rigid body type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SerializableRigidBody {
    Dynamic,
    Static,
    Kinematic,
}

/// Serializable collider shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableCollider {
    Cuboid { hx: f32, hy: f32, hz: f32 },
    Sphere { radius: f32 },
    CapsuleY { half_height: f32, radius: f32 },
    CapsuleX { half_height: f32, radius: f32 },
    CapsuleZ { half_height: f32, radius: f32 },
    CylinderY { half_height: f32, radius: f32 },
}

/// Serializable physics velocity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializablePhysicsVelocity {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

/// Serializable mass properties.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableMass {
    pub mass: f32,
    pub angular_inertia: f32,
}

/// Serializable audio source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAudioSource {
    pub path: String,
    pub volume: f32,
    pub spatial: bool,
    pub looping: bool,
    pub max_distance: f32,
    pub reference_distance: f32,
}

/// Serializable perspective camera.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializablePerspectiveProjection {
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

/// Command for creating a new entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityCommand {
    /// The entity that was/will be created.
    /// For new commands, this is set during execute().
    pub entity: Option<SerializableEntity>,
    /// Components to add to the entity.
    pub components: Vec<ComponentData>,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl CreateEntityCommand {
    /// Creates a new entity creation command.
    pub fn new(components: Vec<ComponentData>) -> Self {
        Self {
            entity: None,
            components,
            executed: false,
        }
    }

    /// Creates a command for a simple entity with transform.
    pub fn with_transform(transform: Transform) -> Self {
        Self::new(vec![ComponentData::Transform(transform.into())])
    }
}

impl EditorCommand for CreateEntityCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_graphics::MaterialProperties;
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let mut entity_mut = world.spawn_empty();
        let entity = entity_mut.id();

        for component in &self.components {
            match component {
                ComponentData::Transform(t) => {
                    entity_mut.insert(Transform::from(*t));
                }
                ComponentData::Name(name) => {
                    entity_mut.insert(Name::new(name.clone()));
                }
                ComponentData::Parent(parent) => {
                    let parent_entity: Entity = (*parent).into();
                    entity_mut.insert(Parent(parent_entity));
                }
                ComponentData::MeshHandle(id) => {
                    entity_mut.insert(MeshHandle::new(id.clone()));
                }
                ComponentData::MaterialHandle(id) => {
                    entity_mut.insert(MaterialHandle::new(id.clone()));
                }
                ComponentData::MaterialProperties(props) => {
                    entity_mut.insert(MaterialPropertiesComponent(
                        MaterialProperties::new()
                            .with_base_color(props.base_color)
                            .with_metallic(props.metallic)
                            .with_roughness(props.roughness)
                            .with_emissive_strength(props.emissive_strength),
                    ));
                }
                ComponentData::RigidBody(rb) => {
                    entity_mut.insert(match rb {
                        SerializableRigidBody::Dynamic => RigidBody::Dynamic,
                        SerializableRigidBody::Static => RigidBody::Static,
                        SerializableRigidBody::Kinematic => RigidBody::Kinematic,
                    });
                }
                ComponentData::Collider(col) => {
                    entity_mut.insert(match col {
                        SerializableCollider::Cuboid { hx, hy, hz } => {
                            Collider::cuboid(*hx, *hy, *hz)
                        }
                        SerializableCollider::Sphere { radius } => Collider::sphere(*radius),
                        SerializableCollider::CapsuleY {
                            half_height,
                            radius,
                        } => Collider::capsule_y(*half_height, *radius),
                        SerializableCollider::CapsuleX {
                            half_height,
                            radius,
                        } => Collider::capsule_x(*half_height, *radius),
                        SerializableCollider::CapsuleZ {
                            half_height,
                            radius,
                        } => Collider::capsule_z(*half_height, *radius),
                        SerializableCollider::CylinderY {
                            half_height,
                            radius,
                        } => Collider::cylinder_y(*half_height, *radius),
                    });
                }
                ComponentData::PhysicsVelocity(vel) => {
                    entity_mut.insert(PhysicsVelocity::new(
                        Vec3::from_array(vel.linear),
                        Vec3::from_array(vel.angular),
                    ));
                }
                ComponentData::Mass(mass) => {
                    entity_mut.insert(Mass::with_inertia(mass.mass, mass.angular_inertia));
                }
                ComponentData::AudioSource(audio) => {
                    entity_mut.insert(
                        AudioSource::new(audio.path.clone())
                            .with_volume(audio.volume)
                            .with_spatial(audio.spatial)
                            .with_looping(audio.looping)
                            .with_max_distance(audio.max_distance)
                            .with_reference_distance(audio.reference_distance),
                    );
                }
                ComponentData::PerspectiveProjection(cam) => {
                    entity_mut.insert(PerspectiveProjection::new(
                        cam.fov,
                        cam.aspect_ratio,
                        cam.near,
                        cam.far,
                    ));
                }
            }
        }

        self.entity = Some(entity.into());
        self.executed = true;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(serializable_entity) = self.entity {
            let entity: Entity = serializable_entity.into();
            if world.despawn(entity) {
                self.executed = false;
                Ok(())
            } else {
                Err("Failed to despawn entity".to_string())
            }
        } else {
            Err("Entity not found".to_string())
        }
    }

    fn redo(&mut self, world: &mut World) -> Result<()> {
        self.execute(world)
    }

    fn description(&self) -> String {
        "Create Entity".to_string()
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::CreateEntity(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "CreateEntity"
    }
}

/// Command for deleting an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEntityCommand {
    /// Entity to delete.
    pub entity: SerializableEntity,
    /// Components that were on the entity (for undo).
    pub stored_components: Vec<ComponentData>,
    /// Parent entity if the entity had one.
    pub parent: Option<SerializableEntity>,
    /// Children entities if the entity had any.
    pub children: Vec<SerializableEntity>,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl DeleteEntityCommand {
    /// Creates a new delete entity command.
    ///
    /// Note: This doesn't automatically capture components.
    /// Use `from_world` to create a command with captured state.
    pub fn new(entity: Entity) -> Self {
        Self {
            entity: entity.into(),
            stored_components: Vec::new(),
            parent: None,
            children: Vec::new(),
            executed: false,
        }
    }

    /// Creates a delete command by capturing entity state from the world.
    pub fn from_world(entity: Entity, world: &World) -> Result<Self> {
        let mut stored_components = Vec::new();

        if let Some(entity_ref) = world.get_entity(entity) {
            if let Some(transform) = entity_ref.get::<Transform>() {
                stored_components.push(ComponentData::Transform((*transform).into()));
            }
            if let Some(name) = entity_ref.get::<Name>() {
                stored_components.push(ComponentData::Name(name.0.clone()));
            }

            let parent = entity_ref.get::<Parent>().map(|p| p.0.into());
            let children = entity_ref
                .get::<Children>()
                .map(|c| c.0.iter().map(|e| (*e).into()).collect())
                .unwrap_or_default();

            Ok(Self {
                entity: entity.into(),
                stored_components,
                parent,
                children,
                executed: false,
            })
        } else {
            Err("Entity not found".to_string())
        }
    }
}

impl EditorCommand for DeleteEntityCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        let entity: Entity = self.entity.into();
        if world.despawn(entity) {
            self.executed = true;
            Ok(())
        } else {
            Err("Failed to despawn entity".to_string())
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_graphics::MaterialProperties;
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let mut entity_mut = world.spawn_empty();

        for component in &self.stored_components {
            match component {
                ComponentData::Transform(t) => {
                    entity_mut.insert(Transform::from(*t));
                }
                ComponentData::Name(name) => {
                    entity_mut.insert(Name::new(name.clone()));
                }
                ComponentData::Parent(parent) => {
                    let parent_entity: Entity = (*parent).into();
                    entity_mut.insert(Parent(parent_entity));
                }
                ComponentData::MeshHandle(id) => {
                    entity_mut.insert(MeshHandle::new(id.clone()));
                }
                ComponentData::MaterialHandle(id) => {
                    entity_mut.insert(MaterialHandle::new(id.clone()));
                }
                ComponentData::MaterialProperties(props) => {
                    entity_mut.insert(MaterialPropertiesComponent(
                        MaterialProperties::new()
                            .with_base_color(props.base_color)
                            .with_metallic(props.metallic)
                            .with_roughness(props.roughness)
                            .with_emissive_strength(props.emissive_strength),
                    ));
                }
                ComponentData::RigidBody(rb) => {
                    entity_mut.insert(match rb {
                        SerializableRigidBody::Dynamic => RigidBody::Dynamic,
                        SerializableRigidBody::Static => RigidBody::Static,
                        SerializableRigidBody::Kinematic => RigidBody::Kinematic,
                    });
                }
                ComponentData::Collider(col) => {
                    entity_mut.insert(match col {
                        SerializableCollider::Cuboid { hx, hy, hz } => {
                            Collider::cuboid(*hx, *hy, *hz)
                        }
                        SerializableCollider::Sphere { radius } => Collider::sphere(*radius),
                        SerializableCollider::CapsuleY {
                            half_height,
                            radius,
                        } => Collider::capsule_y(*half_height, *radius),
                        SerializableCollider::CapsuleX {
                            half_height,
                            radius,
                        } => Collider::capsule_x(*half_height, *radius),
                        SerializableCollider::CapsuleZ {
                            half_height,
                            radius,
                        } => Collider::capsule_z(*half_height, *radius),
                        SerializableCollider::CylinderY {
                            half_height,
                            radius,
                        } => Collider::cylinder_y(*half_height, *radius),
                    });
                }
                ComponentData::PhysicsVelocity(vel) => {
                    entity_mut.insert(PhysicsVelocity::new(
                        Vec3::from_array(vel.linear),
                        Vec3::from_array(vel.angular),
                    ));
                }
                ComponentData::Mass(mass) => {
                    entity_mut.insert(Mass::with_inertia(mass.mass, mass.angular_inertia));
                }
                ComponentData::AudioSource(audio) => {
                    entity_mut.insert(
                        AudioSource::new(audio.path.clone())
                            .with_volume(audio.volume)
                            .with_spatial(audio.spatial)
                            .with_looping(audio.looping)
                            .with_max_distance(audio.max_distance)
                            .with_reference_distance(audio.reference_distance),
                    );
                }
                ComponentData::PerspectiveProjection(cam) => {
                    entity_mut.insert(PerspectiveProjection::new(
                        cam.fov,
                        cam.aspect_ratio,
                        cam.near,
                        cam.far,
                    ));
                }
            }
        }

        if let Some(parent) = self.parent {
            let parent_entity: Entity = parent.into();
            entity_mut.insert(Parent(parent_entity));
        }

        if !self.children.is_empty() {
            let children_entities: Vec<Entity> =
                self.children.iter().map(|e| (*e).into()).collect();
            entity_mut.insert(Children::with_children(children_entities));
        }

        self.executed = false;
        Ok(())
    }

    fn description(&self) -> String {
        let entity: Entity = self.entity.into();
        format!("Delete Entity {entity:?}")
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::DeleteEntity(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "DeleteEntity"
    }
}

/// Command for adding a component to an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddComponentCommand {
    /// Entity to add component to.
    pub entity: SerializableEntity,
    /// Component to add.
    pub component: ComponentData,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl AddComponentCommand {
    /// Creates a new add component command.
    pub fn new(entity: Entity, component: ComponentData) -> Self {
        Self {
            entity: entity.into(),
            component,
            executed: false,
        }
    }
}

impl EditorCommand for AddComponentCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_graphics::MaterialProperties;
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            match &self.component {
                ComponentData::Transform(t) => {
                    entity_mut.insert(Transform::from(*t));
                }
                ComponentData::Name(name) => {
                    entity_mut.insert(Name::new(name.clone()));
                }
                ComponentData::Parent(parent) => {
                    let parent_entity: Entity = (*parent).into();
                    entity_mut.insert(Parent(parent_entity));
                }
                ComponentData::MeshHandle(id) => {
                    entity_mut.insert(MeshHandle::new(id.clone()));
                }
                ComponentData::MaterialHandle(id) => {
                    entity_mut.insert(MaterialHandle::new(id.clone()));
                }
                ComponentData::MaterialProperties(props) => {
                    entity_mut.insert(MaterialPropertiesComponent(
                        MaterialProperties::new()
                            .with_base_color(props.base_color)
                            .with_metallic(props.metallic)
                            .with_roughness(props.roughness)
                            .with_emissive_strength(props.emissive_strength),
                    ));
                }
                ComponentData::RigidBody(rb) => {
                    entity_mut.insert(match rb {
                        SerializableRigidBody::Dynamic => RigidBody::Dynamic,
                        SerializableRigidBody::Static => RigidBody::Static,
                        SerializableRigidBody::Kinematic => RigidBody::Kinematic,
                    });
                }
                ComponentData::Collider(col) => {
                    entity_mut.insert(match col {
                        SerializableCollider::Cuboid { hx, hy, hz } => {
                            Collider::cuboid(*hx, *hy, *hz)
                        }
                        SerializableCollider::Sphere { radius } => Collider::sphere(*radius),
                        SerializableCollider::CapsuleY {
                            half_height,
                            radius,
                        } => Collider::capsule_y(*half_height, *radius),
                        SerializableCollider::CapsuleX {
                            half_height,
                            radius,
                        } => Collider::capsule_x(*half_height, *radius),
                        SerializableCollider::CapsuleZ {
                            half_height,
                            radius,
                        } => Collider::capsule_z(*half_height, *radius),
                        SerializableCollider::CylinderY {
                            half_height,
                            radius,
                        } => Collider::cylinder_y(*half_height, *radius),
                    });
                }
                ComponentData::PhysicsVelocity(vel) => {
                    entity_mut.insert(PhysicsVelocity::new(
                        Vec3::from_array(vel.linear),
                        Vec3::from_array(vel.angular),
                    ));
                }
                ComponentData::Mass(mass) => {
                    entity_mut.insert(Mass::with_inertia(mass.mass, mass.angular_inertia));
                }
                ComponentData::AudioSource(audio) => {
                    entity_mut.insert(
                        AudioSource::new(audio.path.clone())
                            .with_volume(audio.volume)
                            .with_spatial(audio.spatial)
                            .with_looping(audio.looping)
                            .with_max_distance(audio.max_distance)
                            .with_reference_distance(audio.reference_distance),
                    );
                }
                ComponentData::PerspectiveProjection(cam) => {
                    entity_mut.insert(PerspectiveProjection::new(
                        cam.fov,
                        cam.aspect_ratio,
                        cam.near,
                        cam.far,
                    ));
                }
            }
            self.executed = true;
            Ok(())
        } else {
            Err("Entity not found".to_string())
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            match &self.component {
                ComponentData::Transform(_) => {
                    entity_mut.remove::<Transform>();
                }
                ComponentData::Name(_) => {
                    entity_mut.remove::<Name>();
                }
                ComponentData::Parent(_) => {
                    entity_mut.remove::<Parent>();
                }
                ComponentData::MeshHandle(_) => {
                    entity_mut.remove::<MeshHandle>();
                }
                ComponentData::MaterialHandle(_) => {
                    entity_mut.remove::<MaterialHandle>();
                }
                ComponentData::MaterialProperties(_) => {
                    entity_mut.remove::<MaterialPropertiesComponent>();
                }
                ComponentData::RigidBody(_) => {
                    entity_mut.remove::<RigidBody>();
                }
                ComponentData::Collider(_) => {
                    entity_mut.remove::<Collider>();
                }
                ComponentData::PhysicsVelocity(_) => {
                    entity_mut.remove::<PhysicsVelocity>();
                }
                ComponentData::Mass(_) => {
                    entity_mut.remove::<Mass>();
                }
                ComponentData::AudioSource(_) => {
                    entity_mut.remove::<AudioSource>();
                }
                ComponentData::PerspectiveProjection(_) => {
                    entity_mut.remove::<PerspectiveProjection>();
                }
            }
            self.executed = false;
            Ok(())
        } else {
            Err("Entity not found".to_string())
        }
    }

    fn description(&self) -> String {
        let entity: Entity = self.entity.into();
        let component_name = match &self.component {
            ComponentData::Transform(_) => "Transform",
            ComponentData::Name(_) => "Name",
            ComponentData::Parent(_) => "Parent",
            ComponentData::MeshHandle(_) => "MeshHandle",
            ComponentData::MaterialHandle(_) => "MaterialHandle",
            ComponentData::MaterialProperties(_) => "MaterialProperties",
            ComponentData::RigidBody(_) => "RigidBody",
            ComponentData::Collider(_) => "Collider",
            ComponentData::PhysicsVelocity(_) => "PhysicsVelocity",
            ComponentData::Mass(_) => "Mass",
            ComponentData::AudioSource(_) => "AudioSource",
            ComponentData::PerspectiveProjection(_) => "PerspectiveProjection",
        };
        format!("Add {component_name} to {entity:?}")
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::AddComponent(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "AddComponent"
    }
}

/// Command for removing a component from an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveComponentCommand {
    /// Entity to remove component from.
    pub entity: SerializableEntity,
    /// Component that was removed (stored for undo).
    pub component: ComponentData,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl RemoveComponentCommand {
    /// Creates a new remove component command.
    ///
    /// Note: The component data should be captured before removal for proper undo.
    pub fn new(entity: Entity, component: ComponentData) -> Self {
        Self {
            entity: entity.into(),
            component,
            executed: false,
        }
    }
}

impl EditorCommand for RemoveComponentCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            match &self.component {
                ComponentData::Transform(_) => {
                    entity_mut.remove::<Transform>();
                }
                ComponentData::Name(_) => {
                    entity_mut.remove::<Name>();
                }
                ComponentData::Parent(_) => {
                    entity_mut.remove::<Parent>();
                }
                ComponentData::MeshHandle(_) => {
                    entity_mut.remove::<MeshHandle>();
                }
                ComponentData::MaterialHandle(_) => {
                    entity_mut.remove::<MaterialHandle>();
                }
                ComponentData::MaterialProperties(_) => {
                    entity_mut.remove::<MaterialPropertiesComponent>();
                }
                ComponentData::RigidBody(_) => {
                    entity_mut.remove::<RigidBody>();
                }
                ComponentData::Collider(_) => {
                    entity_mut.remove::<Collider>();
                }
                ComponentData::PhysicsVelocity(_) => {
                    entity_mut.remove::<PhysicsVelocity>();
                }
                ComponentData::Mass(_) => {
                    entity_mut.remove::<Mass>();
                }
                ComponentData::AudioSource(_) => {
                    entity_mut.remove::<AudioSource>();
                }
                ComponentData::PerspectiveProjection(_) => {
                    entity_mut.remove::<PerspectiveProjection>();
                }
            }
            self.executed = true;
            Ok(())
        } else {
            Err("Entity not found".to_string())
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_graphics::MaterialProperties;
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            match &self.component {
                ComponentData::Transform(t) => {
                    entity_mut.insert(Transform::from(*t));
                }
                ComponentData::Name(name) => {
                    entity_mut.insert(Name::new(name.clone()));
                }
                ComponentData::Parent(parent) => {
                    let parent_entity: Entity = (*parent).into();
                    entity_mut.insert(Parent(parent_entity));
                }
                ComponentData::MeshHandle(id) => {
                    entity_mut.insert(MeshHandle::new(id.clone()));
                }
                ComponentData::MaterialHandle(id) => {
                    entity_mut.insert(MaterialHandle::new(id.clone()));
                }
                ComponentData::MaterialProperties(props) => {
                    entity_mut.insert(MaterialPropertiesComponent(
                        MaterialProperties::new()
                            .with_base_color(props.base_color)
                            .with_metallic(props.metallic)
                            .with_roughness(props.roughness)
                            .with_emissive_strength(props.emissive_strength),
                    ));
                }
                ComponentData::RigidBody(rb) => {
                    entity_mut.insert(match rb {
                        SerializableRigidBody::Dynamic => RigidBody::Dynamic,
                        SerializableRigidBody::Static => RigidBody::Static,
                        SerializableRigidBody::Kinematic => RigidBody::Kinematic,
                    });
                }
                ComponentData::Collider(col) => {
                    entity_mut.insert(match col {
                        SerializableCollider::Cuboid { hx, hy, hz } => {
                            Collider::cuboid(*hx, *hy, *hz)
                        }
                        SerializableCollider::Sphere { radius } => Collider::sphere(*radius),
                        SerializableCollider::CapsuleY {
                            half_height,
                            radius,
                        } => Collider::capsule_y(*half_height, *radius),
                        SerializableCollider::CapsuleX {
                            half_height,
                            radius,
                        } => Collider::capsule_x(*half_height, *radius),
                        SerializableCollider::CapsuleZ {
                            half_height,
                            radius,
                        } => Collider::capsule_z(*half_height, *radius),
                        SerializableCollider::CylinderY {
                            half_height,
                            radius,
                        } => Collider::cylinder_y(*half_height, *radius),
                    });
                }
                ComponentData::PhysicsVelocity(vel) => {
                    entity_mut.insert(PhysicsVelocity::new(
                        Vec3::from_array(vel.linear),
                        Vec3::from_array(vel.angular),
                    ));
                }
                ComponentData::Mass(mass) => {
                    entity_mut.insert(Mass::with_inertia(mass.mass, mass.angular_inertia));
                }
                ComponentData::AudioSource(audio) => {
                    entity_mut.insert(
                        AudioSource::new(audio.path.clone())
                            .with_volume(audio.volume)
                            .with_spatial(audio.spatial)
                            .with_looping(audio.looping)
                            .with_max_distance(audio.max_distance)
                            .with_reference_distance(audio.reference_distance),
                    );
                }
                ComponentData::PerspectiveProjection(cam) => {
                    entity_mut.insert(PerspectiveProjection::new(
                        cam.fov,
                        cam.aspect_ratio,
                        cam.near,
                        cam.far,
                    ));
                }
            }
            self.executed = false;
            Ok(())
        } else {
            Err("Entity not found".to_string())
        }
    }

    fn description(&self) -> String {
        let entity: Entity = self.entity.into();
        let component_name = match &self.component {
            ComponentData::Transform(_) => "Transform",
            ComponentData::Name(_) => "Name",
            ComponentData::Parent(_) => "Parent",
            ComponentData::MeshHandle(_) => "MeshHandle",
            ComponentData::MaterialHandle(_) => "MaterialHandle",
            ComponentData::MaterialProperties(_) => "MaterialProperties",
            ComponentData::RigidBody(_) => "RigidBody",
            ComponentData::Collider(_) => "Collider",
            ComponentData::PhysicsVelocity(_) => "PhysicsVelocity",
            ComponentData::Mass(_) => "Mass",
            ComponentData::AudioSource(_) => "AudioSource",
            ComponentData::PerspectiveProjection(_) => "PerspectiveProjection",
        };
        format!("Remove {component_name} from {entity:?}")
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::RemoveComponent(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "RemoveComponent"
    }
}

/// Command for changing an entity's parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParentCommand {
    /// Entity whose parent is being changed.
    pub entity: SerializableEntity,
    /// Old parent (None if entity had no parent).
    pub old_parent: Option<SerializableEntity>,
    /// New parent (None to remove parent).
    pub new_parent: Option<SerializableEntity>,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl SetParentCommand {
    /// Creates a new set parent command.
    pub fn new(entity: Entity, old_parent: Option<Entity>, new_parent: Option<Entity>) -> Self {
        Self {
            entity: entity.into(),
            old_parent: old_parent.map(|e| e.into()),
            new_parent: new_parent.map(|e| e.into()),
            executed: false,
        }
    }

    /// Creates a command from current world state.
    pub fn from_world(entity: Entity, new_parent: Option<Entity>, world: &World) -> Result<Self> {
        let old_parent = world
            .get_entity(entity)
            .and_then(|e| e.get::<Parent>())
            .map(|p| p.0.into());

        Ok(Self {
            entity: entity.into(),
            old_parent,
            new_parent: new_parent.map(|e| e.into()),
            executed: false,
        })
    }

    fn apply_parent(&self, world: &mut World, parent: Option<SerializableEntity>) -> Result<()> {
        let entity: Entity = self.entity.into();
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            if let Some(parent_serializable) = parent {
                let parent_entity: Entity = parent_serializable.into();
                entity_mut.insert(Parent(parent_entity));

                if let Some(mut parent_mut) = world.get_entity_mut(parent_entity) {
                    if let Some(mut children) = parent_mut.get_mut::<Children>() {
                        if !children.0.contains(&entity) {
                            children.0.push(entity);
                        }
                    } else {
                        parent_mut.insert(Children::with_children(vec![entity]));
                    }
                }
            } else {
                entity_mut.remove::<Parent>();
            }
            Ok(())
        } else {
            Err("Entity not found".to_string())
        }
    }
}

impl EditorCommand for SetParentCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        self.apply_parent(world, self.new_parent)?;
        self.executed = true;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        self.apply_parent(world, self.old_parent)?;
        self.executed = false;
        Ok(())
    }

    fn description(&self) -> String {
        let entity: Entity = self.entity.into();
        match self.new_parent {
            Some(parent_ser) => {
                let parent: Entity = parent_ser.into();
                format!("Set Parent of {entity:?} to {parent:?}")
            }
            None => format!("Remove Parent from {entity:?}"),
        }
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::SetParent(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "SetParent"
    }
}

/// Command that groups multiple commands together.
///
/// Executes all child commands in order, and undoes them in reverse order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeCommand {
    /// Child commands to execute.
    pub commands: Vec<SerializableCommand>,
    /// Description for the composite command.
    pub description: String,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl CompositeCommand {
    /// Creates a new composite command.
    pub fn new(description: String) -> Self {
        Self {
            commands: Vec::new(),
            description,
            executed: false,
        }
    }

    /// Adds a command to the composite.
    pub fn add_command(&mut self, command: SerializableCommand) {
        self.commands.push(command);
    }

    /// Returns the number of child commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns true if there are no child commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl EditorCommand for CompositeCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        for command in &mut self.commands {
            // Execute directly on the SerializableCommand variant to preserve state
            match command {
                SerializableCommand::TransformEdit(cmd) => cmd.execute(world)?,
                SerializableCommand::CreateEntity(cmd) => cmd.execute(world)?,
                SerializableCommand::DeleteEntity(cmd) => cmd.execute(world)?,
                SerializableCommand::AddComponent(cmd) => cmd.execute(world)?,
                SerializableCommand::RemoveComponent(cmd) => cmd.execute(world)?,
                SerializableCommand::SetParent(cmd) => cmd.execute(world)?,
                SerializableCommand::Composite(cmd) => cmd.execute(world)?,
                SerializableCommand::CopyEntity(cmd) => cmd.execute(world)?,
                SerializableCommand::PasteEntity(cmd) => cmd.execute(world)?,
            }
        }
        self.executed = true;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        // Undo commands in reverse order
        for command in self.commands.iter_mut().rev() {
            match command {
                SerializableCommand::TransformEdit(cmd) => cmd.undo(world)?,
                SerializableCommand::CreateEntity(cmd) => cmd.undo(world)?,
                SerializableCommand::DeleteEntity(cmd) => cmd.undo(world)?,
                SerializableCommand::AddComponent(cmd) => cmd.undo(world)?,
                SerializableCommand::RemoveComponent(cmd) => cmd.undo(world)?,
                SerializableCommand::SetParent(cmd) => cmd.undo(world)?,
                SerializableCommand::Composite(cmd) => cmd.undo(world)?,
                SerializableCommand::CopyEntity(cmd) => cmd.undo(world)?,
                SerializableCommand::PasteEntity(cmd) => cmd.undo(world)?,
            }
        }
        self.executed = false;
        Ok(())
    }

    fn redo(&mut self, world: &mut World) -> Result<()> {
        self.execute(world)
    }

    fn description(&self) -> String {
        if self.commands.len() == 1 {
            self.description.clone()
        } else {
            format!("{} ({} operations)", self.description, self.commands.len())
        }
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::Composite(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "Composite"
    }
}

/// Command for copying an entity to clipboard.
///
/// This command captures the entity's state including all components and hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyEntityCommand {
    /// Entity to copy.
    pub entity: SerializableEntity,
    /// Captured components from the entity.
    pub components: Vec<ComponentData>,
    /// Parent entity if the entity had one.
    pub parent: Option<SerializableEntity>,
    /// Children entities if the entity had any.
    pub children: Vec<SerializableEntity>,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl CopyEntityCommand {
    /// Creates a copy command by capturing entity state from the world.
    pub fn from_world(entity: Entity, world: &World) -> Result<Self> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let mut components = Vec::new();

        if let Some(entity_ref) = world.get_entity(entity) {
            // Capture Transform
            if let Some(transform) = entity_ref.get::<Transform>() {
                components.push(ComponentData::Transform((*transform).into()));
            }

            // Capture Name
            if let Some(name) = entity_ref.get::<Name>() {
                components.push(ComponentData::Name(name.0.clone()));
            }

            // Capture Parent
            let parent = entity_ref.get::<Parent>().map(|p| p.0.into());

            // Capture Children
            let children = entity_ref
                .get::<Children>()
                .map(|c| c.0.iter().map(|e| (*e).into()).collect())
                .unwrap_or_default();

            // Capture MeshHandle
            if let Some(mesh_handle) = entity_ref.get::<MeshHandle>() {
                components.push(ComponentData::MeshHandle(mesh_handle.id().to_string()));
            }

            // Capture MaterialHandle
            if let Some(material_handle) = entity_ref.get::<MaterialHandle>() {
                components.push(ComponentData::MaterialHandle(
                    material_handle.id().to_string(),
                ));
            }

            // Capture MaterialProperties
            if let Some(material_props) = entity_ref.get::<MaterialPropertiesComponent>() {
                let props = &material_props.0;
                components.push(ComponentData::MaterialProperties(
                    SerializableMaterialProperties {
                        base_color: props.base_color(),
                        metallic: props.metallic(),
                        roughness: props.roughness(),
                        emissive_strength: props.emissive_strength(),
                    },
                ));
            }

            // Capture RigidBody
            if let Some(rigid_body) = entity_ref.get::<RigidBody>() {
                let rb_serializable = match rigid_body {
                    RigidBody::Dynamic => SerializableRigidBody::Dynamic,
                    RigidBody::Static => SerializableRigidBody::Static,
                    RigidBody::Kinematic => SerializableRigidBody::Kinematic,
                };
                components.push(ComponentData::RigidBody(rb_serializable));
            }

            // Capture Collider
            if let Some(collider) = entity_ref.get::<Collider>() {
                let serializable = match collider {
                    Collider::Cuboid { hx, hy, hz } => SerializableCollider::Cuboid {
                        hx: *hx,
                        hy: *hy,
                        hz: *hz,
                    },
                    Collider::Sphere { radius } => SerializableCollider::Sphere { radius: *radius },
                    Collider::CapsuleY {
                        half_height,
                        radius,
                    } => SerializableCollider::CapsuleY {
                        half_height: *half_height,
                        radius: *radius,
                    },
                    Collider::CapsuleX {
                        half_height,
                        radius,
                    } => SerializableCollider::CapsuleX {
                        half_height: *half_height,
                        radius: *radius,
                    },
                    Collider::CapsuleZ {
                        half_height,
                        radius,
                    } => SerializableCollider::CapsuleZ {
                        half_height: *half_height,
                        radius: *radius,
                    },
                    Collider::CylinderY {
                        half_height,
                        radius,
                    } => SerializableCollider::CylinderY {
                        half_height: *half_height,
                        radius: *radius,
                    },
                };
                components.push(ComponentData::Collider(serializable));
            }

            // Capture PhysicsVelocity
            if let Some(velocity) = entity_ref.get::<PhysicsVelocity>() {
                components.push(ComponentData::PhysicsVelocity(
                    SerializablePhysicsVelocity {
                        linear: velocity.linear.to_array(),
                        angular: velocity.angular.to_array(),
                    },
                ));
            }

            // Capture Mass
            if let Some(mass) = entity_ref.get::<Mass>() {
                components.push(ComponentData::Mass(SerializableMass {
                    mass: mass.mass(),
                    angular_inertia: mass.angular_inertia(),
                }));
            }

            // Capture AudioSource
            if let Some(audio_source) = entity_ref.get::<AudioSource>() {
                components.push(ComponentData::AudioSource(SerializableAudioSource {
                    path: audio_source.path().to_string(),
                    volume: audio_source.volume(),
                    spatial: audio_source.is_spatial(),
                    looping: audio_source.is_looping(),
                    max_distance: audio_source.max_distance(),
                    reference_distance: audio_source.reference_distance(),
                }));
            }

            // Capture PerspectiveProjection
            if let Some(projection) = entity_ref.get::<PerspectiveProjection>() {
                components.push(ComponentData::PerspectiveProjection(
                    SerializablePerspectiveProjection {
                        fov: projection.fov(),
                        aspect_ratio: projection.aspect_ratio(),
                        near: projection.near(),
                        far: projection.far(),
                    },
                ));
            }

            Ok(Self {
                entity: entity.into(),
                components,
                parent,
                children,
                executed: false,
            })
        } else {
            Err("Entity not found".to_string())
        }
    }
}

impl EditorCommand for CopyEntityCommand {
    fn execute(&mut self, _world: &mut World) -> Result<()> {
        self.executed = true;
        Ok(())
    }

    fn undo(&mut self, _world: &mut World) -> Result<()> {
        self.executed = false;
        Ok(())
    }

    fn description(&self) -> String {
        let entity: Entity = self.entity.into();
        format!("Copy Entity {entity:?}")
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::CopyEntity(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "CopyEntity"
    }
}

/// Command for pasting a copied entity.
///
/// This command creates a new entity with the same components and hierarchy as the copied entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasteEntityCommand {
    /// Components to add to the new entity.
    pub components: Vec<ComponentData>,
    /// Parent entity for the pasted entity (if any).
    pub parent: Option<SerializableEntity>,
    /// The entity that was created during paste.
    pub created_entity: Option<SerializableEntity>,
    /// Position offset to apply to the pasted entity.
    pub offset: Option<[f32; 3]>,
    /// Whether the command has been executed.
    #[serde(skip)]
    executed: bool,
}

impl PasteEntityCommand {
    /// Creates a new paste command from a copy command.
    pub fn from_copy(copy_cmd: &CopyEntityCommand, offset: Option<Vec3>) -> Self {
        let mut components = copy_cmd.components.clone();

        // Apply offset to transform if provided
        if let Some(offset_vec) = offset {
            if let Some(transform_idx) = components
                .iter()
                .position(|c| matches!(c, ComponentData::Transform(_)))
            {
                if let ComponentData::Transform(ref mut transform) = components[transform_idx] {
                    transform.translation[0] += offset_vec.x;
                    transform.translation[1] += offset_vec.y;
                    transform.translation[2] += offset_vec.z;
                }
            }
        }

        // Update name to add " Copy" suffix
        if let Some(name_idx) = components
            .iter()
            .position(|c| matches!(c, ComponentData::Name(_)))
        {
            if let ComponentData::Name(ref mut name) = components[name_idx] {
                *name = format!("{name} Copy");
            }
        }

        Self {
            components,
            parent: copy_cmd.parent,
            created_entity: None,
            offset: offset.map(|v| v.to_array()),
            executed: false,
        }
    }

    /// Creates a paste command with specific components and optional parent.
    pub fn new(
        components: Vec<ComponentData>,
        parent: Option<SerializableEntity>,
        offset: Option<Vec3>,
    ) -> Self {
        Self {
            components,
            parent,
            created_entity: None,
            offset: offset.map(|v| v.to_array()),
            executed: false,
        }
    }
}

impl EditorCommand for PasteEntityCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        use praxis_audio::AudioSource;
        use praxis_ecs::{
            MaterialHandle, MaterialPropertiesComponent, MeshHandle, PerspectiveProjection,
        };
        use praxis_graphics::MaterialProperties;
        use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};

        let mut entity_mut = world.spawn_empty();
        let entity = entity_mut.id();

        for component in &self.components {
            match component {
                ComponentData::Transform(t) => {
                    entity_mut.insert(Transform::from(*t));
                }
                ComponentData::Name(name) => {
                    entity_mut.insert(Name::new(name.clone()));
                }
                ComponentData::Parent(parent) => {
                    let parent_entity: Entity = (*parent).into();
                    entity_mut.insert(Parent(parent_entity));
                }
                ComponentData::MeshHandle(id) => {
                    entity_mut.insert(MeshHandle::new(id.clone()));
                }
                ComponentData::MaterialHandle(id) => {
                    entity_mut.insert(MaterialHandle::new(id.clone()));
                }
                ComponentData::MaterialProperties(props) => {
                    entity_mut.insert(MaterialPropertiesComponent(
                        MaterialProperties::new()
                            .with_base_color(props.base_color)
                            .with_metallic(props.metallic)
                            .with_roughness(props.roughness)
                            .with_emissive_strength(props.emissive_strength),
                    ));
                }
                ComponentData::RigidBody(rb) => {
                    entity_mut.insert(match rb {
                        SerializableRigidBody::Dynamic => RigidBody::Dynamic,
                        SerializableRigidBody::Static => RigidBody::Static,
                        SerializableRigidBody::Kinematic => RigidBody::Kinematic,
                    });
                }
                ComponentData::Collider(col) => {
                    entity_mut.insert(match col {
                        SerializableCollider::Cuboid { hx, hy, hz } => {
                            Collider::cuboid(*hx, *hy, *hz)
                        }
                        SerializableCollider::Sphere { radius } => Collider::sphere(*radius),
                        SerializableCollider::CapsuleY {
                            half_height,
                            radius,
                        } => Collider::capsule_y(*half_height, *radius),
                        SerializableCollider::CapsuleX {
                            half_height,
                            radius,
                        } => Collider::capsule_x(*half_height, *radius),
                        SerializableCollider::CapsuleZ {
                            half_height,
                            radius,
                        } => Collider::capsule_z(*half_height, *radius),
                        SerializableCollider::CylinderY {
                            half_height,
                            radius,
                        } => Collider::cylinder_y(*half_height, *radius),
                    });
                }
                ComponentData::PhysicsVelocity(vel) => {
                    entity_mut.insert(PhysicsVelocity::new(
                        Vec3::from_array(vel.linear),
                        Vec3::from_array(vel.angular),
                    ));
                }
                ComponentData::Mass(mass) => {
                    entity_mut.insert(Mass::with_inertia(mass.mass, mass.angular_inertia));
                }
                ComponentData::AudioSource(audio) => {
                    entity_mut.insert(
                        AudioSource::new(audio.path.clone())
                            .with_volume(audio.volume)
                            .with_spatial(audio.spatial)
                            .with_looping(audio.looping)
                            .with_max_distance(audio.max_distance)
                            .with_reference_distance(audio.reference_distance),
                    );
                }
                ComponentData::PerspectiveProjection(cam) => {
                    entity_mut.insert(PerspectiveProjection::new(
                        cam.fov,
                        cam.aspect_ratio,
                        cam.near,
                        cam.far,
                    ));
                }
            }
        }

        if let Some(parent) = self.parent {
            let parent_entity: Entity = parent.into();
            entity_mut.insert(Parent(parent_entity));
        }

        self.created_entity = Some(entity.into());
        self.executed = true;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(serializable_entity) = self.created_entity {
            let entity: Entity = serializable_entity.into();
            if world.despawn(entity) {
                self.executed = false;
                Ok(())
            } else {
                Err("Failed to despawn entity".to_string())
            }
        } else {
            Err("Entity not found".to_string())
        }
    }

    fn description(&self) -> String {
        "Paste Entity".to_string()
    }

    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::PasteEntity(self.clone());
        ron::to_string(&serializable).map_err(|e| format!("Failed to serialize command: {e}"))
    }

    fn type_id(&self) -> &'static str {
        "PasteEntity"
    }
}

/// Command history system managing undo/redo stacks.
///
/// Maintains two stacks:
/// - Undo stack: Commands that can be undone
/// - Redo stack: Commands that can be redone
///
/// When a new command is executed, the redo stack is cleared.
/// Commands can be serialized to/from RON for persistence.
pub struct CommandHistory {
    /// Stack of commands that can be undone.
    pub(crate) undo_stack: VecDeque<Box<dyn EditorCommand>>,
    /// Stack of commands that can be redone.
    pub(crate) redo_stack: VecDeque<Box<dyn EditorCommand>>,
    /// Maximum number of commands to keep in history.
    max_history_size: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    /// Creates a new command history.
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history_size: MAX_HISTORY_SIZE,
        }
    }

    /// Creates a command history with a custom maximum size.
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history_size: max_size,
        }
    }

    /// Executes a command and adds it to the undo stack.
    ///
    /// This clears the redo stack since executing a new command
    /// invalidates any previously undone commands.
    pub fn execute(
        &mut self,
        world: &mut World,
        mut command: Box<dyn EditorCommand>,
    ) -> Result<()> {
        command.execute(world)?;

        self.redo_stack.clear();
        self.undo_stack.push_back(command);

        if self.undo_stack.len() > self.max_history_size {
            self.undo_stack.pop_front();
        }

        Ok(())
    }

    /// Undoes the last command.
    ///
    /// Returns true if a command was undone, false if the undo stack is empty.
    pub fn undo(&mut self, world: &mut World) -> Result<bool> {
        if let Some(mut command) = self.undo_stack.pop_back() {
            command.undo(world)?;
            self.redo_stack.push_back(command);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redoes the last undone command.
    ///
    /// Returns true if a command was redone, false if the redo stack is empty.
    pub fn redo(&mut self, world: &mut World) -> Result<bool> {
        if let Some(mut command) = self.redo_stack.pop_back() {
            command.redo(world)?;
            self.undo_stack.push_back(command);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if there are commands that can be undone.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if there are commands that can be redone.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Gets a description of the next command that would be undone.
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Gets a description of the next command that would be redone.
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Clears all undo/redo history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Returns the number of commands in the undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns the number of commands in the redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Serializes the entire command history to RON.
    ///
    /// This allows saving and loading command history for features like
    /// session recovery or replay functionality.
    pub fn to_ron(&self) -> Result<String> {
        let mut serialized_commands = Vec::new();

        for command in &self.undo_stack {
            let ron = command.to_ron()?;
            serialized_commands.push(ron);
        }

        #[derive(Serialize)]
        struct HistoryData {
            commands: Vec<String>,
        }

        let data = HistoryData {
            commands: serialized_commands,
        };

        ron::to_string(&data).map_err(|e| format!("Failed to serialize history: {e}"))
    }

    /// Loads command history from RON.
    ///
    /// Note: This clears the current history and replaces it with the loaded commands.
    pub fn from_ron(&mut self, ron: &str) -> Result<()> {
        #[derive(Deserialize)]
        struct HistoryData {
            commands: Vec<String>,
        }

        let data: HistoryData =
            ron::from_str(ron).map_err(|e| format!("Failed to deserialize history: {e}"))?;

        self.clear();

        for command_ron in data.commands {
            let serializable = SerializableCommand::from_ron(&command_ron)?;
            self.undo_stack.push_back(serializable.to_trait_object());
        }

        Ok(())
    }
}

/// Resource wrapper for command history that can be inserted into the ECS world.
///
/// This system provides:
/// - Command execution with undo/redo support
/// - Dirty state tracking for unsaved changes
/// - Keyboard shortcuts (Ctrl+Z, Ctrl+Y)
/// - Menu bar integration
/// - Maximum history size of 100 entries
#[derive(Resource)]
pub struct UndoRedoSystem {
    pub history: CommandHistory,
    /// Tracks whether there are unsaved changes.
    /// Set to true when commands are executed, false when saved.
    dirty: bool,
    /// The undo count when the last save occurred.
    /// Used to determine if we've returned to a saved state.
    saved_undo_count: usize,
}

impl Default for UndoRedoSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoRedoSystem {
    /// Creates a new undo/redo system.
    pub fn new() -> Self {
        Self {
            history: CommandHistory::new(),
            dirty: false,
            saved_undo_count: 0,
        }
    }

    /// Executes a command and marks the state as dirty.
    pub fn execute_command(
        &mut self,
        world: &mut World,
        command: Box<dyn EditorCommand>,
    ) -> Result<()> {
        self.history.execute(world, command)?;
        self.dirty = true;
        Ok(())
    }

    /// Undoes the last command and updates dirty state.
    pub fn undo(&mut self, world: &mut World) -> Result<bool> {
        let result = self.history.undo(world)?;
        if result {
            self.update_dirty_state();
        }
        Ok(result)
    }

    /// Redoes the last undone command and updates dirty state.
    pub fn redo(&mut self, world: &mut World) -> Result<bool> {
        let result = self.history.redo(world)?;
        if result {
            self.update_dirty_state();
        }
        Ok(result)
    }

    /// Returns true if there are commands that can be undone.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Returns true if there are commands that can be redone.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Gets a description of the next command that would be undone.
    pub fn undo_description(&self) -> Option<String> {
        self.history.undo_description()
    }

    /// Gets a description of the next command that would be redone.
    pub fn redo_description(&self) -> Option<String> {
        self.history.redo_description()
    }

    /// Clears all undo/redo history and resets dirty state.
    pub fn clear(&mut self) {
        self.history.clear();
        self.dirty = false;
        self.saved_undo_count = 0;
    }

    /// Returns the number of commands in the undo stack.
    pub fn undo_count(&self) -> usize {
        self.history.undo_count()
    }

    /// Returns the number of commands in the redo stack.
    pub fn redo_count(&self) -> usize {
        self.history.redo_count()
    }

    /// Returns true if there are unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the current state as saved.
    /// This resets the dirty flag and records the current undo count.
    pub fn mark_saved(&mut self) {
        self.dirty = false;
        self.saved_undo_count = self.history.undo_count();
    }

    /// Marks the state as dirty (having unsaved changes).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Updates the dirty state based on whether we're at the saved undo count.
    fn update_dirty_state(&mut self) {
        // If we've returned to the saved undo count, we're no longer dirty
        self.dirty = self.history.undo_count() != self.saved_undo_count;
    }

    /// Serializes the command history to RON.
    pub fn to_ron(&self) -> Result<String> {
        self.history.to_ron()
    }

    /// Loads command history from RON.
    /// This marks the state as dirty since we've loaded new commands.
    pub fn from_ron(&mut self, ron: &str) -> Result<()> {
        self.history.from_ron(ron)?;
        self.dirty = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_history_creation() {
        let history = CommandHistory::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_transform_edit_command() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let old_transform = Transform::default();
        let new_transform = Transform::from_xyz(10.0, 5.0, 3.0);

        let mut command = TransformEditCommand::new(entity, old_transform, new_transform);

        assert!(command.execute(&mut world).is_ok());

        if let Some(transform) = world.get::<Transform>(entity) {
            assert_eq!(transform.translation.x, 10.0);
        }

        assert!(command.undo(&mut world).is_ok());

        if let Some(transform) = world.get::<Transform>(entity) {
            assert_eq!(transform.translation.x, 0.0);
        }
    }

    #[test]
    fn test_create_entity_command() {
        let mut world = World::new();
        let mut command = CreateEntityCommand::with_transform(Transform::from_xyz(5.0, 0.0, 0.0));

        assert!(command.execute(&mut world).is_ok());
        assert!(command.entity.is_some());

        let entity: Entity = command.entity.unwrap().into();
        assert!(world.get_entity(entity).is_some());

        assert!(command.undo(&mut world).is_ok());
    }

    #[test]
    fn test_delete_entity_command() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), Name::new("Test"))).id();

        let command = DeleteEntityCommand::from_world(entity, &world);
        assert!(command.is_ok());

        let mut command = command.unwrap();
        assert!(command.execute(&mut world).is_ok());
    }

    #[test]
    fn test_add_component_command() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut command =
            AddComponentCommand::new(entity, ComponentData::Name("TestEntity".to_string()));

        assert!(command.execute(&mut world).is_ok());
        assert!(world.get::<Name>(entity).is_some());

        assert!(command.undo(&mut world).is_ok());
        assert!(world.get::<Name>(entity).is_none());
    }

    #[test]
    fn test_composite_command() {
        let _world = World::new();
        let mut composite = CompositeCommand::new("Create and Name Entity".to_string());

        let create_cmd = CreateEntityCommand::with_transform(Transform::default());
        composite.add_command(SerializableCommand::CreateEntity(create_cmd));

        assert_eq!(composite.len(), 1);
        assert!(!composite.is_empty());
    }

    #[test]
    fn test_command_history_execute() {
        let mut world = World::new();
        let mut history = CommandHistory::new();
        let entity = world.spawn(Transform::default()).id();

        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(5.0, 0.0, 0.0),
        ));

        assert!(history.execute(&mut world, command).is_ok());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_history_undo_redo() {
        let mut world = World::new();
        let mut history = CommandHistory::new();
        let entity = world.spawn(Transform::default()).id();

        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(5.0, 0.0, 0.0),
        ));

        history.execute(&mut world, command).unwrap();

        let result = history.undo(&mut world);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let result = history.redo(&mut world);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_serialization() {
        let entity = Entity::from_raw(42);
        let command = TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(1.0, 2.0, 3.0),
        );

        let ron = command.to_ron();
        assert!(ron.is_ok());

        let serializable = SerializableCommand::from_ron(&ron.unwrap());
        assert!(serializable.is_ok());
    }

    #[test]
    fn test_undo_redo_system() {
        let system = UndoRedoSystem::new();
        assert!(!system.can_undo());
        assert!(!system.can_redo());
        assert_eq!(system.undo_count(), 0);
        assert_eq!(system.redo_count(), 0);
        assert!(!system.is_dirty());
    }

    #[test]
    fn test_dirty_state_tracking() {
        let mut world = World::new();
        let mut system = UndoRedoSystem::new();

        // Initially clean
        assert!(!system.is_dirty());

        // Execute command - becomes dirty
        let entity = world.spawn(Transform::default()).id();
        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(1.0, 0.0, 0.0),
        ));
        system.execute_command(&mut world, command).unwrap();
        assert!(system.is_dirty());

        // Mark as saved - becomes clean
        system.mark_saved();
        assert!(!system.is_dirty());

        // Execute another command - becomes dirty again
        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::from_xyz(1.0, 0.0, 0.0),
            Transform::from_xyz(2.0, 0.0, 0.0),
        ));
        system.execute_command(&mut world, command).unwrap();
        assert!(system.is_dirty());

        // Undo back to saved state - becomes clean
        system.undo(&mut world).unwrap();
        assert!(!system.is_dirty());

        // Redo - becomes dirty
        system.redo(&mut world).unwrap();
        assert!(system.is_dirty());
    }

    #[test]
    fn test_max_history_size() {
        let history = CommandHistory::new();
        // Verify max history size is 100
        assert_eq!(history.max_history_size, MAX_HISTORY_SIZE);
        assert_eq!(MAX_HISTORY_SIZE, 100);
    }

    #[test]
    fn test_copy_entity_command() {
        let mut world = World::new();
        let entity = world
            .spawn((Transform::from_xyz(5.0, 10.0, 15.0), Name::new("Test")))
            .id();

        let command = CopyEntityCommand::from_world(entity, &world);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.components.len(), 2);
    }

    #[test]
    fn test_paste_entity_command() {
        let mut world = World::new();
        let original = world
            .spawn((Transform::from_xyz(10.0, 0.0, 0.0), Name::new("Original")))
            .id();

        let copy_cmd = CopyEntityCommand::from_world(original, &world).unwrap();
        let mut paste_cmd = PasteEntityCommand::from_copy(&copy_cmd, Some(Vec3::new(5.0, 0.0, 0.0)));

        assert!(paste_cmd.execute(&mut world).is_ok());
        assert!(paste_cmd.created_entity.is_some());

        let pasted: Entity = paste_cmd.created_entity.unwrap().into();
        let pasted_transform = world.get::<Transform>(pasted).unwrap();
        assert_eq!(pasted_transform.translation.x, 15.0);

        let pasted_name = world.get::<Name>(pasted).unwrap();
        assert_eq!(pasted_name.0, "Original Copy");
    }

    #[test]
    fn test_paste_entity_undo() {
        let mut world = World::new();
        let original = world
            .spawn((Transform::default(), Name::new("Original")))
            .id();

        let copy_cmd = CopyEntityCommand::from_world(original, &world).unwrap();
        let mut paste_cmd = PasteEntityCommand::from_copy(&copy_cmd, None);

        paste_cmd.execute(&mut world).unwrap();
        let pasted: Entity = paste_cmd.created_entity.unwrap().into();
        assert!(world.get_entity(pasted).is_some());

        paste_cmd.undo(&mut world).unwrap();
        assert!(world.get_entity(pasted).is_none());
    }
}
