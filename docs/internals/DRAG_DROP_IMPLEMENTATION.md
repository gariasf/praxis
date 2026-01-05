# Drag-and-Drop Asset Instantiation Implementation

## Overview

This implementation adds full drag-and-drop functionality to instantiate assets from the AssetsPanel into the scene viewport. Assets are automatically spawned with appropriate components based on their type.

## Features Implemented

### 1. Asset Type Detection
Assets are categorized by file extension:
- **Models** (`.obj`, `.gltf`, `.glb`) - Spawns entities with MeshHandle + Transform
- **Textures** (`.png`, `.jpg`, `.jpeg`) - Applies TextureHandle to selected entity
- **Audio** (`.wav`, `.ogg`, `.mp3`) - Spawns entities with AudioSource + Transform

### 2. Drag-and-Drop System Integration

#### DragDropSystem Resource
- Manages drag payload state across panels
- Tracks current drag operation
- Handles drop completion and cancellation
- Frame-based state reset

#### AssetsPanel Integration
- Detects drag start on asset items
- Creates DragDropPayload with asset information
- Shows visual drag preview cursor
- Notifies DragDropSystem when drag begins

#### SceneViewPanel Integration  
- Detects when viewport is hovered during drag
- Shows visual feedback (highlight overlay)
- Handles drop completion
- Spawns appropriate entities based on asset type

### 3. Entity Spawning

#### Mesh Models
```rust
// Spawns entity with:
- Name: "Mesh_<filename>"
- Transform: at origin (0, 0, 0)
- MeshHandle: path to asset file
```

#### Textures
```rust
// Applies to selected entity:
- Adds or updates TextureHandle with asset path
- Requires entity to be selected first
```

#### Audio Sources
```rust
// Spawns entity with:
- Name: "Audio_<filename>"
- Transform: at origin (0, 0, 0)  
- AudioSource: configured with spatial audio
  - Volume: 0.5
  - Spatial: true
  - Looping: false
```

### 4. Undo/Redo Support

All spawning operations use `EntityOperations` when `UndoRedoSystem` is available:
- Entity creation is undoable
- Component additions are undoable
- Batch operations for consistency

Without undo system, falls back to direct World spawning.

### 5. Selection Integration

- Newly spawned entities are automatically selected
- Selection is cleared before selecting new entity
- Uses `SelectionSystem` when available

### 6. Visual Feedback

#### During Drag
- Floating preview label follows cursor
- Asset name shown in preview

#### Hover Over Viewport
- Viewport highlights with colored overlay
- Text changes to "Drop here to add to scene"

#### After Drop
- Console logs entity creation
- Entity appears in hierarchy
- Entity is selected in scene

## Code Architecture

### Key Files Modified

1. **`drag_drop.rs`** - Core drag-drop system resource
2. **`scene_view_panel.rs`** - Drop target and entity spawning
3. **`assets_panel.rs`** - Drag source initialization
4. **`editor_state.rs`** - System coordination and panel integration
5. **`undo.rs`** - Component data variants (already existed)

### Extension Traits

Created extension traits to provide drag-drop aware rendering:
- `SceneViewPanelExt::ui_with_world()` - Accepts DragDropSystem
- `AssetsPanelExt::ui_with_drag_drop()` - Passes DragDropSystem to items

### Flow

1. User clicks and drags asset in AssetsPanel
2. `render_asset_item()` detects drag start via `response.drag_started()`
3. Creates `DragDropPayload::Asset` with path and name
4. Calls `DragDropSystem::start_drag()` to set active payload
5. Shows floating preview following cursor
6. User hovers over SceneViewPanel
7. `ui_with_world()` detects hover + drag state
8. Shows highlight overlay on viewport
9. User releases mouse button
10. Detects `pointer.any_released()` while hovering
11. Calls `DragDropSystem::complete_drop()` to get payload
12. Creates `AssetEntry` from payload path
13. Calls `handle_asset_drop()` with asset entry
14. Spawns appropriate entity via `spawn_mesh_entity()`, `apply_texture_to_selected()`, or `spawn_audio_entity()`
15. Logs creation and selects entity

## Usage

### Prerequisites
The ECS World must have `DragDropSystem` resource registered:

```rust
world.insert_resource(DragDropSystem::new());
```

### Basic Usage
1. Open AssetsPanel
2. Drag any asset file (not folder)
3. Drop onto SceneViewPanel viewport
4. Entity spawns at origin with appropriate components

### Texture Application
1. Select an existing entity in scene
2. Drag a texture asset
3. Drop onto viewport
4. Texture is applied to selected entity

### Future Enhancements

Potential improvements:
- Spawn at mouse position in 3D space (raycast to ground plane)
- Preview ghost/wireframe during drag
- Support for dropping into hierarchy panel (parenting)
- Material asset type support
- Scene instantiation support
- Configurable spawn positions
- Multi-select drag for batch instantiation
- Asset validation before drop
- Error dialogs for failed operations
