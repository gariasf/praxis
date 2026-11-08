//! `world`, the streamed Reference scene: a Game world of 40 × 40 Chunks of
//! flat ground, 10.24 km on a side (ADR 0013), with a settlement in each of
//! its 8 × 8 regions and sparse farms between them. Each settlement clads
//! its walls in brick Textures of its own, so the scene's library outgrows
//! the texture pool and freeing and eviction run. Every Instance is placed
//! from a hash of where it stands, so every cook places the same Game world.

#![cfg_attr(not(feature = "cook"), allow(dead_code))]

use bevy_ecs::prelude::*;

use super::town::{FlockGrid, random, spawn_flocks};
use super::{ReferenceScene, WrapsAroundCamera};
use crate::assets::AssetLoader;
use crate::camera::Camera;
use crate::cooked::CHUNK_EDGE_METRES;
#[cfg(feature = "cook")]
use crate::cooked::chunk_origin;
#[cfg(feature = "cook")]
use crate::cooked::{CookedLight, CookedPlacement};

/// Chunks along each side, centred on the origin.
pub(super) const CHUNKS_PER_SIDE: i32 = 40;
const WORLD_HALF_SIZE_METRES: f64 = CHUNKS_PER_SIDE as f64 * CHUNK_EDGE_METRES / 2.0;
const GROUND_TILE_METRES: f64 = 128.0;
const GROUND_TILES_PER_SIDE: u32 = (2.0 * WORLD_HALF_SIZE_METRES / GROUND_TILE_METRES) as u32;

const SETTLEMENTS_PER_SIDE: u32 = 8;
const REGION_METRES: f64 = 2.0 * WORLD_HALF_SIZE_METRES / SETTLEMENTS_PER_SIDE as f64;
const BLOCKS_PER_SIDE: u32 = 13;
const LOTS_PER_BLOCK_SIDE: u32 = 4;
const LOT_PITCH_METRES: f64 = 12.0;
const STREET_WIDTH_METRES: f64 = 10.0;
const BLOCK_PITCH_METRES: f64 = LOTS_PER_BLOCK_SIDE as f64 * LOT_PITCH_METRES + STREET_WIDTH_METRES;
const SETTLEMENT_HALF_SIZE_METRES: f64 = BLOCKS_PER_SIDE as f64 * BLOCK_PITCH_METRES / 2.0;
/// The most a settlement's centre strays from its region's, which keeps
/// 64 m of open land between neighbouring settlements.
const SETTLEMENT_STRAY_METRES: f64 = REGION_METRES / 2.0 - SETTLEMENT_HALF_SIZE_METRES - 32.0;

/// The settlement whose region the camera flies into, column then row.
const CAMERA_SETTLEMENT: [u32; 2] = [4, 4];
/// The camera flies north along a street, over the roofs and the birds, at
/// 50 m/s, from farmland `APPROACH_METRES` south of its settlement's edge,
/// looking at the ground `GAZE_METRES` ahead.
const CAMERA_HEIGHT_METRES: f64 = 60.0;
const CAMERA_METRES_PER_FRAME: f64 = 50.0 / 60.0;
const APPROACH_METRES: f64 = 300.0;
const GAZE_METRES: f64 = 250.0;
/// The bench path's frames, 60 s at 60 Hz, holds aside.
pub(crate) const BENCH_SEGMENT_FRAMES: u32 = 3600;
/// The bench path's Fast travel: on its frame, over the farmland north of
/// `CAMERA_SETTLEMENT`, the camera jumps about 5 km to the middle of
/// another settlement, flying on north through it.
const FAST_TRAVEL_FRAME: u32 = 1800;
const FAST_TRAVEL_SETTLEMENT: [u32; 2] = [1, 6];
/// The traversal flies as the bench path does, half a Chunk in from the
/// world's edge: west to east along its south side, then, after a Fast
/// travel to its north-east corner, its diagonal to the south-west corner.
const TRAVERSAL_CORNER_METRES: f64 = WORLD_HALF_SIZE_METRES - CHUNK_EDGE_METRES / 2.0;
const TRAVERSAL_SIDE_FRAMES: u32 = (2.0 * TRAVERSAL_CORNER_METRES / CAMERA_METRES_PER_FRAME) as u32;
pub(crate) const TRAVERSAL_FRAMES: u32 =
    TRAVERSAL_SIDE_FRAMES + (TRAVERSAL_SIDE_FRAMES as f64 * std::f64::consts::SQRT_2) as u32;

