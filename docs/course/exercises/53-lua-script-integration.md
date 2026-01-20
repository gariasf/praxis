# Exercise 53: Lua Script Integration

**Difficulty**: 🔴 Advanced | **Estimated Time**: 5-6h | **Subsystem**: Scripting

## Overview

Integrate Lua scripting into the game engine, allowing game logic to be written in scripts rather than compiled code. Essential for rapid iteration and modding support.

## Learning Objectives

- Understand scripting engine integration
- Learn Rust-Lua FFI boundaries
- Implement script lifecycle management
- Handle script errors safely

## Requirements

### Functional Requirements

1. **Lua Context**
   - Initialize Lua VM
   - Load and execute scripts
   - Access global variables and functions

2. **Engine API Exposure**
   - Expose entity creation/destruction
   - Provide component access
   - Math utilities (Vec3, Quat, etc.)
   - Input query functions

3. **Script Lifecycle**
   - `init()`: Called once on load
   - `update(delta_time)`: Called every frame
   - `on_event(event)`: Handle game events

4. **Error Handling**
   - Catch and report Lua errors
   - Don't crash engine on script error
   - Provide stack traces

### Non-Functional Requirements

- **Performance**: Script overhead < 1ms per frame
- **Safety**: Scripts can't crash engine or access unsafe memory
- **Debugging**: Clear error messages with line numbers

## API Design

```rust
pub struct ScriptingContext {
    lua: Lua,
    loaded_scripts: HashMap<String, ScriptHandle>,
}

impl ScriptingContext {
    pub fn new() -> Result<Self>;
    
    pub fn load_script(&mut self, name: &str, path: &Path) -> Result<ScriptHandle>;
    pub fn unload_script(&mut self, handle: ScriptHandle);
    
    pub fn call_function(&self, handle: ScriptHandle, func: &str, args: &[Value]) 
        -> Result<Vec<Value>>;
    
    pub fn set_global(&self, name: &str, value: Value) -> Result<()>;
    pub fn get_global(&self, name: &str) -> Result<Value>;
    
    pub fn update_scripts(&mut self, delta_time: f32, world: &mut World);
}

// Example Lua script API
/*
function init()
    print("Script initialized")
end

function update(dt)
    local pos = entity_get_position(player_id)
    entity_set_position(player_id, pos.x + 1.0, pos.y, pos.z)
end

function on_event(event_type, event_data)
    if event_type == "collision" then
        print("Collision detected!")
    end
end
*/
```

## Validation Criteria

### Correctness
- [ ] Scripts load and execute successfully
- [ ] Lua can call engine functions
- [ ] Engine can call Lua functions
- [ ] Errors caught and reported properly
- [ ] Memory doesn't leak across Lua/Rust boundary

### Performance
- [ ] 100 simple scripts update in < 1ms
- [ ] FFI calls < 1µs overhead
- [ ] No GC pauses > 1ms

## Test Cases

```rust
#[test]
fn test_basic_script_execution() {
    let mut ctx = ScriptingContext::new().unwrap();
    
    let script = r#"
        function test()
            return 42
        end
    "#;
    
    let handle = ctx.load_script_from_string("test", script).unwrap();
    let result = ctx.call_function(handle, "test", &[]).unwrap();
    
    assert_eq!(result[0].as_i32().unwrap(), 42);
}

#[test]
fn test_engine_api_call() {
    let mut ctx = ScriptingContext::new().unwrap();
    ctx.register_engine_api();
    
    let script = r#"
        function test()
            local v = vec3_new(1.0, 2.0, 3.0)
            return vec3_length(v)
        end
    "#;
    
    let handle = ctx.load_script_from_string("test", script).unwrap();
    let result = ctx.call_function(handle, "test", &[]).unwrap();
    
    let length = result[0].as_f32().unwrap();
    assert!((length - 3.741).abs() < 0.01);
}

#[test]
fn test_error_handling() {
    let mut ctx = ScriptingContext::new().unwrap();
    
    let script = r#"
        function test()
            error("Test error")
        end
    "#;
    
    let handle = ctx.load_script_from_string("test", script).unwrap();
    let result = ctx.call_function(handle, "test", &[]);
    
    assert!(result.is_err());
}

#[test]
fn test_script_update_loop() {
    let mut ctx = ScriptingContext::new().unwrap();
    let mut world = World::new();
    
    let script = r#"
        counter = 0
        function update(dt)
            counter = counter + 1
        end
    "#;
    
    let handle = ctx.load_script_from_string("test", script).unwrap();
    
    ctx.update_scripts(0.016, &mut world);
    
    let counter = ctx.get_global("counter").unwrap().as_i32().unwrap();
    assert_eq!(counter, 1);
}
```

## Performance Targets

| Operation | Target |
|-----------|--------|
| Script load | < 10ms |
| Function call overhead | < 1µs |
| 100 script updates | < 1ms |
| FFI data conversion | < 100ns |

## Hints & Guidance

### Using mlua
```rust
use mlua::prelude::*;

let lua = Lua::new();

// Define Rust function callable from Lua
let print_fn = lua.create_function(|_, msg: String| {
    println!("{}", msg);
    Ok(())
})?;

lua.globals().set("rust_print", print_fn)?;

// Execute Lua code
lua.load("rust_print('Hello from Lua!')").exec()?;
```

### Exposing Engine API
Create a module with engine functions:
```rust
fn create_engine_module(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;
    
    // Vector math
    let vec3_new = lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
        Ok(Vec3::new(x, y, z))
    })?;
    module.set("vec3_new", vec3_new)?;
    
    // More functions...
    
    Ok(module)
}
```

