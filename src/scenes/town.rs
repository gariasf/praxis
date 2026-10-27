//! town, the dense Reference scene at M3's target scale (ADR 0013): a
//! procedural town of 1 × 1 km on a street grid. Most blocks hold four buildings whose
//! street fronts carry windows and whose roofs carry railings, plant and
//! aerials; the rest are parks. Sidewalks carry trees, parked cars,
//! bollards, benches and bins, utility poles carry wires that go sub-pixel
//! at distance, 1,024 street lamps each hold a point light, and 10,000
//! pigeons circle over the roofs in flocks, its Moving Instances. Its Meshes
//! cook from the generators here, and its spawn places each Instance from a
//! hash of where it stands, so every launch places the same town.

use std::f64::consts::FRAC_PI_2;

use bevy_ecs::prelude::*;
use glam::FloatExt;

use super::{ReferenceScene, Spin};
use crate::assets::{AssetLoader, MaterialHandle, MeshHandle};
use crate::camera::Camera;
use crate::components::{MaterialRef, MeshRef, PointLight, Transform};
use crate::render::DYNAMIC_TLAS_CAPACITY;
use crate::transform::PreviousWorldTransform;

/// The town's Meshes.
#[derive(Clone, Copy)]
enum TownMesh {
    Ground,
    /// A unit cube on its base, scaled per Instance and drawn with its
    /// Instance's Material: slabs, buildings, windows and rails.
    Box,
    AirConditioner,
    Vent,
    WaterTank,
    Dish,
    Aerial,
    SolarPanel,
    RailingPost,
    Lamp,
    UtilityPole,
    Wire,
    Tree,
    Bench,
    RedCar,
    BlueCar,
    Bollard,
    Bin,
    /// A pigeon, which each of the town's Moving Instances draws.
    Bird,
}

impl TownMesh {
    /// Every Mesh, in the order of its discriminant, which indexes its
    /// Handle when loaded.
    const ALL: [TownMesh; 19] = [
        TownMesh::Ground,
        TownMesh::Box,
        TownMesh::AirConditioner,
        TownMesh::Vent,
        TownMesh::WaterTank,
        TownMesh::Dish,
        TownMesh::Aerial,
        TownMesh::SolarPanel,
        TownMesh::RailingPost,
        TownMesh::Lamp,
        TownMesh::UtilityPole,
        TownMesh::Wire,
        TownMesh::Tree,
        TownMesh::Bench,
        TownMesh::RedCar,
        TownMesh::BlueCar,
        TownMesh::Bollard,
        TownMesh::Bin,
        TownMesh::Bird,
    ];

    /// The name it cooks under.
    const fn name(self) -> &'static str {
        match self {
            TownMesh::Ground => "ground",
            TownMesh::Box => "box",
            TownMesh::AirConditioner => "air-conditioner",
            TownMesh::Vent => "vent",
            TownMesh::WaterTank => "water-tank",
            TownMesh::Dish => "dish",
            TownMesh::Aerial => "aerial",
            TownMesh::SolarPanel => "solar-panel",
            TownMesh::RailingPost => "railing-post",
            TownMesh::Lamp => "lamp",
            TownMesh::UtilityPole => "utility-pole",
            TownMesh::Wire => "wire",
            TownMesh::Tree => "tree",
            TownMesh::Bench => "bench",
            TownMesh::RedCar => "red-car",
            TownMesh::BlueCar => "blue-car",
            TownMesh::Bollard => "bollard",
            TownMesh::Bin => "bin",
            TownMesh::Bird => "bird",
        }
    }
}

const _: () = {
    let mut index = 0;
    while index < TownMesh::ALL.len() {
        assert!(TownMesh::ALL[index] as usize == index);
        index += 1;
    }
};

/// The Materials the box's Instances draw with; the other Meshes'
/// Primitives name their own.
#[derive(Clone, Copy)]
enum BoxMaterial {
    Concrete,
    Grass,
    Ochre,
    Cream,
    Rose,
    Slate,
    Glass,
    Steel,
}

impl BoxMaterial {
    /// Every Material, in the order of its discriminant, which indexes its
    /// Handle when loaded.
    const ALL: [BoxMaterial; 8] = [
        BoxMaterial::Concrete,
        BoxMaterial::Grass,
        BoxMaterial::Ochre,
        BoxMaterial::Cream,
        BoxMaterial::Rose,
        BoxMaterial::Slate,
        BoxMaterial::Glass,
        BoxMaterial::Steel,
    ];

    /// The name it cooks under.
    const fn name(self) -> &'static str {
        match self {
            BoxMaterial::Concrete => "concrete",
            BoxMaterial::Grass => "grass",
            BoxMaterial::Ochre => "ochre",
            BoxMaterial::Cream => "cream",
            BoxMaterial::Rose => "rose",
            BoxMaterial::Slate => "slate",
            BoxMaterial::Glass => "glass",
            BoxMaterial::Steel => "steel",
        }
    }
}

const _: () = {
    let mut index = 0;
    while index < BoxMaterial::ALL.len() {
        assert!(BoxMaterial::ALL[index] as usize == index);
        index += 1;
    }
};

/// The plasters the buildings are painted in.
const PLASTERS: [BoxMaterial; 4] = [
    BoxMaterial::Ochre,
    BoxMaterial::Cream,
    BoxMaterial::Rose,
    BoxMaterial::Slate,
];

/// The town spans ±`TOWN_HALF_SIZE` in X and Z: `BLOCKS_PER_SIDE` blocks a
/// side, centred `BLOCK_PITCH` apart, with a street between neighbours and
/// along the edge.
const TOWN_HALF_SIZE: f64 = 500.0;
const BLOCKS_PER_SIDE: u32 = 20;
const BLOCK_PITCH: f64 = 50.0;
const STREET_WIDTH: f64 = 12.0;
/// From a block's centre to its curb.
const BLOCK_HALF_SIDE: f64 = (BLOCK_PITCH - STREET_WIDTH) / 2.0;
const SIDEWALK_HEIGHT: f64 = 0.15;
/// From the curb to a building's street front.
const SIDEWALK_WIDTH: f64 = 2.5;
/// From each building to its block's centre lines.
const BUILDING_GAP: f64 = 0.5;
const BUILDING_HALF_SIDE: f64 = (BLOCK_HALF_SIDE - SIDEWALK_WIDTH - BUILDING_GAP) / 2.0;
const STOREY_HEIGHT: f64 = 3.2;
/// A building has `MIN_STOREYS` and up to `STOREY_RANGE - 1` more.
const MIN_STOREYS: u32 = 3;
const STOREY_RANGE: u32 = 4;
/// The share of blocks that are parks.
const PARK_SHARE: f64 = 0.12;

