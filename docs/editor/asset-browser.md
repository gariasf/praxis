# Asset Browser Guide

## Overview

The Asset Browser Panel provides comprehensive asset management for the Praxis editor, featuring filesystem navigation, thumbnail previews, drag-and-drop functionality, asset import configuration, and automatic hot-reload capabilities.

## Features

- **Filesystem Traversal**: Browse the `assets/` directory hierarchy with breadcrumb navigation
- **Thumbnail Generation**: Automatic thumbnail creation for textures with async loading
- **Drag-and-Drop**: Drag assets from browser to scene view for instant placement
- **Asset Import**: Configure import settings per asset type
- **Hot-Reload**: Automatic detection and update of filesystem changes
- **Search & Filter**: Real-time asset filtering by name
- **Context Menus**: Right-click operations for common tasks

## Architecture

### Core Components

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

### Asset Types

Assets are categorized by file extension:

```rust
pub enum AssetType {
    Texture,  // png, jpg, jpeg
    Model,    // obj, gltf, glb
    Audio,    // wav, ogg, mp3
    Scene,    // scene
    Unknown,  // unsupported
}
```

Each type has associated:
- **Icon**: Visual identifier (emoji or thumbnail)
- **Color**: Type-specific tinting
- **Import Settings**: Type-specific configuration options

## Navigation

### Basic Navigation

Navigate through the asset directory:

```rust
use praxis_editor::AssetsPanel;

let mut panel = AssetsPanel::new();

// Navigate to specific directory
panel.navigate_to("assets/models");

// Navigate up to parent
panel.navigate_up();

// Navigate back/forward
panel.navigate_back();
panel.navigate_forward();

// Refresh current directory
panel.refresh_entries();
```

### Breadcrumb Navigation

Click any path component to jump directly:

```
assets / models / characters / player
  ^        ^          ^            ^
  |        |          |            |
  Click any segment to navigate there
```

### History

Browser-style navigation history:

- **Back** (<): Return to previous directory
- **Forward** (>): Go to next directory in history
- History persists during session
- Clear history on project change

## Thumbnail System

### Automatic Generation

Thumbnails are generated automatically for texture assets:

```rust
// Thumbnail generation pipeline
1. User navigates to directory
2. Visible items rendered
3. Missing thumbnails queued
4. One thumbnail per frame generated
5. Image loaded and downscaled to 96x96
6. Thumbnail cached
7. UI updated
```

### Thumbnail States

```rust
pub enum ThumbnailState {
    NotLoaded,              // Not yet requested
    Loading,                // Currently being generated
    Loaded(TextureId),      // Successfully loaded
    Failed,                 // Generation failed
}
```

### Manual Thumbnail Loading

```rust
// Load thumbnail manually (if needed)
panel.load_thumbnail(&path, texture_id);

// Check loading status
if panel.pending_thumbnail_count() > 0 {
    // Show loading indicator
}

// Process one thumbnail per frame (non-blocking)
panel.process_thumbnail_queue();
```

### Fallback Icons

Non-texture assets display emoji icons:
- 📦 Model files (.obj, .gltf, .glb)
- 🔊 Audio files (.wav, .ogg, .mp3)
- 🎬 Scene files (.scene)
- ❓ Unknown file types

## Drag-and-Drop

### Initiating Drag

Click and hold an asset to start dragging:

```rust
// In your update loop
if panel.is_dragging() {
    // Asset is being dragged
    if let Some(asset) = panel.peek_dragged_asset() {
        show_drag_preview(asset);
    }
}
```

### Drop Handling

Handle dropped assets in scene view:

```rust
use praxis_editor::AssetsPanel;

// Check for dropped asset
if let Some(asset) = assets_panel.get_dragged_asset() {
    match asset.asset_type {
        AssetType::Model => {
            // Spawn model entity at cursor position
            spawn_model_at_cursor(&asset.path, cursor_pos);
        }
        AssetType::Texture => {
            // Apply texture to selected object
            apply_texture_to_selection(&asset.path);
        }
        AssetType::Audio => {
            // Create audio source
            create_audio_source(&asset.path, cursor_pos);
        }
        _ => {}
    }
}
```

### Drop Validation

Check if asset can be dropped:

```rust
if scene_view.can_accept_drop(&asset) {
    // Show valid drop indicator
    ui.painter().rect_filled(drop_zone, 0.0, Color32::GREEN.linear_multiply(0.3));
} else {
    // Show invalid drop indicator
    ui.painter().rect_filled(drop_zone, 0.0, Color32::RED.linear_multiply(0.3));
}
```

## Asset Import

### Import Dialog

Configure import settings before loading:

```rust
pub struct AssetImportConfig {
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub model_scale: f32,          // For models: 0.01 - 10.0x
    pub generate_mipmaps: bool,    // For textures
}
```

