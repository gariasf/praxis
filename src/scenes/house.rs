#![cfg_attr(not(feature = "cook"), allow(dead_code))]

use std::f32::consts::FRAC_PI_2;
use std::ops::Range;

use bevy_ecs::prelude::*;
#[cfg(feature = "cook")]
use glam::dvec3;
use glam::{DVec3, FloatExt};

use super::ReferenceScene;
use crate::assets::AssetLoader;
use crate::camera::Camera;
use crate::components::{DirectionalLight, MeshRef, PointLight, Transform};
#[cfg(feature = "cook")]
use crate::cooked::{CookedLight, CookedPlacement};
use crate::time::{FixedTime, TICK_SECONDS};
use crate::transform::{PreviousWorldTransform, Teleported};

/// The Tick at whose end room A's lamp comes on.
pub const LAMP_SWITCHES_ON_AT_TICK: u64 = 128;
/// The Tick at whose end room A's door stands open; it is shut until then.
pub const DOOR_OPENS_AT_TICK: u64 = 256;
/// The Tick at whose end the sun jumps `SUN_STEP_RADIANS` further west.
pub const SUN_STEPS_AT_TICK: u64 = 1536;

// Every pivot lies in Chunk (0, 0), the one Chunk `chunk_coordinates` lists for the scene.
const HOUSE_CENTRE: DVec3 = DVec3::new(32.0, 0.0, 32.0);

#[cfg(feature = "cook")]
const MATERIALS: [(&str, [f32; 3], f32); 6] = [
    ("plaster", [0.65, 0.62, 0.56], 0.9),
    ("boards", [0.35, 0.24, 0.14], 0.7),
    ("grass", [0.10, 0.26, 0.06], 0.95),
    ("wood", [0.30, 0.19, 0.10], 0.8),
    ("brick", [0.45, 0.20, 0.14], 0.9),
    ("paint", [0.55, 0.04, 0.03], 0.35),
];

const SHARED_WALL_FACE_X: f64 = 0.05;
const END_WALL_INNER_X: f64 = 4.05;
const END_WALL_OUTER_X: f64 = 4.25;
const SIDE_WALL_INNER_Z: f64 = 2.0;
const SIDE_WALL_OUTER_Z: f64 = 2.2;
const FLOOR_TOP_Y: f64 = 0.05;
const WALL_TOP_Y: f64 = 2.65;
const ROOF_TOP_Y: f64 = 2.85;
const DOORWAY_X: Range<f64> = -2.5..-1.6;
const DOORWAY_TOP_Y: f64 = 2.05;
const WINDOW_X: Range<f64> = 2.0..3.2;
const WINDOW_Y: Range<f64> = 0.95..2.15;
// The leaf laps the doorway's jambs and head by 0.05 m and stands on the floor, so closed it shuts room A.
const DOOR_LEAF_METRES: glam::Vec3 = glam::Vec3::new(1.0, 2.05, 0.04);
const DOOR_HINGE: DVec3 = DVec3::new(-1.55, FLOOR_TOP_Y, SIDE_WALL_INNER_Z);
const BALL_RADIUS_METRES: f32 = 0.3;
const YARD_HALF_SIZE_METRES: f32 = 30.0;

pub(super) const SUN_DIRECTION: glam::Vec3 = glam::Vec3::new(-0.4, -0.6, -0.7);
/// Turns about +Y that carry the sun west, as across a southern sky: half a
/// degree a second, and a step of 30°, which moves room B's sun patch off
/// its place.
const SUN_RADIANS_PER_SECOND: f64 = -std::f64::consts::PI / 360.0;
const SUN_STEP_RADIANS: f64 = -std::f64::consts::FRAC_PI_6;
const LAMP_POSITION: DVec3 = DVec3::new(-3.3, 2.25, 1.4);
const LAMP_LIGHT: PointLight = PointLight {
    color: glam::Vec3::new(1.0, 0.85, 0.65),
    intensity: 0.1 * super::SUN_ILLUMINANCE,
    range: 8.0,
};

struct PathKey {
    frame: u32,
    position: DVec3,
    yaw_degrees: f32,
    pitch_degrees: f32,
    stops_under_the_yard: f32,
}

