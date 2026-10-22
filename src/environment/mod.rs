use bevy_ecs::prelude::*;

/// The light a Scene's surroundings give, constant until ADR 0020's
/// irradiance probes (M5) and M6's atmosphere replace it: the illuminance
/// every surface receives from all around, and the luminance of the
/// background, seen wherever nothing is drawn (ADR 0023). The default is
/// dark.
#[derive(Resource, Clone, Copy, Default)]
pub struct Environment {
    pub ambient_illuminance: glam::Vec3,
    pub background_luminance: glam::Vec3,
}
