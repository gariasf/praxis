# Asset Browser Panel Implementation

This document provides a comprehensive overview of the Asset Browser Panel implementation for the Praxis game engine editor.

## Overview

The Asset Browser Panel is a full-featured asset management system integrated into the Praxis editor. It provides filesystem browsing, thumbnail previews, drag-and-drop functionality, asset import configuration, and automatic hot-reload capabilities.

## Features Implemented

### 1. Filesystem Traversal

**Location**: `crates/praxis_editor/src/panels/assets_panel.rs`

The asset browser provides complete filesystem navigation:

- **Directory Navigation**: Browse the `assets/` directory hierarchy
- **Breadcrumb Navigation**: Clickable path components for quick navigation
- **Back/Forward History**: Browser-style navigation history
- **Up Navigation**: Quick parent directory access
- **Search & Filter**: Real-time filtering by asset name

**Key Methods**:
- `navigate_to(path)` - Navigate to specific directory
- `navigate_back()` / `navigate_forward()` - History navigation
- `navigate_up()` - Move to parent directory
- `refresh_entries()` - Reload current directory contents

### 2. Thumbnail Generation

**Implementation**: Async thumbnail loading system

The browser generates thumbnails for texture assets:

- **Automatic Generation**: Thumbnails created on-demand for PNG/JPG/JPEG files
- **Queue-Based Loading**: Non-blocking thumbnail generation using `ThumbnailLoader`
- **Caching**: Thumbnails cached by path to prevent redundant loads
- **Fallback Icons**: Emoji icons for non-texture assets (models, audio, etc.)
- **Size**: Fixed 96x96 pixel thumbnails with proper aspect ratio

**Thumbnail States**:
- `NotLoaded` - Not yet requested
- `Loading` - Currently being generated
- `Loaded(TextureId)` - Successfully loaded
- `Failed` - Generation failed

**Key Methods**:
- `process_thumbnail_queue()` - Processes one thumbnail per frame
- `generate_texture_thumbnail(path)` - Creates downscaled image
- `load_thumbnail(path, texture_id)` - Manually add thumbnail
- `pending_thumbnail_count()` - Check loading queue size

### 3. Asset Type System

**Enum**: `AssetType`

Categorizes assets by file extension:

```rust
pub enum AssetType {
    Texture,  // png, jpg, jpeg
    Model,    // obj, gltf, glb
    Audio,    // wav, ogg, mp3
    Scene,    // scene
    Unknown,  // unsupported
}
```

Each type has:
- Icon emoji for visual identification
- Color coding for different types
- Extension detection logic

### 4. Drag-and-Drop System

**Module**: `crates/praxis_editor/src/drag_drop.rs`

Comprehensive drag-and-drop implementation:

- **Drag Initiation**: Click and drag assets from browser
- **Visual Preview**: Floating preview follows cursor during drag
- **Payload Types**: Support for assets, entities, and file paths
- **Drop Detection**: Integration with scene view panel
- **State Management**: ECS resource for global drag state

**Components**:
- `DragDropPayload` - Data being dragged
- `DragDropSystem` - ECS resource managing drag state
- `AssetEntry` - Asset metadata for drag operations

**Key Methods**:
- `get_dragged_asset()` - Retrieve and clear dragged asset
- `peek_dragged_asset()` - Check drag state without taking
- `is_dragging()` - Query drag status

### 5. Asset Import Dialogs

**Structure**: `AssetImportConfig`

Configurable import settings per asset type:

**Features**:
- Modal dialog with format-specific options
- Model import: Scale adjustment (0.01 - 10.0x)
- Texture import: Mipmap generation toggle
- Path and type display
- Import confirmation workflow

**Dialog Workflow**:
1. Single-click asset → Opens import dialog
2. Configure settings
3. Click Import → Triggers import logic
4. Asset ready for use in scene

### 6. File Watcher (Hot-Reload)

**Implementation**: `notify` crate integration

Automatic detection of filesystem changes:

