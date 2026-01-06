//! ECS systems for script execution.

use crate::context::ScriptingContext;
use crate::script_component::ScriptComponent;
use bevy_ecs::prelude::*;
use praxis_ecs::{Entity, Query};
use praxis_utils::{debug, error, info};
use std::collections::HashSet;

/// Resource that holds the scripting context.
///
/// Note: The Lua VM is not thread-safe, so this resource should only be
/// accessed from the main thread. The ECS scheduler handles this automatically.
#[derive(Resource)]
pub struct ScriptingResource {
    context: ScriptingContext,
    started_scripts: HashSet<String>,
}

// SAFETY: ScriptingContext is only accessed from the main thread via ECS systems.
// The Lua VM is not thread-safe, but bevy_ecs Resource trait requires Send + Sync.
// In practice, all scripting systems run on the main thread sequentially.
unsafe impl Send for ScriptingResource {}
unsafe impl Sync for ScriptingResource {}

impl ScriptingResource {
    /// Creates a new scripting resource.
    pub fn new(context: ScriptingContext) -> Self {
        Self {
            context,
            started_scripts: HashSet::new(),
        }
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
        debug!(
            "Initializing script '{}' on entity {:?}",
            script.name, entity
        );

        match scripting
            .context_mut()
            .load_script(&script.name, &script.script_path)
        {
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
///
/// This system uses exclusive world access to provide scripts with full ECS access.
/// It should run after `script_initialization_system`.
///
/// Note: on_start is called once per script (not per entity), when the script
/// is first loaded. Scripts can query for entities by name or other criteria.
pub fn script_start_system(world: &mut praxis_ecs::World) {
    // Extract the scripting resource temporarily
    let scripting_resource = world.inner_mut().remove_resource::<ScriptingResource>();

    let Some(mut scripting) = scripting_resource else {
        return;
    };

    // Query for entities with scripts that need on_start called
    let mut query = world.inner_mut().query_filtered::<(Entity, &ScriptComponent), (With<ScriptInitialized>, Without<ScriptStarted>)>();
    let scripts_to_start: Vec<(Entity, String)> = query
        .iter(world.inner())
        .filter(|(_, script)| !scripting.started_scripts.contains(&script.name))
        .map(|(entity, script)| (entity, script.name.clone()))
        .collect();

    // Call on_start for each unique script (only once per script name)
    let mut unique_scripts = HashSet::new();
    for (entity, script_name) in scripts_to_start {
        if unique_scripts.insert(script_name.clone()) {
            debug!("Calling on_start for script '{}'", script_name);

            let result = scripting.context.with_world(world, |lua| {
                // Try to get the on_start function
                let globals = lua.globals();
                if let Ok(on_start) = globals.get::<_, crate::mlua::Function>("on_start") {
                    // Call on_start with no arguments (scripts manage their own state)
                    on_start
                        .call::<_, ()>(())
                        .map_err(|e| praxis_utils::eyre::eyre!("Error calling on_start: {}", e))?;
                    debug!("Successfully called on_start for script '{}'", script_name);
                } else {
                    debug!("Script '{}' does not have on_start function", script_name);
                }
                Ok(())
            });

            if let Err(e) = result {
                error!("Error calling on_start for script '{}': {}", script_name, e);
            } else {
                // Mark the script as started globally
                scripting.started_scripts.insert(script_name.clone());
            }
        }

        // Mark this entity's script as started
        if let Some(mut entity_mut) = world.inner_mut().get_entity_mut(entity) {
            entity_mut.insert(ScriptStarted);
        }
    }

    // Put the scripting resource back
    world.inner_mut().insert_resource(scripting);
}

/// Marker component indicating a script's on_start has been called.
#[derive(Component)]
pub struct ScriptStarted;

/// System that calls `on_update` for all active scripts.
///
/// This system uses exclusive world access to provide scripts with full ECS access.
/// It should run during the update phase.
///
/// Note: on_update is called once per frame per unique script (not per entity).
/// Scripts receive delta_time as an argument and can query for entities as needed.
pub fn script_update_system(world: &mut praxis_ecs::World) {
    // Get delta time
    let delta_time = world
        .inner()
        .get_resource::<praxis_ecs::DeltaTime>()
        .map(|dt| dt.0)
        .unwrap_or(0.016);

    // Extract the scripting resource temporarily
    let scripting_resource = world.inner_mut().remove_resource::<ScriptingResource>();

    let Some(scripting) = scripting_resource else {
        return;
    };

    // Query for unique scripts that have been started
    let mut query = world
        .inner_mut()
        .query_filtered::<&ScriptComponent, With<ScriptStarted>>();
    let mut unique_scripts = HashSet::new();
    for script in query.iter(world.inner()) {
        unique_scripts.insert(script.name.clone());
    }

    // Call on_update for each unique script
    for script_name in unique_scripts {
        let result = scripting.context.with_world(world, |lua| {
            // Try to get the on_update function
            let globals = lua.globals();
            if let Ok(on_update) = globals.get::<_, crate::mlua::Function>("on_update") {
                // Call on_update with delta_time
                on_update
                    .call::<_, ()>(delta_time)
                    .map_err(|e| praxis_utils::eyre::eyre!("Error calling on_update: {}", e))?;
            }
            Ok(())
        });

        if let Err(e) = result {
            error!(
                "Error calling on_update for script '{}': {}",
                script_name, e
            );
        }
    }

    // Put the scripting resource back
    world.inner_mut().insert_resource(scripting);
}

/// System that processes hot-reload events for scripts.
///
/// This system should run early in the frame.
///
/// Note: When scripts are hot-reloaded, they are re-executed in the same Lua VM.
/// Currently, on_start is NOT called again after hot-reload. This behavior could
/// be changed in the future by clearing the started_scripts set when scripts reload.
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

        assert!(resource
            .context()
            .lua()
            .globals()
            .get::<_, crate::mlua::Value>("math")
            .is_ok());
    }
}