### Import Workflow

1. **Single-click** asset → Opens import dialog
2. **Configure settings** → Type-specific options
3. **Click Import** → Triggers import logic
4. **Asset ready** → Available in scene

### Example Import Logic

```rust
fn import_asset(config: &AssetImportConfig) -> Result<()> {
    match config.asset_type {
        AssetType::Model => {
            let model = praxis_assets::load_gltf(&config.path)?;
            // Apply scale
            model.transform.scale = Vec3::splat(config.model_scale);
            // Add to scene
            asset_manager.add_model(model);
        }
        AssetType::Texture => {
            let texture = praxis_assets::load_texture(&config.path)?;
            if config.generate_mipmaps {
                texture.generate_mipmaps();
            }
            asset_manager.add_texture(texture);
        }
        _ => {}
    }
    Ok(())
}
```

## File Watcher (Hot-Reload)

### Automatic Detection

The file watcher monitors the `assets/` directory for changes:

```rust
// File watcher flow
1. File system event occurs
2. notify sends event via channel
3. process_file_events() reads channel
4. Event categorized (Create/Modify/Delete)
5. If in current directory: refresh_entries()
6. Cache invalidated for affected files
7. UI automatically updates
```

### Supported Operations

| Operation | Behavior |
|-----------|----------|
| **Create** | New file appears in browser |
| **Modify** | Thumbnail regenerated |
| **Delete** | File removed from browser |
| **Rename** | Treated as delete + create |
| **Directory changes** | Full refresh triggered |

### Event Processing

```rust
// Process file watcher events
panel.process_file_events();

// Events are handled automatically, but you can also
// manually trigger a refresh if needed
panel.refresh_entries();
```

## Search and Filter

### Real-time Filtering

Filter assets by name:

```rust
// Set search filter
panel.set_search_filter("diffuse".to_string());

// Clear filter
panel.set_search_filter(String::new());

// Get filtered count
let filtered_count = panel.filtered_entry_count();
let total_count = panel.entry_count();
```

### Filter Behavior

- **Case-insensitive**: "Diffuse" matches "diffuse.png"
- **Substring match**: "diff" matches "diffuse.png"
- **Real-time**: Updates as you type
- **Persistent**: Filter maintained during navigation

## Context Menus

### File Context Menu

Right-click on file:

```
Import...
Show in Explorer
Copy Path
Delete
```

### Directory Context Menu

Right-click on directory:

```
Open
Show in Explorer
Copy Path
Create Subdirectory
Delete
```

### Context Menu Actions

```rust
match context_action {
    ContextAction::Import => {
        // Open import dialog
        show_import_dialog(&asset);
    }
    ContextAction::ShowInExplorer => {
        // Open system file browser
        #[cfg(target_os = "windows")]
        std::process::Command::new("explorer")
            .arg(asset.path.parent().unwrap())
            .spawn()?;
        
        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg(asset.path.parent().unwrap())
            .spawn()?;
        
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open")
            .arg(asset.path.parent().unwrap())
            .spawn()?;
    }
    ContextAction::CopyPath => {
        // Copy path to clipboard
        clipboard.set_text(asset.path.to_string_lossy().to_string());
    }
    ContextAction::Delete => {
        // Confirm and delete
        if confirm_delete() {
            std::fs::remove_file(&asset.path)?;
        }
    }
}
```

## User Interface

### Grid Layout

Assets displayed in responsive grid:

- **Thumbnail Size**: 96×96 pixels
- **Label**: Asset name below thumbnail
- **Columns**: Auto-calculated based on panel width
- **Hover**: Highlight on mouse over
- **Double-click**: Open directories

### Toolbar

Top toolbar with navigation and controls:

```
[<] [>] [↑] [📁 assets / models / characters] [🔄] [⚙️]
 |   |   |              |                      |     |
Back Fwd Up         Breadcrumb              Refresh Settings
```

### Search Bar

Real-time search with clear button:

```
🔍 [Search assets...] [×]
```

### Status Bar

Bottom status displays information:

```
10 items (2 filtered) | 3 thumbnails loading | assets/models/characters
```

## Performance

### Memory Usage

Per-directory memory footprint:

| Component | Memory |
|-----------|--------|
| **Entries** | ~1 KB per 100 files |
| **Thumbnails** | ~37 KB per texture (96×96 RGBA) |
| **Cache** | Limited by thumbnail count |

### Optimization Tips

1. **Lazy loading**: Thumbnails generated on-demand
2. **One per frame**: Non-blocking thumbnail generation
3. **Cache size limit**: Configure maximum cached thumbnails
4. **Async processing**: File operations on background thread

### Large Directories

For directories with 1000+ files:

