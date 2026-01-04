# Project Settings Panel Implementation

## Overview

Implemented a comprehensive Project Settings Panel for the Praxis editor with a tabbed interface for configuring graphics, physics, audio, and input settings. Settings are saved to and loaded from a `project.ron` configuration file.

## Implementation Details

### Files Created/Modified

1. **Created: `crates/praxis_editor/src/panels/project_settings_panel.rs`**
   - Complete settings panel implementation with tabbed interface
   - RON serialization/deserialization support
   - Five configuration categories: General, Graphics, Physics, Audio, Input

2. **Modified: `crates/praxis_editor/src/panels/mod.rs`**
   - Added project_settings_panel module
   - Exported new types: `ProjectSettingsPanel`, `ProjectSettings`, `GraphicsSettings`, `PhysicsSettings`, `AudioSettings`, `InputSettings`

3. **Modified: `crates/praxis_editor/src/editor_state.rs`**
   - Added `ProjectSettings` to `EditorTab` enum
   - Added `project_settings_panel` field to `EditorState`
   - Integrated panel into `EditorTabViewer`
   - Added `project_settings_panel_mut()` accessor method

4. **Modified: `crates/praxis_editor/src/lib.rs`**
   - Exported all new settings types

5. **Modified: `crates/praxis_editor/src/menu_bar.rs`**
   - Added `ToggleProjectSettings` action
   - Added `project_settings_visible` field to `MenuBarState`
   - Added menu item in View menu
   - Implemented toggle handler

6. **Modified: `.gitignore`**
   - Added `project.ron` to gitignore (user-specific configuration)

7. **Created: `project.ron.example`**
   - Example configuration file showing the structure

## Features

### General Tab
- Project name configuration
- Project version configuration
- Settings file path configuration

### Graphics Tab
- Resolution configuration (width/height)
- MSAA samples (1, 2, 4, 8)
- VSync toggle
- Fullscreen toggle
- Target FPS setting (0 = unlimited)

### Physics Tab
- Gravity vector (X, Y, Z)
- Quick presets: Earth Gravity, Zero Gravity
- Fixed timestep configuration
- Position iterations (solver accuracy)
- Velocity iterations (solver accuracy)

### Audio Tab
- Master volume slider (0.0 - 1.0)
- Music volume slider (0.0 - 1.0)
- SFX volume slider (0.0 - 1.0)
- Doppler effect scale factor
- Speed of sound configuration
- Maximum audio sources limit

### Input Tab
- Mouse sensitivity slider (0.1 - 5.0)
- Invert mouse Y toggle
- Gamepad deadzone slider (0.0 - 0.5)
- Key bindings editor:
  - View existing bindings
  - Remove bindings
  - Add new action/key mappings

### Panel Features
- **Save**: Saves current settings to project.ron
- **Load**: Reloads settings from project.ron
- **Reset to Defaults**: Restores all default values
- **Status Messages**: Visual feedback for save/load/error operations

## Configuration Structure

Settings are stored in RON format with the following structure:

```ron
(
    graphics: (
        resolution_width: 1920,
        resolution_height: 1080,
        msaa_samples: 4,
        vsync: true,
        fullscreen: false,
        target_fps: 60,
    ),
    physics: (
        gravity_x: 0.0,
        gravity_y: -9.81,
        gravity_z: 0.0,
        timestep: 0.016666668,
        position_iterations: 4,
        velocity_iterations: 1,
    ),
    audio: (
        master_volume: 1.0,
        music_volume: 0.7,
        sfx_volume: 0.8,
        doppler_scale: 1.0,
        speed_of_sound: 343.0,
        max_audio_sources: 32,
    ),
    input: (
        mouse_sensitivity: 1.0,
        invert_mouse_y: false,
        gamepad_deadzone: 0.15,
        key_bindings: {
            "Forward": "W",
            "Backward": "S",
            "Left": "A",
            "Right": "D",
            "Jump": "Space",
            "Sprint": "Shift",
        },
    ),
    project_name: "Praxis Project",
    project_version: "0.1.0",
)
```

