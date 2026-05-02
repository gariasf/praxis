use bevy_ecs::prelude::*;
use winit::keyboard::KeyCode;

use crate::assets::MeshHandle;
use crate::camera::Camera;
use crate::components::{MeshRef, Transform};
use crate::input::Input;

#[derive(Resource)]
pub struct HelmetAssets {
    pub mesh: MeshHandle,
}

#[derive(Resource, Default)]
pub struct RuntimeHelmets {
    pub entities: Vec<Entity>,
}

pub fn spawn_helmet(
    mut commands: Commands,
    input: Res<Input>,
    camera: Res<Camera>,
    assets: Res<HelmetAssets>,
    mut runtime_helmets: ResMut<RuntimeHelmets>,
) {
    if !input.just_pressed.contains(&KeyCode::KeyG) {
        return;
    }

    let position = camera.position + camera.forward() * 3.0;
    let entity = commands
        .spawn((
            Transform(glam::Affine3A::from_translation(position)),
            MeshRef(assets.mesh),
        ))
        .id();

    runtime_helmets.entities.push(entity);
    tracing::info!(
        ?entity,
        ?position,
        runtime_count = runtime_helmets.entities.len(),
        "helmet spawned"
    );
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
        tracing::debug!("despawn requested but stack empty");
        return;
    };

    commands.entity(entity).despawn();
    tracing::info!(
        ?entity,
        runtime_count = runtime_helmets.entities.len(),
        "helmet despawned"
    );
}
