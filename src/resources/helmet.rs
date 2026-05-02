use bevy_ecs::resource::Resource;

use crate::assets::MeshHandle;

#[derive(Resource)]
pub struct HelmetHandles {
    pub mesh: MeshHandle,
}
