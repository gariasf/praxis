//! Lag compensation for fair gameplay.

use crate::{NetworkId, ReplicatedTransform};
use bevy_ecs::prelude::*;
use praxis_math::Vec3;
use std::collections::{HashMap, VecDeque};
use praxis_utils::Result;

/// Historical state of an entity at a specific time.
#[derive(Debug, Clone, Copy)]
pub struct HistoricalState {
    /// Timestamp in milliseconds
    pub timestamp: u64,
    
    /// Transform at this time
    pub transform: ReplicatedTransform,
}

impl HistoricalState {
    /// Creates a new historical state.
    pub fn new(timestamp: u64, transform: ReplicatedTransform) -> Self {
        Self {
            timestamp,
            transform,
        }
    }
}

/// Buffer storing historical states for lag compensation.
#[derive(Debug)]
pub struct HistoryBuffer {
    /// Historical states ordered by timestamp (oldest first)
    states: VecDeque<HistoricalState>,
    
    /// Maximum history duration in milliseconds
    max_history_ms: u64,
}

impl HistoryBuffer {
    /// Creates a new history buffer.
    pub fn new(max_history_ms: u64) -> Self {
        Self {
            states: VecDeque::new(),
            max_history_ms,
        }
    }
    
    /// Adds a state to the history.
    pub fn add_state(&mut self, state: HistoricalState) {
        self.states.push_back(state);
        
        // Remove old states
        let cutoff_time = state.timestamp.saturating_sub(self.max_history_ms);
        while let Some(oldest) = self.states.front() {
            if oldest.timestamp < cutoff_time {
                self.states.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// Gets the state at a specific time (or closest before that time).
    pub fn get_state_at(&self, timestamp: u64) -> Option<HistoricalState> {
        // Find the last state before or at the target time
        self.states
            .iter()
            .rev()
            .find(|state| state.timestamp <= timestamp)
            .copied()
    }
    
    /// Interpolates state between two historical states.
    pub fn interpolate_state_at(&self, timestamp: u64) -> Option<HistoricalState> {
        if self.states.len() < 2 {
            return self.get_state_at(timestamp);
        }
        
        // Find surrounding states
        let next_idx = self.states
            .iter()
            .position(|s| s.timestamp > timestamp)?;
        
        if next_idx == 0 {
            return Some(self.states[0]);
        }
        
        let prev = self.states[next_idx - 1];
        let next = self.states[next_idx];
        
        // Interpolate
        let time_diff = next.timestamp - prev.timestamp;
        if time_diff == 0 {
            return Some(prev);
        }
        
        let t = ((timestamp - prev.timestamp) as f32) / (time_diff as f32);
        let t = t.clamp(0.0, 1.0);
        
        let interpolated_transform = ReplicatedTransform {
            translation: prev.transform.translation.lerp(next.transform.translation, t),
            rotation: prev.transform.rotation.slerp(next.transform.rotation, t),
            scale: prev.transform.scale.lerp(next.transform.scale, t),
        };
        
        Some(HistoricalState::new(timestamp, interpolated_transform))
    }
    
    /// Clears all history.
    pub fn clear(&mut self) {
        self.states.clear();
    }
    
    /// Gets the number of states in the buffer.
    pub fn len(&self) -> usize {
        self.states.len()
    }
    
    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

/// Client state history for lag compensation.
#[derive(Component, Debug)]
pub struct ClientStateHistory {
    /// Network ID to history buffer mapping
    histories: HashMap<u64, HistoryBuffer>,
    
    /// Maximum history duration in milliseconds
    max_history_ms: u64,
}

impl ClientStateHistory {
    /// Creates a new client state history.
    pub fn new(max_history_ms: u64) -> Self {
        Self {
            histories: HashMap::new(),
            max_history_ms,
        }
    }
    
    /// Records a state for an entity.
    pub fn record_state(&mut self, network_id: u64, state: HistoricalState) {
        self.histories
            .entry(network_id)
            .or_insert_with(|| HistoryBuffer::new(self.max_history_ms))
            .add_state(state);
    }
    
    /// Gets the state of an entity at a specific time.
    pub fn get_state_at(&self, network_id: u64, timestamp: u64) -> Option<HistoricalState> {
        self.histories
            .get(&network_id)
            .and_then(|buffer| buffer.interpolate_state_at(timestamp))
    }
    
    /// Removes history for an entity.
    pub fn remove_entity(&mut self, network_id: u64) {
        self.histories.remove(&network_id);
    }
    
    /// Clears all history.
    pub fn clear(&mut self) {
        self.histories.clear();
    }
}

/// Lag compensation system.
pub struct LagCompensation {
    /// Client state histories
    client_histories: HashMap<u64, ClientStateHistory>,
    
    /// Maximum history duration in milliseconds
    max_history_ms: u64,
}

impl LagCompensation {
    /// Creates a new lag compensation system.
    pub fn new(max_history_ms: u64) -> Self {
        Self {
            client_histories: HashMap::new(),
            max_history_ms,
        }
    }
    
    /// Records the current state of all entities for a client.
    pub fn record_snapshot(
        &mut self,
        client_id: u64,
        timestamp: u64,
        world: &mut bevy_ecs::world::World,
    ) {
        let history = self.client_histories
            .entry(client_id)
            .or_insert_with(|| ClientStateHistory::new(self.max_history_ms));
        
        // Query all networked entities
        let mut query = world.query::<(&NetworkId, &ReplicatedTransform)>();
        
        for (network_id, transform) in query.iter(world) {
            let state = HistoricalState::new(timestamp, *transform);
            history.record_state(network_id.get(), state);
        }
    }
    
    /// Rewinds the world to a client's perspective at a given time.
    pub fn rewind_to_client_time(
        &self,
        client_id: u64,
        timestamp: u64,
        world: &mut bevy_ecs::world::World,
    ) -> Result<RewindState> {
        let history = self.client_histories.get(&client_id)
            .ok_or_else(|| color_eyre::eyre::eyre!("No history for client {}", client_id))?;
        
        let mut rewound_entities = HashMap::new();
        
        // Store current state and rewind entities
        let mut query = world.query::<(Entity, &NetworkId, &mut ReplicatedTransform)>();
        let mut entities_to_update = Vec::new();
        
        for (entity, network_id, transform) in query.iter(world) {
            let current_state = HistoricalState::new(timestamp, *transform);
            rewound_entities.insert(network_id.get(), current_state);
            
            if let Some(historical_state) = history.get_state_at(network_id.get(), timestamp) {
                entities_to_update.push((entity, historical_state.transform));
            }
        }
        
        // Apply rewound transforms
        for (entity, transform) in entities_to_update {
            if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut transform_mut) = entity_mut.get_mut::<ReplicatedTransform>() {
                    *transform_mut = transform;
                }
            }
        }
        
        Ok(RewindState {
            client_id,
            timestamp,
            original_states: rewound_entities,
        })
    }
    
    /// Restores the world to its original state after rewinding.
    pub fn restore_state(&self, rewind_state: RewindState, world: &mut bevy_ecs::world::World) {
        let mut query = world.query::<(Entity, &NetworkId, &mut ReplicatedTransform)>();
        let mut entities_to_restore = Vec::new();
        
        for (entity, network_id, _) in query.iter(world) {
            if let Some(original_state) = rewind_state.original_states.get(&network_id.get()) {
                entities_to_restore.push((entity, original_state.transform));
            }
        }
        
        // Restore original transforms
        for (entity, transform) in entities_to_restore {
            if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut transform_mut) = entity_mut.get_mut::<ReplicatedTransform>() {
                    *transform_mut = transform;
                }
            }
        }
    }
    
