# Project 08: Scene Editor

**Difficulty**: Advanced  
**Estimated Time**: 4-6 weeks  
**Core Learning**: Editor architecture, undo/redo systems, gizmos, serialization

## Overview

Build a scene editor for placing, manipulating, and saving 3D objects. This project teaches editor tool development, command pattern for undo/redo, transform gizmos, object selection, and scene serialization used in professional game engines.

### Learning Objectives

- Design editor architecture separate from runtime
- Implement selection and manipulation tools
- Build undo/redo system with command pattern
- Create transform gizmos (translate, rotate, scale)
- Serialize and deserialize scene data
- Develop property inspector/editor UI

## Feature Requirements

### Core Features (Minimum Viable)

1. **Object Management**
   - Spawn primitive objects (cube, sphere, plane)
   - Select objects (click to select)
   - Delete selected objects
   - Object hierarchy (parent-child relationships)
   - Scene outliner (list all objects)

2. **Transform Manipulation**
   - Move tool (translate gizmo)
   - Visual gizmo (3 arrows for X/Y/Z axes)
   - Drag gizmo to move object
   - Grid snapping (optional toggle)

3. **Camera Controls**
   - Orbit camera (like Project 01)
   - Focus on selected object (F key)
   - Pan and zoom
   - Camera speed adjustment

4. **Save/Load Scene**
   - Save scene to JSON or binary format
   - Load scene from file
   - File picker dialog
   - Scene versioning

### Extended Features (Recommended)

5. **Advanced Gizmos**
   - Rotate tool (rotation gizmo with rings)
   - Scale tool (scale gizmo with cubes)
   - Tool switching (hotkeys: W/E/R for move/rotate/scale)
   - Local vs. world space toggle
   - Uniform vs. non-uniform scaling

6. **Undo/Redo System**
   - Undo transform changes (Ctrl+Z)
   - Redo (Ctrl+Y or Ctrl+Shift+Z)
   - Undo spawn/delete operations
   - Command history view
   - Unlimited undo stack

7. **Property Inspector**
   - Display selected object properties
   - Edit position/rotation/scale numerically
   - Edit object name
   - Material properties (color, texture)
   - Real-time property updates

### Stretch Goals

8. **Advanced Features**
   - Multi-selection (shift-click, box select)
   - Copy/paste objects (Ctrl+C/V)
   - Duplicate (Ctrl+D)
   - Group objects (create parent)
   - Asset browser (models, materials)

9. **Polish Features**
   - Prefab system (save/load object templates)
   - Snapping to surfaces (raycast to ground)
   - Alignment tools (align to grid, other objects)
   - Grid and axis visualization
   - Viewport shading modes (wireframe, lit, unlit)

## Architecture Guidance

### System Components

```
SceneEditor
├── EditorCore
│   ├── EditorState (mode, selected objects, etc.)
│   ├── ToolManager (active tool)
│   └── ViewportController
├── SelectionSystem
│   ├── ObjectPicker (raycast selection)
│   ├── SelectionSet (currently selected)
│   └── SelectionVisualizer (highlight)
├── GizmoSystem
│   ├── TranslateGizmo
│   ├── RotateGizmo
│   ├── ScaleGizmo
│   └── GizmoRenderer
├── CommandSystem
│   ├── Command (interface)
│   ├── CommandHistory (undo/redo stack)
│   └── Commands (TransformCommand, SpawnCommand, etc.)
├── SceneManager
│   ├── SceneGraph (hierarchy)
│   ├── EntityFactory
│   └── SceneSerializer
└── EditorUI
    ├── SceneOutliner
    ├── PropertyInspector
    ├── Toolbar
    └── MenuBar
```

### Data Structures

**Editor State**
```
EditorState:
  - active_tool: Tool (Move | Rotate | Scale | None)
  - selection: SelectionSet
  - camera: EditorCamera
  - grid_visible: bool
  - grid_size: float
  - snap_enabled: bool
  - transform_space: Local | World

SelectionSet:
  - selected_entities: set of EntityId
  - primary_selection: EntityId (last selected)

Methods:
  - add(entity_id)
  - remove(entity_id)
  - clear()
  - toggle(entity_id)
  - contains(entity_id) -> bool
```

