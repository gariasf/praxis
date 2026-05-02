use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;

#[derive(Resource, Default)]
pub struct RuntimeHelmets {
    pub entities: Vec<Entity>,
}