- **Recursive Monitoring**: Watches entire `assets/` directory tree
- **Event Types**: Create, Modify, Delete
- **Automatic Refresh**: Updates browser when changes detected
- **Cache Invalidation**: Clears thumbnails for modified assets
- **Non-Blocking**: Events processed via channel

**Supported Operations**:
- New file creation → Appears in browser
- File modification → Thumbnail regenerated
- File deletion → Removed from browser
- Directory changes → Full refresh

### 7. User Interface Components

**Grid Layout**:
- Responsive column-based grid
- 96x96px thumbnails with labels
- Hover highlighting
- Double-click to open directories
- Right-click context menus

**Toolbar**:
- Navigation buttons (Back, Forward, Up)
- Breadcrumb path display
- Refresh button
- Sort toggle (by name)
- Hidden files toggle

**Search Bar**:
- Real-time filtering
- Case-insensitive search
- Clear button

**Status Bar**:
- Item count display
- Filtered vs total count
- Thumbnail loading status
- Current path display

**Context Menu**:
- Import... (files)
- Show in Explorer (platform-specific)
- Copy Path to clipboard
- Delete (with confirmation)
- Open (directories)

## Architecture

### Data Structures

```rust
pub struct AssetsPanel {
    current_path: PathBuf,
    entries: Vec<AssetEntry>,
    path_history: Vec<PathBuf>,
    path_forward_history: Vec<PathBuf>,
    search_filter: String,
    thumbnail_cache: HashMap<PathBuf, ThumbnailState>,
    file_watcher: Option<RecommendedWatcher>,
    file_watcher_rx: Option<Receiver<FileWatcherMessage>>,
    dragged_asset: Option<AssetEntry>,
    import_config: AssetImportConfig,
    thumbnail_loader: Arc<Mutex<ThumbnailLoader>>,
}

pub struct AssetEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub asset_type: AssetType,
    pub modified: Option<SystemTime>,
    pub thumbnail: Option<TextureId>,
}
```

### Thumbnail Loading Pipeline

```
1. User navigates to directory
   ↓
2. Visible items rendered
   ↓
3. Missing thumbnails queued
   ↓
4. One thumbnail per frame generated
   ↓
5. Image loaded and downscaled
   ↓
6. Thumbnail cached
   ↓
7. UI updated with texture
```

### File Watcher Flow

```
1. File system event occurs
   ↓
2. notify sends event via channel
   ↓
3. process_file_events() reads channel
   ↓
4. Event categorized (Create/Modify/Delete)
   ↓
5. If in current directory: refresh_entries()
   ↓
6. Cache invalidated for affected files
   ↓
7. UI automatically updates
```

## Integration Points

### With Editor State

The asset browser integrates with `EditorState`:

```rust
// In editor_state.rs
pub fn assets_panel_mut(&mut self) -> &mut AssetsPanel {
    &mut self.assets_panel
}
```

### With Scene View

Scene view can accept dropped assets:

```rust
// In scene_view_panel.rs
pub fn take_dropped_asset(&mut self) -> Option<AssetEntry> {
    self.last_dropped_asset.take()
}

pub fn can_accept_drop(&self, asset: &AssetEntry) -> bool {
    !asset.is_directory
}
```

### With Asset Loading System

Integration with `praxis_assets`:

```rust
// Load dropped model
if let Some(asset) = assets_panel.get_dragged_asset() {
    if asset.asset_type == AssetType::Model {
        praxis_assets::load_obj_mesh(
            mesh_manager,
            &asset.name,
            &asset.path
        )?;
    }
}
```

## Dependencies Added

### Cargo.toml Changes

```toml
[dependencies]
notify = "6.1"      # File system watching
image = "0.25"      # Thumbnail generation
```

## Usage Examples

### Basic Navigation

```rust
use praxis_editor::{AssetsPanel, EditorPanel};

let mut panel = AssetsPanel::new();

// Navigate to models directory
panel.navigate_to("assets/models");

// Go back
panel.navigate_back();

// Search for textures
panel.set_search_filter("diffuse".to_string());
```