const ROOM_A_VIEW: PathKey = PathKey {
    frame: 0,
    position: DVec3::new(-1.6, 1.65, -1.3),
    yaw_degrees: -255.0,
    pitch_degrees: -17.0,
    stops_under_the_yard: 4.0,
};
const ROOM_B_VIEW: PathKey = PathKey {
    frame: 1320,
    position: DVec3::new(0.7, 1.65, -1.6),
    yaw_degrees: -296.0,
    pitch_degrees: -19.0,
    stops_under_the_yard: 4.0,
};

// Still in room A over the lamp's and the door's Ticks, and in room B over the sun's step.
const PATH: [PathKey; 10] = [
    ROOM_A_VIEW,
    PathKey {
        frame: 330,
        ..ROOM_A_VIEW
    },
    PathKey {
        frame: 420,
        position: DVec3::new(-2.05, 1.65, 1.0),
        yaw_degrees: -270.0,
        pitch_degrees: -5.0,
        stops_under_the_yard: 4.0,
    },
    PathKey {
        frame: 510,
        position: DVec3::new(-2.05, 1.65, 3.6),
        yaw_degrees: -270.0,
        pitch_degrees: -5.0,
        stops_under_the_yard: 0.0,
    },
    PathKey {
        frame: 750,
        position: DVec3::new(1.0, 1.65, 14.0),
        yaw_degrees: -455.0,
        pitch_degrees: -2.0,
        stops_under_the_yard: 0.0,
    },
    PathKey {
        frame: 990,
        position: DVec3::new(0.5, 1.65, 3.6),
        yaw_degrees: -450.0,
        pitch_degrees: -5.0,
        stops_under_the_yard: 0.0,
    },
    PathKey {
        frame: 1080,
        position: DVec3::new(2.6, 1.65, 3.6),
        yaw_degrees: -450.0,
        pitch_degrees: -5.0,
        stops_under_the_yard: 0.0,
    },
    PathKey {
        frame: 1170,
        position: DVec3::new(2.6, 1.65, 1.0),
        yaw_degrees: -450.0,
        pitch_degrees: -10.0,
        stops_under_the_yard: 4.0,
    },
    ROOM_B_VIEW,
    PathKey {
        frame: 1560,
        ..ROOM_B_VIEW
    },
];

fn path_segment(frame: u32) -> (&'static PathKey, &'static PathKey, f64) {
    let later = |key: &PathKey| key.frame > frame;
    let next = PATH.iter().position(later).unwrap_or(PATH.len() - 1);
    let (from, to) = (&PATH[next - 1], &PATH[next]);
    let progress = f64::from(frame.min(to.frame) - from.frame) / f64::from(to.frame - from.frame);
    (from, to, progress)
}

pub(super) fn camera_at(frame: u32) -> Camera {
    let (from, to, progress) = path_segment(frame);
    let turned = |from_degrees: f32, to_degrees: f32| {
        from_degrees.lerp(to_degrees, progress as f32).to_radians()
    };
    Camera {
        position: HOUSE_CENTRE + from.position.lerp(to.position, progress),
        yaw: turned(from.yaw_degrees, to.yaw_degrees),
        pitch: turned(from.pitch_degrees, to.pitch_degrees),
        ..Camera::new()
    }
}

pub(super) fn stops_under_the_yard(frame: u32) -> f32 {
    let (from, to, progress) = path_segment(frame);
    from.stops_under_the_yard
        .lerp(to.stops_under_the_yard, progress as f32)
}

/// Room A's door, shut until `DOOR_OPENS_AT_TICK`.
#[derive(Component)]
pub(crate) struct Door;

/// Room A's lamp, dark until `LAMP_SWITCHES_ON_AT_TICK`.
#[derive(Component)]
pub(crate) struct Lamp;

/// house's sun, turned west each Tick and stepped at `SUN_STEPS_AT_TICK`.
#[derive(Component)]
pub(crate) struct TurningSun;

/// Spawns the door, shut, and the lamp, dark, and makes the scene's sun,
/// spawned before, turn.
pub(super) fn spawn_house(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    let door = loader.mesh(ReferenceScene::House.mesh_id("door"))?;
    world.spawn((
        Transform::from_translation(HOUSE_CENTRE + DOOR_HINGE),
        MeshRef(door),
        PreviousWorldTransform::default(),
        Door,
    ));
    world.spawn((
        Transform::from_translation(HOUSE_CENTRE + LAMP_POSITION),
        Lamp,
    ));
    let mut suns = world.query_filtered::<Entity, With<DirectionalLight>>();
    let sun = (suns.single(world)).map_err(|error| anyhow::anyhow!("house's sun: {error}"))?;
    world
        .entity_mut(sun)
        .insert((TurningSun, PreviousWorldTransform::default()));
    Ok(())
}

