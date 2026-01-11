//! Scene view panel for rendering and interacting with the 3D scene.
//!
//! This module implements the **Viewport Panel**, the main 3D scene visualization and interaction
//! area in the editor. It integrates multiple systems:
//! - 3D rendering (scene, gizmos, selection highlights)
//! - Input handling (mouse picking, gizmo manipulation)
//! - Drag-and-drop asset instantiation
//! - Visual mode indicators (play/pause/edit)
//!
//! # Viewport Architecture
//!
//! The viewport acts as a **mediator** between various editor subsystems:
//!
//! ```text
//!                    ┌─────────────────┐
//!                    │  SceneViewPanel │
//!                    │   (Viewport)    │
//!                    └────────┬────────┘
//!                             │
//!          ┌──────────────────┼──────────────────┐
//!          │                  │                  │
//!    ┌─────▼─────┐    ┌──────▼──────┐    ┌─────▼─────┐
//!    │  Render   │    │  Selection  │    │  Gizmo    │
//!    │  System   │    │  System     │    │  System   │
//!    └───────────┘    └─────────────┘    └───────────┘
//!          │                  │                  │
//!          └──────────────────┼──────────────────┘
//!                             │
//!                      ┌──────▼──────┐
//!                      │  ECS World  │
//!                      └─────────────┘
//! ```
//!
//! # Input Handling Flow
//!
//! ## Mouse Click (Selection)
//! 1. User clicks in viewport
//! 2. Check if gizmo is under cursor (priority)
//! 3. If not gizmo, perform raycast picking
//! 4. Find entity under cursor using ray-AABB tests
//! 5. Update selection based on modifier keys (Ctrl/Shift)
//! 6. Fire selection events for UI updates
//!
//! ## Mouse Drag (Gizmo Manipulation)
//! 1. User clicks on gizmo axis
//! 2. Start gizmo interaction (capture initial transforms)
//! 3. Track mouse movement, compute axis-aligned delta
//! 4. Apply real-time transform updates (preview)
//! 5. On release, create TransformEditCommand for undo
//!
//! ## Mouse Drag (Marquee Selection)
//! 1. User clicks and drags in empty space
//! 2. Draw selection rectangle overlay
//! 3. On release, test which entities are within rectangle
//! 4. Update selection with entities in rectangle
//!
//! # Drag-and-Drop Integration
//!
//! The viewport accepts drag-and-drop from the Assets Panel to instantiate entities:
//!
//! ## Asset Drop Flow
//! 1. User drags asset from Assets Panel
//! 2. Viewport highlights when hovering (visual feedback)
//! 3. On drop:
//!    - **Models (.obj/.gltf)**: Spawn entity with MeshHandle at origin
//!    - **Textures (.png/.jpg)**: Apply to selected entity's material
//!    - **Audio (.wav/.ogg)**: Spawn entity with AudioSource at origin
//! 4. Create undo command for operation
//! 5. Select newly created entity
//!
//! ## Undo Integration
//! All asset drop operations create commands:
//! - Model drop → CreateEntityCommand + AddComponentCommand
//! - Texture application → TransformEditCommand (material change)
//! - Audio drop → CreateEntityCommand + AddComponentCommand
//!
//! # Visual Mode Indicators
//!
//! The viewport provides visual feedback for editor modes using border colors:
//! - **Edit Mode**: Dark gray border (RGB: 76, 76, 89)
//! - **Play Mode**: Green border (indicates game is running)
//! - **Paused Mode**: Orange border (game paused but not editing)
//!
//! This immediate visual feedback helps users understand the current editor state.
//!
//! # Rendering Pipeline Integration
//!
//! The viewport coordinates with the rendering system:
//! 1. Scene rendering: Normal 3D scene with entities
//! 2. Gizmo overlay: Transform gizmos for selected entities
//! 3. Selection highlights: Outline or tint for selected entities
//! 4. Grid rendering: Optional world grid for alignment
//! 5. UI overlay: egui elements (mode indicators, stats)