    /// Performs a lag-compensated raycast.
    pub fn raycast_at_client_time(
        &self,
        client_id: u64,
        timestamp: u64,
        world: &mut bevy_ecs::world::World,
        ray_origin: Vec3,
        ray_direction: Vec3,
        max_distance: f32,
    ) -> Result<Option<RaycastHit>> {
        // Rewind world to client's time
        let rewind_state = self.rewind_to_client_time(client_id, timestamp, world)?;
        
        // Perform raycast
        let hit = Self::perform_raycast(world, ray_origin, ray_direction, max_distance);
        
        // Restore world state
        self.restore_state(rewind_state, world);
        
        Ok(hit)
    }
    
    /// Performs a simple raycast (in a real implementation, this would use physics).
    fn perform_raycast(
        world: &mut bevy_ecs::world::World,
        ray_origin: Vec3,
        ray_direction: Vec3,
        max_distance: f32,
    ) -> Option<RaycastHit> {
        let ray_dir = ray_direction.normalize();
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = max_distance;
        
        let mut query = world.query::<(Entity, &NetworkId, &ReplicatedTransform)>();
        
        for (entity, network_id, transform) in query.iter(world) {
            // Simple sphere intersection (in real impl, use actual collision shapes)
            let to_entity = transform.translation - ray_origin;
            let projection = to_entity.dot(ray_dir);
            
            if projection < 0.0 || projection > closest_distance {
                continue;
            }
            
            let closest_point = ray_origin + ray_dir * projection;
            let distance_to_ray = (closest_point - transform.translation).length();
            
            // Assume 1.0 unit radius for simplicity
            if distance_to_ray < 1.0 && projection < closest_distance {
                closest_distance = projection;
                closest_hit = Some(RaycastHit {
                    entity,
                    network_id: network_id.get(),
                    point: closest_point,
                    distance: projection,
                });
            }
        }
        
        closest_hit
    }
}