/// Sets the sun's direction for the end of the running Tick, from the Tick
/// count, so no rounding accumulates over Ticks; its step is a jump.
pub(super) fn turn_sun(
    mut commands: Commands,
    fixed_time: Res<FixedTime>,
    mut suns: Query<(Entity, &mut Transform), With<TurningSun>>,
) {
    let tick = fixed_time.tick;
    let stepped = if tick >= SUN_STEPS_AT_TICK {
        SUN_STEP_RADIANS
    } else {
        0.0
    };
    let radians = SUN_RADIANS_PER_SECOND * (tick + 1) as f64 * TICK_SECONDS + stepped;
    let direction = glam::Quat::from_rotation_y(radians as f32) * SUN_DIRECTION.normalize();
    for (entity, mut transform) in &mut suns {
        transform.rotation = glam::Quat::from_rotation_arc(glam::Vec3::NEG_Z, direction);
        if tick == SUN_STEPS_AT_TICK {
            commands.entity(entity).insert(Teleported);
        }
    }
}

/// Opens the door, a quarter turn into room A, at `DOOR_OPENS_AT_TICK`, as a jump.
pub(super) fn open_door(
    mut commands: Commands,
    fixed_time: Res<FixedTime>,
    mut doors: Query<(Entity, &mut Transform), With<Door>>,
) {
    if fixed_time.tick != DOOR_OPENS_AT_TICK {
        return;
    }
    for (entity, mut transform) in &mut doors {
        transform.rotation = glam::Quat::from_rotation_y(-FRAC_PI_2);
        commands.entity(entity).insert(Teleported);
    }
}

/// Switches the lamp on at `LAMP_SWITCHES_ON_AT_TICK`.
pub(super) fn switch_on_lamp(
    mut commands: Commands,
    fixed_time: Res<FixedTime>,
    lamps: Query<Entity, With<Lamp>>,
) {
    if fixed_time.tick != LAMP_SWITCHES_ON_AT_TICK {
        return;
    }
    for lamp in &lamps {
        commands.entity(lamp).insert(LAMP_LIGHT);
    }
}

#[cfg(feature = "cook")]
fn block(material: &str, x: Range<f64>, y: Range<f64>, z: Range<f64>) -> CookedPlacement {
    let scene = ReferenceScene::House;
    let foot = dvec3((x.start + x.end) / 2.0, y.start, (z.start + z.end) / 2.0);
    let size = dvec3(x.end - x.start, y.end - y.start, z.end - z.start);
    CookedPlacement {
        scale: size.as_vec3(),
        ..super::placed(
            scene.mesh_id("box"),
            Some(scene.material_id(material)),
            HOUSE_CENTRE + foot,
        )
    }
}