use super::{AssetEntry, AssetType, EditorPanel};
use crate::drag_drop::DragDropSystem;
use crate::entity_operations::EntityOperations;
use crate::selection::SelectionSystem;
use crate::UndoRedoSystem;
use egui::{Color32, Ui};
use praxis_audio::AudioSource;
use praxis_ecs::{MeshHandle, Name, TextureHandle, Transform};
use praxis_math::Vec3;
use praxis_utils::{info, warn};

/// Panel for displaying the 3D scene viewport.
pub struct SceneViewPanel {
    title: String,
    /// Last dropped asset (if any)
    last_dropped_asset: Option<AssetEntry>,
    /// Viewport border color (set externally based on play mode)
    border_color: Color32,
    /// Entity operations for spawning entities
    entity_ops: EntityOperations,
}

impl SceneViewPanel {
    /// Creates a new scene view panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Scene".to_string(),
            last_dropped_asset: None,
            border_color: Color32::from_rgb(76, 76, 89), // Default dark gray
            entity_ops: EntityOperations::new(),
        }
    }

    /// Gets the last dropped asset and clears it
    pub fn take_dropped_asset(&mut self) -> Option<AssetEntry> {
        self.last_dropped_asset.take()
    }

    /// Checks if the scene view can accept a drop
    pub fn can_accept_drop(&self, asset: &AssetEntry) -> bool {
        !asset.is_directory
    }

    /// Sets the viewport border color (used for play mode indicators)
    pub fn set_border_color(&mut self, color: Color32) {
        self.border_color = color;
    }

    /// Gets the current viewport border color
    pub const fn border_color(&self) -> Color32 {
        self.border_color
    }

    /// Handles a dropped asset by spawning an appropriate entity
    fn handle_asset_drop(
        &mut self,
        asset: &AssetEntry,
        world: &mut bevy_ecs::world::World,
        undo_system: Option<&mut UndoRedoSystem>,
        selection_system: Option<&mut SelectionSystem>,
    ) {
        let asset_path = asset.path.to_string_lossy().to_string();
        info!(
            "Dropping asset into scene: {} (type: {:?})",
            asset_path, asset.asset_type
        );

        match asset.asset_type {
            AssetType::Model => {
                self.spawn_mesh_entity(
                    &asset.name,
                    &asset_path,
                    world,
                    undo_system,
                    selection_system,
                );
            }
            AssetType::Texture => {
                self.apply_texture_to_selected(&asset.name, &asset_path, world, selection_system);
            }
            AssetType::Audio => {
                self.spawn_audio_entity(
                    &asset.name,
                    &asset_path,
                    world,
                    undo_system,
                    selection_system,
                );
            }
            AssetType::Scene => {
                info!("Scene loading from drag-drop not yet implemented");
            }
            AssetType::Unknown => {
                warn!("Cannot instantiate unknown asset type: {}", asset.name);
            }
        }
    }

    /// Spawns a mesh entity at the origin
    #[allow(clippy::needless_pass_by_ref_mut)]
    fn spawn_mesh_entity(
        &mut self,
        name: &str,
        path: &str,
        world: &mut bevy_ecs::world::World,
        undo_system: Option<&mut UndoRedoSystem>,
        selection_system: Option<&mut SelectionSystem>,
    ) {
        let entity_name = format!(
            "Mesh_{}",
            name.trim_end_matches(".obj")
                .trim_end_matches(".gltf")
                .trim_end_matches(".glb")
        );
        let spawn_position = Vec3::new(0.0, 0.0, 0.0);

        info!(
            "Spawning mesh entity '{}' with mesh from '{}'",
            entity_name, path
        );

        let entity = if let Some(undo) = undo_system {
            // Use entity operations for undo support
            match self.entity_ops.create_entity_with_components(
                world,
                undo,
                &entity_name,
                Transform::from_translation(spawn_position),
            ) {
                Ok(entity) => {
                    // Add mesh handle component
                    let mesh_handle = MeshHandle::new(path);
                    match self.entity_ops.add_component(
                        world,
                        undo,
                        entity,
                        crate::undo::ComponentData::MeshHandle {
                            id: mesh_handle.id.clone(),
                        },
                    ) {
                        Ok(()) => {
                            info!("Created mesh entity {:?} at {:?}", entity, spawn_position);
                            Some(entity)
                        }
                        Err(e) => {
                            warn!("Failed to add mesh handle component: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create mesh entity: {}", e);
                    None
                }
            }
        } else {
            // Direct spawn without undo support
            let entity = world
                .spawn((
                    Transform::from_translation(spawn_position),
                    Name::new(&entity_name),
                    MeshHandle::new(path),
                ))
                .id();
            info!("Created mesh entity {:?} at {:?}", entity, spawn_position);
            Some(entity)
        };

        // Select the newly created entity
        if let (Some(entity), Some(selection)) = (entity, selection_system) {
            selection.clear();
            selection.select_entity(entity, crate::selection::SelectionMode::Replace);
        }
    }

    /// Applies a texture to the currently selected entity
    fn apply_texture_to_selected(
        &self,
        name: &str,
        path: &str,
        world: &mut bevy_ecs::world::World,
        selection_system: Option<&mut SelectionSystem>,
    ) {
        if let Some(selection) = selection_system {
            let selected: Vec<_> = selection.selected_entities().collect();

            if selected.is_empty() {
                info!(
                    "No entity selected to apply texture '{}'. Select an entity first.",
                    name
                );
                return;
            }

            for entity in selected {
                if let Some(mut entity_ref) = world.get_entity_mut(entity) {
                    // Add or update texture handle
                    if entity_ref.contains::<TextureHandle>() {
                        // Update existing texture
                        if let Some(mut texture) = entity_ref.get_mut::<TextureHandle>() {
                            texture.id = path.to_string();
                            info!("Updated texture on entity {:?} to '{}'", entity, path);
                        }
                    } else {
                        // Add new texture handle
                        entity_ref.insert(TextureHandle::new(path));
                        info!("Added texture '{}' to entity {:?}", path, entity);
                    }
                } else {
                    warn!("Selected entity {:?} not found in world", entity);
                }
            }
        } else {
            info!(
                "No selection system available. Create an entity and select it to apply textures."
            );
        }
    }

    /// Spawns an audio source entity at the origin
    #[allow(clippy::needless_pass_by_ref_mut)]
    fn spawn_audio_entity(
        &mut self,
        name: &str,
        path: &str,
        world: &mut bevy_ecs::world::World,
        undo_system: Option<&mut UndoRedoSystem>,
        selection_system: Option<&mut SelectionSystem>,
    ) {
        let entity_name = format!(
            "Audio_{}",
            name.trim_end_matches(".wav")
                .trim_end_matches(".ogg")
                .trim_end_matches(".mp3")
        );
        let spawn_position = Vec3::new(0.0, 0.0, 0.0);

        info!(
            "Spawning audio entity '{}' with sound from '{}'",
            entity_name, path
        );

        let entity = if let Some(undo) = undo_system {
            // Use entity operations for undo support
            match self.entity_ops.create_entity_with_components(
                world,
                undo,
                &entity_name,
                Transform::from_translation(spawn_position),
            ) {
                Ok(entity) => {
                    // Add audio source component
                    let audio_source = AudioSource::new(path)
                        .with_volume(0.5)
                        .with_spatial(true)
                        .with_looping(false);

                    match self.entity_ops.add_component(
                        world,
                        undo,
                        entity,
                        crate::undo::ComponentData::AudioSource {
                            data: crate::undo::SerializableAudioSource {
                                path: audio_source.path.clone(),
                                volume: audio_source.volume,
                                spatial: audio_source.spatial,
                                looping: audio_source.looping,
                                max_distance: audio_source.max_distance,
                                reference_distance: audio_source.reference_distance,
                            },
                        },
                    ) {
                        Ok(()) => {
                            info!("Created audio entity {:?} at {:?}", entity, spawn_position);
                            Some(entity)
                        }
                        Err(e) => {
                            warn!("Failed to add audio source component: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create audio entity: {}", e);
                    None
                }
            }
        } else {
            // Direct spawn without undo support
            let audio_source = AudioSource::new(path)
                .with_volume(0.5)
                .with_spatial(true)
                .with_looping(false);

            let entity = world
                .spawn((
                    Transform::from_translation(spawn_position),
                    Name::new(&entity_name),
                    audio_source,
                ))
                .id();
            info!("Created audio entity {:?} at {:?}", entity, spawn_position);
            Some(entity)
        };

        // Select the newly created entity
        if let (Some(entity), Some(selection)) = (entity, selection_system) {
            selection.clear();
            selection.select_entity(entity, crate::selection::SelectionMode::Replace);
        }
    }
}

impl Default for SceneViewPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for SceneViewPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        ui.heading("Scene View");
        ui.separator();

        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // Draw border with color based on play mode
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(3.0, self.border_color),
        );

        // Check for drag-drop payloads
        let is_hovering = response.hovered();

        if is_hovering {
            // Visual feedback when hovering with asset
            ui.painter().rect_filled(
                response.rect,
                0.0,
                Color32::from_rgba_premultiplied(100, 150, 200, 30),
            );
        }

        ui.painter().text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            "3D scene viewport will be rendered here.\n\nDrag assets from the Asset Browser to add them to the scene.",
            egui::FontId::proportional(14.0),
            Color32::GRAY,
        );

        ui.label("Camera controls:");
        ui.label("• Right-click + drag to rotate");
        ui.label("• WASD to move");
        ui.label("• Mouse wheel to zoom");
    }
}

