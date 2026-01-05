# praxis_input Audit Report

**Audit Date:** January 2026
**Lines of Code:** ~400
**Test Coverage:** 21 tests (excellent coverage)

## Executive Summary

`praxis_input` provides a well-designed input abstraction layer with action mapping, similar to [Unity's Input System](https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/QuickStartGuide.html) and [Unreal's Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine). The implementation supports keyboard and mouse with rebindable controls, 3-state tracking (pressed/just-pressed/just-released), and bidirectional binding maps. **This is production-quality code** with comprehensive test coverage.

**Overall Assessment: EXCELLENT (9/10)**

---

## Features Inventory

### Feature 1: Action System (`action.rs`)

**Location:** `src/action.rs`
**Purpose:** Abstract physical inputs into logical game actions

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [x] Adequate test coverage (via InputMap tests)

#### Code Analysis

**`ActionId` struct:**
```rust
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ActionId(String);
```

**`Action` struct:**
```rust
pub struct Action {
    id: ActionId,
}
```

**Key Traits Implemented:**
- `Hash`, `PartialEq`, `Eq` - For HashMap keys
- `Debug`, `Display` - For logging
- `From<&str>`, `From<String>` - For ergonomic construction
- `Serialize`, `Deserialize` (feature-gated) - For persistence

#### Design Assessment
- **Pattern Used:** Value Object pattern (ActionId wraps String)
- **Industry Alignment:** **Matches** - Similar to Unity/Unreal action concepts
- **Modern Approach:** **Yes** - Type-safe action identifiers

#### Issues Found

1. **Action Creates New ActionId Each Call** (Severity: LOW)
   - **Location:** `src/input_map.rs:218` (usage pattern)
   - **Problem:** Creating `Action::new("jump")` twice creates two separate allocations
   - **Impact:** Minor performance overhead; users should cache Action instances
   - **Proposed Fix:** Document best practice, consider `Cow<'static, str>`:
     ```rust
     // Current (allocates)
     if input_map.is_action_pressed(&Action::new("jump"), &input_state) { ... }

     // Better (reuse)
     let jump = Action::new("jump");
     if input_map.is_action_pressed(&jump, &input_state) { ... }

     // Alternative: intern strings or use &'static str
     ```
   - **References:** String interning patterns

#### Positive Findings
- Clean separation between ActionId and Action
- Proper trait implementations
- Optional serialization support
- Ergonomic construction methods

---

### Feature 2: Input State Tracking (`input_state.rs`)

**Location:** `src/input_state.rs`
**Purpose:** Track current state of all input devices

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [x] Excellent test coverage (5 tests)

#### Code Analysis

**`InputState` resource:**
```rust
#[derive(Debug, Clone, Resource)]
pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    just_pressed_keys: HashSet<KeyCode>,
    just_released_keys: HashSet<KeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    just_pressed_mouse_buttons: HashSet<MouseButton>,
    just_released_mouse_buttons: HashSet<MouseButton>,
    mouse_position: (f64, f64),
    mouse_delta: (f64, f64),
    scroll_delta: (f32, f32),
}
```

**3-State Tracking Logic:**
- `pressed` - Currently held down
- `just_pressed` - Pressed this frame (cleared on update())
- `just_released` - Released this frame (cleared on update())

**Key Methods:**
- `update()` - Clear frame-based states
- `handle_keyboard_input()` - Process winit keyboard events
- `handle_mouse_button()` - Process mouse button events
- `handle_cursor_moved()` - Update position and delta
- `handle_mouse_wheel()` - Update scroll delta
- `press_key()` / `release_key()` - Programmatic control
- Various `is_*_pressed()` / `is_*_just_pressed()` queries

#### Design Assessment
- **Pattern Used:** State snapshot with frame clearing
- **Industry Alignment:** **Matches** - Standard input state pattern
- **Modern Approach:** **Yes** - ECS resource integration

#### Issues Found

1. **`just_pressed` Detection on Key Repeat** (Severity: LOW)
   - **Location:** `src/input_state.rs:131-133`
   - **Problem:** Uses `HashSet::insert()` return to detect new press, which correctly ignores key repeat
   - **Impact:** None - this is actually correct behavior
   - **Note:** Marking as positive finding