**Command Interface**
```
trait Command {
  fn execute(&mut self, scene: &mut Scene)
  fn undo(&mut self, scene: &mut Scene)
  fn redo(&mut self, scene: &mut Scene) {
    self.execute(scene)  # Default: redo = execute
  }
  fn description(&self) -> String
  fn merge(&self, other: &Command) -> Option<Command> {
    None  # Optional: merge consecutive similar commands
  }
}

CommandHistory:
  - undo_stack: Vec<Box<dyn Command>>
  - redo_stack: Vec<Box<dyn Command>>
  - max_history: usize (e.g., 100)

Methods:
  - execute(command)
  - undo()
  - redo()
  - clear()
  - can_undo() -> bool
  - can_redo() -> bool
```

**Example Commands**
```
TransformCommand:
  - entity_id: EntityId
  - old_transform: Transform
  - new_transform: Transform

SpawnCommand:
  - entity_id: EntityId
  - entity_data: SerializedEntity
  - parent_id: Option<EntityId>

DeleteCommand:
  - entities: Vec<(EntityId, SerializedEntity, parent)>

SetPropertyCommand:
  - entity_id: EntityId
  - property_name: String
  - old_value: Value
  - new_value: Value
```

**Gizmo State**
```
Gizmo:
  - gizmo_type: Translate | Rotate | Scale
  - space: Local | World
  - selected_axis: None | X | Y | Z | XY | XZ | YZ | XYZ
  - is_dragging: bool
  - drag_start_pos: vec3
  - drag_current_pos: vec3

TranslateGizmo:
  - axis_length: float
  - arrow_size: float
  - plane_size: float
  
  Components:
  - X axis (red arrow)
  - Y axis (green arrow)
  - Z axis (blue arrow)
  - XY plane (small square)
  - XZ plane
  - YZ plane

Methods:
  - render(camera, selection_center)
  - hit_test(ray) -> Option<GizmoPart>
  - calculate_delta(mouse_ray) -> vec3
```

**Scene Serialization**
```
SerializedScene:
  - version: String
  - entities: Vec<SerializedEntity>

SerializedEntity:
  - id: Uuid
  - name: String
  - parent_id: Option<Uuid>
  - transform: Transform
  - components: Map<String, ComponentData>

ComponentData:
  - type_name: String
  - properties: Map<String, Value>

Example JSON:
{
  "version": "1.0",
  "entities": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "name": "Cube",
      "parent_id": null,
      "transform": {
        "position": [0, 1, 0],
        "rotation": [0, 0, 0, 1],
        "scale": [1, 1, 1]
      },
      "components": {
        "Mesh": {
          "type_name": "MeshRenderer",
          "mesh_path": "assets/cube.obj",
          "material": "red"
        }
      }
    }
  ]
}
```

### Selection via Raycasting

```
on_mouse_click(mouse_pos):
  ray = camera.screen_point_to_ray(mouse_pos)
  
  # Test all selectable objects
  closest_hit = None
  closest_distance = infinity
  
  for entity in scene.entities:
    if entity.has_component<Selectable>():
      bounding_box = entity.get_bounding_box()
      hit = ray_box_intersection(ray, bounding_box)
      
      if hit and hit.distance < closest_distance:
        closest_hit = entity
        closest_distance = hit.distance
  
  if closest_hit:
    if shift_pressed:
      selection.toggle(closest_hit)
    else:
      selection.clear()
      selection.add(closest_hit)
```

### Gizmo Manipulation