/// Windows along each street front, each storey, and their size: wide,
/// tall, and deep, half of it proud of the wall.
const WINDOW_COLUMNS: u32 = 4;
const WINDOW_SIZE: glam::Vec3 = glam::Vec3::new(1.4, 1.8, 0.12);
/// Railing posts along each side of a roof, inset from its edge, and the
/// height of the rail they carry.
const RAILING_POSTS_PER_SIDE: u32 = 20;
const RAILING_INSET: f64 = 0.3;
const RAILING_HEIGHT: f64 = 1.0;
/// A roof's plant stands on a grid of cells `ROOF_CELL` metres square.
const ROOF_CELLS_PER_SIDE: u32 = 6;
const ROOF_CELL: f64 = 2.2;

/// Street trees along each side of a block, and a park's grid of trees.
const STREET_TREES_PER_SIDE: u32 = 4;
const PARK_TREES_PER_SIDE: u32 = 5;
/// Parking places along each side of a block, `CAR_SPACING` apart, a car
/// in each with this chance.
const CARS_PER_SIDE: u32 = 6;
const CAR_SPACING: f64 = 5.5;
const CAR_CHANCE: f64 = 0.7;
/// The chance that a side of a block has a row of bollards,
/// `BOLLARD_SPACING` apart, a bench or a bin.
const BOLLARD_CHANCE: f64 = 0.3;
const BOLLARD_SPACING: f64 = 2.0;
const BENCH_CHANCE: f64 = 0.3;
const BIN_CHANCE: f64 = 0.4;

/// Street lamps stand at 1,024 of the blocks' corners. Each lamp's light
/// hangs below its head, outside its Mesh, so it reaches what it lights.
const LAMP_COUNT: usize = 1024;
const LAMP_LIGHT_POSITION: glam::DVec3 = glam::DVec3::new(0.0, 4.7, 1.2);
/// Warm white, giving the sun's illuminance 4.5 m away and nearly as much
/// on the ground below, so its pool shows in daylight, and reaching 18 m.
const LAMP_LIGHT: PointLight = PointLight {
    color: glam::Vec3::new(1.0, 0.85, 0.65),
    intensity: 20.0 * super::SUN_ILLUMINANCE,
    range: 18.0,
};

/// Flocks of pigeons over the town, `FLOCKS_PER_SIDE` a side on a grid
/// `FLOCK_PITCH` apart, each of `BIRDS_PER_FLOCK` circling its centre in
/// the fixed schedule: 10,000 Moving Instances, the dynamic TLAS's capacity
/// (ADR 0020). Centres stand `FLOCK_HEIGHT` up, clear of every roof's plant,
/// and turn at `FLOCK_SPIN` radians a second, either way.
const FLOCKS_PER_SIDE: u32 = 10;
const FLOCK_PITCH: f64 = 100.0;
const BIRDS_PER_FLOCK: u32 = 100;
const _: () = assert!(FLOCKS_PER_SIDE * FLOCKS_PER_SIDE * BIRDS_PER_FLOCK == DYNAMIC_TLAS_CAPACITY);
const FLOCK_HEIGHT: std::ops::Range<f64> = 30.0..55.0;
const FLOCK_SPIN: std::ops::Range<f64> = 0.2..0.6;
/// A bird's distance from its flock's centre, and how far above or below
/// it.
const FLOCK_RADIUS: std::ops::Range<f64> = 6.0..30.0;
const FLOCK_DEPTH: f64 = 4.0;
/// How far a flock's centre strays from its grid cell's middle, each way:
/// as far as keeps the whole flock within its cell, so within the town.
const FLOCK_STRAY: f64 = FLOCK_PITCH / 2.0 - FLOCK_RADIUS.end;

/// Utility poles along every street, `POLE_SPACING` apart, each span between
/// two holding a wire at each of `WIRE_HEIGHTS`.
const POLE_SPACING: f64 = 25.0;
const WIRE_HEIGHTS: [f64; 2] = [7.6, 7.0];

