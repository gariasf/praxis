# Script Bindings

Script bindings expose engine functionality to scripting languages like Lua, Python, or JavaScript. This enables rapid iteration, modding support, and empowers non-programmers to create game content without recompiling the engine.

## The Core Problem

Game engines are typically written in systems programming languages (C++, Rust) for performance, but these languages are:

- **Slow to compile** - Long iteration cycles
- **Complex** - Steep learning curve
- **Unsafe** - Easy to crash the entire engine
- **Inflexible** - Requires recompilation for changes

Scripting languages solve these problems but introduce new challenges:

- **Type safety** - How to prevent runtime errors?
- **Performance** - Scripts are slower than compiled code
- **Memory safety** - Who owns objects? When are they freed?
- **Security** - How to sandbox untrusted scripts?

## FFI Architecture Patterns

### 1. C FFI with Wrapper Layer

**Concept**: Most scripting language runtimes provide C FFI. Create C wrappers around engine APIs.

=== "Rust → Lua (Praxis)"

    ```rust
    // Praxis uses mlua for Lua bindings
    // From crates/praxis_scripting/src/bindings.rs

    use mlua::{Lua, UserData, UserDataMethods, Result};

    // Wrapper type for Entity (not directly usable from Lua)
    #[derive(Clone, Copy)]
    pub struct LuaEntity(Entity);

    impl UserData for LuaEntity {
        fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
            // Expose methods to Lua
            methods.add_method("id", |_, this, ()| {
                Ok(this.0.index())
            });

            methods.add_method("generation", |_, this, ()| {
                Ok(this.0.generation())
            });
        }
    }

    // Expose engine functions to Lua
    pub fn create_bindings(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        // Create namespace table
        let engine = lua.create_table()?;

        // Bind function: engine.spawn_entity()
        let spawn_fn = lua.create_function(|lua, ()| {
            with_world(|world| {
                let entity = world.spawn_empty();
                LuaEntity(entity)
            })
        })?;
        engine.set("spawn_entity", spawn_fn)?;

        // Bind function: engine.add_component(entity, component)
        let add_component_fn = lua.create_function(|lua, (entity, comp_type, data): (LuaEntity, String, mlua::Table)| {
            with_world(|world| {
                match comp_type.as_str() {
                    "Transform" => {
                        let x: f32 = data.get("x")?;
                        let y: f32 = data.get("y")?;
                        let z: f32 = data.get("z")?;
                        world.insert(entity.0, Transform::from_xyz(x, y, z));
                        Ok(())
                    }
                    _ => Err(mlua::Error::RuntimeError("Unknown component".into()))
                }
            })
        })?;
        engine.set("add_component", add_component_fn)?;

        globals.set("engine", engine)?;
        Ok(())
    }
    ```

    **Usage from Lua**:
    ```lua
    -- Lua script can now call engine functions
    local entity = engine.spawn_entity()
    engine.add_component(entity, "Transform", { x = 0, y = 0, z = 0 })
    ```

=== "C++ → Lua (Unreal-style)"

    ```cpp
    // Traditional C++ approach using lua C API
    #include <lua.hpp>

    // C function callable from Lua
    int lua_spawn_entity(lua_State* L) {
        // Create entity
        Entity entity = g_World->SpawnEntity();
        
        // Push entity ID to Lua stack
        lua_pushinteger(L, entity.GetID());
        return 1;  // Number of return values
    }

    int lua_add_component(lua_State* L) {
        // Get arguments from Lua stack
        int entity_id = luaL_checkinteger(L, 1);
        const char* comp_type = luaL_checkstring(L, 2);
        
        if (strcmp(comp_type, "Transform") == 0) {
            lua_getfield(L, 3, "x");
            float x = lua_tonumber(L, -1);
            lua_pop(L, 1);
            
            // ... get y, z similarly
            
            Entity entity = g_World->GetEntity(entity_id);
            entity.AddComponent<Transform>(x, y, z);
        }
        
        return 0;
    }

    // Register functions
    void RegisterBindings(lua_State* L) {
        lua_newtable(L);
        
        lua_pushcfunction(L, lua_spawn_entity);
        lua_setfield(L, -2, "spawn_entity");
        
        lua_pushcfunction(L, lua_add_component);
        lua_setfield(L, -2, "add_component");
        
        lua_setglobal(L, "engine");
    }
    ```