/// The flocks about the camera: 320 m a side, centres between the roofs and
/// the camera, each kept within 160 m of the camera on X and Z, so that no
/// bird flies past the Load radius.
const FLOCK_PITCH_METRES: f64 = 32.0;
const FLOCK_HEIGHT_METRES: std::ops::Range<f64> = 34.0..50.0;
const FLOCK_RADIUS_METRES: std::ops::Range<f64> = 6.0..14.0;

/// Each settlement's own brick Materials, one Texture each.
const FACADES_PER_SETTLEMENT: u32 = 24;
const FACADE_TEXTURE_SIDE: u32 = 2048;
/// The wall each facade Texture covers before it repeats, in metres.
const FACADE_METRES: f32 = 4.0;
const BRICK_TEXELS: [u32; 2] = [64, 24];
const MORTAR_TEXELS: u32 = 3;

const STOREY_HEIGHT_METRES: f32 = 3.2;
const WINDOW_PITCH_METRES: f32 = 1.6;
const WINDOW_SIZE_METRES: glam::Vec3 = glam::Vec3::new(1.2, 1.6, 0.12);
const WINDOW_SILL_HEIGHT_METRES: f32 = 0.9;
const ROOF_OVERHANG_METRES: f32 = 0.3;
const ROOF_THICKNESS_METRES: f32 = 0.3;

/// A chance per ground tile outside the settlements of holding a farm.
const FARM_CHANCE: f64 = 0.25;

/// A building's shape: its walls' Mesh, its roof's, its footprint across X
/// and Z, and its storeys.
struct Building {
    walls: &'static str,
    roof: &'static str,
    width: f32,
    depth: f32,
    storeys: u32,
}

const BUILDINGS: [Building; 8] = [
    Building::new("cottage", "cottage roof", 7.0, 6.0, 2),
    Building::new("house", "house roof", 8.0, 8.0, 3),
    Building::new("terrace", "terrace roof", 10.0, 6.0, 4),
    Building::new("hall", "hall roof", 10.0, 9.0, 3),
    Building::new("townhouse", "townhouse roof", 6.0, 9.0, 6),
    Building::new("tower", "tower roof", 8.0, 8.0, 9),
    Building::new("shop", "shop roof", 9.0, 7.0, 2),
    Building::new("block", "block roof", 10.0, 10.0, 8),
];

/// The building each farm is.
const FARM: usize = 3;

impl Building {
    const fn new(
        walls: &'static str,
        roof: &'static str,
        width: f32,
        depth: f32,
        storeys: u32,
    ) -> Self {
        Self {
            walls,
            roof,
            width,
            depth,
            storeys,
        }
    }

    fn height(&self) -> f32 {
        self.storeys as f32 * STOREY_HEIGHT_METRES
    }
}

/// What a random value decides, so that no two values for one place share
/// a seed.
#[derive(Clone, Copy)]
enum Choice {
    SettlementX,
    SettlementZ,
    Building,
    Facade,
    Yaw,
    Farm,
    FarmX,
    FarmZ,
    Plaster,
    Brick,
}

/// The name of facade `facade` of settlement `settlement`, which its
/// Material and Texture share.
fn facade_name(settlement: u32, facade: u32) -> String {
    format!("settlement {settlement} facade {facade}")
}

/// The centre of the settlement in region `[column, row]`, on the ground.
fn settlement_centre([column, row]: [u32; 2]) -> glam::DVec3 {
    let region_centre =
        |index: u32| -WORLD_HALF_SIZE_METRES + (f64::from(index) + 0.5) * REGION_METRES;
    let stray = |choice: Choice| {
        let value = random([column.into(), row.into(), choice as u64, 0]);
        SETTLEMENT_STRAY_METRES * (2.0 * value - 1.0)
    };
    glam::dvec3(
        region_centre(column) + stray(Choice::SettlementX),
        0.0,
        region_centre(row) + stray(Choice::SettlementZ),
    )
}