/// A well-mixed 64-bit value of `seed`: SplitMix64's output function
/// (Steele, Lea and Flood, "Fast Splittable Pseudorandom Number
/// Generators", OOPSLA 2014), its constants from Sebastiano Vigna's
/// splitmix64.c, which is in the public domain.
fn mix(seed: u64) -> u64 {
    let mut value = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// A value in [0, 1) from `parts`, the same for the same parts.
fn random(parts: [u64; 4]) -> f64 {
    let hash = parts.into_iter().fold(0, |hash, part| mix(hash ^ part));
    (hash >> 11) as f64 / (1u64 << 53) as f64
}

/// What a random value decides, so that no two values for one place share
/// a seed: each choice numbers its items apart.
#[derive(Clone, Copy)]
enum Choice {
    Park,
    Storeys,
    Plaster,
    RoofPlant,
    PlantYaw,
    StreetTreeYaw,
    ParkTreeYaw,
    Car,
    Bollards,
    Bench,
    Bin,
    Lamp,
    Flock,
    Bird,
}

/// A Renderable the spawn places: its Mesh, its Material where it
/// overrides the Mesh's, and its Transform.
struct TownRenderable {
    mesh: TownMesh,
    material: Option<BoxMaterial>,
    transform: Transform,
}

impl TownRenderable {
    fn new(mesh: TownMesh, position: glam::DVec3, rotation: glam::Quat) -> Self {
        Self {
            mesh,
            material: None,
            transform: Transform {
                rotation,
                ..Transform::from_translation(position)
            },
        }
    }

    /// The unit cube scaled to `size`, on `position`, turned by `rotation`.
    fn cube(
        material: BoxMaterial,
        position: glam::DVec3,
        rotation: glam::Quat,
        size: glam::Vec3,
    ) -> Self {
        Self {
            mesh: TownMesh::Box,
            material: Some(material),
            transform: Transform {
                translation: position,
                rotation,
                scale: size,
            },
        }
    }
}

/// The horizontal axis across `direction`, which lies along X or Z.
fn across(direction: glam::DVec3) -> glam::DVec3 {
    glam::dvec3(direction.z.abs(), 0.0, direction.x.abs())
}

/// The rotation about +Y that turns +Z to the horizontal `direction`.
fn facing(direction: glam::DVec3) -> glam::Quat {
    glam::Quat::from_rotation_y(direction.x.atan2(direction.z) as f32)
}

/// Every Instance the town places, block by block, then its lamps, poles
/// and wires, and its lamps' lights.
fn placements() -> (Vec<TownRenderable>, Vec<glam::DVec3>) {
    let mut placed = vec![TownRenderable::new(
        TownMesh::Ground,
        glam::DVec3::ZERO,
        glam::Quat::IDENTITY,
    )];
    let mut corners = Vec::new();
    for row in 0..BLOCKS_PER_SIDE {
        for column in 0..BLOCKS_PER_SIDE {
            let block = Block::new(column, row);
            block.place(&mut placed);
            corners.extend(block.lamp_corners());
        }
    }
    // The lamps stand at the corners whose draws come lowest.
    corners.sort_by(|(rank, ..), (other, ..)| rank.total_cmp(other));
    corners.truncate(LAMP_COUNT);
    let mut lights = Vec::with_capacity(LAMP_COUNT);
    for &(_, position, rotation) in &corners {
        placed.push(TownRenderable::new(TownMesh::Lamp, position, rotation));
        lights.push(position + rotation.as_dquat() * LAMP_LIGHT_POSITION);
    }
    place_poles_and_wires(&mut placed);
    (placed, lights)
}

/// One block of the grid, `column` along X and `row` along Z.
struct Block {
    column: u32,
    row: u32,
    centre: glam::DVec3,
    park: bool,
}

/// A block's sides, counter-clockwise seen from above: each one's outward
/// direction, and the direction along it.
const SIDES: [(glam::DVec3, glam::DVec3); 4] = [
    (glam::DVec3::X, glam::DVec3::NEG_Z),
    (glam::DVec3::NEG_Z, glam::DVec3::NEG_X),
    (glam::DVec3::NEG_X, glam::DVec3::Z),
    (glam::DVec3::Z, glam::DVec3::X),
];

impl Block {
    fn new(column: u32, row: u32) -> Self {
        let offset = |index: u32| -TOWN_HALF_SIZE + BLOCK_PITCH * (f64::from(index) + 0.5);
        let mut block = Self {
            column,
            row,
            centre: glam::dvec3(offset(column), 0.0, offset(row)),
            park: false,
        };
        block.park = block.random(Choice::Park, 0) < PARK_SHARE;
        block
    }

    /// A value in [0, 1) for `choice` about `item` of the block, the same
    /// on every launch.
    fn random(&self, choice: Choice, item: u64) -> f64 {
        random([self.column.into(), self.row.into(), choice as u64, item])
    }

    /// A turn about +Y, at random for `choice` of `item`.
    fn yaw(&self, choice: Choice, item: u64) -> glam::Quat {
        glam::Quat::from_rotation_y((self.random(choice, item) * std::f64::consts::TAU) as f32)
    }

    /// The block's slab, then its buildings or its park, then what stands
    /// along its sides.
    fn place(&self, placed: &mut Vec<TownRenderable>) {
        let slab_side = 2.0 * BLOCK_HALF_SIDE as f32;
        placed.push(TownRenderable::cube(
            if self.park {
                BoxMaterial::Grass
            } else {
                BoxMaterial::Concrete
            },
            self.centre,
            glam::Quat::IDENTITY,
            glam::vec3(slab_side, SIDEWALK_HEIGHT as f32, slab_side),
        ));
        if self.park {
            self.place_park(placed);
        } else {
            for building in 0..4 {
                self.place_building(building, placed);
            }
        }
        for (side, &(outward, along)) in (0..).zip(&SIDES) {
            self.place_side(side, outward, along, placed);
        }
    }

    /// Building `index`, in the block's quadrant `index`: its walls, the
    /// windows of its two street fronts, and its roof's railing and plant.
    fn place_building(&self, index: u64, placed: &mut Vec<TownRenderable>) {
        let quadrant = glam::dvec3(
            if index.is_multiple_of(2) { -1.0 } else { 1.0 },
            0.0,
            if index < 2 { -1.0 } else { 1.0 },
        );
        let centre = self.centre + quadrant * (BUILDING_GAP + BUILDING_HALF_SIDE);
        let storeys =
            MIN_STOREYS + (self.random(Choice::Storeys, index) * f64::from(STOREY_RANGE)) as u32;
        let height = f64::from(storeys) * STOREY_HEIGHT;
        let plaster =
            PLASTERS[(self.random(Choice::Plaster, index) * PLASTERS.len() as f64) as usize];
        let side = 2.0 * BUILDING_HALF_SIDE as f32;
        placed.push(TownRenderable::cube(
            plaster,
            centre,
            glam::Quat::IDENTITY,
            glam::vec3(side, height as f32, side),
        ));
        for front in [glam::DVec3::X * quadrant.x, glam::DVec3::Z * quadrant.z] {
            let across = across(front);
            for storey in 0..storeys {
                // Centred on the storey.
                let sill =
                    (f64::from(storey) + 0.5) * STOREY_HEIGHT - f64::from(WINDOW_SIZE.y) / 2.0;
                for column in 0..WINDOW_COLUMNS {
                    let along = (f64::from(column) + 0.5) / f64::from(WINDOW_COLUMNS) * 2.0 - 1.0;
                    let position = centre
                        + front * BUILDING_HALF_SIDE
                        + across * (along * BUILDING_HALF_SIDE)
                        + glam::DVec3::Y * sill;
                    placed.push(TownRenderable::cube(
                        BoxMaterial::Glass,
                        position,
                        facing(front),
                        WINDOW_SIZE,
                    ));
                }
            }
        }
        self.place_roof(index, centre + glam::DVec3::Y * height, placed);
    }

    /// A railing along the edge of the roof at `roof`, posts and a top rail
    /// each side, and plant on a grid of cells inside it, each cell drawing
    /// what stands there, if anything.
    fn place_roof(&self, building: u64, roof: glam::DVec3, placed: &mut Vec<TownRenderable>) {
        let half_side = BUILDING_HALF_SIDE - RAILING_INSET;
        let post_spacing = 2.0 * half_side / f64::from(RAILING_POSTS_PER_SIDE);
        for &(outward, along) in &SIDES {
            let corner = roof + (outward - along) * half_side;
            for post in 0..RAILING_POSTS_PER_SIDE {
                let position = corner + along * (post_spacing * f64::from(post));
                placed.push(TownRenderable::new(
                    TownMesh::RailingPost,
                    position,
                    glam::Quat::IDENTITY,
                ));
            }
            placed.push(TownRenderable::cube(
                BoxMaterial::Steel,
                roof + outward * half_side + glam::DVec3::Y * RAILING_HEIGHT,
                facing(along),
                glam::vec3(0.05, 0.05, 2.0 * half_side as f32),
            ));
        }
        let first_cell = -ROOF_CELL * f64::from(ROOF_CELLS_PER_SIDE - 1) / 2.0;
        for cell in 0..ROOF_CELLS_PER_SIDE * ROOF_CELLS_PER_SIDE {
            let item = building << 8 | u64::from(cell);
            let mesh = match self.random(Choice::RoofPlant, item) {
                share if share < 0.30 => TownMesh::AirConditioner,
                share if share < 0.52 => TownMesh::Vent,
                share if share < 0.76 => TownMesh::SolarPanel,
                share if share < 0.82 => TownMesh::WaterTank,
                share if share < 0.89 => TownMesh::Dish,
                share if share < 0.95 => TownMesh::Aerial,
                _ => continue,
            };
            let position = roof
                + glam::dvec3(
                    first_cell + ROOF_CELL * f64::from(cell % ROOF_CELLS_PER_SIDE),
                    0.0,
                    first_cell + ROOF_CELL * f64::from(cell / ROOF_CELLS_PER_SIDE),
                );
            placed.push(TownRenderable::new(
                mesh,
                position,
                self.yaw(Choice::PlantYaw, item),
            ));
        }
    }

    /// A grid of trees, and on each side a bench between the outer trees
    /// and the next, facing the park's middle.
    fn place_park(&self, placed: &mut Vec<TownRenderable>) {
        let spacing = 2.0 * (BLOCK_HALF_SIDE - SIDEWALK_WIDTH) / f64::from(PARK_TREES_PER_SIDE);
        let first = -spacing * f64::from(PARK_TREES_PER_SIDE - 1) / 2.0;
        let ground = self.centre + glam::DVec3::Y * SIDEWALK_HEIGHT;
        for tree in 0..PARK_TREES_PER_SIDE * PARK_TREES_PER_SIDE {
            let position = ground
                + glam::dvec3(
                    first + spacing * f64::from(tree % PARK_TREES_PER_SIDE),
                    0.0,
                    first + spacing * f64::from(tree / PARK_TREES_PER_SIDE),
                );
            placed.push(TownRenderable::new(
                TownMesh::Tree,
                position,
                self.yaw(Choice::ParkTreeYaw, tree.into()),
            ));
        }
        for &(outward, _) in &SIDES {
            let position = ground + outward * (-first - spacing / 2.0);
            placed.push(TownRenderable::new(
                TownMesh::Bench,
                position,
                facing(-outward),
            ));
        }
    }

    /// Along side `side`, facing `outward`: street trees and bollards along
    /// the curb, a bench and a bin against the buildings, and parked cars
    /// in the street.
    fn place_side(
        &self,
        side: u64,
        outward: glam::DVec3,
        along: glam::DVec3,
        placed: &mut Vec<TownRenderable>,
    ) {
        let curb = self.centre + outward * BLOCK_HALF_SIDE;
        let sidewalk = curb + glam::DVec3::Y * SIDEWALK_HEIGHT;
        let tree_spacing = 2.0 * BLOCK_HALF_SIDE / f64::from(STREET_TREES_PER_SIDE);
        for tree in 0..STREET_TREES_PER_SIDE {
            let offset = tree_spacing * (f64::from(tree) + 0.5) - BLOCK_HALF_SIDE;
            let position = sidewalk - outward + along * offset;
            let item = side << 8 | u64::from(tree);
            placed.push(TownRenderable::new(
                TownMesh::Tree,
                position,
                self.yaw(Choice::StreetTreeYaw, item),
            ));
        }
        if self.random(Choice::Bollards, side) < BOLLARD_CHANCE {
            // Clear of the lamps at the corners.
            let reach = BLOCK_HALF_SIDE - 2.0;
            let count = (2.0 * reach / BOLLARD_SPACING) as u32;
            for bollard in 0..=count {
                let offset = BOLLARD_SPACING * f64::from(bollard) - reach;
                let position = sidewalk - outward * 0.3 + along * offset;
                placed.push(TownRenderable::new(
                    TownMesh::Bollard,
                    position,
                    glam::Quat::IDENTITY,
                ));
            }
        }
        let wall = sidewalk - outward * (SIDEWALK_WIDTH - 0.5);
        if self.random(Choice::Bench, side) < BENCH_CHANCE {
            placed.push(TownRenderable::new(
                TownMesh::Bench,
                wall + along * 4.0,
                facing(outward),
            ));
        }
        if self.random(Choice::Bin, side) < BIN_CHANCE {
            placed.push(TownRenderable::new(
                TownMesh::Bin,
                wall + along * 6.0,
                glam::Quat::IDENTITY,
            ));
        }
        let first_car = -CAR_SPACING * f64::from(CARS_PER_SIDE - 1) / 2.0;
        for car in 0..CARS_PER_SIDE {
            let chance = self.random(Choice::Car, side << 8 | u64::from(car));
            if chance >= CAR_CHANCE {
                continue;
            }
            let mesh = if chance < CAR_CHANCE / 2.0 {
                TownMesh::RedCar
            } else {
                TownMesh::BlueCar
            };
            let position =
                curb + outward * 1.2 + along * (first_car + CAR_SPACING * f64::from(car));
            placed.push(TownRenderable::new(mesh, position, facing(along)));
        }
    }

    /// Where a lamp could stand at each corner of the block, a metre in from
    /// both curbs, its arm over the street of the side that ends there, with
    /// the draw that ranks it.
    fn lamp_corners(&self) -> impl Iterator<Item = (f64, glam::DVec3, glam::Quat)> + '_ {
        (0..).zip(&SIDES).map(|(corner, &(outward, along))| {
            let position = self.centre
                + (outward + along) * (BLOCK_HALF_SIDE - 1.0)
                + glam::DVec3::Y * SIDEWALK_HEIGHT;
            (self.random(Choice::Lamp, corner), position, facing(outward))
        })
    }
}

