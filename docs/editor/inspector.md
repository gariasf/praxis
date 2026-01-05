# Inspector Panel

The Inspector Panel provides comprehensive component viewing and editing capabilities for selected entities, enabling real-time modification of entity properties with full undo/redo support.

## Overview

The Inspector Panel is the primary interface for viewing and editing entity components in the Praxis editor. It dynamically displays all components attached to the selected entity and provides type-specific editors for each component.

### Key Features

- **Component Viewing**: Display all components on selected entity
- **Real-Time Editing**: Immediate property updates with visual feedback
- **Type-Specific Editors**: Custom UI for each component type
- **Add/Remove Components**: Dynamic component management
- **Undo/Redo Integration**: All changes tracked for undo
- **Multi-Component Support**: Edit multiple component types simultaneously
- **Read-Only Components**: View computed values like `GlobalTransform`
- **Validation**: Automatic input validation and clamping

## Architecture

### Panel Structure

```rust
pub struct InspectorPanel {
    title: String,
    visible: bool,
    
    // Undo support
    cached_transforms: HashMap<Entity, Transform>,
    
    // UI state
    add_component_open: bool,
}
```

### Supported Components

The Inspector Panel provides editors for the following component categories:

#### Transform & Hierarchy
- `Name` - Entity identifier
- `Transform` - Local position, rotation, scale
- `GlobalTransform` - Computed world-space transform (read-only)
- `Parent` - Parent entity reference (read-only)
- `Children` - Child entity list (read-only)

#### Rendering
- `MeshHandle` - Mesh asset reference
- `TextureHandle` - Texture asset reference
- `MaterialHandle` - Material asset reference
- `MaterialPropertiesComponent` - PBR material properties
- `Visibility` - Visibility state

#### Camera
- `Camera` - Camera component (active state, priority)
- `PerspectiveProjection` - Perspective camera parameters
- `OrthographicProjection` - Orthographic camera parameters

#### Lighting
- `DirectionalLight` - Directional light properties
- `PointLight` - Point light properties

#### Physics
- `RigidBody` - Physics body type (Dynamic/Static/Kinematic)
- `Collider` - Collision shape and size
- `PhysicsVelocity` - Linear and angular velocity
- `Mass` - Mass and angular inertia

#### Audio
- `AudioSource` - Audio playback configuration
- `AudioListener` - Audio listener marker

## Usage

### Basic Setup

```rust
use praxis_editor::panels::InspectorPanel;
use praxis_editor::SelectionSystem;
use praxis_ecs::World;

let mut inspector_panel = InspectorPanel::new();
let mut world = World::new();

// In your editor update loop
inspector_panel.ui_with_world(ui, &mut world);
```

### Integration with Selection

The Inspector automatically displays components for the selected entity:

```rust
// Get selection from hierarchy or viewport
let selected_entity = selection_system.primary_selection();

// Inspector reads selection from World resource
world.insert_resource(selection_system);

// Inspector automatically shows components for selected entity
inspector_panel.ui_with_world(ui, &mut world);
```

### Integration with EditorState

The Inspector Panel is automatically included in `EditorState`:

```rust
use praxis_editor::EditorState;

let mut editor_state = EditorState::new();

// Inspector panel is automatically rendered
// Access via editor_state.inspector_panel if needed
```

## Component Editors

### Name Component

Edits entity display name:

```rust
// UI Elements:
// - Text input field
// - Remove component button

// Example usage:
world.entity_mut(entity).insert(Name::new("Player"));
```

**Editor Features**:
- Single-line text input
- Real-time updates
- No validation (any string allowed)

### Transform Component

Edits local transform with separate translation, rotation, and scale controls:

```rust
// UI Elements:
// - Translation: 3x drag value (X, Y, Z)
// - Rotation: 3x drag value (Euler angles in degrees)
// - Scale: 3x drag value (X, Y, Z)
// - Remove component button

// Example usage:
let transform = Transform {
    translation: Vec3::new(1.0, 2.0, 3.0),
    rotation: Quat::from_euler(EulerRot::XYZ, 0.0, 45.0_f32.to_radians(), 0.0),
    scale: Vec3::ONE,
};
world.entity_mut(entity).insert(transform);
```