2. **Mouse Position Starts at (0, 0)** (Severity: LOW)
   - **Location:** `src/input_state.rs:103`
   - **Problem:** Initial mouse position is (0, 0), not actual cursor position
   - **Impact:** First delta calculation may be incorrect
   - **Proposed Fix:** Accept initial position or query cursor on creation:
     ```rust
     pub fn with_initial_mouse_position(position: (f64, f64)) -> Self {
         let mut state = Self::new();
         state.mouse_position = position;
         state
     }
     ```

3. **Scroll Delta Overwrites Instead of Accumulates** (Severity: MEDIUM)
   - **Location:** `src/input_state.rs:179-181`
   - **Problem:** `handle_mouse_wheel` overwrites, doesn't accumulate scroll events
   - **Impact:** If multiple scroll events arrive between updates, only last is recorded
   - **Proposed Fix:**
     ```rust
     // Before
     pub const fn handle_mouse_wheel(&mut self, delta: (f32, f32)) {
         self.scroll_delta = delta;
     }

     // After
     pub fn handle_mouse_wheel(&mut self, delta: (f32, f32)) {
         self.scroll_delta.0 += delta.0;
         self.scroll_delta.1 += delta.1;
     }
     ```

#### Positive Findings
- **Excellent 3-state tracking** - Industry standard approach
- **HashSet for O(1) lookups** - Efficient state queries
- **ECS Resource integration** - Derives `Resource`
- **Proper key repeat handling** - Ignores repeated presses
- **Programmatic control** - Useful for testing and simulation
- **Comprehensive iterators** - Access to all pressed/just-pressed sets

---

### Feature 3: Input Mapping (`input_map.rs`)

**Location:** `src/input_map.rs`
**Purpose:** Map physical inputs to logical actions

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [x] Excellent test coverage (4 tests)

#### Code Analysis

**`InputBinding` enum:**
```rust
pub enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
}
```

**`InputMap` resource:**
```rust
pub struct InputMap {
    action_bindings: HashMap<ActionId, HashSet<InputBinding>>,
    binding_actions: HashMap<InputBinding, HashSet<ActionId>>,
}
```

**Key Features:**
- **Bidirectional mapping** - Action→Bindings and Binding→Actions
- **Multiple bindings per action** - Jump = Space OR W
- **Multiple actions per binding** - Possible but rare use case
- **Full CRUD operations** - bind, unbind, unbind_all, clear

#### Design Assessment
- **Pattern Used:** Bidirectional map with N:M relationship
- **Industry Alignment:** **Matches** - Similar to Unity/Unreal action maps
- **Modern Approach:** **Yes** - Flexible rebinding support

#### Issues Found