/// Utility poles along every street, on the sidewalk of the block on its
/// lower side, or the higher one along the town's lower edge, and wires
/// from each pole to the next.
fn place_poles_and_wires(placed: &mut Vec<TownRenderable>) {
    let poles_per_street = (2.0 * TOWN_HALF_SIZE / POLE_SPACING) as u32;
    for street in 0..=BLOCKS_PER_SIDE {
        let middle = -TOWN_HALF_SIZE + BLOCK_PITCH * f64::from(street);
        let curb_offset = STREET_WIDTH / 2.0 + 0.6;
        let line = if street == 0 {
            middle + curb_offset
        } else {
            middle - curb_offset
        };
        // Along X, and along −Z, the wire Mesh's +X turned to −Z, so each
        // wire runs from its pole to the next.
        for (direction, rotation) in [
            (glam::DVec3::X, glam::Quat::IDENTITY),
            (
                glam::DVec3::NEG_Z,
                glam::Quat::from_rotation_y(FRAC_PI_2 as f32),
            ),
        ] {
            let across = across(direction);
            for pole in 0..poles_per_street {
                let offset = -TOWN_HALF_SIZE + POLE_SPACING * (f64::from(pole) + 0.5);
                let position =
                    direction * offset + across * line + glam::DVec3::Y * SIDEWALK_HEIGHT;
                placed.push(TownRenderable::new(
                    TownMesh::UtilityPole,
                    position,
                    rotation,
                ));
                if pole + 1 < poles_per_street {
                    for height in WIRE_HEIGHTS {
                        let start = position + glam::DVec3::Y * height;
                        placed.push(TownRenderable::new(TownMesh::Wire, start, rotation));
                    }
                }
            }
        }
    }
}