### Drag-and-Drop Handling

```rust
// In your scene update loop
if let Some(asset) = assets_panel.get_dragged_asset() {
    match asset.asset_type {
        AssetType::Model => {
            // Spawn model entity at cursor position
            spawn_model_entity(world, &asset.path);
        }
        AssetType::Texture => {
            // Apply texture to selected object
            apply_texture(&asset.path);
        }
        _ => {}
    }
}
```

### Thumbnail Management

```rust
// Process thumbnails each frame (non-blocking)
assets_panel.process_thumbnail_queue();

// Check status
if assets_panel.pending_thumbnail_count() > 0 {
    // Show loading indicator
}

// Manually load thumbnail (if needed)
assets_panel.load_thumbnail(&path, texture_id);
```

## Performance Characteristics

- **Startup**: O(n) where n = files in current directory
- **Navigation**: O(n) for new directory, O(1) for history
- **Search**: O(n) filtering, real-time
- **Thumbnail Loading**: O(1) per frame (queue-based)
- **File Watching**: O(1) event processing (async notifications)
- **Memory**: O(n) for entries + O(m) for cached thumbnails

## Platform Support

- **Windows**: Full support (explorer integration)
- **macOS**: Full support (Finder integration)
- **Linux**: Full support (xdg-open integration)

## Future Enhancements

Potential improvements for future development:

1. **Model Thumbnails**: Generate previews for 3D models
2. **Audio Waveforms**: Visual previews for audio files
3. **Batch Operations**: Multi-select and batch import
4. **Asset Metadata**: Tags, ratings, descriptions
5. **Import Profiles**: Save/load import presets
6. **Thumbnail Cache Persistence**: Save thumbnails to disk
7. **Virtual Filesystem**: Support for packed asset archives
8. **Asset Dependencies**: Track asset references
9. **Version Control Integration**: Git status indicators
10. **Custom Asset Types**: Plugin system for new formats

## Testing Recommendations

### Manual Testing Checklist

- [ ] Navigate through multiple directory levels
- [ ] Test back/forward navigation
- [ ] Search for assets by name
- [ ] Drag asset to scene view
- [ ] Open import dialog and configure settings
- [ ] Create new file in assets/ (should appear automatically)
- [ ] Modify existing texture (thumbnail should update)
- [ ] Delete file (should disappear from browser)
- [ ] Right-click context menu on file
- [ ] Right-click context menu on directory
- [ ] Test with large directories (100+ files)
- [ ] Test with nested directory structures
- [ ] Test with various asset types
- [ ] Test platform-specific "Show in Explorer"

### Integration Testing

1. **Asset Loading**: Verify dropped assets load correctly
2. **Scene Integration**: Test asset instantiation in scene
3. **Memory**: Check for thumbnail memory leaks
4. **Performance**: Profile with 1000+ assets
5. **File Watcher**: Test with rapid file changes

## Code Organization

```
crates/praxis_editor/
├── src/
│   ├── lib.rs                     # Module exports
│   ├── drag_drop.rs               # Drag-drop system
│   └── panels/
│       ├── mod.rs                 # Panel exports
│       ├── assets_panel.rs        # Main implementation
│       └── scene_view_panel.rs    # Drop target
```

## Documentation

- Module-level documentation: Complete
- Type documentation: Complete
- Method documentation: Complete
- Usage examples: Complete
- Architecture diagrams: This document

## Conclusion

The Asset Browser Panel provides a production-ready asset management system for the Praxis editor. It combines modern UI patterns (breadcrumbs, search, drag-drop) with robust technical features (async loading, hot-reload, caching) to create an efficient and user-friendly workflow for game developers.

The implementation is extensible, performant, and follows Praxis coding conventions. It integrates seamlessly with the existing editor infrastructure and provides a solid foundation for future asset management features.