**Editor Features**:
- Translation: 0.1 units/step
- Rotation: 1 degree/step, automatic quaternion conversion
- Scale: 0.01 units/step
- Real-time updates with undo support
- Cached values for undo on mouse release

**Undo Behavior**:
```rust
// Changes applied immediately for visual feedback
// Undo command created on mouse button release
if ui.input(|i| i.pointer.any_released()) {
    let command = TransformEditCommand::new(entity, old_transform, new_transform);
    undo_system.history.undo_stack.push_back(Box::new(command));
}
```

### GlobalTransform Component (Read-Only)

Displays computed world-space transform:

```rust
// UI Elements:
// - World Position (read-only)
// - World Scale (read-only)
// - Collapsible header (default collapsed)
```

**Note**: This is computed by the transform propagation system and cannot be edited directly.

### Material Properties Component

Edits PBR material properties:

```rust
// UI Elements:
// - Base Color: RGBA sliders/drag values
// - Metallic: Slider (0.0-1.0)
// - Roughness: Slider (0.0-1.0)
// - Emissive Strength: Drag value (0.0-100.0)
// - Remove component button

// Example usage:
let props = MaterialProperties {
    base_color: [1.0, 0.5, 0.2, 1.0],
    metallic: 0.0,
    roughness: 0.8,
    emissive_strength: 0.0,
};
world.entity_mut(entity).insert(MaterialPropertiesComponent(props));
```

**Editor Features**:
- Color picker for base color
- Sliders with visual feedback
- Real-time material updates
- Proper value clamping

### Camera Components

#### Camera Component
```rust
// UI Elements:
// - Active checkbox
// - Priority drag value
// - Remove component button

let camera = Camera {
    is_active: true,
    priority: 0,
};
world.entity_mut(entity).insert(camera);
```

#### PerspectiveProjection Component
```rust
// UI Elements:
// - FOV: Drag value (1.0-179.0 degrees)
// - Aspect Ratio: Drag value (0.1-10.0)
// - Near Plane: Drag value (0.001-100.0)
// - Far Plane: Drag value (1.0-10000.0)
// - Remove component button

let projection = PerspectiveProjection {
    fov: 60.0_f32.to_radians(),
    aspect_ratio: 16.0 / 9.0,
    near: 0.1,
    far: 1000.0,
};
world.entity_mut(entity).insert(projection);
```

#### OrthographicProjection Component
```rust
// UI Elements:
// - Left, Right, Bottom, Top: Drag values
// - Near/Far Plane: Drag values
// - Remove component button

let projection = OrthographicProjection {
    left: -10.0,
    right: 10.0,
    bottom: -10.0,
    top: 10.0,
    near: 0.1,
    far: 1000.0,
};
world.entity_mut(entity).insert(projection);
```

### Lighting Components

#### DirectionalLight Component
```rust
// UI Elements:
// - Direction: 3x drag value (X, Y, Z) - auto-normalized
// - Color: 3x drag value (RGB, 0.0-1.0)
// - Intensity: Drag value (0.0-100.0)
// - Remove component button

let light = DirectionalLight {
    direction: Vec3::new(0.0, -1.0, 0.0).normalize(),
    color: Vec3::new(1.0, 1.0, 0.9),
    intensity: 5.0,
};
world.entity_mut(entity).insert(light);
```

**Note**: Direction is automatically normalized after editing.

#### PointLight Component
```rust
// UI Elements:
// - Color: 3x drag value (RGB, 0.0-1.0)
// - Intensity: Drag value (0.0-100.0)
// - Range: Drag value (0.1-1000.0)
// - Remove component button

let light = PointLight {
    color: Vec3::ONE,
    intensity: 10.0,
    range: 20.0,
};
world.entity_mut(entity).insert(light);
```

### Physics Components

#### RigidBody Component
```rust
// UI Elements:
// - Type: Radio buttons (Dynamic/Static/Kinematic)
// - Remove component button

let rigid_body = RigidBody::Dynamic;
world.entity_mut(entity).insert(rigid_body);
```

**Physics Body Types**:
- **Dynamic**: Affected by forces and gravity
- **Static**: Immovable, used for environment
- **Kinematic**: Manually controlled, not affected by forces