```
on_gizmo_drag_start(mouse_pos):
  ray = camera.screen_point_to_ray(mouse_pos)
  
  # Hit test gizmo parts
  hit_part = gizmo.hit_test(ray)
  
  if hit_part:
    gizmo.is_dragging = true
    gizmo.selected_axis = hit_part.axis
    gizmo.drag_start_pos = selection_center
    gizmo.drag_plane = create_drag_plane(hit_part.axis, camera)

on_gizmo_drag_update(mouse_pos):
  if not gizmo.is_dragging:
    return
  
  ray = camera.screen_point_to_ray(mouse_pos)
  hit_point = ray.intersect_plane(gizmo.drag_plane)
  
  delta = calculate_delta(hit_point, gizmo.drag_start_pos, gizmo.selected_axis)
  
  # Apply delta to selected objects
  for entity in selection:
    entity.transform.position += delta

on_gizmo_drag_end():
  # Create undo command
  command = TransformCommand {
    entities: selection.clone(),
    old_transforms: stored_original_transforms,
    new_transforms: current_transforms
  }
  command_history.execute(command)
  
  gizmo.is_dragging = false
```

### Command Pattern Example

```
# User modifies object position
original_pos = object.position
object.position = new_position

# Create command
command = TransformCommand {
  entity_id: object.id,
  old_transform: Transform { position: original_pos, ... },
  new_transform: Transform { position: new_position, ... }
}

# Execute (already done) and store
command_history.execute(command)

# Later: Undo
command_history.undo()
# This calls: command.undo(scene)
# Which sets: object.position = original_pos

# Redo
command_history.redo()
# This calls: command.execute(scene)
# Which sets: object.position = new_position
```

## Milestone Plan

### Milestone 1: Basic Editor Shell (Week 1, Days 1-3)

**Goal**: Editor window with camera and grid

**Tasks**:
- Create editor application (separate from game runtime)
- Implement orbit camera controls
- Render 3D grid on ground plane
- Render world axis indicators (X/Y/Z)
- Basic UI layout (toolbar, outliner placeholders)
- Empty scene with default lighting

**Deliverable**: Empty editor viewport with camera

### Milestone 2: Object Spawning and Selection (Week 1, Days 4-5)

**Goal**: Spawn and select objects

**Tasks**:
- Add spawn buttons (cube, sphere, plane)
- Implement object spawning at origin or cursor
- Implement click-to-select via raycasting
- Highlight selected object (outline or color change)
- Display selection in outliner
- Delete selected object (Del key)

**Deliverable**: Spawn, select, delete objects

### Milestone 3: Transform Gizmo (Week 1, Days 6-7)

**Goal**: Move objects with translate gizmo

**Tasks**:
- Render translate gizmo (3 colored arrows)
- Implement gizmo hit testing (ray-arrow intersection)
- Implement axis-constrained dragging
- Update object position while dragging
- Add plane handles (XY, XZ, YZ)
- Local vs. world space toggle

**Deliverable**: Interactive move gizmo

### Milestone 4: Undo/Redo System (Week 2, Days 1-3)

**Goal**: Undo transform changes

**Tasks**:
- Design Command interface
- Implement CommandHistory
- Create TransformCommand
- Capture transform before/after modification
- Execute command on gizmo drag end
- Bind Ctrl+Z (undo) and Ctrl+Y (redo)
- Display undo history in UI

**Deliverable**: Working undo/redo for transforms

### Milestone 5: Property Inspector (Week 2, Days 4-5)

**Goal**: Edit properties numerically

**Tasks**:
- Create property inspector UI panel
- Display selected object name
- Show position/rotation/scale fields
- Edit values with text input
- Apply changes on Enter or focus loss
- Create commands for property edits (undo support)

**Deliverable**: Editable property panel

### Milestone 6: Scene Serialization (Week 3, Days 1-3)

**Goal**: Save and load scenes

**Tasks**:
- Design scene file format (JSON recommended)
- Implement scene serialization (write entities to file)
- Implement scene deserialization (load from file)
- Add File menu (New, Open, Save, Save As)
- Handle file dialogs
- Clear scene before loading

**Deliverable**: Save/load functional scenes

### Milestone 7: Advanced Gizmos (Week 3-4, Days 4-7)

