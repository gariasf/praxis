//! Console commands for engine introspection and runtime modifications.
//!
//! This module provides Lua functions that can be used from a REPL/console
//! to query and modify the ECS World at runtime.

use mlua::Lua;
use praxis_ecs::{Entity, GlobalTransform, Name, Transform};
use praxis_utils::Result;

/// Registers console introspection commands with the Lua environment.
///
/// This creates a `console` table with utility functions for querying
/// and modifying the ECS World. These functions are designed for
/// interactive use from a REPL/console interface.
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
    console.set(
        "list_entities",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let mut entities = Vec::new();
                
                // Query all entities with optional Name component
                let mut query = world.inner_mut().query::<(Entity, Option<&Name>)>();
                
                for (entity, name) in query.iter(world.inner()) {
                    let name_str = name.map(|n| n.as_str()).unwrap_or("<unnamed>");
                    entities.push(format!("Entity({}) - {}", entity.index(), name_str));
                }
                
                if entities.is_empty() {
                    Ok("No entities in the world".to_string())
                } else {
                    Ok(format!("Entities ({}):\n  {}", entities.len(), entities.join("\n  ")))
                }
            })
        })?,
    )?;

    // Get total entity count
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
    console.set(
        "inspect",
        lua.create_function(|_lua, entity_id: u32| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);
                
                let entity_ref = world.inner().get_entity(entity)
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Entity {entity_id} not found")))?;
                
                let mut components = Vec::new();
                
                // Check for common components
                if let Some(name) = entity_ref.get::<Name>() {
                    components.push(format!("  Name: \"{}\"", name.as_str()));
                }
                
                if let Some(transform) = entity_ref.get::<Transform>() {
                    components.push(format!(
                        "  Transform: pos=({:.2}, {:.2}, {:.2}), scale=({:.2}, {:.2}, {:.2})",
                        transform.translation.x, transform.translation.y, transform.translation.z,
                        transform.scale.x, transform.scale.y, transform.scale.z
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
    console.set(
        "get_transform",
        lua.create_function(|lua, entity_id: u32| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);
                
                let transform = world.inner().get::<Transform>(entity)
                    .ok_or_else(|| mlua::Error::RuntimeError(
                        format!("Entity {entity_id} does not have a Transform component")
                    ))?;
                
                let table = lua.create_table()?;
                table.set("x", transform.translation.x)?;
                table.set("y", transform.translation.y)?;
                table.set("z", transform.translation.z)?;
                Ok(table)
            })
        })?,
    )?;

    // Set entity's transform position
    console.set(
        "set_transform",
        lua.create_function(|_, (entity_id, x, y, z): (u32, f32, f32, f32)| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);
                
                let mut transform = world.inner_mut().get_mut::<Transform>(entity)
                    .ok_or_else(|| mlua::Error::RuntimeError(
                        format!("Entity {entity_id} does not have a Transform component")
                    ))?;
                
                transform.translation.x = x;
                transform.translation.y = y;
                transform.translation.z = z;
                
                Ok(format!("Set transform for Entity({entity_id}) to ({x:.2}, {y:.2}, {z:.2})"))
            })
        })?,
    )?;

    // Spawn a new entity with a name
    console.set(
        "spawn",
        lua.create_function(|_, name: String| {
            super::ecs_api::with_world_raw(|world| {
                let entity = world.spawn((
                    Name::new(name.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                ));
                
                Ok(format!("Spawned Entity({}) with name \"{}\"", entity.index(), name))
            })
        })?,
    )?;

    // Despawn an entity
    console.set(
        "despawn",
        lua.create_function(|_, entity_id: u32| {
            super::ecs_api::with_world_raw(|world| {
                let entity = Entity::from_bits(entity_id as u64);
                
                world.despawn(entity)
                    .map_err(|_| mlua::Error::RuntimeError(format!("Failed to despawn Entity({entity_id})")))?;
                
                Ok(format!("Despawned Entity({entity_id})"))
            })
        })?,
    )?;

    // Query entities by component
    console.set(
        "query_with_name",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let mut entities = Vec::new();
                
                let mut query = world.inner_mut().query::<(Entity, &Name)>();
                
                for (entity, name) in query.iter(world.inner()) {
                    entities.push(format!("Entity({}) - \"{}\"", entity.index(), name.as_str()));
                }
                
                if entities.is_empty() {
                    Ok("No entities with Name component".to_string())
                } else {
                    Ok(format!("Entities with Name ({}):\n  {}", entities.len(), entities.join("\n  ")))
                }
            })
        })?,
    )?;

    // Query entities by component
    console.set(
        "query_with_transform",
        lua.create_function(|_, ()| {
            super::ecs_api::with_world_raw(|world| {
                let mut entities = Vec::new();
                
                let mut query = world.inner_mut().query::<(Entity, &Transform, Option<&Name>)>();
                
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
                    Ok(format!("Entities with Transform ({}):\n  {}", entities.len(), entities.join("\n  ")))
                }
            })
        })?,
    )?;

    // Set the console table as a global
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
        let has_list: bool = lua.load("return type(console.list_entities) == 'function'").eval().unwrap();
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
