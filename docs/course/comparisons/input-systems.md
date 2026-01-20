# Input Systems: Multi-Engine Comparison

**Complexity**: Beginner  
**Curriculum Module**: [Module 8 - Input Abstraction](../modules/08-input-abstraction.md)

## Problem Statement

Game engines need flexible input handling across multiple device types. Key challenges:

- How do we abstract different input devices (keyboard, mouse, gamepad, touch)?
- How do we map raw inputs to high-level game actions?
- How do we support input rebinding for players?
- How do we handle device connection/disconnection?
- How do we manage input state across frames (pressed vs. held vs. released)?

## Design Philosophy Comparison

| Engine | Input Model | Action Mapping | Rebinding Support |
|--------|-------------|----------------|-------------------|
| **Unity** | Old Input + New Input System | Action/binding system (new) | Built-in UI (new) |
| **Unreal** | Enhanced Input System (UE5) | Input Mapping Contexts | Blueprint-friendly |
| **Godot** | Input Actions (centralized) | Project-level action map | Built-in InputMap |
| **Praxis** | Direct state polling | Manual action mapping | DIY implementation |

## Implementation Examples

### Reading Raw Input

#### Unity (C# - Old Input System)

```csharp
using UnityEngine;

public class OldInputExample : MonoBehaviour
{
    void Update()
    {
        // Keyboard
        if (Input.GetKeyDown(KeyCode.Space))  // Just pressed this frame
        {
            Debug.Log("Space pressed");
        }
        
        if (Input.GetKey(KeyCode.W))  // Held down
        {
            Debug.Log("W held");
        }
        
        if (Input.GetKeyUp(KeyCode.Space))  // Just released this frame
        {
            Debug.Log("Space released");
        }
        
        // Mouse
        if (Input.GetMouseButtonDown(0))  // Left click
        {
            Vector3 mousePos = Input.mousePosition;
            Debug.Log("Clicked at " + mousePos);
        }
        
        float mouseX = Input.GetAxis("Mouse X");  // Mouse delta
        float mouseY = Input.GetAxis("Mouse Y");
        
        // Gamepad
        float horizontal = Input.GetAxis("Horizontal");  // -1 to 1
        float vertical = Input.GetAxis("Vertical");
        
        if (Input.GetButtonDown("Jump"))  // Virtual button (configured in Input Manager)
        {
            Debug.Log("Jump button pressed");
        }
    }
}
```

#### Unity (C# - New Input System)

```csharp
using UnityEngine;
using UnityEngine.InputSystem;

public class NewInputExample : MonoBehaviour
{
    private PlayerInput playerInput;
    private InputAction moveAction;
    private InputAction jumpAction;
    
    void Awake()
    {
        playerInput = GetComponent<PlayerInput>();
        
        // Get actions from Input Action Asset
        moveAction = playerInput.actions["Move"];
        jumpAction = playerInput.actions["Jump"];
        
        // Subscribe to events
        jumpAction.performed += OnJump;
        jumpAction.canceled += OnJumpReleased;
    }
    
    void Update()
    {
        // Read action value
        Vector2 moveInput = moveAction.ReadValue<Vector2>();
        
        // Check if button is pressed
        bool isJumpPressed = jumpAction.IsPressed();
    }
    
    void OnJump(InputAction.CallbackContext context)
    {
        Debug.Log("Jump performed!");
    }
    
    void OnJumpReleased(InputAction.CallbackContext context)
    {
        Debug.Log("Jump released!");
    }
    
    void OnDestroy()
    {
        jumpAction.performed -= OnJump;
        jumpAction.canceled -= OnJumpReleased;
    }
}

// Input Action Asset (JSON configuration)
/*
{
  "name": "PlayerControls",
  "maps": [
    {
      "name": "Gameplay",
      "actions": [
        {
          "name": "Move",
          "type": "Value",
          "bindings": [
            { "path": "<Keyboard>/w", "processors": "up" },
            { "path": "<Keyboard>/a", "processors": "left" },
            { "path": "<Gamepad>/leftStick" }
          ]
        },
        {
          "name": "Jump",
          "type": "Button",
          "bindings": [
            { "path": "<Keyboard>/space" },
            { "path": "<Gamepad>/buttonSouth" }
          ]
        }
      ]
    }
  ]
}
*/
```

