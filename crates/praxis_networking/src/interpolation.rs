//! Interpolation and extrapolation for smooth remote entity movement.
//!
//! # The Problem: Jittery Movement
//!
//! Network updates arrive at discrete intervals (e.g., 20 times per second), but
//! games render at 60+ FPS. Without smoothing, entities would "teleport" between
//! positions, creating jittery, unnatural movement.
//!
//! ```text
//! Without smoothing (60 FPS render, 20 Hz updates):
//! Frame 1-3: Position A (looks frozen)
//! Frame 4-6: Position B (sudden jump!)
//! Frame 7-9: Position C (another jump!)
//! Result: Choppy, jarring movement
//! ```
//!
//! # Solution 1: Interpolation (Most Common)
//!
//! **Interpolation** renders entities *slightly in the past*, smoothly blending
//! between received snapshots.
//!
//! ## How It Works
//!
//! 1. **Buffer snapshots**: Store last few position updates
//! 2. **Delay rendering**: Render 100-200ms in the past
//! 3. **Blend positions**: Smoothly transition between snapshots
//!
//! ```text
//! Time:     0ms    100ms   200ms   300ms   (server time)
//! Server:   [S1]   [S2]    [S3]    [S4]
//!            ↓      ↓       ↓       ↓
//! Client:   recv   recv    recv    recv
//! Render:          [S1-S2] [S2-S3] [S3-S4]  (interpolating between)
//! ```
//!
//! ## Advantages
//!
//! - **Perfectly smooth**: Always have two snapshots to blend
//! - **No guessing**: Only render known, authoritative data
//! - **No corrections**: Server data is always correct
//!
//! ## Trade-offs
//!
//! - **Added latency**: Entities rendered 100-200ms behind reality
//! - **Not for local player**: Local player needs instant feedback
//! - **Buffer required**: Need memory for snapshot history
//!
//! ## When to Use
//!
//! Perfect for **remote entities** (other players, NPCs, vehicles):
//! - Their 100ms delay isn't noticeable
//! - Smoothness is more important than instant response
//! - They're not controlled by you, so latency doesn't affect input
//!
//! # Solution 2: Extrapolation (Riskier)
//!
//! **Extrapolation** predicts where entities *will be* based on velocity,
//! rendering them ahead of the last known position.
//!
//! ## How It Works
//!
//! 1. **Receive position + velocity**: Server sends both
//! 2. **Predict forward**: Use velocity to estimate future position
//! 3. **Correct when wrong**: Snap to real position when update arrives
//!
//! ```text
//! Last update at 0ms: Position (10, 0), Velocity (1, 0)
//!
//! Frame at 16ms: Predict (10 + 1*0.016, 0) = (10.016, 0)
//! Frame at 33ms: Predict (10 + 1*0.033, 0) = (10.033, 0)
//!
//! Update arrives at 100ms: Actual position is (10.1, 0.5)
//!                          Snap to correct position
//! ```
//!
//! ## Advantages
//!
//! - **Zero added latency**: Render at current time
//! - **Good for constant velocity**: Works great for moving vehicles
//! - **Responsive feel**: Entities appear up-to-date
//!
//! ## Trade-offs
//!
//! - **Prediction errors**: Movement changes cause visible snapping
//! - **Worse for erratic movement**: Player strafing/jumping hard to predict
//! - **Visual artifacts**: Occasional "rubber-banding" when predictions wrong
//!
//! ## When to Use
//!
//! Best for **predictable movement**:
//! - Physics objects (falling, sliding)
//! - Vehicles moving in straight lines
//! - Projectiles with constant velocity
//!
//! Avoid for:
//! - Player-controlled characters (too erratic)
//! - Entities that frequently change direction
//!
//! # Hybrid Approach (Best of Both Worlds)
//!
//! Many games use **both**:
//! - **Interpolation** as default (smooth, safe)
//! - **Extrapolation** when packets late (prevent freezing)
//!
//! ```text
//! Normal: Interpolate between S1 and S2
//! Packet loss: No S3? Extrapolate from S2 using velocity
//! Packet arrives: Snap back to S3, resume interpolation
//! ```
//!
//! # Implementation Details
//!
//! ## Snapshot Buffer
//!
//! Stores historical positions with timestamps:
//! ```rust,ignore
//! buffer.add_snapshot(Snapshot {
//!     timestamp: 1000.0,
//!     transform: current_transform,
//!     velocity: Some(current_velocity),
//! });
//! ```
//!
//! ## Interpolation Math
//!
//! Linear interpolation (lerp) between two positions:
//! ```rust,ignore
//! let t = (target_time - prev.time) / (next.time - prev.time);
//! let pos = prev.pos.lerp(next.pos, t);
//! let rot = prev.rot.slerp(next.rot, t); // Spherical lerp for rotations
//! ```
//!
//! ## Extrapolation Math
//!
//! Simple physics prediction:
//! ```rust,ignore
//! let dt = time_since_last_update;
//! let predicted_pos = last_pos + velocity * dt;
//! ```
//!
//! # Choosing Delay Amount
//!
//! Typical interpolation delays:
//! - **100ms**: Good balance (default)
//! - **50ms**: Lower latency, riskier (might not have next snapshot)
//! - **150-200ms**: Very smooth, but noticeable lag
//!
//! Formula: `delay ≥ update_interval + jitter`
//! - Update interval: Time between server updates (e.g., 50ms for 20 Hz)
//! - Jitter: Network timing variation (e.g., ±50ms)
//!
//! # Visual Example
//!
//! ```text
//! Server sends updates:  A---B---C---D---E
//!                       0ms 50  100 150 200
//!
//! Client interpolates with 100ms delay:
//! Time 100ms: Interpolate between A and B
//! Time 150ms: Interpolate between B and C
//! Time 200ms: Interpolate between C and D
//!
//! Result: Smooth movement, but 100ms behind server
//! ```