/// A flock: its centre, how fast it turns, in radians a second, and its
/// birds, placed about its centre.
struct Flock {
    centre: glam::DVec3,
    radians_per_second: f64,
    birds: Vec<Transform>,
}

/// The town's flocks, row by row: each centre at random about its grid
/// cell's middle, and each bird at random about its centre, facing the way
/// it circles.
fn flocks() -> Vec<Flock> {
    let offset = |index: u32| -TOWN_HALF_SIZE + FLOCK_PITCH * (f64::from(index) + 0.5);
    (0..FLOCKS_PER_SIDE * FLOCKS_PER_SIDE)
        .map(|flock| {
            let (column, row) = (flock % FLOCKS_PER_SIDE, flock / FLOCKS_PER_SIDE);
            let flock_random = |choice: Choice, item: u64| {
                random([column.into(), row.into(), choice as u64, item])
            };
            let centre = glam::dvec3(
                offset(column) + FLOCK_STRAY * (2.0 * flock_random(Choice::Flock, 0) - 1.0),
                FLOCK_HEIGHT
                    .start
                    .lerp(FLOCK_HEIGHT.end, flock_random(Choice::Flock, 1)),
                offset(row) + FLOCK_STRAY * (2.0 * flock_random(Choice::Flock, 2) - 1.0),
            );
            let turning = if flock_random(Choice::Flock, 3) < 0.5 {
                1.0
            } else {
                -1.0
            };
            let radians_per_second = turning
                * FLOCK_SPIN
                    .start
                    .lerp(FLOCK_SPIN.end, flock_random(Choice::Flock, 4));
            let birds = (0..BIRDS_PER_FLOCK)
                .map(|bird| {
                    let bird_random =
                        |item: u64| flock_random(Choice::Bird, u64::from(bird) << 4 | item);
                    // Uniform over the ring's area, so the flock is no
                    // denser towards its middle than towards its edge.
                    let [inner_squared, outer_squared] =
                        [FLOCK_RADIUS.start, FLOCK_RADIUS.end].map(|radius| radius * radius);
                    let radius = inner_squared.lerp(outer_squared, bird_random(0)).sqrt();
                    let angle = std::f64::consts::TAU * bird_random(1);
                    // A turn about +Y by the flock's spin moves this point
                    // along (−sin, 0, −cos) of its angle.
                    let position = glam::dvec3(
                        radius * angle.cos(),
                        FLOCK_DEPTH * (2.0 * bird_random(2) - 1.0),
                        -radius * angle.sin(),
                    );
                    let heading = glam::dvec3(-angle.sin(), 0.0, -angle.cos()) * turning;
                    Transform {
                        rotation: facing(heading),
                        ..Transform::from_translation(position)
                    }
                })
                .collect();
            Flock {
                centre,
                radians_per_second,
                birds,
            }
        })
        .collect()
}

/// Loads the town's Meshes and the Materials its boxes draw with, then
/// places its Instances, its lamps' point lights, and its flocks: each a
/// spinning centre with its birds as its children, all moved in the fixed
/// schedule, so each carries its previous World transform (ADR 0022).
pub(crate) fn spawn_town(world: &mut World, loader: &mut AssetLoader) -> anyhow::Result<()> {
    let scene = ReferenceScene::Town;
    let meshes: Vec<MeshHandle> = TownMesh::ALL
        .iter()
        .map(|mesh| loader.mesh(scene.mesh_id(mesh.name())))
        .collect::<anyhow::Result<_>>()?;
    let materials: Vec<MaterialHandle> = BoxMaterial::ALL
        .iter()
        .map(|material| loader.material(scene.material_id(material.name())))
        .collect::<anyhow::Result<_>>()?;
    let (placed, lights) = placements();
    let (boxes, others): (Vec<_>, Vec<_>) = placed
        .into_iter()
        .partition(|renderable| renderable.material.is_some());
    world.spawn_batch(others.into_iter().map(|renderable| {
        (
            renderable.transform,
            MeshRef(meshes[renderable.mesh as usize]),
        )
    }));
    world.spawn_batch(boxes.into_iter().filter_map(|renderable| {
        let material = materials[renderable.material? as usize];
        Some((
            renderable.transform,
            MeshRef(meshes[renderable.mesh as usize]),
            MaterialRef(material),
        ))
    }));
    world.spawn_batch(
        lights
            .into_iter()
            .map(|position| (Transform::from_translation(position), LAMP_LIGHT)),
    );
    let bird = meshes[TownMesh::Bird as usize];
    for flock in flocks() {
        let centre = world
            .spawn((
                Transform::from_translation(flock.centre),
                Spin {
                    radians_per_second: flock.radians_per_second,
                },
                PreviousWorldTransform::default(),
            ))
            .id();
        world.spawn_batch(flock.birds.into_iter().map(|transform| {
            (
                transform,
                ChildOf(centre),
                MeshRef(bird),
                PreviousWorldTransform::default(),
            )
        }));
    }
    Ok(())
}