#### Unreal (C++)

```cpp
#include "InputAction.h"
#include "InputMappingContext.h"
#include "EnhancedInputComponent.h"
#include "EnhancedInputSubsystems.h"

class AMyCharacter : public ACharacter
{
public:
    // Input Actions (defined in editor)
    UPROPERTY(EditAnywhere, BlueprintReadOnly, Category = "Input")
    UInputAction* MoveAction;
    
    UPROPERTY(EditAnywhere, BlueprintReadOnly, Category = "Input")
    UInputAction* JumpAction;
    
    UPROPERTY(EditAnywhere, BlueprintReadOnly, Category = "Input")
    UInputMappingContext* DefaultMappingContext;
    
    void BeginPlay() override
    {
        Super::BeginPlay();
        
        // Add Input Mapping Context
        if (APlayerController* PC = Cast<APlayerController>(GetController()))
        {
            if (UEnhancedInputLocalPlayerSubsystem* Subsystem = 
                ULocalPlayer::GetSubsystem<UEnhancedInputLocalPlayerSubsystem>(PC->GetLocalPlayer()))
            {
                Subsystem->AddMappingContext(DefaultMappingContext, 0);
            }
        }
    }
    
    void SetupPlayerInputComponent(UInputComponent* PlayerInputComponent) override
    {
        Super::SetupPlayerInputComponent(PlayerInputComponent);
        
        UEnhancedInputComponent* EnhancedInput = Cast<UEnhancedInputComponent>(PlayerInputComponent);
        
        // Bind actions
        EnhancedInput->BindAction(MoveAction, ETriggerEvent::Triggered, this, &AMyCharacter::Move);
        EnhancedInput->BindAction(JumpAction, ETriggerEvent::Started, this, &AMyCharacter::Jump);
        EnhancedInput->BindAction(JumpAction, ETriggerEvent::Completed, this, &AMyCharacter::StopJumping);
    }
    
    void Move(const FInputActionValue& Value)
    {
        FVector2D MovementVector = Value.Get<FVector2D>();
        
        // Add movement input
        AddMovementInput(GetActorForwardVector(), MovementVector.Y);
        AddMovementInput(GetActorRightVector(), MovementVector.X);
    }
    
    void Jump()
    {
        ACharacter::Jump();
    }
    
    void StopJumping()
    {
        ACharacter::StopJumping();
    }
};

// Input Mapping Context configuration (created in editor):
// - Move: Keyboard (WASD), Gamepad (Left Stick)
// - Jump: Keyboard (Space), Gamepad (South Button)
```

#### Godot (GDScript)

```gdscript
extends Node

func _ready():
    # Input actions defined in Project Settings > Input Map
    # Example: "move_forward" = W, Up Arrow, Gamepad D-Pad Up
    pass

func _process(delta):
    # Check if action is pressed
    if Input.is_action_pressed("move_forward"):
        print("Moving forward")
    
    # Just pressed this frame
    if Input.is_action_just_pressed("jump"):
        print("Jump!")
    
    # Just released this frame
    if Input.is_action_just_released("jump"):
        print("Jump released")
    
    # Get axis value (-1 to 1)
    var move_x = Input.get_axis("move_left", "move_right")
    var move_y = Input.get_axis("move_forward", "move_back")
    
    # Get action strength (0 to 1, useful for analog inputs)
    var jump_strength = Input.get_action_strength("jump")
    
    # Mouse
    var mouse_pos = get_viewport().get_mouse_position()
    
    # Raw keyboard check
    if Input.is_key_pressed(KEY_SPACE):
        print("Space key held")

func _input(event):
    # Event-based input (alternative to polling)
    if event is InputEventKey:
        if event.pressed and event.keycode == KEY_ESCAPE:
            print("Escape pressed")
    
    if event is InputEventMouseButton:
        if event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
            print("Left click at", event.position)
    
    if event is InputEventMouseMotion:
        print("Mouse moved:", event.relative)
```

#### Praxis (Rust)