use crate::{NetworkExtrapolation, NetworkInterpolation, ReplicatedTransform, ReplicatedVelocity};
use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// Snapshot of entity state at a point in time.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    /// Timestamp in milliseconds
    pub timestamp: f32,

    /// Transform at this time
    pub transform: ReplicatedTransform,

    /// Velocity at this time
    pub velocity: Option<ReplicatedVelocity>,
}

impl Snapshot {
    /// Creates a new snapshot.
    pub fn new(
        timestamp: f32,
        transform: ReplicatedTransform,
        velocity: Option<ReplicatedVelocity>,
    ) -> Self {
        Self {
            timestamp,
            transform,
            velocity,
        }
    }
}

/// Buffer storing snapshots for interpolation.
#[derive(Component, Debug)]
pub struct SnapshotBuffer {
    /// Snapshots ordered by timestamp (oldest first)
    snapshots: VecDeque<Snapshot>,

    /// Maximum number of snapshots to keep
    max_snapshots: usize,
}

impl SnapshotBuffer {
    /// Creates a new snapshot buffer.
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_snapshots),
            max_snapshots,
        }
    }

    /// Adds a snapshot to the buffer.
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        // Insert in sorted order
        let insert_pos = self
            .snapshots
            .iter()
            .position(|s| s.timestamp > snapshot.timestamp)
            .unwrap_or(self.snapshots.len());

        self.snapshots.insert(insert_pos, snapshot);

        // Remove old snapshots if buffer is full
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.pop_front();
        }
    }

    /// Gets snapshots surrounding a given timestamp.
    pub fn get_surrounding_snapshots(&self, timestamp: f32) -> Option<(Snapshot, Snapshot)> {
        if self.snapshots.len() < 2 {
            return None;
        }

        // Find the first snapshot after the target time
        let next_idx = self
            .snapshots
            .iter()
            .position(|s| s.timestamp > timestamp)?;

        if next_idx == 0 {
            return None;
        }

        let prev = self.snapshots[next_idx - 1];
        let next = self.snapshots[next_idx];

        Some((prev, next))
    }

    /// Gets the most recent snapshot.
    pub fn latest(&self) -> Option<&Snapshot> {
        self.snapshots.back()
    }

    /// Gets the oldest snapshot.
    pub fn oldest(&self) -> Option<&Snapshot> {
        self.snapshots.front()
    }

    /// Clears all snapshots.
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Gets the number of snapshots in the buffer.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

impl Default for SnapshotBuffer {
    fn default() -> Self {
        Self::new(32)
    }
}

/// Interpolation buffer for remote entities.
#[derive(Component, Debug)]
pub struct InterpolationBuffer {
    /// Snapshot buffer
    pub buffer: SnapshotBuffer,

    /// Current interpolation time
    pub current_time: f32,
}

impl InterpolationBuffer {
    /// Creates a new interpolation buffer.
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            buffer: SnapshotBuffer::new(max_snapshots),
            current_time: 0.0,
        }
    }

    /// Advances the interpolation time.
    pub fn advance_time(&mut self, delta_time: f32) {
        self.current_time += delta_time;
    }
}

impl Default for InterpolationBuffer {
    fn default() -> Self {
        Self::new(32)
    }
}

/// System for interpolating entity positions.
pub struct InterpolationSystem;

impl InterpolationSystem {
    /// Interpolates transform between two snapshots.
    pub fn interpolate_transform(
        prev: &ReplicatedTransform,
        next: &ReplicatedTransform,
        t: f32,
    ) -> ReplicatedTransform {
        let t = t.clamp(0.0, 1.0);

        ReplicatedTransform {
            translation: prev.translation.lerp(next.translation, t),
            rotation: prev.rotation.slerp(next.rotation, t),
            scale: prev.scale.lerp(next.scale, t),
        }
    }

