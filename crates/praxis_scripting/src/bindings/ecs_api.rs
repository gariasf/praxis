//! ECS API bindings for Lua scripts.

use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use praxis_ecs::{Entity, World, Transform, Name};
use praxis_math::Vec3;
use praxis_utils::Result;
use std::cell::RefCell;

thread_local! {
    static WORLD_CONTEXT: RefCell<Option<*mut World>> = RefCell::new(None);
}

/// Sets the current ECS World context for the current thread.
///
/// # Safety
///
/// The world pointer must remain valid for the duration of script execution.
pub fn set_world_context(lua: &Lua, world: &mut World) -> Result<()> {
    WORLD_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(world as *mut World);
    });
    
    let world_table = create_world_table(lua)?;
    lua.globals().set("world", world_table)?;
    
    Ok(())
}

/// Clears the current ECS World context.
pub fn clear_world_context(lua: &Lua) -> Result<()> {
    WORLD_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
    
    lua.globals().set("world", Value::Nil)?;
    
    Ok(())
}

fn with_world<F, R>(f: F) -> mlua::Result<R>
where
    F: FnOnce(&mut World) -> mlua::Result<R>,
{
    WORLD_CONTEXT.with(|ctx| {
        let world_ptr = ctx.borrow()
            .ok_or_else(|| mlua::Error::RuntimeError("No world context available".to_string()))?;
        
        #[allow(unsafe_code)]
        unsafe {
            let world = &mut *world_ptr;
            f(world)
        }
    })
}

fn create_world_table(lua: &Lua) -> Result<Table> {
    let table = lua.create_table()?;
    
    table.set("spawn", lua.create_function(|_, ()| {
        with_world(|world| {
            let entity = world.spawn_empty();
            Ok(LuaEntity { id: entity })
        })
    })?)?;
    
    table.set("despawn", lua.create_function(|_, entity: LuaEntity| {
        with_world(|world| {
            world.despawn(entity.id);
            Ok(())
        })
    })?)?;
    
    table.set("get_entity_by_name", lua.create_function(|_, name: String| {
        with_world(|world| {
            use praxis_ecs::Query;
            let mut query = world.inner_mut().query::<(Entity, &Name)>();
            
            for (entity, entity_name) in query.iter(world.inner()) {
                if entity_name.as_str() == name {
                    return Ok(Some(LuaEntity { id: entity }));
                }
            }
            
            Ok(None)
        })
    })?)?;
    
    table.set("add_component_transform", lua.create_function(|_, (entity, x, y, z): (LuaEntity, f32, f32, f32)| {
        with_world(|world| {
            let transform = Transform::from_xyz(x, y, z);
            world.inner_mut().entity_mut(entity.id).insert(transform);
            Ok(())
        })
    })?)?;
    
    table.set("add_component_name", lua.create_function(|_, (entity, name): (LuaEntity, String)| {
        with_world(|world| {
            world.inner_mut().entity_mut(entity.id).insert(Name::new(name));
            Ok(())
        })
    })?)?;
    
    table.set("get_component_transform", lua.create_function(|_, entity: LuaEntity| {
        with_world(|world| {
            world.inner()
                .get::<Transform>(entity.id)
                .map(|t| LuaTransform { inner: *t })
                .ok_or_else(|| mlua::Error::RuntimeError("Entity does not have Transform component".to_string()))
        })
    })?)?;
    
    table.set("set_component_transform", lua.create_function(|_, (entity, transform): (LuaEntity, LuaTransform)| {
        with_world(|world| {
            if let Some(mut t) = world.inner_mut().get_mut::<Transform>(entity.id) {
                *t = transform.inner;
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError("Entity does not have Transform component".to_string()))
            }
        })
    })?)?;
    
    table.set("get_component_name", lua.create_function(|_, entity: LuaEntity| {
        with_world(|world| {
            world.inner()
                .get::<Name>(entity.id)
                .map(|n| n.as_str().to_string())
                .ok_or_else(|| mlua::Error::RuntimeError("Entity does not have Name component".to_string()))
        })
    })?)?;
    
    Ok(table)
}

/// Registers ECS API with the Lua environment.
pub fn register_ecs_api(lua: &Lua) -> Result<()> {
    Ok(())
}

#[derive(Clone, Copy)]
struct LuaEntity {
    id: Entity,
}

impl UserData for LuaEntity {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
            Ok(format!("Entity({})", this.id.index()))
        });
        
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaEntity| {
            Ok(this.id == other.id)
        });
    }
}

#[derive(Clone, Copy)]
struct LuaTransform {
    inner: Transform,
}

impl UserData for LuaTransform {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("translation", |lua, this| {
            let table = lua.create_table()?;
            table.set("x", this.inner.translation.x)?;
            table.set("y", this.inner.translation.y)?;
            table.set("z", this.inner.translation.z)?;
            Ok(table)
        });
        
        fields.add_field_method_set("translation", |_, this, table: Table| {
            let x: f32 = table.get("x")?;
            let y: f32 = table.get("y")?;
            let z: f32 = table.get("z")?;
            this.inner.translation = Vec3::new(x, y, z);
            Ok(())
        });
    }
    
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("translate", |_, this, (x, y, z): (f32, f32, f32)| {
            let mut transform = this.inner;
            transform.translation += Vec3::new(x, y, z);
            Ok(LuaTransform { inner: transform })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_context() {
        let lua = Lua::new();
        let mut world = World::new();
        
        set_world_context(&lua, &mut world).unwrap();
        
        let has_world: bool = lua.load("return world ~= nil").eval().unwrap();
        assert!(has_world);
        
        clear_world_context(&lua).unwrap();
        
        let has_world: bool = lua.load("return world == nil").eval().unwrap();
        assert!(has_world);
    }
}