=== "C# → Lua (Unity with MoonSharp)"

    ```csharp
    using MoonSharp.Interpreter;

    public class LuaBindings 
    {
        private Script script;

        public void Initialize() 
        {
            script = new Script();
            
            // Register CLR type
            UserData.RegisterType<Transform>();
            
            // Expose global function
            script.Globals["spawn_entity"] = (Func<Entity>)(() => {
                return World.Spawn();
            });

            script.Globals["add_component"] = (Action<Entity, string, Table>)((entity, type, data) => {
                if (type == "Transform") 
                {
                    var x = (float)data["x"];
                    var y = (float)data["y"];
                    var z = (float)data["z"];
                    entity.AddComponent(new Transform(x, y, z));
                }
            });
        }

        public void Execute(string code) 
        {
            script.DoString(code);
        }
    }
    ```

**Trade-offs**:

✅ **Strengths**:
- Full control over exposed API surface
- Can implement safety checks
- Performance optimization at boundary
- Works with any scripting language

❌ **Weaknesses**:
- Lots of boilerplate
- Manual memory management
- Easy to introduce memory leaks
- Tedious to maintain

### 2. Automatic Binding Generation

**Concept**: Use code generation or reflection to automatically create bindings.

=== "Rust (Macro-Based)"

    ```rust
    // Hypothetical macro-based approach
    #[lua_export]
    impl World {
        #[lua_method]
        pub fn spawn_entity(&mut self) -> Entity {
            self.inner_mut().spawn_empty()
        }

        #[lua_method]
        pub fn add_transform(&mut self, entity: Entity, x: f32, y: f32, z: f32) {
            self.insert(entity, Transform::from_xyz(x, y, z));
        }
    }

    // Macro generates all FFI boilerplate
    // Usage from Lua:
    // world:spawn_entity()
    // world:add_transform(entity, 0, 0, 0)
    ```

=== "C++ (SWIG)"

    ```cpp
    // SWIG interface file (world.i)
    %module engine
    
    %{
    #include "World.h"
    #include "Entity.h"
    #include "Transform.h"
    %}

    // Tell SWIG about these types
    %include "World.h"
    %include "Entity.h"
    %include "Transform.h"

    // SWIG generates bindings for Python, Lua, etc.
    // Automatically handles type conversion
    ```

    Generated Python usage:
    ```python
    import engine
    
    world = engine.World()
    entity = world.spawn_entity()
    world.add_transform(entity, 0, 0, 0)
    ```

=== "C# (Reflection-Based)"

    ```csharp
    // C# can use reflection for dynamic bindings
    public class ScriptBridge 
    {
        private Dictionary<string, MethodInfo> methods = new();

        public void RegisterType<T>() 
        {
            foreach (var method in typeof(T).GetMethods()) 
            {
                if (method.GetCustomAttribute<ScriptExposed>() != null) 
                {
                    methods[typeof(T).Name + "." + method.Name] = method;
                }
            }
        }

        public object Call(string methodName, params object[] args) 
        {
            if (methods.TryGetValue(methodName, out var method)) 
            {
                return method.Invoke(null, args);
            }
            throw new Exception($"Method {methodName} not found");
        }
    }

    [AttributeUsage(AttributeTargets.Method)]
    public class ScriptExposed : Attribute { }

    public class World 
    {
        [ScriptExposed]
        public static Entity SpawnEntity() { /* ... */ }
    }
    ```

**Trade-offs**:

✅ **Strengths**:
- Minimal boilerplate
- Easy to add new bindings
- Consistent API between native and script
- Less error-prone

❌ **Weaknesses**:
- Less control over API surface
- Harder to debug generated code
- May expose unsafe APIs accidentally
- Build system complexity