fn settlement_index([column, row]: [u32; 2]) -> u32 {
    row * SETTLEMENTS_PER_SIDE + column
}

/// The camera at `frame` of its path, the bench path: into a settlement,
/// over the farmland beyond, then a Fast travel into another settlement.
pub(crate) fn camera_at(frame: u32) -> Camera {
    let (street_x, start_z, frames_flown) = if frame < FAST_TRAVEL_FRAME {
        let centre = settlement_centre(CAMERA_SETTLEMENT);
        let start_z = centre.z + SETTLEMENT_HALF_SIZE_METRES + APPROACH_METRES;
        (centre.x - BLOCK_PITCH_METRES / 2.0, start_z, frame)
    } else {
        let centre = settlement_centre(FAST_TRAVEL_SETTLEMENT);
        let frames_flown = frame - FAST_TRAVEL_FRAME;
        (centre.x - BLOCK_PITCH_METRES / 2.0, centre.z, frames_flown)
    };
    let position = glam::dvec3(
        street_x,
        CAMERA_HEIGHT_METRES,
        start_z - CAMERA_METRES_PER_FRAME * f64::from(frames_flown),
    );
    let gaze = glam::dvec3(0.0, -CAMERA_HEIGHT_METRES, -GAZE_METRES);
    Camera {
        teleported: frame == FAST_TRAVEL_FRAME,
        ..Camera::looking_at(position, position + gaze)
    }
}

/// The camera at `frame` of the traversal, exposed as the scene's path is.
pub(crate) fn traversal_camera_at(frame: u32) -> Camera {
    let corner = TRAVERSAL_CORNER_METRES;
    let (start, end, frames_flown) = if frame < TRAVERSAL_SIDE_FRAMES {
        (
            glam::dvec3(-corner, 0.0, corner),
            glam::dvec3(corner, 0.0, corner),
            frame,
        )
    } else {
        let frames_flown = frame - TRAVERSAL_SIDE_FRAMES;
        (
            glam::dvec3(corner, 0.0, -corner),
            glam::dvec3(-corner, 0.0, corner),
            frames_flown,
        )
    };
    let heading = (end - start).normalize();
    let flown = heading * (CAMERA_METRES_PER_FRAME * f64::from(frames_flown));
    let position = start + flown + glam::DVec3::Y * CAMERA_HEIGHT_METRES;
    let gaze = heading * GAZE_METRES - glam::DVec3::Y * CAMERA_HEIGHT_METRES;
    Camera {
        teleported: frame == TRAVERSAL_SIDE_FRAMES,
        ev100: ReferenceScene::World.ev100(),
        ..Camera::looking_at(position, position + gaze)
    }
}

/// Spawns 10,000 birds in flocks about the camera's first position, each
/// flock kept within half the flocks' span of the camera as it flies.
pub(crate) fn spawn_world(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    let bird = loader.mesh(ReferenceScene::World.mesh_id("bird"))?;
    let grid = FlockGrid {
        middle: camera_at(0).position * glam::DVec3::new(1.0, 0.0, 1.0),
        pitch_metres: FLOCK_PITCH_METRES,
        height_metres: FLOCK_HEIGHT_METRES,
        radius_metres: FLOCK_RADIUS_METRES,
    };
    let span_metres = grid.span_metres();
    for centre in spawn_flocks(world, bird, &grid) {
        world
            .entity_mut(centre)
            .insert(WrapsAroundCamera { span_metres });
    }
    Ok(())
}

#[cfg(feature = "cook")]
pub(crate) use generators::{materials, meshes, static_content, static_content_reaching, textures};

#[cfg(feature = "cook")]
mod generators {
    use std::sync::Arc;

    use glam::{Quat, Vec3, vec3};

    use super::*;
    use crate::assets::MaterialChannel;
    use crate::cook::{ImageGenerator, LoadedImage};
    use crate::cooked::CookedMaterial;
    use crate::scenes::town::bird;
    use crate::scenes::{Geometry, ScenePrimitive, ground_plane, ground_tile_centres};

    /// The Materials `world`'s Meshes name: a base colour in linear sRGB
    /// and a roughness.
    const MATERIALS: [(&str, [f32; 3], f32); 3] = [
        ("grass", [0.10, 0.26, 0.06], 0.95),
        ("slate", [0.32, 0.36, 0.41], 0.80),
        ("glass", [0.04, 0.05, 0.07], 0.15),
    ];