#### Collider Component
```rust
// UI Elements:
// - Shape Type: Combo box dropdown
// - Shape-specific parameters (drag values)
// - Remove component button

// Available shapes:
let collider = Collider::Cuboid { hx: 0.5, hy: 0.5, hz: 0.5 };
let collider = Collider::Sphere { radius: 0.5 };
let collider = Collider::CapsuleY { half_height: 1.0, radius: 0.5 };
let collider = Collider::CapsuleX { half_height: 1.0, radius: 0.5 };
let collider = Collider::CapsuleZ { half_height: 1.0, radius: 0.5 };
let collider = Collider::CylinderY { half_height: 1.0, radius: 0.5 };

world.entity_mut(entity).insert(collider);
```

**Editor Features**:
- Dropdown to change shape type
- Shape-specific parameter editors
- All values clamped to valid ranges (> 0.01)

#### PhysicsVelocity Component
```rust
// UI Elements:
// - Linear Velocity: 3x drag value (X, Y, Z)
// - Angular Velocity: 3x drag value (X, Y, Z)
// - Remove component button

let velocity = PhysicsVelocity {
    linear: Vec3::new(1.0, 0.0, 0.0),
    angular: Vec3::new(0.0, 0.5, 0.0),
};
world.entity_mut(entity).insert(velocity);
```

#### Mass Component
```rust
// UI Elements:
// - Mass: Drag value (0.0-max)
// - Angular Inertia: Drag value (0.0-max)
// - Remove component button

let mass = Mass {
    mass: 1.0,
    angular_inertia: 1.0,
};
world.entity_mut(entity).insert(mass);
```

### Audio Components

#### AudioSource Component
```rust
// UI Elements:
// - Audio Path: Text input
// - Volume: Slider (0.0-1.0)
// - Spatial: Checkbox
// - Looping: Checkbox
// - Doppler Enabled: Checkbox
// - Max Distance: Drag value (conditional, if spatial)
// - Reference Distance: Drag value (conditional, if spatial)
// - Doppler Scale: Drag value (conditional, if doppler enabled)
// - Playback Controls: Play/Pause/Stop buttons
// - State Display: Current playback state
// - Remove component button

let audio = AudioSource {
    path: "sounds/ambient.ogg".to_string(),
    volume: 1.0,
    spatial: true,
    looping: true,
    doppler_enabled: false,
    max_distance: 100.0,
    reference_distance: 1.0,
    doppler_scale: 1.0,
    state: AudioState::Stopped,
};
world.entity_mut(entity).insert(audio);
```

**Editor Features**:
- Conditional UI (spatial/doppler parameters only shown when enabled)
- Real-time playback control
- State display for debugging

#### AudioListener Component
```rust
// UI Elements:
// - Marker component label
// - Remove component button

world.entity_mut(entity).insert(AudioListener);
```

**Note**: AudioListener is a marker component with no properties.

### Visibility Component
```rust
// UI Elements:
// - Visible checkbox
// - Remove component button

let visibility = Visibility::visible();
world.entity_mut(entity).insert(visibility);
```

## Adding Components

### Via Add Component Menu

1. Click "➕ Add Component" button
2. Select component type from dropdown
3. Component is added with default values
4. Undo/redo supported

```rust
// UI provides dropdown with all available components
// Only shows components not already on the entity
if ui.selectable_label(false, "Transform").clicked() {
    world.insert_component(entity, Transform::default());
    add_component_open = false;
}
```

### Programmatically

```rust
use praxis_editor::EntityOperations;

let mut entity_ops = EntityOperations::new();

// Add transform component
entity_ops.add_transform(
    &mut world,
    &mut undo_system,
    entity,
    Transform::from_xyz(0.0, 1.0, 0.0),
)?;

// Add name component
entity_ops.add_name(&mut world, &mut undo_system, entity, "MyEntity")?;

// Add generic component
entity_ops.add_component(
    &mut world,
    &mut undo_system,
    entity,
    ComponentData::RigidBody(RigidBody::Dynamic),
)?;
```

## Removing Components

### Via Inspector UI