### 3. Object Handle Pattern

**Concept**: Scripts hold opaque handles; engine validates before dereferencing.

=== "Rust (Praxis)"

    ```rust
    // Thread-local world access pattern from Praxis
    use std::cell::RefCell;

    thread_local! {
        static WORLD_CONTEXT: RefCell<Option<*mut World>> = RefCell::new(None);
    }

    pub fn set_world_context(world: &mut World) {
        WORLD_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = Some(world as *mut World);
        });
    }

    pub fn clear_world_context() {
        WORLD_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = None;
        });
    }

    pub fn with_world<F, R>(f: F) -> mlua::Result<R>
    where
        F: FnOnce(&mut World) -> R,
    {
        WORLD_CONTEXT.with(|ctx| {
            let ctx_ref = ctx.borrow();
            match *ctx_ref {
                Some(world_ptr) => unsafe {
                    // SAFETY: Caller must ensure world_ptr is valid
                    // during the call to with_world
                    Ok(f(&mut *world_ptr))
                },
                None => Err(mlua::Error::RuntimeError(
                    "No world context available".into()
                )),
            }
        })
    }

    // Usage in Lua binding:
    let spawn_fn = lua.create_function(|_, ()| {
        with_world(|world| {
            LuaEntity(world.spawn_empty())
        })
    })?;
    ```

    **Safety pattern**:
    1. Set world context before executing script
    2. Script calls functions that use `with_world`
    3. Clear world context after script completes
    4. Prevents dangling pointers

=== "C++ (Smart Handles)"

    ```cpp
    // Handle-based API for safety
    class EntityHandle {
        uint32_t id_;
        uint32_t generation_;
        World* world_;  // Non-owning pointer

    public:
        EntityHandle(uint32_t id, uint32_t gen, World* world)
            : id_(id), generation_(gen), world_(world) {}

        // Validate before dereferencing
        Entity* Get() const {
            if (world_->IsValid(id_, generation_)) {
                return world_->GetEntity(id_);
            }
            return nullptr;
        }
    };

    // Lua binding returns handle, not raw pointer
    int lua_spawn_entity(lua_State* L) {
        Entity entity = g_World->SpawnEntity();
        
        // Create handle and push to Lua
        auto* handle = new EntityHandle(
            entity.GetID(), 
            entity.GetGeneration(), 
            g_World
        );
        lua_pushlightuserdata(L, handle);
        return 1;
    }

    int lua_get_transform(lua_State* L) {
        auto* handle = (EntityHandle*)lua_touserdata(L, 1);
        
        Entity* entity = handle->Get();
        if (!entity) {
            return luaL_error(L, "Entity no longer valid");
        }
        
        Transform* transform = entity->GetComponent<Transform>();
        // ... push transform data to Lua
    }
    ```

## Real-World Examples

### Praxis: ECS Script Bindings

```rust
// From crates/praxis_scripting/src/bindings.rs

// Expose console commands for ECS introspection
pub fn create_console_bindings(lua: &Lua) -> Result<()> {
    let console = lua.create_table()?;

    // List all entities
    let list_entities = lua.create_function(|_, ()| {
        with_world(|world| {
            let mut output = String::from("Entities:\n");
            for entity in world.iter() {
                output.push_str(&format!("  - {:?}\n", entity));
            }
            Ok(output)
        })
    })?;
    console.set("list_entities", list_entities)?;

    // Inspect entity components
    let inspect_entity = lua.create_function(|_, entity: LuaEntity| {
        with_world(|world| {
            if let Some(name) = world.get::<Name>(entity.0) {
                Ok(format!("Entity: {}", name.as_str()))
            } else {
                Ok(format!("Entity: {:?}", entity.0))
            }
        })
    })?;
    console.set("inspect", inspect_entity)?;

    lua.globals().set("console", console)?;
    Ok(())
}
```

**Lua usage**:
```lua
-- List all entities
print(console.list_entities())

-- Inspect specific entity
local entity = engine.spawn_entity()
print(console.inspect(entity))
```

### Unity: C# Scripting