```rust
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use praxis_input::InputState;

// Input state structure
pub struct InputState {
    keys_pressed: HashSet<VirtualKeyCode>,
    keys_just_pressed: HashSet<VirtualKeyCode>,
    keys_just_released: HashSet<VirtualKeyCode>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
}

impl InputState {
    pub fn update(&mut self) {
        // Clear frame-based state
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_delta = Vec2::ZERO;
    }
    
    pub fn handle_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => {
                    if !self.keys_pressed.contains(&keycode) {
                        self.keys_just_pressed.insert(keycode);
                    }
                    self.keys_pressed.insert(keycode);
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&keycode);
                    self.keys_just_released.insert(keycode);
                }
            }
        }
    }
    
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }
    
    pub fn is_key_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }
    
    pub fn is_key_just_released(&self, key: VirtualKeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }
}

// Usage in game loop
fn handle_input(input: &InputState) {
    // Check key states
    if input.is_key_just_pressed(VirtualKeyCode::Space) {
        println!("Jump!");
    }
    
    if input.is_key_pressed(VirtualKeyCode::W) {
        println!("Moving forward");
    }
    
    // Mouse
    let mouse_pos = input.mouse_position();
    let mouse_delta = input.mouse_delta();
}
```

### Action Mapping System

#### Unity (New Input System)

```csharp
// Action mapping defined in Input Action Asset
// Accessed at runtime:

public class ActionMappingExample : MonoBehaviour
{
    private InputActionAsset inputActions;
    
    void Awake()
    {
        inputActions = GetComponent<PlayerInput>().actions;
        
        // Rebind at runtime
        InputAction jumpAction = inputActions.FindAction("Jump");
        jumpAction.ApplyBindingOverride("<Keyboard>/return");  // Rebind to Enter key
        
        // Composite bindings (e.g., WASD as 2D vector)
        InputAction moveAction = inputActions.FindAction("Move");
        // Automatically handles WASD, Arrow Keys, Gamepad Stick
    }
    
    // Save/load rebindings
    void SaveBindings()
    {
        string rebinds = inputActions.SaveBindingOverridesAsJson();
        PlayerPrefs.SetString("InputRebinds", rebinds);
    }
    
    void LoadBindings()
    {
        string rebinds = PlayerPrefs.GetString("InputRebinds");
        inputActions.LoadBindingOverridesFromJson(rebinds);
    }
}
```

#### Unreal (Enhanced Input)

```cpp
// Input Mapping Context (IMC) defined in editor
// Can switch contexts at runtime

void AMyCharacter::SwitchToVehicleControls()
{
    if (APlayerController* PC = Cast<APlayerController>(GetController()))
    {
        if (UEnhancedInputLocalPlayerSubsystem* Subsystem = 
            ULocalPlayer::GetSubsystem<UEnhancedInputLocalPlayerSubsystem>(PC->GetLocalPlayer()))
        {
            // Remove default context
            Subsystem->RemoveMappingContext(DefaultMappingContext);
            
            // Add vehicle context
            Subsystem->AddMappingContext(VehicleMappingContext, 0);
        }
    }
}

// Input Modifiers and Triggers
// - Modifiers: Negate, Scale, Smooth, Dead Zone
// - Triggers: Pressed, Released, Hold, Tap, Double Tap
```

#### Godot (Input Map)

```gdscript
# Action map defined in Project Settings > Input Map
# Example:
# - "jump": Space, Gamepad Button A
# - "move_forward": W, Up Arrow, Gamepad D-Pad Up

# Runtime rebinding
func rebind_action(action_name: String, new_key: int):
    # Remove existing bindings
    InputMap.action_erase_events(action_name)
    
    # Add new binding
    var event = InputEventKey.new()
    event.keycode = new_key
    InputMap.action_add_event(action_name, event)

# Save/load
func save_input_map():
    var config = ConfigFile.new()
    for action in InputMap.get_actions():
        var events = InputMap.action_get_events(action)
        config.set_value("input", action, events)
    config.save("user://input_map.cfg")

func load_input_map():
    var config = ConfigFile.new()
    config.load("user://input_map.cfg")
    for action in config.get_section_keys("input"):
        var events = config.get_value("input", action)
        InputMap.action_erase_events(action)
        for event in events:
            InputMap.action_add_event(action, event)
```

