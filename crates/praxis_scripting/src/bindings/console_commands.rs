//! Console commands for engine introspection and runtime modifications.
//!
//! # REPL Integration Pattern
//!
//! This module implements a **command API** for interactive console/REPL usage.
//! It provides Lua functions specifically designed for:
//! - Quick entity querying without writing full scripts
//! - Live debugging of ECS state during development
//! - Runtime modification of game state for testing
//! - Interactive exploration of the entity hierarchy
//!
//! ## Design Principles
//!
//! ### 1. Namespace Isolation
//! All commands live under the `console` global table to avoid polluting
//! the global namespace. This prevents conflicts with user scripts:
//! ```lua
//! -- Good: Namespaced
//! console.list_entities()
//!
//! -- Bad: Global pollution
//! list_entities()
//! ```
//!
//! ### 2. Human-Readable Output
//! Return values are formatted strings, not raw data structures. This makes
//! REPL output immediately useful without additional formatting:
//! ```lua
//! > console.list_entities()
//! Entities (3):
//!   Entity(0) - Player
//!   Entity(1) - Enemy
//!   Entity(2) - Pickup
//! ```
//!
//! ### 3. Error Handling
//! Invalid operations return descriptive error messages, not panics:
//! ```lua
//! > console.inspect(999)
//! Error: Entity 999 not found
//! ```
//!
//! ### 4. World Access via Thread-Local
//! Console commands need to query/modify the ECS World. They use the
//! `with_world_raw()` helper to access the thread-local World pointer
//! set by `ScriptingContext::with_world()`.
//!
//! ## ECS Access Pattern
//!
//! Console commands demonstrate the standard pattern for ECS access from Lua:
//!
//! ```rust
//! lua.create_function(|_lua, args| {
//!     with_world_raw(|world| {
//!         // Access World here
//!         let entities = world.inner().entities();
//!         // Process and return result
//!         Ok(result)
//!     })
//! })
//! ```
//!
//! The `with_world_raw()` function:
//! - Retrieves the World pointer from thread-local storage
//! - Dereferences it (unsafe) to get `&mut World`
//! - Calls the provided closure with World access
//! - Returns the result or error
//!
//! ## Available Commands
//!
//! ### Query Commands (Read-Only)
//! - `console.list_entities()` - List all entities
//! - `console.entity_count()` - Get total count
//! - `console.inspect(entity_id)` - Show entity details
//! - `console.find_entity(name)` - Find by name
//! - `console.get_transform(entity_id)` - Get position/rotation/scale
//! - `console.query_with_name()` - List entities with Name component
//! - `console.query_with_transform()` - List entities with Transform
//!
//! ### Mutation Commands (Write)
//! - `console.set_transform(entity_id, x, y, z)` - Move entity
//! - `console.spawn(name)` - Create new entity
//! - `console.despawn(entity_id)` - Remove entity
//!
//! # Usage Examples
//!
//! ## Interactive Debugging Session
//! ```lua
//! -- Find the player entity
//! local player = console.find_entity("Player")
//!
//! -- Check its current position
//! console.inspect(player)
//!
//! -- Move it to origin
//! console.set_transform(player, 0, 0, 0)
//!
//! -- Spawn a test enemy nearby
//! console.spawn("DebugEnemy")
//! ```
//!
//! ## Quick Entity Count
//! ```lua
//! > console.entity_count()
//! 42
//! ```
//!
//! ## Finding Memory Leaks
//! ```lua
//! -- Check entity count before test
//! local before = console.entity_count()
//!
//! -- Run some game logic...
//!
//! -- Check after
//! local after = console.entity_count()
//! print("Leaked entities:", after - before)
//! ```

use mlua::Lua;
use praxis_ecs::{Entity, GlobalTransform, Name, Transform};
use praxis_utils::Result;