/// The camera's path: at street level up the avenue at X = 0, facing north
/// (−Z), for `STREET_FRAMES`; then for `RISE_FRAMES` up and back over the
/// rooftops to a circle round the town's middle, inside its edge, at
/// `ORBIT_HEIGHT`, turning to face its middle; then round the circle,
/// facing the middle. Its position and its speed change smoothly
/// throughout.
const STREET_FRAMES: u32 = 420;
const RISE_FRAMES: u32 = 360;
/// In the avenue's right-hand lane, at eye height, and moving at 4 m/s.
const STREET_START: glam::DVec3 = glam::DVec3::new(2.0, 1.7, 445.0);
const STREET_SPEED: f64 = 4.0 / 60.0;
const ORBIT_RADIUS: f64 = 480.0;
const ORBIT_HEIGHT: f64 = 260.0;
/// Radians a frame round the circle, reached over `ORBIT_EASE_FRAMES`.
const ORBIT_SPEED: f64 = 0.0006;
const ORBIT_EASE_FRAMES: f64 = 120.0;

/// The camera at `frame` of its path.
pub(crate) fn camera_at(frame: u32) -> Camera {
    let frame = f64::from(frame);
    let street_end = STREET_START - glam::DVec3::Z * (STREET_SPEED * f64::from(STREET_FRAMES));
    let orbit_frame = frame - f64::from(STREET_FRAMES + RISE_FRAMES);
    if frame < f64::from(STREET_FRAMES) {
        let position = STREET_START - glam::DVec3::Z * (STREET_SPEED * frame);
        Camera::looking_at(position, position - glam::DVec3::Z)
    } else if orbit_frame < 0.0 {
        // A cubic from the street's end, leaving at the street's speed, to
        // the circle's start, arriving at rest; the gaze turns from far up
        // the avenue to the town's middle.
        let progress = (frame - f64::from(STREET_FRAMES)) / f64::from(RISE_FRAMES);
        let orbit_start = glam::dvec3(0.0, ORBIT_HEIGHT, ORBIT_RADIUS);
        let leaving = -glam::DVec3::Z * (STREET_SPEED * f64::from(RISE_FRAMES));
        let squared = progress * progress;
        let cubed = squared * progress;
        let arrived = 3.0 * squared - 2.0 * cubed;
        let position = street_end * (1.0 - arrived)
            + leaving * (cubed - 2.0 * squared + progress)
            + orbit_start * arrived;
        let up_the_avenue = street_end - glam::DVec3::Z * TOWN_HALF_SIZE;
        Camera::looking_at(position, up_the_avenue * (1.0 - arrived))
    } else {
        // At rest on arrival, then at `ORBIT_SPEED`, with no jolt between.
        let eased = if orbit_frame < ORBIT_EASE_FRAMES {
            orbit_frame * orbit_frame / (2.0 * ORBIT_EASE_FRAMES)
        } else {
            orbit_frame - ORBIT_EASE_FRAMES / 2.0
        };
        let angle = ORBIT_SPEED * eased;
        let position = glam::dvec3(
            ORBIT_RADIUS * angle.sin(),
            ORBIT_HEIGHT,
            ORBIT_RADIUS * angle.cos(),
        );
        Camera::looking_at(position, glam::DVec3::ZERO)
    }
}

#[cfg(feature = "cook")]
pub(crate) use generators::{materials, meshes};

/// town's Materials and the Meshes they cook from, which only the cook
/// reads.
#[cfg(feature = "cook")]
mod generators {
    use std::f32::consts::TAU;

    use glam::{Quat, Vec3, vec3};

    use super::TownMesh;
    use crate::cooked::CookedMaterial;
    use crate::scenes::{Geometry, ScenePrimitive, ground_plane};

    /// Every Material town's Meshes name or its boxes draw with: a base
    /// colour in linear sRGB, metallic, roughness.
    const MATERIALS: [(&str, [f32; 3], f32, f32); 18] = [
        ("concrete", [0.42, 0.41, 0.39], 0.0, 0.85),
        ("grass", [0.10, 0.26, 0.06], 0.0, 0.95),
        ("ochre", [0.66, 0.45, 0.22], 0.0, 0.80),
        ("cream", [0.78, 0.73, 0.60], 0.0, 0.80),
        ("rose", [0.65, 0.36, 0.32], 0.0, 0.80),
        ("slate", [0.32, 0.36, 0.41], 0.0, 0.80),
        ("glass", [0.04, 0.05, 0.07], 0.0, 0.15),
        ("steel", [0.56, 0.57, 0.58], 1.0, 0.45),
        ("asphalt", [0.10, 0.10, 0.11], 0.0, 0.90),
        ("paint", [0.80, 0.80, 0.77], 0.0, 0.50),
        ("wood", [0.30, 0.19, 0.10], 0.0, 0.80),
        ("bark", [0.20, 0.13, 0.08], 0.0, 0.90),
        ("leaves", [0.08, 0.23, 0.06], 0.0, 0.85),
        ("red", [0.55, 0.04, 0.03], 0.0, 0.35),
        ("blue", [0.04, 0.12, 0.45], 0.0, 0.35),
        ("rubber", [0.03, 0.03, 0.03], 0.0, 0.90),
        ("cable", [0.02, 0.02, 0.02], 0.0, 0.60),
        ("panel", [0.03, 0.05, 0.12], 0.0, 0.20),
    ];

    pub(crate) fn materials() -> Vec<(String, CookedMaterial)> {
        MATERIALS
            .iter()
            .map(|&(name, [red, green, blue], metallic, roughness)| {
                let material = CookedMaterial {
                    base_color: [red, green, blue, 1.0],
                    metallic,
                    roughness,
                    emissive: [0.0; 3],
                    normal_scale: 1.0,
                    textures: [None; 5],
                };
                (name.to_owned(), material)
            })
            .collect()
    }