#### Praxis (Manual Action Mapping)

```rust
use std::collections::HashMap;

// Action mapping system
pub struct InputActionMap {
    actions: HashMap<String, Vec<InputBinding>>,
}

#[derive(Clone)]
pub enum InputBinding {
    Key(VirtualKeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
}

impl InputActionMap {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }
    
    pub fn add_action(&mut self, action: &str, binding: InputBinding) {
        self.actions.entry(action.to_string())
            .or_insert_with(Vec::new)
            .push(binding);
    }
    
    pub fn is_action_pressed(&self, action: &str, input: &InputState) -> bool {
        if let Some(bindings) = self.actions.get(action) {
            for binding in bindings {
                match binding {
                    InputBinding::Key(key) => {
                        if input.is_key_pressed(*key) {
                            return true;
                        }
                    }
                    InputBinding::MouseButton(btn) => {
                        if input.is_mouse_button_pressed(*btn) {
                            return true;
                        }
                    }
                    // ... other binding types
                }
            }
        }
        false
    }
    
    pub fn rebind(&mut self, action: &str, bindings: Vec<InputBinding>) {
        self.actions.insert(action.to_string(), bindings);
    }
}

// Usage
fn setup_input_actions() -> InputActionMap {
    let mut actions = InputActionMap::new();
    
    // Jump action
    actions.add_action("jump", InputBinding::Key(VirtualKeyCode::Space));
    actions.add_action("jump", InputBinding::GamepadButton(GamepadButton::South));
    
    // Move forward
    actions.add_action("move_forward", InputBinding::Key(VirtualKeyCode::W));
    actions.add_action("move_forward", InputBinding::Key(VirtualKeyCode::Up));
    
    actions
}

fn handle_gameplay_input(input: &InputState, actions: &InputActionMap) {
    if actions.is_action_pressed("jump", input) {
        // Jump logic
    }
    
    if actions.is_action_pressed("move_forward", input) {
        // Move forward logic
    }
}
```

## Gamepad Support

### Unity

```csharp
using UnityEngine.InputSystem;

public class GamepadExample : MonoBehaviour
{
    void Update()
    {
        // Check if gamepad is connected
        if (Gamepad.current != null)
        {
            // Read buttons
            if (Gamepad.current.buttonSouth.wasPressedThisFrame)
            {
                Debug.Log("A button pressed");
            }
            
            // Read sticks
            Vector2 leftStick = Gamepad.current.leftStick.ReadValue();
            Vector2 rightStick = Gamepad.current.rightStick.ReadValue();
            
            // Read triggers (0 to 1)
            float leftTrigger = Gamepad.current.leftTrigger.ReadValue();
            float rightTrigger = Gamepad.current.rightTrigger.ReadValue();
            
            // Rumble
            Gamepad.current.SetMotorSpeeds(0.5f, 0.5f);  // Low and high frequency motors
        }
    }
}
```

### Unreal

```cpp
// Gamepad input automatically mapped through Enhanced Input
// Vibration:
void AMyCharacter::TriggerRumble()
{
    if (APlayerController* PC = Cast<APlayerController>(GetController()))
    {
        PC->PlayDynamicForceFeedback(
            0.5f,  // Intensity
            0.2f,  // Duration
            true,  // Large motor
            true,  // Small motor
            true,  // Left trigger
            true   // Right trigger
        );
    }
}
```

### Godot

```gdscript
func _process(delta):
    # Check connected joysticks
    var joysticks = Input.get_connected_joypads()
    
    if joysticks.size() > 0:
        var joy_id = joysticks[0]
        
        # Read axis
        var left_stick_x = Input.get_joy_axis(joy_id, JOY_AXIS_LEFT_X)
        var left_stick_y = Input.get_joy_axis(joy_id, JOY_AXIS_LEFT_Y)
        
        # Read button
        if Input.is_joy_button_pressed(joy_id, JOY_BUTTON_A):
            print("A button pressed")
        
        # Rumble
        Input.start_joy_vibration(joy_id, 0.5, 0.5, 0.2)  # weak, strong, duration
```

### Praxis