Unity's entire scripting API is C#—no FFI needed! But it uses reflection for bindings:

```csharp
// Unity exposes C++ engine through managed C# API
public class GameObject 
{
    // Native pointer to C++ object
    private IntPtr m_CachedPtr;

    // P/Invoke to native code
    [MethodImpl(MethodImplOptions.InternalCall)]
    private extern void GetComponent_Internal(Type type, out Component component);

    public T GetComponent<T>() where T : Component 
    {
        GetComponent_Internal(typeof(T), out var component);
        return (T)component;
    }

    // Property backed by native call
    public Transform transform 
    {
        get 
        {
            return GetComponent<Transform>();
        }
    }
}

// User scripts derive from MonoBehaviour
public class MyScript : MonoBehaviour 
{
    void Update() 
    {
        // Calls native engine code under the hood
        transform.position += Vector3.forward * Time.deltaTime;
    }
}
```

Unity's approach:
- C++ engine exposes C API
- C# wrapper layer provides managed API
- User scripts use pure C#
- No manual FFI needed

### Bevy: No Scripting (Pure Rust)

Bevy doesn't include scripting by default—it's a Rust-native engine:

```rust
// No FFI—just Rust
fn player_movement(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in query.iter_mut() {
        if keyboard.pressed(KeyCode::W) {
            transform.translation.z -= 0.1;
        }
    }
}

// Third-party crates add scripting:
// - bevy_mod_scripting: Lua/Rhai scripting
// - bevy_script_api: Experimental script bindings
```

**Trade-off**: Fast iteration via hot-reloading Rust code, but requires Rust knowledge.

## Type Mapping Strategies

### Primitive Types

Most scripting FFIs automatically convert primitives:

| Rust | Lua | Python | C# |
|------|-----|--------|-----|
| `i32`, `i64` | `number` | `int` | `int`, `long` |
| `f32`, `f64` | `number` | `float` | `float`, `double` |
| `bool` | `boolean` | `bool` | `bool` |
| `String` | `string` | `str` | `string` |
| `Vec<T>` | `table` | `list` | `List<T>` |

```rust
// mlua handles these automatically
let func = lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
    Ok(Transform::from_xyz(x, y, z))
})?;
```

### Complex Types

**Option 1: Copy/Clone**

```rust
// Copy data from native to script
impl UserData for LuaTransform {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.translation.x));
        fields.add_field_method_get("y", |_, this| Ok(this.translation.y));
        fields.add_field_method_get("z", |_, this| Ok(this.translation.z));
    }
}

// Lua gets a copy, not a reference
let get_transform = lua.create_function(|_, entity: LuaEntity| {
    with_world(|world| {
        let transform = world.get::<Transform>(entity.0)?;
        Ok(LuaTransform { translation: transform.translation })
    })
})?;
```

**Option 2: Opaque Handle**

```rust
// Script holds handle, can't inspect directly
struct TransformHandle {
    entity: Entity,
}

impl UserData for TransformHandle {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_x", |_, this, ()| {
            with_world(|world| {
                let transform = world.get::<Transform>(this.entity)?;
                Ok(transform.translation.x)
            })
        });

        methods.add_method("set_x", |_, this, x: f32| {
            with_world(|world| {
                let mut transform = world.get_mut::<Transform>(this.entity)?;
                transform.translation.x = x;
                Ok(())
            })
        });
    }
}
```

**Option 3: Tables/Dictionaries**

```rust
// Convert to Lua table
let to_table = lua.create_function(|lua, transform: LuaTransform| {
    let table = lua.create_table()?;
    table.set("x", transform.translation.x)?;
    table.set("y", transform.translation.y)?;
    table.set("z", transform.translation.z)?;
    Ok(table)
})?;

// Lua receives plain table
local transform = engine.get_transform(entity)
print(transform.x, transform.y, transform.z)
transform.x = 10  -- Just modifies table, not engine state!
```

## Memory Management

### Reference Counting