**Goal**: Rotate and scale gizmos

**Tasks**:
- Implement rotation gizmo (3 colored rings)
- Handle rotation dragging (quaternion math)
- Implement scale gizmo (3 colored cubes + center cube)
- Handle uniform vs. non-uniform scaling
- Tool switching (W/E/R hotkeys)
- Update undo system for all tools

**Deliverable**: Full transform gizmo suite

### Milestone 8: Polish and Advanced Features (Week 4-6, Days 1+)

**Goal**: Production-ready editor

**Tasks**:
- Implement multi-selection (Shift+click)
- Add copy/paste functionality
- Add duplicate (Ctrl+D)
- Implement scene outliner (tree view with hierarchy)
- Add parent-child relationships
- Grid snapping
- Focus on selected (F key)
- Viewport shading modes
- Performance optimization

**Deliverable**: Polished, usable editor

## Technical Challenges

### Challenge 1: Gizmo Screen-Space Size

**Problem**: Gizmo should stay same size regardless of distance

**Approach**:
- Scale gizmo based on distance to camera
- Use perspective division to maintain visual size
- Common formula: `size = distance * tan(fov / 2) * scale_factor`

**Implementation**:
```
render_gizmo(camera, position):
  distance = length(camera.position - position)
  
  # Maintain constant screen size
  scale = distance * 0.1  # Tunable constant
  
  # Render gizmo geometry with this scale
  render_arrow(position, vec3(1, 0, 0) * scale, RED)
  render_arrow(position, vec3(0, 1, 0) * scale, GREEN)
  render_arrow(position, vec3(0, 0, 1) * scale, BLUE)
```

### Challenge 2: Axis-Constrained Dragging

**Problem**: Dragging along a 3D axis with 2D mouse input

**Approach**:
- Create drag plane perpendicular to camera and containing axis
- Intersect mouse ray with drag plane
- Project intersection point onto axis
- Calculate delta along axis only

**Algorithm**:
```
calculate_axis_delta(mouse_ray, axis, start_pos, camera_forward):
  # Create plane for dragging
  plane_normal = normalize(cross(axis, camera_forward))
  if length(plane_normal) < 0.01:
    plane_normal = camera_up  # Fallback if axis parallel to view
  
  drag_plane = Plane(start_pos, plane_normal)
  
  # Intersect ray with plane
  hit_point = mouse_ray.intersect(drag_plane)
  
  # Project onto axis
  offset = hit_point - start_pos
  delta = dot(offset, axis) * axis
  
  return delta
```

### Challenge 3: Undo/Redo with Hierarchies

**Problem**: Moving parent should affect children, but undo should restore all

**Approach**:
- Store transforms for entire hierarchy in command
- On undo, restore all affected transforms
- Consider storing only root transform and recomputing children
- Handle deleted parents (unparent children or delete cascade)

**Hierarchical Transform Command**:
```
HierarchyTransformCommand:
  - root_entity: EntityId
  - old_transforms: Map<EntityId, Transform>
  - new_transforms: Map<EntityId, Transform>
  - affected_entities: Vec<EntityId>  # Root + descendants

execute():
  for entity in affected_entities:
    entity.transform = new_transforms[entity]

undo():
  for entity in affected_entities:
    entity.transform = old_transforms[entity]
```

### Challenge 4: Command Merging

**Problem**: Small drag movements create many undo steps

**Approach**:
- Merge consecutive similar commands (same entity, same type)
- Only keep final state in history
- Implement `Command::merge()` method
- Merge during drag (not on each pixel), create final command on mouse up

**Alternative Pattern**:
```
on_drag_start():
  start_transform = entity.transform

on_drag_update():
  # Update transform in-place, don't create command yet

on_drag_end():
  end_transform = entity.transform
  
  command = TransformCommand {
    old: start_transform,
    new: end_transform
  }
  
  command_history.execute(command)
```

### Challenge 5: Serialization Versioning

**Problem**: Scene format changes break old save files