    /// Wall colours in sRGB, which each facade Texture varies.
    const PLASTERS: [[u8; 3]; 6] = [
        [196, 150, 92],
        [222, 210, 180],
        [190, 120, 110],
        [150, 70, 50],
        [150, 150, 145],
        [200, 180, 140],
    ];

    pub(crate) fn materials() -> Vec<(String, CookedMaterial)> {
        let factors = |base_color: [f32; 3], roughness, albedo| CookedMaterial {
            base_color: [base_color[0], base_color[1], base_color[2], 1.0],
            metallic: 0.0,
            roughness,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [albedo, None, None, None, None],
        };
        let named = MATERIALS.iter().map(|&(name, base_color, roughness)| {
            (name.to_owned(), factors(base_color, roughness, None))
        });
        let facades = facade_names().map(|name| {
            let albedo = ReferenceScene::World.texture_id(MaterialChannel::Albedo, &name);
            let material = factors([1.0; 3], 0.85, Some(albedo));
            (name, material)
        });
        named.chain(facades).collect()
    }

    pub(crate) fn textures() -> Vec<(String, MaterialChannel, ImageGenerator)> {
        facade_names()
            .enumerate()
            .map(|(seed, name)| {
                let generate: ImageGenerator = Arc::new(move || facade_texture(seed as u64));
                (name, MaterialChannel::Albedo, generate)
            })
            .collect()
    }

    fn facade_names() -> impl Iterator<Item = String> {
        let settlements = SETTLEMENTS_PER_SIDE * SETTLEMENTS_PER_SIDE;
        (0..settlements).flat_map(|settlement| {
            (0..FACADES_PER_SETTLEMENT).map(move |facade| facade_name(settlement, facade))
        })
    }

    /// Brick courses in mortar over a plaster colour, each brick a shade of
    /// its own, all from `seed`, top row first.
    fn facade_texture(seed: u64) -> LoadedImage {
        let side = FACADE_TEXTURE_SIDE;
        let pick = |choice: Choice, part: u64| random([seed, choice as u64, part, 0]);
        let plaster = PLASTERS[(pick(Choice::Plaster, 0) * PLASTERS.len() as f64) as usize];
        let plaster = plaster.map(|channel| {
            let jitter = 24.0 * (pick(Choice::Plaster, 1) - 0.5);
            (f64::from(channel) + jitter).clamp(0.0, 255.0)
        });
        let mortar = [205u8, 200, 190, 255];
        let [brick_width, brick_height] = BRICK_TEXELS;
        let mut pixels = Vec::with_capacity((side * side * 4) as usize);
        for y in 0..side {
            let course = y / brick_height;
            let shift = course % 2 * brick_width / 2;
            for x in 0..side {
                let along = x + shift;
                if y % brick_height < MORTAR_TEXELS || along % brick_width < MORTAR_TEXELS {
                    pixels.extend(mortar);
                    continue;
                }
                let brick = u64::from(along / brick_width);
                let shade = 0.8 + 0.3 * random([seed, Choice::Brick as u64, course.into(), brick]);
                let [red, green, blue] = plaster.map(|channel| (channel * shade).min(255.0) as u8);
                pixels.extend([red, green, blue, 255]);
            }
        }
        LoadedImage {
            pixels,
            width: side,
            height: side,
        }
    }

    pub(crate) fn meshes() -> Vec<(&'static str, Vec<ScenePrimitive>)> {
        let half_tile_metres = (GROUND_TILE_METRES / 2.0) as f32;
        let mut meshes = vec![
            (
                "ground",
                vec![(ground_plane(half_tile_metres, 1.0), Some("grass"))],
            ),
            ("window", vec![(window(), Some("glass"))]),
            ("bird", bird()),
        ];
        for building in &BUILDINGS {
            meshes.push((building.walls, vec![(walls(building), None)]));
            meshes.push((building.roof, vec![(roof(building), Some("slate"))]));
        }
        meshes
    }