```rust
// Lua's GC manages script-side objects
impl UserData for LuaEntity {
    // No explicit cleanup—Lua GC handles it
}

// But underlying Entity is still in World
// Script just holds ID, not ownership
```

### Lifetime Validation

```rust
// Generational indices detect use-after-free
pub struct Entity {
    index: u32,
    generation: u32,
}

impl World {
    pub fn get<T>(&self, entity: Entity) -> Option<&T> {
        let entry = self.entities.get(entity.index)?;
        
        // Check generation matches
        if entry.generation != entity.generation {
            return None;  // Entity was destroyed and recreated
        }
        
        entry.get_component::<T>()
    }
}

// Lua can keep stale Entity handle
// Engine safely returns None instead of crashing
```

### Garbage Collection Integration

```lua
-- Lua manages script objects
local entities = {}
for i = 1, 100 do
    table.insert(entities, engine.spawn_entity())
end

-- Lua GC can collect entities table
-- But underlying Engine entities still exist!
-- Scripts don't own engine objects
```

**Key principle**: Scripts hold *references* (IDs/handles), not ownership.

## Sandboxing and Security

### Praxis Sandboxing

```rust
// From crates/praxis_scripting/src/sandbox.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    None,      // No restrictions
    Moderate,  // Remove dangerous functions
    Strict,    // Minimal standard library
}

pub fn apply_sandbox(lua: &Lua, level: SandboxLevel) -> Result<()> {
    match level {
        SandboxLevel::None => Ok(()),
        
        SandboxLevel::Moderate => {
            let globals = lua.globals();
            
            // Remove file I/O
            globals.set("io", mlua::Nil)?;
            globals.set("dofile", mlua::Nil)?;
            globals.set("loadfile", mlua::Nil)?;
            
            // Remove OS access
            globals.set("os", mlua::Nil)?;
            
            // Remove arbitrary code execution
            globals.set("load", mlua::Nil)?;
            globals.set("loadstring", mlua::Nil)?;
            
            Ok(())
        }
        
        SandboxLevel::Strict => {
            // Create minimal environment
            let env = lua.create_table()?;
            
            // Only provide safe functions
            env.set("print", lua.globals().get::<_, mlua::Function>("print")?)?;
            env.set("tonumber", lua.globals().get::<_, mlua::Function>("tonumber")?)?;
            env.set("tostring", lua.globals().get::<_, mlua::Function>("tostring")?)?;
            
            // Set as global environment
            lua.globals().set("_G", env)?;
            
            Ok(())
        }
    }
}
```

### Instruction Limits

```rust
// Prevent infinite loops
lua.set_hook(HookTriggers::every_nth_instruction(10000), |_lua, _debug| {
    // Check execution time
    if exceeded_time_limit() {
        Err(mlua::Error::RuntimeError("Script timeout".into()))
    } else {
        Ok(())
    }
});
```

### Memory Limits

```lua
-- Lua can set memory limits
lua.set_memory_limit(10 * 1024 * 1024);  -- 10MB limit

-- Script that allocates too much will error
local huge_table = {}
for i = 1, 1000000 do
    table.insert(huge_table, { data = string.rep("x", 1000) })
end
-- Error: memory limit exceeded
```

## Performance Optimization

### Minimize FFI Crossings

```rust
// Bad: Many small calls
for i = 1, 100 do
    local entity = engine.spawn_entity()
    engine.add_transform(entity, i, 0, 0)
    engine.add_velocity(entity, 1, 0, 0)
end

// Good: Batch operations
local entities = engine.spawn_entities(100)
engine.add_transforms_batch(entities, positions)
engine.add_velocities_batch(entities, velocities)
```

### Cache Frequently Accessed Data

```lua
-- Bad: Query every frame
function update()
    local player = engine.find_entity("Player")
    local transform = engine.get_transform(player)
    -- Use transform
end

-- Good: Cache during initialization
local player = nil
local player_transform_handle = nil

function init()
    player = engine.find_entity("Player")
    player_transform_handle = engine.get_transform_handle(player)
end

function update()
    -- Direct handle access, no search
    local pos = player_transform_handle:get_position()
end
```