    pub(crate) fn meshes() -> Vec<(&'static str, Vec<ScenePrimitive>)> {
        // A Primitive of `parts`, which names `material`.
        let of = |material, parts: Vec<Geometry>| (merged(parts), Some(material));
        let car = |paint| {
            let wheels = [(-0.8, -1.35), (0.8, -1.35), (-0.8, 1.35), (0.8, 1.35)].map(|(x, z)| {
                let axle = Quat::from_rotation_z(TAU / 4.0);
                moved(
                    cylinder(0.32, 0.32, 0.22, 10),
                    axle,
                    vec3(x + 0.11, 0.32, z),
                )
            });
            vec![
                of(
                    paint,
                    vec![cuboid(vec3(-0.85, 0.3, -2.1), vec3(0.85, 0.95, 2.1))],
                ),
                of(
                    "glass",
                    vec![cuboid(vec3(-0.75, 0.95, -1.0), vec3(0.75, 1.45, 1.1))],
                ),
                of("rubber", wheels.into()),
            ]
        };
        let tank_legs = [(-0.6, -0.6), (0.6, -0.6), (-0.6, 0.6), (0.6, 0.6)]
            .map(|(x, z)| cuboid(vec3(x - 0.06, 0.0, z - 0.06), vec3(x + 0.06, 1.2, z + 0.06)));
        let aerial_bars = [(2.0, 0.6), (2.6, 0.45), (3.2, 0.3)].map(|(height, half_length)| {
            cuboid(
                vec3(-half_length, height, -0.015),
                vec3(half_length, height + 0.03, 0.015),
            )
        });
        let crown = [(1.8, 1.8, 2.2), (3.0, 1.4, 2.0), (4.1, 1.0, 1.8)]
            .map(|(base, radius, height)| raised(cylinder(radius, 0.0, height, 10), base));
        let meshes = [
            (
                TownMesh::Ground,
                vec![of("asphalt", vec![ground_plane(550.0, 1.0)])],
            ),
            (
                TownMesh::Box,
                vec![(cuboid(vec3(-0.5, 0.0, -0.5), vec3(0.5, 1.0, 0.5)), None)],
            ),
            (
                TownMesh::AirConditioner,
                vec![of(
                    "paint",
                    vec![
                        cuboid(vec3(-0.6, 0.0, -0.45), vec3(0.6, 0.8, 0.45)),
                        raised(cylinder(0.32, 0.32, 0.08, 24), 0.8),
                        raised(cylinder(0.08, 0.08, 0.12, 8), 0.8),
                    ],
                )],
            ),
            (
                TownMesh::Vent,
                vec![of(
                    "steel",
                    vec![
                        cylinder(0.12, 0.12, 0.7, 16),
                        raised(cylinder(0.24, 0.0, 0.18, 16), 0.7),
                    ],
                )],
            ),
            (
                TownMesh::WaterTank,
                vec![of(
                    "wood",
                    [
                        raised(cylinder(1.0, 1.0, 1.8, 16), 1.2),
                        raised(cylinder(1.05, 0.0, 0.5, 16), 3.0),
                    ]
                    .into_iter()
                    .chain(tank_legs)
                    .collect(),
                )],
            ),
            (
                TownMesh::Dish,
                vec![of(
                    "paint",
                    vec![
                        cuboid(vec3(-0.04, 0.0, -0.04), vec3(0.04, 1.0, 0.04)),
                        moved(
                            cylinder(0.05, 0.45, 0.2, 24),
                            Quat::from_rotation_x(0.7),
                            vec3(0.0, 1.0, 0.0),
                        ),
                    ],
                )],
            ),
            (
                TownMesh::Aerial,
                vec![of(
                    "steel",
                    [cylinder(0.03, 0.02, 3.5, 6)]
                        .into_iter()
                        .chain(aerial_bars)
                        .collect(),
                )],
            ),
            (
                TownMesh::SolarPanel,
                vec![
                    of(
                        "steel",
                        vec![
                            cuboid(vec3(-0.7, 0.0, 0.3), vec3(-0.65, 0.75, 0.35)),
                            cuboid(vec3(0.65, 0.0, 0.3), vec3(0.7, 0.75, 0.35)),
                        ],
                    ),
                    of(
                        "panel",
                        vec![moved(
                            cuboid(vec3(-0.8, -0.02, -0.5), vec3(0.8, 0.02, 0.5)),
                            Quat::from_rotation_x(0.45),
                            vec3(0.0, 0.55, 0.05),
                        )],
                    ),
                ],
            ),
            (
                TownMesh::RailingPost,
                vec![of("steel", vec![cylinder(0.025, 0.025, 1.0, 6)])],
            ),
            (
                TownMesh::Lamp,
                vec![of(
                    "steel",
                    vec![
                        cylinder(0.09, 0.06, 5.2, 8),
                        cuboid(vec3(-0.03, 4.95, 0.0), vec3(0.03, 5.05, 1.3)),
                        cuboid(vec3(-0.18, 4.85, 0.95), vec3(0.18, 5.05, 1.45)),
                    ],
                )],
            ),
            (
                TownMesh::UtilityPole,
                vec![of(
                    "wood",
                    vec![
                        cylinder(0.13, 0.1, 8.0, 8),
                        cuboid(vec3(-0.06, 7.75, -0.9), vec3(0.06, 7.9, 0.9)),
                    ],
                )],
            ),
            (
                TownMesh::Wire,
                vec![of(
                    "cable",
                    vec![wire(super::POLE_SPACING as f32, 0.6, 0.015, 15, 3)],
                )],
            ),
            (
                TownMesh::Tree,
                vec![
                    of("bark", vec![cylinder(0.15, 0.12, 2.4, 6)]),
                    of("leaves", crown.into()),
                ],
            ),
            (
                TownMesh::Bench,
                vec![of(
                    "wood",
                    vec![
                        cuboid(vec3(-0.8, 0.42, -0.2), vec3(0.8, 0.47, 0.2)),
                        cuboid(vec3(-0.8, 0.5, -0.22), vec3(0.8, 0.9, -0.18)),
                        cuboid(vec3(-0.75, 0.0, -0.2), vec3(-0.65, 0.42, 0.2)),
                        cuboid(vec3(0.65, 0.0, -0.2), vec3(0.75, 0.42, 0.2)),
                    ],
                )],
            ),
            (TownMesh::RedCar, car("red")),
            (TownMesh::BlueCar, car("blue")),
            (
                TownMesh::Bollard,
                vec![of(
                    "steel",
                    vec![
                        cylinder(0.1, 0.1, 0.85, 8),
                        raised(cylinder(0.1, 0.0, 0.08, 8), 0.85),
                    ],
                )],
            ),
            (
                TownMesh::Bird,
                vec![of(
                    "slate",
                    vec![
                        cuboid(vec3(-0.04, -0.03, -0.2), vec3(0.04, 0.03, 0.2)),
                        cuboid(vec3(-0.3, -0.005, -0.06), vec3(0.3, 0.005, 0.08)),
                    ],
                )],
            ),
            (
                TownMesh::Bin,
                vec![of(
                    "steel",
                    vec![
                        cylinder(0.28, 0.3, 0.85, 12),
                        raised(cylinder(0.31, 0.31, 0.06, 12), 0.85),
                    ],
                )],
            ),
        ];
        meshes
            .into_iter()
            .map(|(mesh, primitives): (TownMesh, _)| (mesh.name(), primitives))
            .collect()
    }