/// Registers console introspection commands with the Lua environment.
///
/// This creates a `console` table with utility functions for querying
/// and modifying the ECS World. These functions are designed for
/// interactive use from a REPL/console interface.
///
/// # Implementation Note
///
/// Each function is created with `lua.create_function()`, which returns
/// a closure that can be called from Lua. The closure captures no external
/// state (stateless) and accesses the World via thread-local storage.
///
/// # Available Commands
///
/// - `console.list_entities()` - List all entities with their IDs and names
/// - `console.entity_count()` - Get the total number of entities
/// - `console.inspect(entity)` - Inspect an entity's components
/// - `console.find_entity(name)` - Find an entity by name
/// - `console.get_transform(entity)` - Get an entity's transform
/// - `console.set_transform(entity, x, y, z)` - Set an entity's position
/// - `console.spawn(name)` - Spawn a new entity with a name
/// - `console.despawn(entity)` - Remove an entity from the world
///
/// # Example
///
/// ```lua
/// -- List all entities
/// console.list_entities()
///
/// -- Find an entity by name
/// local entity = console.find_entity("Player")
///
/// -- Inspect its components
/// console.inspect(entity)
///
/// -- Modify its position
/// console.set_transform(entity, 10, 5, 0)
///
/// -- Spawn a new entity
/// local new_entity = console.spawn("TestEntity")
/// ```
pub fn register_console_commands(lua: &Lua) -> Result<()> {
    let console = lua.create_table()?;

    // List all entities in the world
    //
    // Returns a formatted string with all entities and their names.
    // Uses optional Name component - entities without names show as "<unnamed>".
    console.set(
        "list_entities",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let mut entities = Vec::new();

                // Query all entities with optional Name component
                // Note: Query must be created with inner_mut() but iteration uses inner()
                // to avoid mutable aliasing issues
                let mut query = world.inner_mut().query::<(Entity, Option<&Name>)>();

                for (entity, name) in query.iter(world.inner()) {
                    let name_str = name.map(|n| n.as_str()).unwrap_or("<unnamed>");
                    entities.push(format!("Entity({}) - {}", entity.index(), name_str));
                }

                if entities.is_empty() {
                    Ok("No entities in the world".to_string())
                } else {
                    Ok(format!(
                        "Entities ({}):\n  {}",
                        entities.len(),
                        entities.join("\n  ")
                    ))
                }
            })
        })?,
    )?;

    // Get total entity count
    //
    // Returns a single number (usize). This is more efficient than list_entities()
    // when you only need the count.
    console.set(
        "entity_count",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let count = world.inner().entities().len();
                Ok(count)
            })
        })?,
    )?;

    // Inspect an entity's components
    //
    // Shows all recognized components for debugging. Add more component types
    // here as the engine grows.
    //
    // Arguments:
    // - entity_id: Entity index as u32
    //
    // Returns: Formatted string with component details
    console.set(
        "inspect",
        lua.create_function(|_lua, entity_id: u32| {
            super::ecs_api::with_world_raw(|world| {
                // Convert u32 entity index to Entity handle
                // Note: This assumes entity generation is 0. For production, store full Entity.
                let entity = Entity::from_bits(entity_id as u64);

                // Get entity reference for component queries
                let entity_ref = world.inner().get_entity(entity).ok_or_else(|| {
                    mlua::Error::RuntimeError(format!("Entity {entity_id} not found"))
                })?;

                let mut components = Vec::new();

                // Check for common components and format them nicely
                if let Some(name) = entity_ref.get::<Name>() {
                    components.push(format!("  Name: \"{}\"", name.as_str()));
                }

                if let Some(transform) = entity_ref.get::<Transform>() {
                    components.push(format!(
                        "  Transform: pos=({:.2}, {:.2}, {:.2}), scale=({:.2}, {:.2}, {:.2})",
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                        transform.scale.x,
                        transform.scale.y,
                        transform.scale.z
                    ));
                }

                if let Some(global_transform) = entity_ref.get::<GlobalTransform>() {
                    components.push(format!(
                        "  GlobalTransform: pos=({:.2}, {:.2}, {:.2})",
                        global_transform.translation().x,
                        global_transform.translation().y,
                        global_transform.translation().z
                    ));
                }

                if components.is_empty() {
                    Ok(format!("Entity({entity_id}) - No recognized components"))
                } else {
                    Ok(format!("Entity({entity_id}):\n{}", components.join("\n")))
                }
            })
        })?,
    )?;

    // Find entity by name
    //
    // Linear search through all entities. For production with many entities,
    // consider maintaining a name->entity index.
    //
    // Arguments:
    // - name: String to search for
    //
    // Returns: Option<u32> (entity index if found, nil if not found)
    console.set(
        "find_entity",
        lua.create_function(|_, name: String| {
            super::ecs_api::with_world_raw(|world| {
                let mut query = world.inner_mut().query::<(Entity, &Name)>();

                for (entity, entity_name) in query.iter(world.inner()) {
                    if entity_name.as_str() == name {
                        return Ok(Some(entity.index()));
                    }
                }

                Ok(None)
            })
        })?,
    )?;

    // Get entity's transform
    //
    // Returns a Lua table with x, y, z fields.
    //
    // Arguments:
    // - entity_id: Entity index as u32
    //
    // Returns: Table { x, y, z }
    console.set(
        "get_transform",
        lua.create_function(|lua, entity_id: u32| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);

                let transform = world.inner().get::<Transform>(entity).ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "Entity {entity_id} does not have a Transform component"
                    ))
                })?;

                // Create a Lua table with the transform data
                let table = lua.create_table()?;
                table.set("x", transform.translation.x)?;
                table.set("y", transform.translation.y)?;
                table.set("z", transform.translation.z)?;
                Ok(table)
            })
        })?,
    )?;

    // Set entity's transform position
    //
    // Modifies the entity's local Transform component. If the entity has a parent,
    // this affects its position relative to the parent.
    //
    // Arguments:
    // - entity_id: Entity index as u32
    // - x, y, z: New position coordinates as f32
    //
    // Returns: Confirmation string
    console.set(
        "set_transform",
        lua.create_function(|_, (entity_id, x, y, z): (u32, f32, f32, f32)| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);

                // Get mutable reference to Transform component
                let mut transform =
                    world
                        .inner_mut()
                        .get_mut::<Transform>(entity)
                        .ok_or_else(|| {
                            mlua::Error::RuntimeError(format!(
                                "Entity {entity_id} does not have a Transform component"
                            ))
                        })?;

                // Update translation
                transform.translation.x = x;
                transform.translation.y = y;
                transform.translation.z = z;

                Ok(format!(
                    "Set transform for Entity({entity_id}) to ({x:.2}, {y:.2}, {z:.2})"
                ))
            })
        })?,
    )?;

    // Spawn a new entity with a name
    //
    // Creates a new entity with Name, Transform, and GlobalTransform components.
    // This is the minimum set for a visible entity in the scene graph.
    //
    // Arguments:
    // - name: String name for the entity
    //
    // Returns: Confirmation string with entity ID
    console.set(
        "spawn",
        lua.create_function(|_, name: String| {
            super::ecs_api::with_world_raw(|world| {
                // Spawn entity with basic components
                let entity = world.spawn((
                    Name::new(name.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                ));

                Ok(format!(
                    "Spawned Entity({}) with name \"{}\"",
                    entity.index(),
                    name
                ))
            })
        })?,
    )?;

    // Despawn an entity
    //
    // Removes the entity and all its components from the World.
    // If the entity has children, they become orphans (not automatically despawned).
    //
    // Arguments:
    // - entity_id: Entity index as u32
    //
    // Returns: Confirmation string
    console.set(
        "despawn",
        lua.create_function(|_, entity_id: u32| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);

                world.despawn(entity).map_err(|_| {
                    mlua::Error::RuntimeError(format!("Failed to despawn Entity({entity_id})"))
                })?;

                Ok(format!("Despawned Entity({entity_id})"))
            })
        })?,
    )?;

    // Query entities by component: Name
    //
    // Lists all entities that have a Name component. Useful for finding named
    // entities vs. unnamed temporary entities.
    //
    // Returns: Formatted string with matching entities
    console.set(
        "query_with_name",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let mut entities = Vec::new();

                let mut query = world.inner_mut().query::<(Entity, &Name)>();

                for (entity, name) in query.iter(world.inner()) {
                    entities.push(format!(
                        "Entity({}) - \"{}\"",
                        entity.index(),
                        name.as_str()
                    ));
                }

                if entities.is_empty() {
                    Ok("No entities with Name component".to_string())
                } else {
                    Ok(format!(
                        "Entities with Name ({}):\n  {}",
                        entities.len(),
                        entities.join("\n  ")
                    ))
                }
            })
        })?,
    )?;

    // Query entities by component: Transform
    //
    // Lists all entities that have a Transform component, showing their positions.
    // Useful for debugging scene layout and entity placement.
    //
    // Returns: Formatted string with matching entities and positions
    console.set(
        "query_with_transform",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let mut entities = Vec::new();

                let mut query = world
                    .inner_mut()
                    .query::<(Entity, &Transform, Option<&Name>)>();

                for (entity, transform, name) in query.iter(world.inner()) {
                    let name_str = name.map(|n| n.as_str()).unwrap_or("<unnamed>");
                    entities.push(format!(
                        "Entity({}) - {} at ({:.2}, {:.2}, {:.2})",
                        entity.index(),
                        name_str,
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z
                    ));
                }

                if entities.is_empty() {
                    Ok("No entities with Transform component".to_string())
                } else {
                    Ok(format!(
                        "Entities with Transform ({}):\n  {}",
                        entities.len(),
                        entities.join("\n  ")
                    ))
                }
            })
        })?,
    )?;

    // Register the console table as a global
    lua.globals().set("console", console)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_ecs::World;

    #[test]
    fn test_register_console_commands() {
        let lua = Lua::new();
        let result = register_console_commands(&lua);
        assert!(result.is_ok());

        // Verify console table exists
        let has_console: bool = lua.load("return console ~= nil").eval().unwrap();
        assert!(has_console);

        // Verify commands are registered
        let has_list: bool = lua
            .load("return type(console.list_entities) == 'function'")
            .eval()
            .unwrap();
        assert!(has_list);
    }

    #[test]
    fn test_console_commands_with_world() {
        let lua = Lua::new();
        register_console_commands(&lua).unwrap();

        let mut world = World::new();

        // Spawn some test entities
        world.spawn((
            Name::new("TestEntity1"),
            Transform::default(),
            GlobalTransform::default(),
        ));
        world.spawn((
            Name::new("TestEntity2"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        // Test with world context
        super::super::ecs_api::set_world_context(&lua, &mut world).unwrap();

        let result: String = lua.load("return console.list_entities()").eval().unwrap();
        assert!(result.contains("TestEntity1"));
        assert!(result.contains("TestEntity2"));

        let count: usize = lua.load("return console.entity_count()").eval().unwrap();
        assert_eq!(count, 2);

        super::super::ecs_api::clear_world_context(&lua).unwrap();
    }
}