```rust
// Virtualize grid (only render visible items)
let visible_range = calculate_visible_range(scroll_offset, panel_height);
for i in visible_range {
    render_asset_entry(&entries[i]);
}

// Prioritize thumbnail loading for visible items
for i in visible_range {
    if !has_thumbnail(&entries[i]) {
        queue_thumbnail(&entries[i]);
    }
}
```

## Integration with EditorState

### Adding to Editor

```rust
use praxis_editor::{EditorState, AssetsPanel};

let mut editor_state = EditorState::new();

// Assets panel automatically included
// Access via editor state
editor_state.assets_panel_mut().navigate_to("assets/textures");
```

### Scene Integration

Handle dropped assets in scene view:

```rust
// In scene update
if let Some(asset) = editor_state.assets_panel_mut().get_dragged_asset() {
    if editor_state.scene_view_panel().is_hovered() {
        let world_pos = screen_to_world(input.mouse_position());
        spawn_asset(&mut world, &asset, world_pos);
    }
}
```

## Advanced Usage

### Custom Asset Types

Extend with custom asset types:

```rust
impl AssetType {
    pub fn from_extension_custom(extension: &str) -> Self {
        match extension {
            "png" | "jpg" | "jpeg" => AssetType::Texture,
            "obj" | "gltf" | "glb" => AssetType::Model,
            "wav" | "ogg" | "mp3" => AssetType::Audio,
            "scene" => AssetType::Scene,
            "prefab" => AssetType::Prefab,  // Custom type
            "mat" => AssetType::Material,    // Custom type
            _ => AssetType::Unknown,
        }
    }
}
```

### Custom Thumbnails

Generate thumbnails for custom types:

```rust
// Generate model thumbnails
fn generate_model_thumbnail(path: &Path) -> Result<DynamicImage> {
    let model = load_model(path)?;
    let thumbnail = render_model_preview(&model, 96, 96)?;
    Ok(thumbnail)
}

// Register custom thumbnail generator
panel.register_thumbnail_generator(AssetType::Model, generate_model_thumbnail);
```

### Batch Operations

Perform operations on multiple assets:

```rust
// Multi-select assets (hold Ctrl)
let selected = panel.get_selected_assets();

// Batch import
for asset in selected {
    import_asset(&asset)?;
}

// Batch delete
for asset in selected {
    delete_asset(&asset)?;
}
```

## Troubleshooting

### Thumbnails Not Loading

**Problem**: Thumbnails show as loading indefinitely

**Solutions**:
- Check if `process_thumbnail_queue()` is called each frame
- Verify image file is valid (not corrupted)
- Check file permissions
- Increase queue processing rate

### File Watcher Not Working

**Problem**: Changes to files not reflected in browser

**Solutions**:
- Verify file watcher is initialized
- Check `process_file_events()` is called each frame
- Manually call `refresh_entries()` as workaround
- Check OS file watcher limits (inotify on Linux)

### Drag-and-Drop Not Working

**Problem**: Assets don't drop in scene view

**Solutions**:
- Verify scene view calls `can_accept_drop()`
- Check `get_dragged_asset()` is called on mouse release
- Ensure drop zone is hovered during release
- Debug with `peek_dragged_asset()` to check drag state

### Performance Issues

**Problem**: UI is slow with large directories

**Solutions**:
- Implement virtual scrolling for grid
- Limit thumbnails per frame
- Reduce thumbnail resolution
- Cache aggressively
- Use lazy loading for subdirectories

## Example: Complete Integration

```rust
use praxis_editor::*;
use praxis_ecs::World;

// Setup
fn setup_editor() -> EditorState {
    let mut editor = EditorState::new();
    
    // Configure assets panel
    editor.assets_panel_mut().navigate_to("assets");
    
    editor
}

// Update loop
fn update(
    editor: &mut EditorState,
    world: &mut World,
    input: &InputState,
) {
    // Process thumbnails (one per frame)
    editor.assets_panel_mut().process_thumbnail_queue();
    
    // Process file watcher events
    editor.assets_panel_mut().process_file_events();
    
    // Handle drag-and-drop
    if let Some(asset) = editor.assets_panel_mut().get_dragged_asset() {
        if editor.scene_view_panel().is_hovered() {
            match asset.asset_type {
                AssetType::Model => {
                    let pos = screen_to_world(input.mouse_position());
                    spawn_model(world, &asset.path, pos);
                }
                AssetType::Texture => {
                    if let Some(entity) = editor.selection().first_selected() {
                        apply_texture(world, entity, &asset.path);
                    }
                }
                _ => {}
            }
        }
    }
}
```

## See Also

- [Editor Overview](README.md)
- [Scene View Panel](panels.md)
- [Inspector Panel](inspector.md)
- [Asset Loading Guide](../guides/assets.md)