### JIT-Friendly Code

```lua
-- LuaJIT can optimize this
local function distance_squared(x1, y1, x2, y2)
    local dx = x2 - x1
    local dy = y2 - y1
    return dx * dx + dy * dy
end

-- Avoid FFI in hot loops
for i = 1, 10000 do
    local dist = distance_squared(entities[i].x, entities[i].y, player.x, player.y)
    if dist < 100 then
        -- Process nearby entity
    end
end
```

## Design Guidelines

### What to Expose

✅ **Expose to scripts**:
- Gameplay logic (AI, quests, dialogue)
- Content creation (spawn entities, configure components)
- High-level events (player died, level completed)
- Configuration and tuning (health values, speeds)

❌ **Don't expose to scripts**:
- Performance-critical loops (rendering, physics)
- Low-level memory management
- Engine initialization/shutdown
- Security-critical operations

### API Design Principles

**1. Make invalid states unrepresentable**

```rust
// Bad: Scripts can pass invalid entity
function damage_entity(entity_id)
    -- What if entity_id is invalid?
    engine.modify_health(entity_id, -10)
end

// Good: Engine validates
function damage_entity(entity_handle)
    -- Returns error if entity invalid
    local ok, err = pcall(function()
        entity_handle:modify_health(-10)
    end)
    if not ok then
        print("Entity no longer exists")
    end
end
```

**2. Provide both low-level and high-level APIs**

```lua
-- High-level: Easy for beginners
engine.spawn_enemy("Goblin", {x = 10, y = 0, z = 5})

-- Low-level: Flexible for advanced users
local enemy = engine.spawn_entity()
engine.add_component(enemy, "Transform", {x = 10, y = 0, z = 5})
engine.add_component(enemy, "Mesh", {model = "goblin.gltf"})
engine.add_component(enemy, "AI", {behavior = "aggressive"})
```

**3. Use callbacks for engine events**

```lua
-- Register callback for collision events
engine.on_collision(function(entity_a, entity_b)
    print("Collision between", entity_a, entity_b)
end)

-- Engine calls this from native code when collisions occur
```

## Testing Script Bindings

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_spawn_entity() {
        let lua = Lua::new();
        let mut world = World::new();
        
        set_world_context(&mut world);
        create_bindings(&lua).unwrap();
        
        lua.load(r#"
            local entity = engine.spawn_entity()
            assert(entity ~= nil)
        "#).exec().unwrap();
        
        clear_world_context();
    }

    #[test]
    fn test_lua_sandbox() {
        let lua = Lua::new();
        apply_sandbox(&lua, SandboxLevel::Strict).unwrap();
        
        // Should error—io is removed
        let result = lua.load(r#"
            local file = io.open("secret.txt", "r")
        "#).exec();
        
        assert!(result.is_err());
    }
}
```

## Summary

| Approach | Pros | Cons | Best For |
|----------|------|------|----------|
| **Manual FFI** | Full control, optimized | Boilerplate, error-prone | Performance-critical |
| **Auto-Generated** | Less work, consistent | Less control | Large APIs |
| **Reflection-Based** | Dynamic, flexible | Runtime overhead | C#/Java engines |
| **Handle-Based** | Safe, validates access | Indirection cost | Persistent objects |

**Key principles**:
1. **Safety first** - Validate all script inputs
2. **Clear ownership** - Scripts reference, don't own
3. **Minimize crossings** - Batch operations when possible
4. **Sandbox untrusted code** - Remove dangerous functions
5. **Test thoroughly** - Scripts can crash the engine

Choose scripting based on:
- **Audience**: Modders need sandboxing, internal tools don't
- **Performance**: Gameplay logic OK, rendering not OK
- **Iteration speed**: Scripts allow hot-reload
- **Security**: Untrusted code must be sandboxed

## Related Patterns

- [Declarative APIs](declarative-vs-imperative.md) - Scripts often use declarative style
- [Language Constraints](language-constraints.md) - FFI and type system interactions
- [Component APIs](component-apis.md) - Exposing ECS to scripts