### Error Handling Pattern
```rust
match lua.load(script).exec() {
    Ok(()) => { /* Success */ }
    Err(LuaError::RuntimeError(msg)) => {
        eprintln!("Script error: {}", msg);
        // Continue execution
    }
    Err(e) => {
        eprintln!("Lua error: {}", e);
    }
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use mlua::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use glam::{Vec3, Quat};

pub type ScriptHandle = usize;

pub struct ScriptingContext {
    lua: Lua,
    scripts: HashMap<ScriptHandle, String>,
    next_handle: ScriptHandle,
}

impl ScriptingContext {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();
        
        // Register engine API
        Self::register_engine_api(&lua)?;
        
        Ok(Self {
            lua,
            scripts: HashMap::new(),
            next_handle: 1,
        })
    }
    
    fn register_engine_api(lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();
        
        // Math functions
        let vec3_new = lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            Ok((x, y, z))
        })?;
        globals.set("vec3_new", vec3_new)?;
        
        let vec3_length = lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            let v = Vec3::new(x, y, z);
            Ok(v.length())
        })?;
        globals.set("vec3_length", vec3_length)?;
        
        let vec3_normalize = lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            let v = Vec3::new(x, y, z).normalize();
            Ok((v.x, v.y, v.z))
        })?;
        globals.set("vec3_normalize", vec3_normalize)?;
        
        // Logging
        let log = lua.create_function(|_, msg: String| {
            println!("[Lua] {}", msg);
            Ok(())
        })?;
        globals.set("log", log)?;
        
        Ok(())
    }
    
    pub fn load_script_from_file(&mut self, name: &str, path: &Path) -> LuaResult<ScriptHandle> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to read file: {}", e)))?;
        
        self.load_script_from_string(name, &content)
    }
    
    pub fn load_script_from_string(&mut self, name: &str, content: &str) -> LuaResult<ScriptHandle> {
        // Execute the script to define functions
        self.lua.load(content).set_name(name)?.exec()?;
        
        // Call init if it exists
        if let Ok(init) = self.lua.globals().get::<_, LuaFunction>("init") {
            init.call::<_, ()>(())?;
        }
        
        let handle = self.next_handle;
        self.next_handle += 1;
        self.scripts.insert(handle, name.to_string());
        
        Ok(handle)
    }
    
    pub fn unload_script(&mut self, handle: ScriptHandle) {
        self.scripts.remove(&handle);
    }
    
    pub fn call_function(&self, handle: ScriptHandle, func_name: &str, args: &[LuaValue]) 
        -> LuaResult<Vec<LuaValue>> 
    {
        if !self.scripts.contains_key(&handle) {
            return Err(LuaError::RuntimeError("Invalid script handle".to_string()));
        }
        
        let func: LuaFunction = self.lua.globals().get(func_name)?;
        
        match args.len() {
            0 => {
                let result = func.call::<_, LuaValue>(())?;
                Ok(vec![result])
            }
            1 => {
                let result = func.call::<_, LuaValue>(args[0].clone())?;
                Ok(vec![result])
            }
            _ => {
                let multi_value = LuaMultiValue::from_vec(args.to_vec());
                let results = func.call::<_, LuaMultiValue>(multi_value)?;
                Ok(results.into_vec())
            }
        }
    }
    
    pub fn set_global(&self, name: &str, value: LuaValue) -> LuaResult<()> {
        self.lua.globals().set(name, value)
    }
    
    pub fn get_global(&self, name: &str) -> LuaResult<LuaValue> {
        self.lua.globals().get(name)
    }
    
    pub fn update_scripts(&mut self, delta_time: f32) -> LuaResult<()> {
        // Call update function if it exists
        if let Ok(update) = self.lua.globals().get::<_, LuaFunction>("update") {
            update.call::<_, ()>(delta_time)?;
        }
        
        Ok(())
    }
    
    pub fn on_event(&self, event_type: &str, event_data: LuaValue) -> LuaResult<()> {
        if let Ok(on_event) = self.lua.globals().get::<_, LuaFunction>("on_event") {
            on_event.call::<_, ()>((event_type, event_data))?;
        }
        
        Ok(())
    }
}

// Example usage
fn example() -> LuaResult<()> {
    let mut scripting = ScriptingContext::new()?;
    
    let script = r#"
        player_pos = {x = 0, y = 0, z = 0}
        velocity = 5.0
        
        function init()
            log("Player script initialized")
        end
        
        function update(dt)
            -- Move player
            player_pos.x = player_pos.x + velocity * dt
            
            -- Normalize position vector
            local len = vec3_length(player_pos.x, player_pos.y, player_pos.z)
            if len > 100 then
                local nx, ny, nz = vec3_normalize(player_pos.x, player_pos.y, player_pos.z)
                player_pos.x = nx * 100
                player_pos.y = ny * 100
                player_pos.z = nz * 100
            end
        end
        
        function on_event(event_type, data)
            if event_type == "collision" then
                log("Player collided!")
                velocity = -velocity
            end
        end
        
        function get_position()
            return player_pos.x, player_pos.y, player_pos.z
        end
    "#;
    
    let handle = scripting.load_script_from_string("player", script)?;
    
    // Game loop
    for _ in 0..60 {
        scripting.update_scripts(0.016)?;
    }
    
    // Get player position
    let result = scripting.call_function(handle, "get_position", &[])?;
    println!("Player position: {:?}", result);
    
    // Trigger event
    scripting.on_event("collision", LuaValue::Nil)?;
    
    Ok(())
}
```

</details>

## Related Resources

- [mlua Documentation](https://docs.rs/mlua/)
- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)
- [Praxis Scripting Guide](../../guides/scripting.md)
- [Programming in Lua](https://www.lua.org/pil/)

## Next Steps

- Add hot-reload support (Exercise 07)
- Implement sandboxing for untrusted scripts
- Profile script performance
- Study `praxis_scripting` implementation
