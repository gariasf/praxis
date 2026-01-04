//! Level of Detail (LOD) system for distance-based mesh switching.
//!
//! LOD reduces rendering cost by using simpler mesh representations for distant objects.

use bevy_ecs::entity::Entity;
use praxis_math::Vec3;
use std::collections::HashMap;

/// A single LOD level with its distance threshold and mesh identifier.
#[derive(Debug, Clone, PartialEq)]
pub struct LodLevel {
    /// Maximum distance at which this LOD is used (exclusive).
    pub distance: f32,
    /// Mesh identifier for this LOD level.
    pub mesh_id: String,
}

impl LodLevel {
    /// Creates a new LOD level.
    pub fn new(distance: f32, mesh_id: impl Into<String>) -> Self {
        Self {
            distance,
            mesh_id: mesh_id.into(),
        }
    }
}

/// A group of LOD levels for a single object type.
///
/// LOD levels should be ordered from highest detail (distance 0) to lowest detail.
#[derive(Debug, Clone)]
pub struct LodGroup {
    /// Name/identifier for this LOD group.
    pub name: String,
    /// LOD levels sorted by distance.
    pub levels: Vec<LodLevel>,
}

impl LodGroup {
    /// Creates a new LOD group.
    pub fn new(name: impl Into<String>, levels: Vec<LodLevel>) -> Self {
        let mut group = Self {
            name: name.into(),
            levels,
        };
        group.sort_levels();
        group
    }

    /// Sorts LOD levels by distance.
    fn sort_levels(&mut self) {
        self.levels
            .sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    }

    /// Selects the appropriate mesh ID for the given distance.
    pub fn select_lod(&self, distance: f32) -> Option<&str> {
        for level in &self.levels {
            if distance < level.distance {
                return Some(&level.mesh_id);
            }
        }

        self.levels.last().map(|l| l.mesh_id.as_str())
    }

    /// Returns the number of LOD levels.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

/// Result of LOD selection for an entity.
#[derive(Debug, Clone)]
pub struct LodSelection {
    /// Entity being evaluated.
    pub entity: Entity,
    /// Selected mesh ID.
    pub mesh_id: String,
    /// LOD level index (0 = highest detail).
    pub level_index: usize,
    /// Distance from camera.
    pub distance: f32,
}

/// LOD management system.
///
/// Manages LOD groups and provides distance-based mesh selection.
pub struct LodManager {
    /// Registered LOD groups.
    groups: HashMap<String, LodGroup>,
    /// Map from entity to its LOD group name.
    entity_lod_groups: HashMap<Entity, String>,
}

impl LodManager {
    /// Creates a new LOD manager.
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            entity_lod_groups: HashMap::new(),
        }
    }

    /// Registers a new LOD group.
    pub fn register_lod_group(&mut self, group: LodGroup) {
        self.groups.insert(group.name.clone(), group);
    }

    /// Registers a LOD group from levels.
    pub fn register_lod_levels(&mut self, name: impl Into<String>, levels: Vec<LodLevel>) {
        let group = LodGroup::new(name, levels);
        self.register_lod_group(group);
    }

    /// Assigns an entity to a LOD group.
    pub fn assign_entity(&mut self, entity: Entity, group_name: impl Into<String>) {
        self.entity_lod_groups.insert(entity, group_name.into());
    }

    /// Removes an entity from LOD management.
    pub fn remove_entity(&mut self, entity: Entity) {
        self.entity_lod_groups.remove(&entity);
    }

    /// Selects the appropriate LOD for an entity based on distance from camera.
    pub fn select_lod(
        &self,
        entity: Entity,
        camera_position: Vec3,
        entity_position: Vec3,
    ) -> Option<LodSelection> {
        let group_name = self.entity_lod_groups.get(&entity)?;
        let group = self.groups.get(group_name)?;

        let distance = camera_position.distance(entity_position);
        let mesh_id = group.select_lod(distance)?;

        let level_index = group
            .levels
            .iter()
            .position(|l| l.mesh_id == mesh_id)
            .unwrap_or(0);

        Some(LodSelection {
            entity,
            mesh_id: mesh_id.to_string(),
            level_index,
            distance,
        })
    }

    /// Selects LODs for multiple entities.
    pub fn select_lods(
        &self,
        entities: &[(Entity, Vec3)],
        camera_position: Vec3,
    ) -> Vec<LodSelection> {
        entities
            .iter()
            .filter_map(|(entity, position)| self.select_lod(*entity, camera_position, *position))
            .collect()
    }

    /// Returns the number of registered LOD groups.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Returns the number of entities assigned to LOD groups.
    pub fn entity_count(&self) -> usize {
        self.entity_lod_groups.len()
    }

    /// Gets a LOD group by name.
    pub fn get_group(&self, name: &str) -> Option<&LodGroup> {
        self.groups.get(name)
    }

    /// Clears all LOD groups and entity assignments.
    pub fn clear(&mut self) {
        self.groups.clear();
        self.entity_lod_groups.clear();
    }
}