/// State of the world before rewinding.
pub struct RewindState {
    /// Client ID
    pub client_id: u64,
    
    /// Timestamp we rewound to
    pub timestamp: u64,
    
    /// Original entity states
    original_states: HashMap<u64, HistoricalState>,
}

/// Result of a raycast hit.
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// Entity that was hit
    pub entity: Entity,
    
    /// Network ID of the hit entity
    pub network_id: u64,
    
    /// Point of intersection
    pub point: Vec3,
    
    /// Distance from ray origin
    pub distance: f32,
}

/// System for lag compensation.
pub struct LagCompensationSystem;

impl LagCompensationSystem {
    /// Updates lag compensation history.
    pub fn update(
        lag_comp: &mut LagCompensation,
        client_id: u64,
        world: &mut bevy_ecs::world::World,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        lag_comp.record_snapshot(client_id, timestamp, world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_math::Quat;

    #[test]
    fn test_history_buffer() {
        let mut buffer = HistoryBuffer::new(1000);
        
        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let state = HistoricalState::new(100, transform);
        
        buffer.add_state(state);
        assert_eq!(buffer.len(), 1);
        
        let retrieved = buffer.get_state_at(100);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_history_buffer_cleanup() {
        let mut buffer = HistoryBuffer::new(100);
        
        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        
        buffer.add_state(HistoricalState::new(0, transform));
        buffer.add_state(HistoricalState::new(50, transform));
        buffer.add_state(HistoricalState::new(200, transform));
        
        // Old state should be removed
        assert_eq!(buffer.len(), 2);
        assert!(buffer.get_state_at(0).is_none());
    }

    #[test]
    fn test_client_state_history() {
        let mut history = ClientStateHistory::new(1000);
        
        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let state = HistoricalState::new(100, transform);
        
        history.record_state(1, state);
        
        let retrieved = history.get_state_at(1, 100);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_lag_compensation_creation() {
        let lag_comp = LagCompensation::new(1000);
        assert_eq!(lag_comp.max_history_ms, 1000);
    }

    #[test]
    fn test_interpolate_state() {
        let mut buffer = HistoryBuffer::new(1000);
        
        let t1 = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let t2 = ReplicatedTransform::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE);
        
        buffer.add_state(HistoricalState::new(100, t1));
        buffer.add_state(HistoricalState::new(200, t2));
        
        let interpolated = buffer.interpolate_state_at(150);
        assert!(interpolated.is_some());
        
        let state = interpolated.unwrap();
        assert!((state.transform.translation.x - 5.0).abs() < 0.1);
    }
}