    /// `geometry` raised by `height` along +Y.
    fn raised(geometry: Geometry, height: f32) -> Geometry {
        moved(geometry, Quat::IDENTITY, vec3(0.0, height, 0.0))
    }

    /// `parts` as one Geometry.
    fn merged(parts: impl IntoIterator<Item = Geometry>) -> Geometry {
        let mut geometry = Geometry::default();
        for part in parts {
            let first = geometry.positions.len() as u32;
            geometry.positions.extend(part.positions);
            geometry.normals.extend(part.normals);
            geometry.uvs.extend(part.uvs);
            geometry
                .indices
                .extend(part.indices.iter().map(|index| first + index));
        }
        geometry
    }

    /// `geometry` turned by `rotation`, then moved by `translation`.
    fn moved(mut geometry: Geometry, rotation: Quat, translation: Vec3) -> Geometry {
        for position in &mut geometry.positions {
            *position = (rotation * Vec3::from(*position) + translation).into();
        }
        for normal in &mut geometry.normals {
            *normal = (rotation * Vec3::from(*normal)).into();
        }
        geometry
    }

    /// Appends the quad from `corner` along `first` then `second`, facing
    /// their cross product, its UVs the unit square.
    fn quad(geometry: &mut Geometry, corner: Vec3, first: Vec3, second: Vec3) {
        let base = geometry.positions.len() as u32;
        let normal = first.cross(second).normalize();
        for (offset, uv) in [
            (Vec3::ZERO, [0.0, 0.0]),
            (first, [1.0, 0.0]),
            (first + second, [1.0, 1.0]),
            (second, [0.0, 1.0]),
        ] {
            geometry.positions.push((corner + offset).into());
            geometry.normals.push(normal.into());
            geometry.uvs.push(uv);
        }
        geometry
            .indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// A box from `min` to `max`, each face with its own vertices.
    fn cuboid(min: Vec3, max: Vec3) -> Geometry {
        let size = max - min;
        let (x, y, z) = (Vec3::X * size.x, Vec3::Y * size.y, Vec3::Z * size.z);
        let mut geometry = Geometry::default();
        quad(&mut geometry, vec3(max.x, min.y, max.z), -z, y);
        quad(&mut geometry, min, z, y);
        quad(&mut geometry, vec3(min.x, max.y, max.z), x, -z);
        quad(&mut geometry, min, x, z);
        quad(&mut geometry, vec3(min.x, min.y, max.z), x, y);
        quad(&mut geometry, vec3(max.x, min.y, min.z), -x, y);
        geometry
    }

    /// A cone's frustum about +Y from radius `bottom` at the origin to `top`
    /// at `height`, of `sides` faces, smooth, closed by a cap at each end
    /// that has a radius.
    fn cylinder(bottom: f32, top: f32, height: f32, sides: u32) -> Geometry {
        let mut geometry = Geometry::default();
        // Round from +X towards −Z, so each face winds counter-clockwise
        // seen from outside.
        let around = |side: u32| {
            let angle = TAU * side as f32 / sides as f32;
            vec3(angle.cos(), 0.0, -angle.sin())
        };
        for side in 0..=sides {
            let radial = around(side);
            let normal = (radial * height + Vec3::Y * (bottom - top)).normalize();
            let u = side as f32 / sides as f32;
            for (radius, y, v) in [(bottom, 0.0, 0.0), (top, height, 1.0)] {
                geometry
                    .positions
                    .push((radial * radius + Vec3::Y * y).into());
                geometry.normals.push(normal.into());
                geometry.uvs.push([u, v]);
            }
        }
        for side in 0..sides {
            let [bottom_left, top_left] = [2 * side, 2 * side + 1];
            let [bottom_right, top_right] = [bottom_left + 2, top_left + 2];
            geometry
                .indices
                .extend([bottom_left, bottom_right, top_right]);
            if top > 0.0 {
                geometry.indices.extend([bottom_left, top_right, top_left]);
            }
        }
        for (radius, y, normal) in [(bottom, 0.0, -Vec3::Y), (top, height, Vec3::Y)] {
            if radius <= 0.0 {
                continue;
            }
            let base = geometry.positions.len() as u32;
            for side in 0..sides {
                let radial = around(side);
                geometry
                    .positions
                    .push((radial * radius + Vec3::Y * y).into());
                geometry.normals.push(normal.into());
                geometry
                    .uvs
                    .push([0.5 + 0.5 * radial.x, 0.5 - 0.5 * radial.z]);
            }
            for side in 1..sides - 1 {
                let fan = [base, base + side, base + side + 1];
                // The bottom cap faces down, so it winds the other way.
                geometry.indices.extend(if normal.y > 0.0 {
                    fan
                } else {
                    [fan[0], fan[2], fan[1]]
                });
            }
        }
        geometry
    }

    /// A wire hung from the origin to `span` along +X, sagging `sag` at
    /// mid-span along a parabola, the catenary's shallow-sag form: a tube of
    /// `sides` faces and `radius` over `segments` straight lengths.
    fn wire(span: f32, sag: f32, radius: f32, segments: u32, sides: u32) -> Geometry {
        let mut geometry = Geometry::default();
        for segment in 0..=segments {
            let along = segment as f32 / segments as f32;
            let centre = vec3(span * along, -4.0 * sag * along * (1.0 - along), 0.0);
            let tangent = vec3(span, -4.0 * sag * (1.0 - 2.0 * along), 0.0).normalize();
            let normal = Vec3::Z.cross(tangent);
            for side in 0..=sides {
                let angle = TAU * side as f32 / sides as f32;
                let outward = normal * angle.cos() + Vec3::Z * angle.sin();
                geometry.positions.push((centre + outward * radius).into());
                geometry.normals.push(outward.into());
                geometry.uvs.push([side as f32 / sides as f32, along]);
            }
        }
        let ring = sides + 1;
        for segment in 0..segments {
            for side in 0..sides {
                let corner = segment * ring + side;
                let [next_side, next_segment] = [corner + 1, corner + ring];
                geometry.indices.extend([
                    corner,
                    next_side,
                    next_segment + 1,
                    corner,
                    next_segment + 1,
                    next_segment,
                ]);
            }
        }
        geometry
    }
}