Each component header has a "🗑" (trash) button:
- Click to remove component
- Component state captured for undo
- Full undo/redo support

### Programmatically

```rust
use praxis_editor::EntityOperations;

let mut entity_ops = EntityOperations::new();

// Remove transform component
entity_ops.remove_transform(&mut world, &mut undo_system, entity)?;

// Remove generic component
entity_ops.remove_component(
    &mut world,
    &mut undo_system,
    entity,
    ComponentData::Name("Previous Name".to_string()),
)?;
```

**Note**: The previous component value must be provided for proper undo support.

## Undo/Redo Integration

### Transform Editing Pattern

Transform edits use a two-phase approach:
1. **Immediate Updates**: Changes applied in real-time for visual feedback
2. **Deferred Undo**: Command created on mouse release

```rust
// Cache original transform when starting edit
cached_transforms.entry(entity).or_insert(original_transform);

// Apply changes immediately
if changed {
    world.entity_mut(entity).insert(transform);
}

// Create undo command on mouse release
if ui.input(|i| i.pointer.any_released()) {
    let old_transform = cached_transforms.get(&entity).copied()?;
    let command = TransformEditCommand::new(entity, old_transform, transform);
    undo_system.add_command(command);
    
    // Update cache with new value
    cached_transforms.insert(entity, transform);
}
```

### Component Add/Remove Pattern

```rust
// Add component command
let command = AddComponentCommand::new(entity, ComponentData::Transform(transform));
undo_system.execute_command(&mut world, Box::new(command))?;

// Remove component command (captures current value for undo)
let current_transform = world.get::<Transform>(entity).copied()?;
let command = RemoveComponentCommand::new(
    entity,
    ComponentData::Transform(current_transform),
);
undo_system.execute_command(&mut world, Box::new(command))?;
```

## Extending the Inspector

### Adding Custom Component Editors

To add a custom component editor:

1. **Implement the editor function**:
```rust
fn render_my_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    let mut query = world.query::<&mut MyComponent>();
    if let Ok(mut component) = query.get_mut(world.inner_mut(), entity) {
        let mut open = true;
        egui::CollapsingHeader::new("My Component")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("My Property:");
                    if ui.button("🗑").on_hover_text("Remove component").clicked() {
                        open = false;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    ui.add(egui::DragValue::new(&mut component.value).speed(0.1));
                });
            });
        
        if !open {
            world.remove_component::<MyComponent>(entity);
        }
    }
}
```

2. **Call in render loop**:
```rust
fn render_selected_entity(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // ... existing component editors ...
    
    self.render_my_component(ui, world, entity);
}
```

3. **Add to component menu**:
```rust
fn render_add_component_menu(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    let has_my_component = world.inner().get::<MyComponent>(entity).is_some();
    
    if !has_my_component && ui.selectable_label(false, "My Component").clicked() {
        let _ = world.insert_component(entity, MyComponent::default());
        self.add_component_open = false;
    }
}
```

### Example: Custom Component

```rust
use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

// Inspector implementation
impl InspectorPanel {
    fn render_health_component(&self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut query = world.query::<&mut Health>();
        if let Ok(mut health) = query.get_mut(world.inner_mut(), entity) {
            let mut open = true;
            egui::CollapsingHeader::new("Health")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Health Component");
                        if ui.button("🗑").on_hover_text("Remove component").clicked() {
                            open = false;
                        }
                    });
                    
                    // Current health
                    ui.horizontal(|ui| {
                        ui.label("Current:");
                        ui.add(
                            egui::DragValue::new(&mut health.current)
                                .speed(1.0)
                                .range(0.0..=health.max)
                        );
                    });
                    
                    // Max health
                    ui.horizontal(|ui| {
                        ui.label("Max:");
                        ui.add(
                            egui::DragValue::new(&mut health.max)
                                .speed(1.0)
                                .range(1.0..=f32::MAX)
                        );
                    });
                    
                    // Health bar visualization
                    let health_ratio = health.current / health.max;
                    ui.add(
                        egui::ProgressBar::new(health_ratio)
                            .text(format!("{:.0}/{:.0}", health.current, health.max))
                    );
                    
                    // Quick actions
                    ui.horizontal(|ui| {
                        if ui.button("Full Heal").clicked() {
                            health.current = health.max;
                        }
                        if ui.button("Set to Half").clicked() {
                            health.current = health.max * 0.5;
                        }
                    });
                });
            
            if !open {
                world.remove_component::<Health>(entity);
            }
        }
    }
}
```