1. **No Input Context/Mapping Context Support** (Severity: MEDIUM)
   - **Location:** `src/input_map.rs`
   - **Problem:** No concept of switching input contexts (e.g., gameplay vs menu vs vehicle)
   - **Impact:** Harder to implement different control schemes for different game states
   - **Proposed Fix:** Add InputContext concept:
     ```rust
     pub struct InputContext {
         name: String,
         priority: i32,
         bindings: InputMap,
     }

     pub struct InputContextStack {
         contexts: Vec<InputContext>,
     }

     impl InputContextStack {
         pub fn push_context(&mut self, context: InputContext) { /* ... */ }
         pub fn pop_context(&mut self) -> Option<InputContext> { /* ... */ }
         pub fn is_action_pressed(&self, action: &Action, state: &InputState) -> bool {
             // Check contexts in priority order
         }
     }
     ```
   - **References:** [Unreal's Mapping Contexts](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine)

2. **No Axis/Value Input Support** (Severity: MEDIUM)
   - **Location:** `src/input_map.rs`
   - **Problem:** Only boolean (pressed/released), no analog/axis support
   - **Impact:** Can't easily implement gamepad sticks, smooth movement
   - **Proposed Fix:** Add value-based actions:
     ```rust
     pub enum InputValue {
         Digital(bool),
         Axis1D(f32),        // -1.0 to 1.0
         Axis2D(f32, f32),   // Stick
     }

     // WASD as axis:
     input_map.bind_axis(&move_action, KeyCode::KeyW, InputValue::Axis1D(1.0));
     input_map.bind_axis(&move_action, KeyCode::KeyS, InputValue::Axis1D(-1.0));
     ```

3. **No Dead Zone / Input Processing** (Severity: LOW)
   - **Location:** `src/input_map.rs`
   - **Problem:** No input modifiers (dead zones, scaling, inversion)
   - **Impact:** Required for proper gamepad support when added
   - **Note:** Acceptable to defer until gamepad implementation

#### Positive Findings
- **Bidirectional maps** - Efficient lookup in both directions
- **Multiple bindings** - Industry standard rebinding support
- **Clean unbind API** - unbind, unbind_all operations
- **ECS Resource** - Integrates with bevy_ecs
- **Optional serialization** - Save/load bindings

---

### Feature 4: Gamepad Support

**Location:** `src/lib.rs` (documentation mentions gamepad)
**Status:** NOT IMPLEMENTED

#### Implementation Status
- [ ] Not implemented (documented as future)

#### Analysis

The crate documentation mentions gamepad support:
> "This crate provides functionality for handling keyboard, mouse, and gamepad input"

However, there's no actual gamepad implementation.

#### Issues Found

1. **Gamepad Documented But Not Implemented** (Severity: MEDIUM)
   - **Location:** `src/lib.rs:3`
   - **Problem:** Documentation claims gamepad support that doesn't exist
   - **Impact:** Misleading documentation
   - **Proposed Fix:** Either implement or clarify docs:
     ```rust
     // Update documentation
     //! This crate provides functionality for handling keyboard and mouse input,
     //! with an action mapping system for rebindable controls.
     //!
     //! **Note:** Gamepad support is planned for a future release.
     ```

---

## Research Context

### Industry Standards Consulted
- [Unity Input System](https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/QuickStartGuide.html)
- [Unreal Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine)
- [Build a Game Engine - Input Design](https://buildagameengine.com/input/input-design-and-devices)
- Bevy Input resources

### Modern Best Practices (2024-2025)

| Practice | Praxis Status | Industry Standard |
|----------|---------------|-------------------|
| Action abstraction | **Matches** | Unity/Unreal both use this |
| Rebindable controls | **Matches** | Essential for accessibility |
| 3-state tracking | **Matches** | Standard pattern |
| Input contexts | **Missing** | Unreal has Mapping Contexts |
| Axis/analog input | **Missing** | Required for gamepads |
| Input modifiers | **Missing** | Dead zones, scaling |
| Gamepad support | **Missing** | Expected in 2024+ |

### Deprecated Approaches Avoided
- Not using polling-only input (uses event-driven)
- Not hardcoding key bindings
- Not using strings for key comparisons (uses KeyCode enum)

---

## Recommendations Summary

### Critical (Must Fix)
*None*

### High Priority
*None*

### Medium Priority
1. Fix scroll delta accumulation (overwrites vs accumulates)
2. Add Input Context / Mapping Context support
3. Add axis/analog input support for future gamepad
4. Correct documentation about gamepad support

### Low Priority / Nice to Have
1. Consider string interning for ActionId
2. Accept initial mouse position
3. Add input modifiers (dead zones, scaling)
4. Implement gamepad via gilrs crate

### Positive Highlights
- **Excellent test coverage** - 21 tests covering all features
- **Clean API design** - Similar to industry leaders
- **3-state tracking** - Professional input handling
- **Bidirectional maps** - Efficient both directions
- **ECS integration** - Resource-based design
- **Serialization support** - Save/load bindings
- **Programmatic control** - Great for testing

---

## Final Rating

| Category | Score | Notes |
|----------|-------|-------|
| Implementation Completeness | 8/10 | Missing gamepad, contexts |
| Logic Correctness | 9/10 | Minor scroll accumulation issue |
| Design Quality | 9/10 | Excellent architecture |
| Modernness | 8/10 | Missing modern features |
| Performance | 10/10 | HashSet O(1) lookups |
| **Overall** | **9/10** | Excellent |

---

*Report generated: January 2026*
