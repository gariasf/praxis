//! Drag and drop system for the editor.
//!
//! This module provides a drag-and-drop system for transferring assets and entities
//! within the editor UI.

use crate::panels::AssetEntry;
use praxis_ecs::{Entity, Resource};
use praxis_utils::info;
use std::path::PathBuf;

/// Payload for drag-and-drop operations
#[derive(Debug, Clone)]
pub enum DragDropPayload {
    /// Asset being dragged from the asset browser
    Asset {
        /// Path to the asset file
        path: PathBuf,
        /// Asset name
        name: String,
    },
    /// Entity being dragged within the hierarchy
    Entity(Entity),
    /// Generic file path being dragged
    FilePath(PathBuf),
}

impl DragDropPayload {
    /// Creates a drag payload from an asset entry
    pub fn from_asset(entry: &AssetEntry) -> Self {
        Self::Asset {
            path: entry.path.clone(),
            name: entry.name.clone(),
        }
    }

    /// Gets the asset path if this is an asset payload
    pub fn as_asset_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Asset { path, .. } => Some(path),
            Self::FilePath(path) => Some(path),
            _ => None,
        }
    }

    /// Gets the entity if this is an entity payload
    pub fn as_entity(&self) -> Option<Entity> {
        match self {
            Self::Entity(entity) => Some(*entity),
            _ => None,
        }
    }
}

/// ECS resource for managing drag-and-drop operations
#[derive(Debug, Clone, Default, Resource)]
pub struct DragDropSystem {
    /// Current payload being dragged (if any)
    current_payload: Option<DragDropPayload>,
    /// Whether a drop was just completed
    drop_completed: bool,
}

impl DragDropSystem {
    /// Creates a new drag-drop system
    pub fn new() -> Self {
        Self {
            current_payload: None,
            drop_completed: false,
        }
    }

    /// Starts a drag operation with the given payload
    pub fn start_drag(&mut self, payload: DragDropPayload) {
        info!("Starting drag operation: {:?}", payload);
        self.current_payload = Some(payload);
        self.drop_completed = false;
    }

    /// Checks if a drag operation is in progress
    pub fn is_dragging(&self) -> bool {
        self.current_payload.is_some()
    }

    /// Gets the current drag payload (if any)
    pub fn current_payload(&self) -> Option<&DragDropPayload> {
        self.current_payload.as_ref()
    }

    /// Completes a drop operation
    pub fn complete_drop(&mut self) -> Option<DragDropPayload> {
        if let Some(payload) = self.current_payload.take() {
            info!("Drop completed: {:?}", payload);
            self.drop_completed = true;
            Some(payload)
        } else {
            None
        }
    }

    /// Cancels the current drag operation
    pub fn cancel_drag(&mut self) {
        if self.current_payload.is_some() {
            info!("Drag operation cancelled");
            self.current_payload = None;
            self.drop_completed = false;
        }
    }

    /// Checks if a drop was just completed this frame
    pub fn drop_just_completed(&self) -> bool {
        self.drop_completed
    }

    /// Resets the drop completed flag (should be called each frame)
    pub fn reset_frame(&mut self) {
        self.drop_completed = false;
    }
}