    /// Updates interpolated entities.
    pub fn update(
        mut query: Query<(
            &mut ReplicatedTransform,
            &mut InterpolationBuffer,
            &NetworkInterpolation,
        )>,
        delta_time: f32,
    ) {
        for (mut transform, mut buffer, interpolation) in query.iter_mut() {
            if !interpolation.enabled {
                continue;
            }

            // Advance time
            buffer.advance_time(delta_time * 1000.0); // Convert to milliseconds

            // Calculate target interpolation time (current time - delay)
            let target_time = buffer.current_time - interpolation.delay_ms;

            // Get surrounding snapshots
            if let Some((prev, next)) = buffer.buffer.get_surrounding_snapshots(target_time) {
                // Calculate interpolation factor
                let time_diff = next.timestamp - prev.timestamp;
                if time_diff > 0.0 {
                    let t = (target_time - prev.timestamp) / time_diff;
                    *transform = Self::interpolate_transform(&prev.transform, &next.transform, t);
                }
            } else if let Some(latest) = buffer.buffer.latest() {
                // If no surrounding snapshots, use latest
                *transform = latest.transform;
            }
        }
    }
}

/// System for extrapolating entity positions.
pub struct ExtrapolationSystem;

impl ExtrapolationSystem {
    /// Extrapolates transform based on velocity.
    pub fn extrapolate_transform(
        transform: &ReplicatedTransform,
        velocity: &ReplicatedVelocity,
        delta_time: f32,
    ) -> ReplicatedTransform {
        let dt = delta_time / 1000.0; // Convert to seconds

        ReplicatedTransform {
            translation: transform.translation + velocity.linear * dt,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    /// Updates extrapolated entities.
    pub fn update(
        mut query: Query<(
            &mut ReplicatedTransform,
            &ReplicatedVelocity,
            &mut NetworkExtrapolation,
            &InterpolationBuffer,
        )>,
        delta_time: f32,
    ) {
        for (mut transform, velocity, mut extrapolation, buffer) in query.iter_mut() {
            if !extrapolation.enabled {
                continue;
            }

            // Update time since last snapshot
            extrapolation.time_since_update += delta_time * 1000.0; // Convert to milliseconds

            // Check if we have recent snapshots
            if let Some(latest) = buffer.buffer.latest() {
                let time_diff = buffer.current_time - latest.timestamp;

                if time_diff < extrapolation.max_time_ms {
                    // Extrapolate from latest snapshot
                    *transform =
                        Self::extrapolate_transform(&latest.transform, velocity, time_diff);
                } else {
                    // Freeze at last known position
                    *transform = latest.transform;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_math::{Quat, Vec3};

    #[test]
    fn test_snapshot_buffer() {
        let mut buffer = SnapshotBuffer::new(5);
        assert!(buffer.is_empty());

        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let snapshot = Snapshot::new(100.0, transform, None);

        buffer.add_snapshot(snapshot);
        assert_eq!(buffer.len(), 1);

        assert!(buffer.latest().is_some());
    }

    #[test]
    fn test_snapshot_ordering() {
        let mut buffer = SnapshotBuffer::new(10);
        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        buffer.add_snapshot(Snapshot::new(300.0, transform, None));
        buffer.add_snapshot(Snapshot::new(100.0, transform, None));
        buffer.add_snapshot(Snapshot::new(200.0, transform, None));

        assert_eq!(buffer.oldest().unwrap().timestamp, 100.0);
        assert_eq!(buffer.latest().unwrap().timestamp, 300.0);
    }

    #[test]
    fn test_surrounding_snapshots() {
        let mut buffer = SnapshotBuffer::new(10);
        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        buffer.add_snapshot(Snapshot::new(100.0, transform, None));
        buffer.add_snapshot(Snapshot::new(200.0, transform, None));
        buffer.add_snapshot(Snapshot::new(300.0, transform, None));

        let result = buffer.get_surrounding_snapshots(150.0);
        assert!(result.is_some());

        let (prev, next) = result.unwrap();
        assert_eq!(prev.timestamp, 100.0);
        assert_eq!(next.timestamp, 200.0);
    }

    #[test]
    fn test_interpolate_transform() {
        let t1 = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let t2 = ReplicatedTransform::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE);

        let result = InterpolationSystem::interpolate_transform(&t1, &t2, 0.5);

        assert!((result.translation.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_extrapolate_transform() {
        let transform = ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let velocity = ReplicatedVelocity::new(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO);

        let result = ExtrapolationSystem::extrapolate_transform(&transform, &velocity, 1000.0);

        assert!((result.translation.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolation_buffer() {
        let mut buffer = InterpolationBuffer::new(10);
        assert_eq!(buffer.current_time, 0.0);

        buffer.advance_time(16.0);
        assert_eq!(buffer.current_time, 16.0);
    }
}
