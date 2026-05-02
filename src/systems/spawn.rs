use bevy_ecs::system::{Commands, Res, ResMut};
use winit::keyboard::KeyCode;

use crate::components::{MeshRef, Transform};
use crate::resources::{Camera, HelmetHandles, Input, RuntimeHelmets};

pub fn spawn_helmet(
    mut commands: Commands,
    input: Res<Input>,
    camera: Res<Camera>,
    handles: Res<HelmetHandles>,
    mut runtime_helmets: ResMut<RuntimeHelmets>,
) {
    if !input.just_pressed.contains(&KeyCode::KeyG) {
        return;
    }

    let position = camera.position + camera.forward() * 3.0;
    let entity = commands
        .spawn((
            Transform(glam::Affine3A::from_translation(position)),
            MeshRef(handles.mesh),
        ))
        .id();

    runtime_helmets.entities.push(entity);
    tracing::info!("spawned helmet {entity:?} at {position:?}");
}

pub fn despawn_helmet(
    mut commands: Commands,
    input: Res<Input>,
    mut runtime_helmets: ResMut<RuntimeHelmets>,
) {
    if !input.just_pressed.contains(&KeyCode::KeyH) {
        return;
    }

    let Some(entity) = runtime_helmets.entities.pop() else {
        return;
    };

    commands.entity(entity).despawn();
    tracing::info!("despawned helmet {entity:?}");
}