## Best Practices

### Performance

1. **Cache Values**: Cache expensive computations or lookups
2. **Avoid per-Frame Allocations**: Reuse buffers and collections
3. **Collapse Unused Sections**: Default headers to collapsed for large component lists
4. **Batch Updates**: Group related changes when possible

### User Experience

1. **Provide Visual Feedback**: Show which entity is being edited
2. **Use Appropriate Widgets**: Sliders for bounded values, drag values for unbounded
3. **Show Units**: Label numeric fields with units (degrees, meters, etc.)
4. **Validation**: Clamp or validate inputs to prevent invalid states
5. **Contextual Help**: Use `.on_hover_text()` for tooltips

### Organization

```rust
// Group related properties
ui.collapsing("Transform", |ui| {
    ui.label("Position:");
    // position controls
    
    ui.separator();
    
    ui.label("Rotation:");
    // rotation controls
    
    ui.separator();
    
    ui.label("Scale:");
    // scale controls
});

// Use consistent spacing
ui.add_space(4.0);
```

## Common Patterns

### Conditional Properties

Show/hide properties based on other values:

```rust
ui.checkbox(&mut audio_source.spatial, "Spatial Audio");

// Only show spatial properties if spatial is enabled
if audio_source.spatial {
    ui.horizontal(|ui| {
        ui.label("Max Distance:");
        ui.add(
            egui::DragValue::new(&mut audio_source.max_distance)
                .speed(1.0)
                .range(0.1..=1000.0)
        );
    });
}
```

### Value Clamping

Ensure values stay within valid ranges:

```rust
ui.horizontal(|ui| {
    ui.label("FOV (degrees):");
    let mut fov_degrees = projection.fov.to_degrees();
    if ui.add(
        egui::DragValue::new(&mut fov_degrees)
            .speed(1.0)
            .range(1.0..=179.0)  // Clamp to valid FOV range
    ).changed() {
        projection.fov = fov_degrees.to_radians();
    }
});
```

### Enum Selection

Use radio buttons or combo boxes for enum selection:

```rust
// Radio buttons
ui.horizontal(|ui| {
    if ui.radio(rigid_body.is_dynamic(), "Dynamic").clicked() {
        *rigid_body = RigidBody::Dynamic;
    }
    if ui.radio(rigid_body.is_static(), "Static").clicked() {
        *rigid_body = RigidBody::Static;
    }
    if ui.radio(rigid_body.is_kinematic(), "Kinematic").clicked() {
        *rigid_body = RigidBody::Kinematic;
    }
});

// Combo box
egui::ComboBox::from_label("Shape Type")
    .selected_text(current_type)
    .show_ui(ui, |ui| {
        if ui.selectable_label(current_type == "Cuboid", "Cuboid").clicked() {
            *collider = Collider::cuboid(0.5, 0.5, 0.5);
        }
        if ui.selectable_label(current_type == "Sphere", "Sphere").clicked() {
            *collider = Collider::sphere(0.5);
        }
    });
```

### Vector/Color Editing

Provide appropriate editors for vectors and colors:

```rust
// Vector3 editing
ui.label("Direction:");
ui.horizontal(|ui| {
    ui.label("X:");
    ui.add(egui::DragValue::new(&mut direction.x).speed(0.01));
    ui.label("Y:");
    ui.add(egui::DragValue::new(&mut direction.y).speed(0.01));
    ui.label("Z:");
    ui.add(egui::DragValue::new(&mut direction.z).speed(0.01));
});

// Color editing
let mut color = egui::Color32::from_rgba_premultiplied(
    (base_color[0] * 255.0) as u8,
    (base_color[1] * 255.0) as u8,
    (base_color[2] * 255.0) as u8,
    (base_color[3] * 255.0) as u8,
);
if ui.color_edit_button_srgba(&mut color).changed() {
    base_color = [
        color.r() as f32 / 255.0,
        color.g() as f32 / 255.0,
        color.b() as f32 / 255.0,
        color.a() as f32 / 255.0,
    ];
}
```