```rust
use gilrs::{Gilrs, Button, Axis};

pub struct GamepadState {
    gilrs: Gilrs,
}

impl GamepadState {
    pub fn new() -> Self {
        Self {
            gilrs: Gilrs::new().unwrap(),
        }
    }
    
    pub fn update(&mut self) {
        while let Some(event) = self.gilrs.next_event() {
            // Handle gamepad events
        }
    }
    
    pub fn is_button_pressed(&self, button: Button) -> bool {
        if let Some(gamepad) = self.gilrs.gamepads().next() {
            gamepad.1.is_pressed(button)
        } else {
            false
        }
    }
    
    pub fn axis_value(&self, axis: Axis) -> f32 {
        if let Some(gamepad) = self.gilrs.gamepads().next() {
            gamepad.1.value(axis)
        } else {
            0.0
        }
    }
}
```

## Trade-Off Analysis

### Unity (New Input System)

**Pros**:
- Powerful action mapping with rebinding UI
- Cross-platform device abstraction
- Composite bindings (WASD as Vector2)
- Input debugging tools
- Touch input support

**Cons**:
- Learning curve (new system is complex)
- Old Input System still in use (fragmentation)
- Package must be installed separately
- More overhead than old system

### Unreal (Enhanced Input)

**Pros**:
- Context switching (vehicle vs. on-foot)
- Modifiers and triggers (hold, double-tap, etc.)
- Blueprint-friendly
- Excellent gamepad support
- Priority system for multiple contexts

**Cons**:
- UE5 only (older projects use old system)
- Complex for simple use cases
- Editor-heavy workflow

### Godot (Input Actions)

**Pros**:
- Simple centralized input map
- Easy rebinding at runtime
- Built into engine (no packages)
- Good documentation
- Supports all device types

**Cons**:
- Less sophisticated than Unity/Unreal
- String-based action names (typo-prone)
- No modifier system (dead zones manual)
- Limited composite bindings

### Praxis (Manual)

**Pros**:
- Full control over implementation
- Zero abstraction overhead
- Can optimize for specific use case
- No hidden behavior

**Cons**:
- Must implement everything
- No built-in rebinding UI
- Manual device abstraction
- More boilerplate code

## Key Takeaways

### Universal Principles

1. **Action Mapping > Raw Input**: Map "Jump" not "Space" for rebindability
2. **Frame-Based State**: Track pressed/held/released separately
3. **Device Abstraction**: Support keyboard, gamepad, touch with same actions
4. **Context Switching**: Different control schemes for different game modes
5. **Deadzone Handling**: Analog sticks need deadzone thresholds

### Design Patterns to Steal

- **Input Action Asset**: External configuration (Unity, Unreal)
- **Binding Overrides**: Runtime rebinding without changing source
- **Event vs. Polling**: Both patterns have uses (events for UI, polling for gameplay)
- **Input Buffering**: Store inputs for next frame (fighting games)
- **Player-Specific Input**: Multiplayer needs per-player input contexts

### Common Pitfalls

- **Reading Input in Wrong Update**: Use FixedUpdate for physics, Update for camera
- **No Deadzone**: Analog sticks drift without deadzone
- **Forgetting to Clear Frame State**: Just pressed/released must reset each frame
- **Hardcoded Keys**: Always use action mapping for player-facing controls
- **No Controller Disconnection Handling**: Pause game or show warning

## Further Reading

### Unity
- [New Input System](https://docs.unity3d.com/Packages/com.unity.inputsystem@latest)
- [Input System Workflows](https://docs.unity3d.com/Packages/com.unity.inputsystem@latest/manual/Workflows.html)

### Unreal
- [Enhanced Input](https://docs.unrealengine.com/5.0/en-US/enhanced-input-in-unreal-engine/)
- [Input Mapping Context](https://docs.unrealengine.com/5.0/en-US/API/Plugins/EnhancedInput/UInputMappingContext/)

### Godot
- [Inputs](https://docs.godotengine.org/en/stable/tutorials/inputs/index.html)
- [InputMap](https://docs.godotengine.org/en/stable/classes/class_inputmap.html)

### Praxis
- [Praxis Input](../../../crates/praxis_input/README.md)
- [winit Documentation](https://docs.rs/winit/)

### General
- [Game Input Programming Patterns](http://gameprogrammingpatterns.com/command.html)