    /// A building's four walls about its base's centre, their UVs in
    /// `FACADE_METRES` along the wall and up from the ground.
    fn walls(building: &Building) -> Geometry {
        let (half_width, half_depth) = (building.width / 2.0, building.depth / 2.0);
        let up = Vec3::Y * building.height();
        let mut geometry = Geometry::default();
        for (corner, along) in [
            (vec3(-half_width, 0.0, half_depth), Vec3::X * building.width),
            (
                vec3(half_width, 0.0, half_depth),
                Vec3::NEG_Z * building.depth,
            ),
            (
                vec3(half_width, 0.0, -half_depth),
                Vec3::NEG_X * building.width,
            ),
            (
                vec3(-half_width, 0.0, -half_depth),
                Vec3::Z * building.depth,
            ),
        ] {
            let [width, height] =
                [along.length(), up.length()].map(|metres| metres / FACADE_METRES);
            quad(&mut geometry, corner, along, up, [width, -height]);
        }
        geometry
    }

    /// A flat slab over a building's walls, overhanging them.
    fn roof(building: &Building) -> Geometry {
        let half = vec3(
            building.width / 2.0 + ROOF_OVERHANG_METRES,
            0.0,
            building.depth / 2.0 + ROOF_OVERHANG_METRES,
        );
        let height = building.height();
        slab(
            vec3(-half.x, height, -half.z),
            vec3(half.x, height + ROOF_THICKNESS_METRES, half.z),
        )
    }

    /// A window's pane, standing on its bottom edge's middle and facing +Z.
    fn window() -> Geometry {
        let half = WINDOW_SIZE_METRES / 2.0;
        slab(
            vec3(-half.x, 0.0, -half.z),
            vec3(half.x, WINDOW_SIZE_METRES.y, half.z),
        )
    }

    /// A box from `min` to `max` without its bottom, which sits on the
    /// ground or on walls, its UVs the unit square on each face.
    fn slab(min: Vec3, max: Vec3) -> Geometry {
        let size = max - min;
        let [width_edge, height_edge, depth_edge] =
            [Vec3::X * size.x, Vec3::Y * size.y, Vec3::Z * size.z];
        let mut geometry = Geometry::default();
        for (corner, first, second) in [
            (vec3(max.x, min.y, max.z), -depth_edge, height_edge),
            (min, depth_edge, height_edge),
            (vec3(min.x, max.y, max.z), width_edge, -depth_edge),
            (vec3(min.x, min.y, max.z), width_edge, height_edge),
            (vec3(max.x, min.y, min.z), -width_edge, height_edge),
        ] {
            quad(&mut geometry, corner, first, second, [1.0, 1.0]);
        }
        geometry
    }