**Approach**:
- Include version number in scene file
- Implement migration functions for old versions
- Validate schema before loading
- Provide clear error messages for unsupported versions

**Versioned Loading**:
```
load_scene(file_path):
  data = parse_json(file_path)
  version = data["version"]
  
  if version == "1.0":
    return load_scene_v1(data)
  elif version == "1.1":
    return load_scene_v1_1(data)
  elif version == "2.0":
    return load_scene_v2(data)
  else:
    error("Unsupported scene version: " + version)
```

## Reference Implementations

### Praxis Engine (Rust)
- **Files**: 
  - `examples/editor_demo.rs`
  - `examples/command_system_demo.rs`
  - `examples/selection_demo.rs`
- **Crates**: `praxis_editor`, `praxis_gui`
- **Concepts**: Selection, gizmos, undo/redo, serialization

### Other Engines/Frameworks

**Unity (C#)**
- Tutorial: "Editor Scripting" (official docs)
- System: Unity Editor architecture
- Key APIs: `EditorWindow`, `Editor`, `SerializedObject`, `Handles` (gizmos)

**Unreal Engine (C++)**
- Tutorial: "Building Editor Tools"
- System: Unreal Editor modes, widgets
- Key APIs: `FEdMode`, `FUICommandList`, Transactions (undo)

**Godot (GDScript/C++)**
- Tutorial: "Editor Plugins" (official docs)
- System: EditorPlugin API
- Key APIs: `EditorPlugin`, `EditorInterface`, `UndoRedo`

**Blender (Python)**
- Documentation: Blender Python API
- Operators, panels, properties
- Inspiration for mature editor UX

**imgui (C++/Rust)**
- Library: Dear ImGui for immediate-mode UI
- Example: Many editor implementations using imgui
- Pattern: Immediate-mode UI for tools

## Extension Ideas

### Beginner Extensions
- Object renaming
- Visibility toggle (show/hide objects)
- Lock transform (prevent edits)
- Scene thumbnails

### Intermediate Extensions
- Prefab system (save object as template)
- Asset browser (drag-drop models)
- Material editor (visual node-based)
- Play mode (test scene in editor)

### Advanced Extensions
- Collaborative editing (multiple users)
- Version control integration (Git)
- Custom component editors
- Visual scripting integration

## Success Criteria

Your scene editor should:

1. ✅ Spawn and manipulate 3D objects intuitively
2. ✅ Provide precise transform controls (gizmos + numeric input)
3. ✅ Support unlimited undo/redo without crashes
4. ✅ Save/load scenes without data loss
5. ✅ Handle complex hierarchies correctly
6. ✅ Run at 60 FPS with moderate scene complexity
7. ✅ Feel responsive and polished (no lag, clear feedback)

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Core Features** | Spawn, select, move, save | + Rotate, scale, undo/redo | + Multi-select, hierarchy, prefabs |
| **UI/UX** | Functional, basic UI | Intuitive, keyboard shortcuts | Polished, professional feel |
| **Robustness** | Works for simple scenes | Handles complex scenes, edge cases | Production-ready, error handling |
| **Performance** | 30 FPS, small scenes | 60 FPS, medium scenes | Optimized, large scenes |

## Common Pitfalls

1. **No Separation of Concerns**: Keep editor code separate from runtime
2. **Ignoring Command Pattern**: Essential for robust undo/redo
3. **Immediate Apply**: Batch changes, create command on action complete
4. **Mutable State**: Commands should store immutable snapshots
5. **Gizmo Z-Fighting**: Render gizmos on top (disable depth test or offset)
6. **Fixed Gizmo Size**: Always scale based on camera distance
7. **Serialization Fragility**: Version your format, handle errors gracefully

## Next Steps

After completing this project, you're ready for:
- **Project 05**: Procedural Terrain Generator (add terrain editing tools)
- **Project 07**: Multiplayer Arena (collaborative editing)
- **Project 10**: Mini Game Engine (integrate editor as primary workflow)