## Examples

### Example 1: Batch Component Editing

```rust
use praxis_editor::EntityOperations;

// Select multiple entities
let selected_entities: Vec<Entity> = selection_system
    .selected_entities()
    .collect();

// Apply same material to all
entity_ops.begin_batch("Set Material");

for entity in &selected_entities {
    if world.get::<MeshHandle>(*entity).is_some() {
        world.entity_mut(*entity).insert(MaterialHandle::new("metal"));
    }
}

entity_ops.end_batch(&mut world, &mut undo_system)?;
```

### Example 2: Component Presets

```rust
// Define component presets
fn apply_player_preset(world: &mut World, entity: Entity) {
    world.entity_mut(entity).insert((
        Name::new("Player"),
        Transform::from_xyz(0.0, 1.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule_y(1.0, 0.5),
        Mass { mass: 80.0, angular_inertia: 1.0 },
    ));
}

fn apply_camera_preset(world: &mut World, entity: Entity) {
    world.entity_mut(entity).insert((
        Name::new("Main Camera"),
        Transform::from_xyz(0.0, 5.0, 10.0),
        Camera { is_active: true, priority: 0 },
        PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        },
    ));
}

// Use in inspector
ui.menu_button("Apply Preset", |ui| {
    if ui.button("Player").clicked() {
        apply_player_preset(&mut world, entity);
    }
    if ui.button("Camera").clicked() {
        apply_camera_preset(&mut world, entity);
    }
});
```

### Example 3: Copy/Paste Components

```rust
use std::collections::HashMap;

struct ComponentClipboard {
    components: HashMap<String, ComponentData>,
}

impl ComponentClipboard {
    fn copy_from(&mut self, world: &World, entity: Entity) {
        self.components.clear();
        
        // Copy transform
        if let Some(transform) = world.get::<Transform>(entity) {
            self.components.insert(
                "Transform".to_string(),
                ComponentData::Transform((*transform).into()),
            );
        }
        
        // Copy name
        if let Some(name) = world.get::<Name>(entity) {
            self.components.insert(
                "Name".to_string(),
                ComponentData::Name(name.0.clone()),
            );
        }
        
        // Add more components as needed
    }
    
    fn paste_to(&self, world: &mut World, entity: Entity, entity_ops: &mut EntityOperations, undo_system: &mut UndoRedoSystem) -> Result<()> {
        entity_ops.begin_batch("Paste Components");
        
        for (name, component) in &self.components {
            entity_ops.add_component(world, undo_system, entity, component.clone())?;
        }
        
        entity_ops.end_batch(world, undo_system)?;
        Ok(())
    }
}

// Use in inspector
if ui.button("📋 Copy Components").clicked() {
    clipboard.copy_from(&world, selected_entity);
}

if ui.button("📄 Paste Components").clicked() {
    clipboard.paste_to(&mut world, selected_entity, &mut entity_ops, &mut undo_system)?;
}
```

## Troubleshooting

### Component Not Showing

**Problem**: Added component doesn't appear in inspector.

**Solution**: Ensure component is registered and has a render function:
```rust
// Add render function in inspector.rs
self.render_my_component(ui, world, entity);
```

### Undo Not Working

**Problem**: Component changes don't undo properly.

**Solution**: Ensure changes go through the undo system:
```rust
// Wrong: Direct modification
world.entity_mut(entity).insert(component);

// Right: Through undo system
entity_ops.add_component(&mut world, &mut undo_system, entity, component)?;
```

### Transform Updates Not Propagating

**Problem**: Transform changes don't affect child entities.

**Solution**: Ensure transform propagation system is running:
```rust
schedule.add_systems(propagate_transforms_system);
```

## See Also

- [Hierarchy Panel](hierarchy.md) - Entity hierarchy management
- [Selection System](selection.md) - Entity selection
- [Undo/Redo System](undo-redo.md) - Command history
- [Entity Operations](../../crates/praxis_editor/src/entity_operations.rs) - Component operations API