    /// Appends the quad from `corner` along `first` then `second`, facing
    /// their cross product, its UVs from 0 to `uv_extent` along each.
    fn quad(geometry: &mut Geometry, corner: Vec3, first: Vec3, second: Vec3, uv_extent: [f32; 2]) {
        let base = geometry.positions.len() as u32;
        let normal = first.cross(second).normalize();
        let [u_extent, v_extent] = uv_extent;
        for (offset, uv) in [
            (Vec3::ZERO, [0.0, 0.0]),
            (first, [u_extent, 0.0]),
            (first + second, [u_extent, v_extent]),
            (second, [0.0, v_extent]),
        ] {
            geometry.positions.push((corner + offset).into());
            geometry.normals.push(normal.into());
            geometry.uvs.push(uv);
        }
        geometry
            .indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub(crate) fn static_content() -> (Vec<CookedPlacement>, Vec<CookedLight>) {
        (placements_reaching(&|_, _| true), Vec::new())
    }

    /// The Placements of each ground tile, lot and farm that reaches the
    /// Chunk at `coordinates`, in `static_content`'s order: every Placement
    /// whose pivot the Chunk holds, and some of its neighbours'.
    pub(crate) fn static_content_reaching(
        coordinates: [i32; 2],
    ) -> (Vec<CookedPlacement>, Vec<CookedLight>) {
        let lowest = chunk_origin(coordinates);
        let highest = lowest + glam::DVec3::splat(CHUNK_EDGE_METRES);
        let reaches = |centre: glam::DVec3, reach_metres: f64| {
            let above_lowest = centre - lowest + reach_metres;
            let below_highest = highest - centre + reach_metres;
            above_lowest.x >= 0.0
                && above_lowest.z >= 0.0
                && below_highest.x >= 0.0
                && below_highest.z >= 0.0
        };
        (placements_reaching(&reaches), Vec::new())
    }

    /// The Placements of each ground tile, settlement lot and farm for which
    /// `reaches` holds, given the centre of its square and how far its
    /// Placements' pivots lie from it at most on either axis.
    fn placements_reaching(reaches: &dyn Fn(glam::DVec3, f64) -> bool) -> Vec<CookedPlacement> {
        let scene = ReferenceScene::World;
        let ground = scene.mesh_id("ground");
        let mut placements: Vec<_> = ground_tile_centres(GROUND_TILE_METRES, GROUND_TILES_PER_SIDE)
            .filter(|centre| reaches(*centre, 0.0))
            .map(|centre| placement(ground, None, centre, Quat::IDENTITY))
            .collect();
        for row in 0..SETTLEMENTS_PER_SIDE {
            for column in 0..SETTLEMENTS_PER_SIDE {
                let region = [column, row];
                if reaches(settlement_centre(region), SETTLEMENT_HALF_SIZE_METRES) {
                    place_settlement(region, reaches, &mut placements);
                }
            }
        }
        for centre in ground_tile_centres(GROUND_TILE_METRES, GROUND_TILES_PER_SIDE) {
            if reaches(centre, GROUND_TILE_METRES / 2.0) {
                place_farm(centre, &mut placements);
            }
        }
        placements
    }

    /// Places the settlement of `region`: blocks of lots between streets,
    /// a building on each lot `reaches` holds for.
    fn place_settlement(
        region: [u32; 2],
        reaches: &dyn Fn(glam::DVec3, f64) -> bool,
        placements: &mut Vec<CookedPlacement>,
    ) {
        let settlement = settlement_index(region);
        let corner = settlement_centre(region)
            - glam::dvec3(
                SETTLEMENT_HALF_SIZE_METRES,
                0.0,
                SETTLEMENT_HALF_SIZE_METRES,
            );
        let lots_per_side = BLOCKS_PER_SIDE * LOTS_PER_BLOCK_SIDE;
        for lot_row in 0..lots_per_side {
            for lot_column in 0..lots_per_side {
                let offset = |lot: u32| {
                    let block = f64::from(lot / LOTS_PER_BLOCK_SIDE);
                    let within = f64::from(lot % LOTS_PER_BLOCK_SIDE);
                    block * BLOCK_PITCH_METRES
                        + STREET_WIDTH_METRES / 2.0
                        + (within + 0.5) * LOT_PITCH_METRES
                };
                let centre = corner + glam::dvec3(offset(lot_column), 0.0, offset(lot_row));
                if !reaches(centre, LOT_PITCH_METRES / 2.0) {
                    continue;
                }
                let pick = |choice: Choice| {
                    random([
                        settlement.into(),
                        lot_column.into(),
                        lot_row.into(),
                        choice as u64,
                    ])
                };
                let building = (pick(Choice::Building) * BUILDINGS.len() as f64) as usize;
                place_building(&BUILDINGS[building], settlement, centre, pick, placements);
            }
        }
    }

    /// Places a farm on the ground tile at `centre`, by `FARM_CHANCE`, unless
    /// a settlement reaches the tile; it is clad in its region's
    /// settlement's facades.
    fn place_farm(centre: glam::DVec3, placements: &mut Vec<CookedPlacement>) {
        let region = [centre.x, centre.z]
            .map(|metres| ((metres + WORLD_HALF_SIZE_METRES) / REGION_METRES).floor() as u32);
        let reach_metres = SETTLEMENT_HALF_SIZE_METRES + GROUND_TILE_METRES / 2.0;
        let from_settlement = (centre - settlement_centre(region)).abs();
        if from_settlement.x < reach_metres && from_settlement.z < reach_metres {
            return;
        }
        let tile = [centre.x, centre.z].map(|metres| metres.to_bits());
        let pick = |choice: Choice| random([tile[0], tile[1], choice as u64, 1]);
        if pick(Choice::Farm) >= FARM_CHANCE {
            return;
        }
        let spread_metres = GROUND_TILE_METRES / 2.0 - 16.0;
        let position = centre
            + glam::dvec3(
                spread_metres * (2.0 * pick(Choice::FarmX) - 1.0),
                0.0,
                spread_metres * (2.0 * pick(Choice::FarmZ) - 1.0),
            );
        let settlement = settlement_index(region);
        place_building(&BUILDINGS[FARM], settlement, position, pick, placements);
    }

    /// Places `building` at `centre`, turned by quarter turns `pick` chooses,
    /// its walls clad in the facade of `settlement` it chooses, with its roof
    /// and a window every `WINDOW_PITCH_METRES` along each wall on each
    /// storey.
    fn place_building(
        building: &Building,
        settlement: u32,
        centre: glam::DVec3,
        pick: impl Fn(Choice) -> f64,
        placements: &mut Vec<CookedPlacement>,
    ) {
        let facade = (pick(Choice::Facade) * f64::from(FACADES_PER_SETTLEMENT)) as u32;
        let quarter_turns = (pick(Choice::Yaw) * 4.0) as u32;
        let scene = ReferenceScene::World;
        let rotation = Quat::from_rotation_y(quarter_turns as f32 * std::f32::consts::FRAC_PI_2);
        let material = scene.material_id(&facade_name(settlement, facade));
        placements.push(placement(
            scene.mesh_id(building.walls),
            Some(material),
            centre,
            rotation,
        ));
        placements.push(placement(
            scene.mesh_id(building.roof),
            None,
            centre,
            rotation,
        ));
        let window = scene.mesh_id("window");
        let (half_width, half_depth) = (building.width / 2.0, building.depth / 2.0);
        for (outward, along, half_length) in [
            (Vec3::Z * half_depth, Vec3::X, half_width),
            (Vec3::X * half_width, Vec3::NEG_Z, half_depth),
            (Vec3::NEG_Z * half_depth, Vec3::NEG_X, half_width),
            (Vec3::NEG_X * half_width, Vec3::Z, half_depth),
        ] {
            let columns = ((2.0 * half_length) / WINDOW_PITCH_METRES).floor() as u32;
            let facing = Quat::from_rotation_arc(Vec3::Z, outward.normalize());
            for storey in 0..building.storeys {
                for column in 0..columns {
                    let across = (column as f32 + 0.5 - columns as f32 / 2.0) * WINDOW_PITCH_METRES;
                    let height = storey as f32 * STOREY_HEIGHT_METRES + WINDOW_SILL_HEIGHT_METRES;
                    let local = outward + along * across + Vec3::Y * height;
                    let translation = centre + (rotation * local).as_dvec3();
                    placements.push(placement(window, None, translation, rotation * facing));
                }
            }
        }
    }

    fn placement(
        mesh: crate::cooked::AssetId,
        material: Option<crate::cooked::AssetId>,
        translation: glam::DVec3,
        rotation: Quat,
    ) -> CookedPlacement {
        CookedPlacement {
            mesh,
            material,
            translation,
            rotation,
            scale: Vec3::ONE,
        }
    }
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Write;

    use super::*;
    use crate::render::{DYNAMIC_TLAS_CAPACITY, RenderCapacities};
    use crate::scenes::ReferenceScene;
    use crate::streaming::distance_metres;

    /// The jumps of `path`, frame by frame from 0 to `frames`: each frame
    /// that teleported and how far it jumped. Every other frame flies
    /// 50 m/s at 60 Hz.
    fn jumps(path: fn(u32) -> Camera, frames: u32) -> Vec<(u32, f64)> {
        assert!(!path(0).teleported);
        let mut jumps = Vec::new();
        for frame in 1..frames {
            let (before, camera) = (path(frame - 1), path(frame));
            let metres = before.position.distance(camera.position);
            if camera.teleported {
                jumps.push((frame, metres));
            } else {
                assert!(
                    (metres - 50.0 / 60.0).abs() < 1e-6,
                    "frame {frame} flew {metres} m"
                );
            }
        }
        jumps
    }

    fn over_a_settlement(position: glam::DVec3) -> bool {
        (0..SETTLEMENTS_PER_SIDE).any(|column| {
            (0..SETTLEMENTS_PER_SIDE).any(|row| {
                let centre = settlement_centre([column, row]);
                let half = SETTLEMENT_HALF_SIZE_METRES;
                (position.x - centre.x).abs() < half && (position.z - centre.z).abs() < half
            })
        })
    }

    #[test]
    fn the_bench_path_flies_into_a_settlement_over_farmland_then_fast_travels_into_another() {
        let segment_frames = 60 * 60;
        assert_eq!(BENCH_SEGMENT_FRAMES, segment_frames);
        let [(jump_frame, jump_metres)] = jumps(camera_at, segment_frames)[..] else {
            panic!("the bench path jumps other than once");
        };
        // Past twice the 512 m hold radius, so no Chunk stays loaded.
        assert!(jump_metres > 1024.0, "{jump_metres} m");
        assert!(over_a_settlement(camera_at(599).position));
        let before = camera_at(jump_frame - 1).position;
        assert!(!over_a_settlement(before), "{before}");
        assert!(over_a_settlement(camera_at(jump_frame).position));
    }

    #[test]
    fn the_traversal_flies_a_side_then_the_diagonal_through_one_fast_travel() {
        let [(jump_frame, _)] = jumps(traversal_camera_at, TRAVERSAL_FRAMES)[..] else {
            panic!("the traversal jumps other than once");
        };
        let start = traversal_camera_at(0).position;
        let side = traversal_camera_at(jump_frame - 1).position - start;
        assert!(
            side.z.abs() < 1e-9 && (9_900.0..10_240.0).contains(&side.x.abs()),
            "{side}"
        );
        let diagonal = traversal_camera_at(TRAVERSAL_FRAMES - 1).position
            - traversal_camera_at(jump_frame).position;
        assert!((diagonal.x - diagonal.z).abs() < 1e-6 || (diagonal.x + diagonal.z).abs() < 1e-6);
        assert!(
            (9_900.0..10_240.0).contains(&diagonal.x.abs()),
            "{diagonal}"
        );
        for frame in (0..TRAVERSAL_FRAMES).step_by(60) {
            let position = traversal_camera_at(frame).position;
            assert!(
                position.x.abs().max(position.z.abs()) < 5_120.0,
                "{position}"
            );
        }
    }

    /// From a camera anywhere on a 64 m grid, the static Instances of the
    /// Chunks within the Load radius fit the static TLAS, and those within
    /// the hold radius, with the 10,000 Moving Instances, fit the Instance
    /// buffer, at the checked-in Budgets.
    #[test]
    fn the_densest_view_fits_the_capacities_the_budgets_give() {
        let (placements, _) = static_content();
        let mut per_chunk = BTreeMap::<[i32; 2], u32>::new();
        for placement in &placements {
            let translation = placement.translation;
            let coordinates = [translation.x, translation.z]
                .map(|metres| (metres / CHUNK_EDGE_METRES).floor() as i32);
            *per_chunk.entry(coordinates).or_default() += 1;
        }
        let load_radius_metres = ReferenceScene::World.load_radius_metres();
        let hold_radius_metres = load_radius_metres + CHUNK_EDGE_METRES;
        let step_metres = 64.0;
        let steps = (2.0 * WORLD_HALF_SIZE_METRES / step_metres) as u32;
        let (mut most_loaded, mut most_held) = (0, 0);
        for column in 0..=steps {
            for row in 0..=steps {
                let at = |index: u32| -WORLD_HALF_SIZE_METRES + f64::from(index) * step_metres;
                let camera = glam::dvec3(at(column), 0.0, at(row));
                let within = |radius: f64| -> u32 {
                    (per_chunk.iter())
                        .filter(|(coordinates, _)| distance_metres(camera, **coordinates) <= radius)
                        .map(|(_, instances)| instances)
                        .sum()
                };
                most_loaded = most_loaded.max(within(load_radius_metres));
                most_held = most_held.max(within(hold_radius_metres));
            }
        }
        let capacities = RenderCapacities::checked_in();
        writeln!(
            std::io::stderr(),
            "{} static Instances, {most_loaded} at most within the Load radius, {most_held} within the hold radius",
            placements.len()
        )
        .ok();
        assert!(
            most_loaded <= capacities.static_tlas_instances,
            "{most_loaded} loaded"
        );
        assert!(
            most_held + DYNAMIC_TLAS_CAPACITY <= capacities.instances,
            "{most_held} held"
        );
    }
}