impl Default for LodManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level_creation() {
        let level = LodLevel::new(50.0, "tree_medium");
        assert_eq!(level.distance, 50.0);
        assert_eq!(level.mesh_id, "tree_medium");
    }

    #[test]
    fn test_lod_group_creation() {
        let levels = vec![
            LodLevel::new(0.0, "tree_high"),
            LodLevel::new(50.0, "tree_medium"),
            LodLevel::new(100.0, "tree_low"),
        ];
        let group = LodGroup::new("tree", levels);
        assert_eq!(group.level_count(), 3);
    }

    #[test]
    fn test_lod_group_selection() {
        let levels = vec![
            LodLevel::new(0.0, "tree_high"),
            LodLevel::new(50.0, "tree_medium"),
            LodLevel::new(100.0, "tree_low"),
        ];
        let group = LodGroup::new("tree", levels);

        assert_eq!(group.select_lod(10.0), Some("tree_high"));
        assert_eq!(group.select_lod(60.0), Some("tree_medium"));
        assert_eq!(group.select_lod(150.0), Some("tree_low"));
    }

    #[test]
    fn test_lod_manager_registration() {
        let mut manager = LodManager::new();
        let levels = vec![
            LodLevel::new(0.0, "rock_high"),
            LodLevel::new(30.0, "rock_low"),
        ];
        manager.register_lod_levels("rock", levels);

        assert_eq!(manager.group_count(), 1);
    }

    #[test]
    fn test_lod_manager_entity_assignment() {
        let mut manager = LodManager::new();
        let entity = Entity::from_raw(1);

        manager.register_lod_levels(
            "tree",
            vec![
                LodLevel::new(0.0, "tree_high"),
                LodLevel::new(50.0, "tree_low"),
            ],
        );

        manager.assign_entity(entity, "tree");
        assert_eq!(manager.entity_count(), 1);
    }

    #[test]
    fn test_lod_manager_selection() {
        let mut manager = LodManager::new();
        let entity = Entity::from_raw(1);

        manager.register_lod_levels(
            "tree",
            vec![
                LodLevel::new(0.0, "tree_high"),
                LodLevel::new(50.0, "tree_low"),
            ],
        );

        manager.assign_entity(entity, "tree");

        let camera_pos = Vec3::ZERO;
        let entity_pos = Vec3::new(30.0, 0.0, 0.0);

        let selection = manager.select_lod(entity, camera_pos, entity_pos);
        assert!(selection.is_some());
        assert_eq!(selection.unwrap().mesh_id, "tree_high");
    }

    #[test]
    fn test_lod_manager_batch_selection() {
        let mut manager = LodManager::new();

        manager.register_lod_levels(
            "tree",
            vec![
                LodLevel::new(0.0, "tree_high"),
                LodLevel::new(50.0, "tree_low"),
            ],
        );

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        manager.assign_entity(entity1, "tree");
        manager.assign_entity(entity2, "tree");

        let entities = vec![
            (entity1, Vec3::new(20.0, 0.0, 0.0)),
            (entity2, Vec3::new(60.0, 0.0, 0.0)),
        ];

        let selections = manager.select_lods(&entities, Vec3::ZERO);
        assert_eq!(selections.len(), 2);
        assert_eq!(selections[0].mesh_id, "tree_high");
        assert_eq!(selections[1].mesh_id, "tree_low");
    }
}