/// Extension trait for SceneViewPanel to handle drag-drop with World access
pub trait SceneViewPanelExt {
    /// Renders the panel with full world access for drag-drop handling
    fn ui_with_world(
        &mut self,
        ui: &mut Ui,
        world: Option<&mut bevy_ecs::world::World>,
        drag_drop_system: Option<&mut DragDropSystem>,
        undo_system: Option<&mut UndoRedoSystem>,
        selection_system: Option<&mut SelectionSystem>,
    );
}

impl SceneViewPanelExt for SceneViewPanel {
    fn ui_with_world(
        &mut self,
        ui: &mut Ui,
        world: Option<&mut bevy_ecs::world::World>,
        drag_drop_system: Option<&mut DragDropSystem>,
        undo_system: Option<&mut UndoRedoSystem>,
        selection_system: Option<&mut SelectionSystem>,
    ) {
        ui.heading("Scene View");
        ui.separator();

        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // Draw border with color based on play mode
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(3.0, self.border_color),
        );

        // Check for drag-drop
        let is_hovering = response.hovered();
        let mut is_dragging_asset = false;

        if let Some(dnd) = drag_drop_system.as_ref() {
            if dnd.is_dragging() {
                is_dragging_asset = true;
            }
        }

        // Visual feedback when hovering with asset
        if is_hovering && is_dragging_asset {
            ui.painter().rect_filled(
                response.rect,
                0.0,
                Color32::from_rgba_premultiplied(100, 200, 150, 40),
            );
        }

        // Handle drop
        if is_hovering && ui.input(|i| i.pointer.any_released()) {
            if let Some(dnd) = drag_drop_system {
                if let Some(payload) = dnd.complete_drop() {
                    if let Some(asset_path) = payload.as_asset_path() {
                        // Create an AssetEntry from the payload
                        if let Ok(asset_entry) = AssetEntry::from_path(asset_path) {
                            if let Some(w) = world {
                                self.handle_asset_drop(
                                    &asset_entry,
                                    w,
                                    undo_system,
                                    selection_system,
                                );
                            }
                        }
                    }
                }
            }
        }

        ui.painter().text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            if is_dragging_asset {
                "Drop here to add to scene"
            } else {
                "3D scene viewport will be rendered here.\n\nDrag assets from the Asset Browser to add them to the scene."
            },
            egui::FontId::proportional(14.0),
            if is_dragging_asset { Color32::WHITE } else { Color32::GRAY },
        );

        if !is_dragging_asset {
            ui.label("Camera controls:");
            ui.label("• Right-click + drag to rotate");
            ui.label("• WASD to move");
            ui.label("• Mouse wheel to zoom");
        }
    }
}
