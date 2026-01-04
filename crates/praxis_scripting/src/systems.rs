//! ECS systems for script execution.

use crate::context::ScriptingContext;
use crate::script_component::ScriptComponent;
use bevy_ecs::prelude::*;
use praxis_ecs::{Entity, Query, World as PraxisWorld};
use praxis_utils::{debug, error, info};

/// Resource that holds the scripting context.
/// 
/// Note: The Lua VM is not thread-safe, so this resource should only be
/// accessed from the main thread. The ECS scheduler handles this automatically.
pub struct ScriptingResource {
    context: ScriptingContext,
}

// SAFETY: ScriptingContext is only accessed from the main thread via ECS systems.
// The Lua VM is not thread-safe, but bevy_ecs Resource trait requires Send + Sync.
// In practice, all scripting systems run on the main thread sequentially.
unsafe impl Send for ScriptingResource {}
unsafe impl Sync for ScriptingResource {}

impl ScriptingResource {
    /// Creates a new scripting resource.
    pub fn new(context: ScriptingContext) -> Self {
        Self { context }
    }
    
    /// Gets a reference to the scripting context.
    pub fn context(&self) -> &ScriptingContext {
        &self.context
    }
    
    /// Gets a mutable reference to the scripting context.
    pub fn context_mut(&mut self) -> &mut ScriptingContext {
        &mut self.context
    }
}

/// System that initializes scripts on entities with `ScriptComponent`.
///
/// This system should run early in the frame, before update systems.
pub fn script_initialization_system(
    mut scripting: ResMut<ScriptingResource>,
    mut query: Query<(Entity, &mut ScriptComponent), Without<ScriptInitialized>>,
    mut commands: Commands,
) {
    for (entity, mut script) in query.iter_mut() {
        debug!("Initializing script '{}' on entity {:?}", script.name, entity);
        
        match scripting.context_mut().load_script(&script.name, &script.script_path) {
            Ok(_) => {
                script.initialized = true;
                commands.entity(entity).insert(ScriptInitialized);
                info!("Script '{}' initialized successfully", script.name);
            }
            Err(e) => {
                error!("Failed to initialize script '{}': {}", script.name, e);
            }
        }
    }
}

/// Marker component indicating a script has been initialized.
#[derive(Component)]
pub struct ScriptInitialized;

/// System that calls `on_start` for newly initialized scripts.
pub fn script_start_system(
    scripting: Res<ScriptingResource>,
    mut query: Query<(Entity, &ScriptComponent), (With<ScriptInitialized>, Without<ScriptStarted>)>,
    mut commands: Commands,
    mut world: ResMut<PraxisWorld>,
) {
    for (entity, script) in query.iter() {
        debug!("Calling on_start for script '{}'", script.name);
        
        let result = scripting.context().with_world(&mut world, |lua| {
            let globals = lua.globals();
            
            if let Ok(on_start) = globals.get::<_, mlua::Function>("on_start") {
                on_start.call::<_, ()>(()).map_err(|e| {
                    praxis_utils::eyre::eyre!("Error in on_start: {}", e)
                })?;
            }
            
            Ok(())
        });
        
        match result {
            Ok(_) => {
                commands.entity(entity).insert(ScriptStarted);
            }
            Err(e) => {
                error!("Error calling on_start for '{}': {}", script.name, e);
            }
        }
    }
}

/// Marker component indicating a script's on_start has been called.
#[derive(Component)]
pub struct ScriptStarted;

/// System that calls `on_update` for all active scripts.
///
/// This system should run every frame.
pub fn script_update_system(
    scripting: Res<ScriptingResource>,
    query: Query<&ScriptComponent, With<ScriptStarted>>,
    mut world: ResMut<PraxisWorld>,
    time: Res<praxis_ecs::DeltaTime>,
) {
    let delta = time.0;
    
    for script in query.iter() {
        let result = scripting.context().with_world(&mut world, |lua| {
            let globals = lua.globals();
            
            if let Ok(on_update) = globals.get::<_, mlua::Function>("on_update") {
                on_update.call::<_, ()>(delta).map_err(|e| {
                    praxis_utils::eyre::eyre!("Error in on_update: {}", e)
                })?;
            }
            
            Ok(())
        });
        
        if let Err(e) = result {
            error!("Error calling on_update for '{}': {}", script.name, e);
        }
    }
}

/// System that processes hot-reload events for scripts.
///
/// This system should run early in the frame.
pub fn script_hot_reload_system(mut scripting: ResMut<ScriptingResource>) {
    if let Err(e) = scripting.context_mut().process_hot_reload() {
        error!("Error processing hot-reload: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScriptingConfig, ScriptingContext};
    
    #[test]
    fn test_scripting_resource() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();
        let resource = ScriptingResource::new(context);
        
        assert!(resource.context().lua().globals().get::<_, mlua::Value>("math").is_ok());
    }
}