#[cfg(feature = "cook")]
pub(super) fn static_content() -> (Vec<CookedPlacement>, Vec<CookedLight>) {
    let scene = ReferenceScene::House;
    let (shared_x, inner_x, outer_x) = (SHARED_WALL_FACE_X, END_WALL_INNER_X, END_WALL_OUTER_X);
    let (inner_z, outer_z) = (SIDE_WALL_INNER_Z, SIDE_WALL_OUTER_Z);
    let (floor_top, wall_top) = (FLOOR_TOP_Y, WALL_TOP_Y);
    let ball_height = f64::from(BALL_RADIUS_METRES);
    let end_wall = |x: Range<f64>| block("plaster", x, 0.0..wall_top, -inner_z..inner_z);
    let south_wall = |x: Range<f64>, y: Range<f64>| block("plaster", x, y, inner_z..outer_z);
    let floor = |x: Range<f64>| block("boards", x, 0.0..floor_top, -inner_z..inner_z);
    let crate_on = |x: f64, y: f64, z: f64, side: f64| {
        let half = side / 2.0;
        block("wood", x - half..x + half, y..y + side, z - half..z + half)
    };
    let prop =
        |mesh, x, y, z| super::placed(scene.mesh_id(mesh), None, HOUSE_CENTRE + dvec3(x, y, z));
    let north_wall = block(
        "plaster",
        -outer_x..outer_x,
        0.0..wall_top,
        -outer_z..-inner_z,
    );
    let roof = block(
        "plaster",
        -outer_x..outer_x,
        wall_top..ROOF_TOP_Y,
        -outer_z..outer_z,
    );
    let freestanding_wall = block("brick", 2.6..2.7, 0.0..2.0, 4.5..7.5);
    let placements = vec![
        prop("yard", 0.0, 0.0, 0.0),
        north_wall,
        end_wall(-outer_x..-inner_x),
        end_wall(-shared_x..shared_x),
        end_wall(inner_x..outer_x),
        south_wall(-outer_x..DOORWAY_X.start, 0.0..wall_top),
        south_wall(DOORWAY_X, DOORWAY_TOP_Y..wall_top),
        south_wall(DOORWAY_X.end..WINDOW_X.start, 0.0..wall_top),
        south_wall(WINDOW_X, 0.0..WINDOW_Y.start),
        south_wall(WINDOW_X, WINDOW_Y.end..wall_top),
        south_wall(WINDOW_X.end..outer_x, 0.0..wall_top),
        roof,
        floor(-inner_x..-shared_x),
        floor(shared_x..inner_x),
        freestanding_wall,
        crate_on(-3.2, 0.0, 6.5, 0.8),
        crate_on(-3.0, floor_top, 0.5, 0.5),
        crate_on(0.5, floor_top, 1.5, 0.5),
        prop("ball", 1.2, ball_height, 8.5),
        prop("ball", 3.3, floor_top + ball_height, 1.2),
        prop("table", -3.3, floor_top, 1.5),
    ];
    (placements, Vec::new())
}

#[cfg(feature = "cook")]
pub(super) fn materials() -> Vec<(String, crate::cooked::CookedMaterial)> {
    MATERIALS
        .into_iter()
        .map(|(name, [red, green, blue], roughness)| {
            let material = crate::cooked::CookedMaterial {
                base_color: [red, green, blue, 1.0],
                metallic: 0.0,
                roughness,
                emissive: [0.0; 3],
                normal_scale: 1.0,
                textures: [None; 5],
            };
            (name.to_owned(), material)
        })
        .collect()
}

#[cfg(feature = "cook")]
pub(super) fn meshes() -> Vec<(&'static str, Vec<super::ScenePrimitive>)> {
    use glam::vec3;

    use super::town::{cuboid, merged};

    let table_legs = [(-0.55, -0.3), (0.55, -0.3), (-0.55, 0.3), (0.55, 0.3)].map(|(x, z)| {
        cuboid(
            vec3(x - 0.025, 0.0, z - 0.025),
            vec3(x + 0.025, 0.71, z + 0.025),
        )
    });
    let table_top = cuboid(vec3(-0.6, 0.71, -0.35), vec3(0.6, 0.75, 0.35));
    let table = merged(table_legs.into_iter().chain([table_top]));
    let door_leaf = cuboid(
        vec3(-DOOR_LEAF_METRES.x, 0.0, -DOOR_LEAF_METRES.z),
        vec3(0.0, DOOR_LEAF_METRES.y, 0.0),
    );
    let yard = super::ground_plane(YARD_HALF_SIZE_METRES, 1.0);
    let unit_box = cuboid(vec3(-0.5, 0.0, -0.5), vec3(0.5, 1.0, 0.5));
    vec![
        ("yard", vec![(yard, Some("grass"))]),
        ("box", vec![(unit_box, None)]),
        ("door", vec![(door_leaf, Some("wood"))]),
        ("table", vec![(table, Some("wood"))]),
        (
            "ball",
            vec![(super::scene_sphere(BALL_RADIUS_METRES), Some("paint"))],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cameras_ev100_is_the_sunlit_scenes_in_the_yard_and_four_stops_under_it_in_each_room() {
        let ev100 = |frame| ReferenceScene::House.camera_at(frame, DVec3::ZERO).ev100;
        let within = |actual: f32, expected: f32| (actual - expected).abs() < 1e-3;
        assert!(within(ev100(59), 12.3466), "room A: {}", ev100(59));
        assert!(within(ev100(750), 16.3466), "yard: {}", ev100(750));
        assert!(within(ev100(1320), 12.3466), "room B: {}", ev100(1320));
    }
}