## Default Values

### Graphics
- Resolution: 1920x1080
- MSAA: 1 (disabled)
- VSync: enabled
- Fullscreen: disabled
- Target FPS: 60

### Physics
- Gravity: (0.0, -9.81, 0.0) - Earth gravity
- Timestep: 1/60 seconds (60 Hz)
- Position Iterations: 4
- Velocity Iterations: 1

### Audio
- Master Volume: 1.0 (100%)
- Music Volume: 0.7 (70%)
- SFX Volume: 0.8 (80%)
- Doppler Scale: 1.0
- Speed of Sound: 343.0 units/s
- Max Audio Sources: 32

### Input
- Mouse Sensitivity: 1.0
- Invert Mouse Y: false
- Gamepad Deadzone: 0.15
- Default Key Bindings: WASD movement, Space for jump, Shift for sprint

## Usage

### Accessing the Panel

The Project Settings panel can be accessed via:
1. View menu > Project Settings
2. The panel can be docked anywhere in the editor

### Programmatic Access

```rust
// Get reference to the panel
let panel = editor_state.project_settings_panel_mut();

// Access settings
let settings = panel.settings();
println!("Resolution: {}x{}", 
    settings.graphics.resolution_width,
    settings.graphics.resolution_height);

// Modify settings
panel.settings_mut().graphics.vsync = false;

// Save to file
panel.save();

// Load from file
panel.load();

// Reset to defaults
panel.reset_to_defaults();
```

### Loading Settings on Startup

```rust
// Load settings with default path
let panel = ProjectSettingsPanel::new();

// Load settings with custom path
let panel = ProjectSettingsPanel::with_path("custom_config.ron".to_string());

// Load settings directly
let settings = ProjectSettings::load_from_file("project.ron")
    .unwrap_or_default();
```

## Integration Points

The settings can be used to configure engine subsystems:

```rust
// Apply graphics settings
let graphics_settings = &settings.graphics;
// Configure window, MSAA, VSync, etc.

// Apply physics settings
let physics_config = PhysicsConfig {
    gravity: Vec3::new(
        settings.physics.gravity_x,
        settings.physics.gravity_y,
        settings.physics.gravity_z
    ),
    timestep: settings.physics.timestep,
};
world.insert_resource(physics_config);

// Apply audio settings
audio_manager.set_master_volume(settings.audio.master_volume);
audio_manager.set_doppler_scale(settings.audio.doppler_scale);

// Apply input settings
// Configure input map with key bindings
for (action, key) in &settings.input.key_bindings {
    input_map.bind_key(&Action::new(action), parse_key(key));
}
```

## Technical Details

### Serialization
- Uses RON (Rusty Object Notation) format via the `ron` crate
- Pretty-printed output with 4-space indentation
- All settings structs implement `Serialize` and `Deserialize`
- Graceful fallback to defaults if file doesn't exist or is invalid

### Error Handling
- File I/O errors are displayed in the UI as status messages
- Parse errors show descriptive messages
- Missing files automatically create defaults
- Invalid values are clamped to valid ranges

### UI Components
- Tab selection using `selectable_value`
- Sliders for normalized values (0.0-1.0)
- Drag values for numeric configuration
- Combo boxes for discrete options (MSAA samples)
- Checkboxes for boolean toggles
- Scrollable areas for key bindings list
- Status messages with colored text feedback

## Future Enhancements

Potential improvements:
1. Real-time application of settings changes
2. Settings validation with warnings for problematic values
3. Keyboard shortcut capture for key bindings
4. Profile system (save/load different configurations)
5. Import/export settings
6. Settings categories expansion (rendering quality presets, etc.)
7. Integration with engine hot-reload system
8. Per-scene override settings
