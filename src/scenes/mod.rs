use std::f32::consts::{PI, TAU};
use std::time::Duration;

use anyhow::Context;
use bevy_ecs::prelude::*;

use crate::assets::loader::generate_tangents;
use crate::assets::{
    LoadedImage, MaterialChannel, MaterialData, MaterialHandle, MaterialPool, MeshHandle, MeshPool,
    load_model,
};
use crate::camera::Camera;
use crate::components::{MaterialRef, MeshRef, Transform};
#[cfg(feature = "cook")]
use crate::cooked::{AssetId, AssetKind, AssetName, AssetSource, CookedMaterial};
use crate::render::{LoadStaging, Vertex};
use crate::time::{FixedTime, SIXTY_HERTZ, TICK_SECONDS};
use crate::transform::PreviousWorldTransform;

/// The Reference scenes `capture` renders, each with a camera path in frames
/// and a frame delta.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReferenceScene {
    /// SciFiHelmet and a material sweep, fetched into `assets/`; local runs only.
    Helmet,
    /// Spheres across metallic (rows) and roughness (columns) on neutral Layers.
    PbrSweep,
    /// A ground plane with a generated grid texture, tiled twice.
    UvGrid,
    /// Khronos NormalTangentTest, committed in `assets/`, seen face on: real
    /// bumps beside normal-mapped quads that should shade the same.
    NormalTangent,
    /// Two ground planes of contrasting colour 1 cm apart, 4 km square and
    /// centred 2 km in front of the camera, seen at a grazing angle: the upper plane covers the lower one everywhere
    /// under reversed depth, and they interleave under standard depth.
    FarCoplanar,
    /// Five textured spheres, each circling and spinning in its parent's
    /// frame, moved per Tick and drawn blended between Ticks (ADR 0022).
    Orrery,
}

impl ReferenceScene {
    pub const ALL: [Self; 6] = [
        Self::Helmet,
        Self::PbrSweep,
        Self::UvGrid,
        Self::NormalTangent,
        Self::FarCoplanar,
        Self::Orrery,
    ];

    /// The name the capture command selects the scene by.
    pub fn name(self) -> &'static str {
        match self {
            Self::Helmet => "helmet",
            Self::PbrSweep => "pbr-sweep",
            Self::UvGrid => "uv-grid",
            Self::NormalTangent => "normal-tangent",
            Self::FarCoplanar => "far-coplanar",
            Self::Orrery => "orrery",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scene| scene.name() == name)
    }

    /// The scene's procedural Materials by name; glTF Materials cook from
    /// their manifest source.
    #[cfg(feature = "cook")]
    pub(crate) fn materials(self) -> Vec<(String, CookedMaterial)> {
        let factors = |base_color, metallic, roughness, albedo: Option<&str>| CookedMaterial {
            base_color,
            metallic,
            roughness,
            emissive: [0.0; 3],
            textures: [
                albedo.map(|name| {
                    let source = AssetSource::Scene(self.name());
                    let kind = AssetKind::Texture(MaterialChannel::Albedo);
                    AssetId::new(source, kind, AssetName::Name(name))
                }),
                None,
                None,
                None,
                None,
            ],
        };
        let sweep = |base_color| {
            SWEEP_METALLIC.into_iter().flat_map(move |metallic| {
                SWEEP_ROUGHNESS.into_iter().map(move |roughness| {
                    let name = format!("metallic-{metallic}-roughness-{roughness}");
                    (name, factors(base_color, metallic, roughness, None))
                })
            })
        };
        match self {
            Self::Helmet => sweep(HELMET_SWEEP_BASE_COLOR).collect(),
            Self::PbrSweep => sweep(SWEEP_BASE_COLOR).collect(),
            Self::UvGrid => vec![(
                "grid".into(),
                factors([1.0; 4], 0.0, UV_GRID_ROUGHNESS, Some("grid")),
            )],
            Self::NormalTangent => Vec::new(),
            Self::FarCoplanar => ["upper", "lower"]
                .into_iter()
                .zip(FAR_COPLANAR_COLORS)
                .map(|(name, base_color)| {
                    let material = factors(base_color, 0.0, FAR_COPLANAR_ROUGHNESS, None);
                    (name.into(), material)
                })
                .collect(),
            Self::Orrery => vec![(
                "grid".into(),
                factors([1.0; 4], 0.0, ORRERY_ROUGHNESS, Some("grid")),
            )],
        }
    }

    /// The scene's generated Textures by name, with their Channel.
    #[cfg(feature = "cook")]
    pub(crate) fn textures(self) -> Vec<(&'static str, MaterialChannel, LoadedImage)> {
        match self {
            Self::UvGrid | Self::Orrery => {
                let grid = LoadedImage {
                    pixels: grid_texture(),
                    width: GRID_TEXTURE_SIZE,
                    height: GRID_TEXTURE_SIZE,
                };
                vec![("grid", MaterialChannel::Albedo, grid)]
            }
            Self::Helmet | Self::PbrSweep | Self::NormalTangent | Self::FarCoplanar => Vec::new(),
        }
    }

    /// The scene's procedural Meshes by name, each one Primitive with the
    /// name of its Material, if its Renderables do not each override it;
    /// glTF Meshes cook from their manifest source.
    #[cfg(feature = "cook")]
    pub(crate) fn meshes(self) -> Vec<(&'static str, Geometry, Option<&'static str>)> {
        match self {
            Self::Helmet | Self::NormalTangent => Vec::new(),
            Self::PbrSweep => vec![("sphere", uv_sphere(SPHERE_RADIUS), None)],
            Self::UvGrid => vec![("plane", ground_plane(4.0, 2.0), Some("grid"))],
            Self::FarCoplanar => vec![(
                "plane",
                ground_plane(FAR_COPLANAR_DISTANCE as f32, 1.0),
                None,
            )],
            Self::Orrery => vec![("sphere", uv_sphere(SPHERE_RADIUS), Some("grid"))],
        }
    }

    /// The camera at `frame`, moved by `shift` metres. The path is a function
    /// of the frame number, not of time, so it does not depend on machine
    /// speed.
    pub(crate) fn camera_at(self, frame: u32, shift: glam::DVec3) -> Camera {
        let mut camera = match self {
            Self::Helmet => Camera::new(),
            Self::PbrSweep => orbit(frame, 6.5, 1.0),
            Self::UvGrid => orbit(frame, 9.0, 7.0),
            // A fixed camera, face on: normal mapping holds up best there, per
            // github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/NormalTangentTest
            Self::NormalTangent => Camera::looking_at(
                glam::dvec3(0.0, NORMAL_TANGENT_CENTRE_HEIGHT, 3.2),
                glam::dvec3(0.0, NORMAL_TANGENT_CENTRE_HEIGHT, 0.0),
            ),
            Self::FarCoplanar => Camera::looking_at(
                glam::dvec3(0.0, FAR_COPLANAR_CAMERA_HEIGHT, 0.0),
                glam::dvec3(0.0, 0.0, -FAR_COPLANAR_DISTANCE),
            ),
            Self::Orrery => {
                Camera::looking_at(glam::dvec3(0.5, 3.6, 4.2), glam::dvec3(1.1, 0.0, -0.7))
            }
        };
        camera.position += shift;
        camera
    }

    /// The time each frame of a capture advances by. The orrery's frames, at
    /// 144 Hz, land at a different point between Ticks each time, and the
    /// last lands two thirds of the way from one Tick to the next.
    pub(crate) fn frame_delta(self) -> Duration {
        match self {
            Self::Orrery => Duration::from_nanos(6_944_444),
            Self::Helmet
            | Self::PbrSweep
            | Self::UvGrid
            | Self::NormalTangent
            | Self::FarCoplanar => SIXTY_HERTZ,
        }
    }
}

/// A camera on a circle of `radius` at `height` around the origin, looking at
/// it, starting a little left of +Z and moving a quarter degree per frame.
fn orbit(frame: u32, radius: f32, height: f32) -> Camera {
    let angle = (-7.5 + 0.25 * frame as f32).to_radians();
    let position = glam::vec3(radius * angle.sin(), height, radius * angle.cos());
    Camera::looking_at(position.as_dvec3(), glam::DVec3::ZERO)
}

/// Linear sRGB reflectance of gold at normal incidence: metals tint their
/// highlight with it and dielectrics their diffuse, so the two rows differ.
const SWEEP_BASE_COLOR: [f32; 4] = [1.0, 0.71, 0.29, 1.0];
/// The helmet scene's sweep is white, so the factors alone show.
pub(crate) const HELMET_SWEEP_BASE_COLOR: [f32; 4] = [1.0; 4];
pub(crate) const SWEEP_METALLIC: [f32; 2] = [0.0, 1.0];
pub(crate) const SWEEP_ROUGHNESS: [f32; 5] = [0.1, 0.3, 0.5, 0.7, 0.9];
const SPHERE_RADIUS: f32 = 0.5;
const SPHERE_SPACING: f32 = 1.3;

pub(crate) fn spawn_pbr_sweep(
    world: &mut World,
    queue: &wgpu::Queue,
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) -> anyhow::Result<()> {
    let sphere = insert_mesh(load_staging, mesh_pool, &uv_sphere(SPHERE_RADIUS))?;
    for (row, metallic) in SWEEP_METALLIC.into_iter().enumerate() {
        for (column, roughness) in SWEEP_ROUGHNESS.into_iter().enumerate() {
            let material = material_pool.insert(
                queue,
                factor_material(
                    SWEEP_BASE_COLOR,
                    metallic,
                    roughness,
                    MaterialPool::DEFAULT_LAYER,
                ),
            );
            let position = glam::vec3(
                (column as f32 - (SWEEP_ROUGHNESS.len() - 1) as f32 / 2.0) * SPHERE_SPACING,
                (0.5 - row as f32) * SPHERE_SPACING,
                0.0,
            );
            world.spawn((
                Transform::from_translation(position.as_dvec3()),
                MeshRef(sphere),
                MaterialRef(material),
            ));
        }
    }
    Ok(())
}

/// Rough so the highlight does not wash out the texture.
const UV_GRID_ROUGHNESS: f32 = 0.8;

pub(crate) fn spawn_uv_grid(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) -> anyhow::Result<()> {
    let plane = insert_mesh(load_staging, mesh_pool, &ground_plane(4.0, 2.0))?;
    let albedo_layer = material_pool.upload(
        device,
        queue,
        MaterialChannel::Albedo,
        &grid_texture(),
        GRID_TEXTURE_SIZE,
        GRID_TEXTURE_SIZE,
    );
    let material = material_pool.insert(
        queue,
        factor_material([1.0; 4], 0.0, UV_GRID_ROUGHNESS, albedo_layer),
    );
    world.spawn((Transform::IDENTITY, MeshRef(plane), MaterialRef(material)));
    Ok(())
}

/// Committed, so CI renders it; 1.8 MB, under the 2 MB commit ceiling
/// (`docs/research/test-assets.md`).
const NORMAL_TANGENT_ASSET: &str = "assets/NormalTangentTest/NormalTangentTest.glb";
/// The middle of the model's bounds, which run from -1.2 to 0.95 in Y.
const NORMAL_TANGENT_CENTRE_HEIGHT: f64 = -0.125;

pub(crate) fn spawn_normal_tangent(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) -> anyhow::Result<()> {
    let (mesh, material) = insert_gltf_model(
        device,
        queue,
        load_staging,
        mesh_pool,
        material_pool,
        NORMAL_TANGENT_ASSET,
    )?;
    world.spawn((Transform::IDENTITY, MeshRef(mesh), MaterialRef(material)));
    Ok(())
}

/// From the camera to the planes' centre, and from the centre to each edge.
const FAR_COPLANAR_DISTANCE: f64 = 2000.0;
/// Looks down about 8.5° at the planes' centre, so a ray crosses the 1 cm gap
/// over about 7 cm.
const FAR_COPLANAR_CAMERA_HEIGHT: f64 = 300.0;
/// Between the two planes, in metres.
const FAR_COPLANAR_GAP: f64 = 0.01;
/// The upper plane's base colour, then the lower one's.
const FAR_COPLANAR_COLORS: [[f32; 4]; 2] = [[0.8, 0.1, 0.1, 1.0], [0.1, 0.8, 0.1, 1.0]];
const FAR_COPLANAR_ROUGHNESS: f32 = 0.8;

pub(crate) fn spawn_far_coplanar(
    world: &mut World,
    queue: &wgpu::Queue,
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) -> anyhow::Result<()> {
    let plane = insert_mesh(
        load_staging,
        mesh_pool,
        &ground_plane(FAR_COPLANAR_DISTANCE as f32, 1.0),
    )?;
    let centre = glam::dvec3(0.0, 0.0, -FAR_COPLANAR_DISTANCE);
    for (height, base_color) in [0.0, -FAR_COPLANAR_GAP]
        .into_iter()
        .zip(FAR_COPLANAR_COLORS)
    {
        let material = material_pool.insert(
            queue,
            factor_material(
                base_color,
                0.0,
                FAR_COPLANAR_ROUGHNESS,
                MaterialPool::DEFAULT_LAYER,
            ),
        );
        world.spawn((
            Transform::from_translation(centre + glam::DVec3::Y * height),
            MeshRef(plane),
            MaterialRef(material),
        ));
    }
    Ok(())
}

/// Each orrery body's spin about its own +Y, the root's first, in radians per
/// second: its child circles it at that rate.
const ORRERY_SPIN_RATES: [f64; 5] = [0.5, 1.5, 2.5, 3.5, 4.5];
/// Each orrery body's size relative to its parent's.
const ORRERY_SCALE: f32 = 0.7;
/// Each orrery body's distance from its parent, in its parent's diameters.
const ORRERY_ORBIT_RADIUS: f64 = 1.6;
const ORRERY_ROUGHNESS: f32 = 0.5;

/// A chain of five spheres from a root at the origin, each a child of the
/// one before, on its parent's +X axis, and all spinning about their own +Y
/// in the fixed schedule, so each circles its parent.
pub(crate) fn spawn_orrery(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) -> anyhow::Result<()> {
    let sphere = insert_mesh(load_staging, mesh_pool, &uv_sphere(SPHERE_RADIUS))?;
    // The grid shows each sphere's own spin.
    let albedo_layer = material_pool.upload(
        device,
        queue,
        MaterialChannel::Albedo,
        &grid_texture(),
        GRID_TEXTURE_SIZE,
        GRID_TEXTURE_SIZE,
    );
    let material = material_pool.insert(
        queue,
        factor_material([1.0; 4], 0.0, ORRERY_ROUGHNESS, albedo_layer),
    );
    let body = |radians_per_second| {
        (
            MeshRef(sphere),
            MaterialRef(material),
            Spin { radians_per_second },
            PreviousWorldTransform::default(),
        )
    };
    let mut parent = world
        .spawn((Transform::IDENTITY, body(ORRERY_SPIN_RATES[0])))
        .id();
    for &radians_per_second in &ORRERY_SPIN_RATES[1..] {
        let transform = Transform {
            scale: glam::Vec3::splat(ORRERY_SCALE),
            ..Transform::from_translation(glam::DVec3::X * ORRERY_ORBIT_RADIUS)
        };
        parent = world
            .spawn((transform, ChildOf(parent), body(radians_per_second)))
            .id();
    }
    Ok(())
}

/// Turns an entity about its own +Y axis at a constant rate from angle 0 at
/// Tick 0.
#[derive(Component)]
pub(crate) struct Spin {
    pub(crate) radians_per_second: f64,
}

/// Sets each spinning entity's rotation for the end of the running Tick, from
/// the Tick count, so no rounding accumulates over Ticks.
pub(crate) fn spin(fixed_time: Res<FixedTime>, mut spinning: Query<(&Spin, &mut Transform)>) {
    let seconds = (fixed_time.tick + 1) as f64 * TICK_SECONDS;
    for (spin, mut transform) in &mut spinning {
        let angle = (spin.radians_per_second * seconds).rem_euclid(std::f64::consts::TAU);
        transform.rotation = glam::Quat::from_rotation_y(angle as f32);
    }
}

/// Loads the glTF file at `path`, relative to the working directory, as one
/// Mesh, and inserts the Material of its first Primitive with each of its
/// textures uploaded to its Channel's array.
pub(crate) fn insert_gltf_model(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
    path: &str,
) -> anyhow::Result<(MeshHandle, MaterialHandle)> {
    let model = load_model(path).with_context(|| format!("load {path}"))?;

    let primitives: Vec<_> = model
        .primitives
        .iter()
        .map(|primitive| (primitive.vertices.as_slice(), primitive.indices.as_slice()))
        .collect();
    let mesh_handle = mesh_pool
        .load(load_staging, &primitives)
        .with_context(|| format!("{path}: the Arenas refused its Mesh"))?;

    // Multi-material meshes (one MaterialRef per primitive) are deferred.
    if model.primitives.len() > 1 {
        tracing::warn!(
            primitives = model.primitives.len(),
            "multi-primitive model; only primitive 0's material is used (multi-material deferred)"
        );
    }
    let first_primitive = model
        .primitives
        .first()
        .ok_or_else(|| anyhow::anyhow!("model has no primitives"))?;
    let mut upload_channel = |channel: MaterialChannel, image: Option<&LoadedImage>| -> u32 {
        match image {
            Some(image) => material_pool.upload(
                device,
                queue,
                channel,
                &image.pixels,
                image.width,
                image.height,
            ),
            None => {
                tracing::debug!(
                    ?channel,
                    "model has no texture for channel; using the neutral layer"
                );
                MaterialPool::DEFAULT_LAYER
            }
        }
    };
    let albedo_layer = upload_channel(
        MaterialChannel::Albedo,
        first_primitive.base_color_image.as_ref(),
    );
    let normal_layer = upload_channel(
        MaterialChannel::Normal,
        first_primitive.normal_image.as_ref(),
    );
    let metallic_roughness_layer = upload_channel(
        MaterialChannel::MetallicRoughness,
        first_primitive.metallic_roughness_image.as_ref(),
    );
    let occlusion_layer = upload_channel(
        MaterialChannel::Occlusion,
        first_primitive.occlusion_image.as_ref(),
    );
    let emissive_layer = upload_channel(
        MaterialChannel::Emissive,
        first_primitive.emissive_image.as_ref(),
    );
    let material = MaterialData {
        base_color: first_primitive.base_color_factor,
        metallic_roughness: [
            first_primitive.metallic_factor,
            first_primitive.roughness_factor,
            0.0,
            0.0,
        ],
        emissive: [
            first_primitive.emissive_factor[0],
            first_primitive.emissive_factor[1],
            first_primitive.emissive_factor[2],
            1.0,
        ],
        texture_indices: [
            albedo_layer,
            normal_layer,
            metallic_roughness_layer,
            occlusion_layer,
        ],
        extra: [emissive_layer, 0, 0, 0],
    };
    let material_handle = material_pool.insert(queue, material);

    Ok((mesh_handle, material_handle))
}

/// Factors only, with `albedo_layer` and the neutral Layer for every other
/// Channel.
fn factor_material(
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    albedo_layer: u32,
) -> MaterialData {
    MaterialData {
        base_color,
        metallic_roughness: [metallic, roughness, 0.0, 0.0],
        emissive: [0.0, 0.0, 0.0, 1.0],
        texture_indices: [
            albedo_layer,
            MaterialPool::DEFAULT_LAYER,
            MaterialPool::DEFAULT_LAYER,
            MaterialPool::DEFAULT_LAYER,
        ],
        extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
    }
}

/// An indexed triangle list, counter-clockwise when seen from outside.
#[derive(Clone, Default)]
pub(crate) struct Geometry {
    pub(crate) positions: Vec<[f32; 3]>,
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) uvs: Vec<[f32; 2]>,
    pub(crate) indices: Vec<u32>,
}

/// Loads `geometry` as a one-Primitive Mesh, with MikkTSpace tangents as a
/// glTF file without tangents gets them.
fn insert_mesh(
    load_staging: &mut LoadStaging,
    mesh_pool: &mut MeshPool,
    geometry: &Geometry,
) -> anyhow::Result<MeshHandle> {
    let tangents = generate_tangents(
        &geometry.positions,
        &geometry.normals,
        &geometry.uvs,
        &geometry.indices,
    );
    let vertices: Vec<Vertex> = (0..geometry.positions.len())
        .map(|vertex_index| Vertex {
            position: geometry.positions[vertex_index],
            normal: geometry.normals[vertex_index],
            uv: geometry.uvs[vertex_index],
            tangent: tangents[vertex_index],
        })
        .collect();
    mesh_pool
        .load(load_staging, &[(&vertices, &geometry.indices)])
        .context("the Arenas refused a generated Mesh")
}

/// A sphere of `radius` around the origin. `u` runs once around from +Z
/// towards +X, `v` from the north pole to the south pole, as a glTF texture
/// maps top row first.
fn uv_sphere(radius: f32) -> Geometry {
    const LONGITUDE_SEGMENTS: u32 = 48;
    const LATITUDE_SEGMENTS: u32 = 24;

    let mut geometry = Geometry::default();
    for latitude in 0..=LATITUDE_SEGMENTS {
        let v = latitude as f32 / LATITUDE_SEGMENTS as f32;
        let polar_angle = PI * v;
        for longitude in 0..=LONGITUDE_SEGMENTS {
            let u = longitude as f32 / LONGITUDE_SEGMENTS as f32;
            let azimuth = TAU * u;
            let normal = [
                polar_angle.sin() * azimuth.sin(),
                polar_angle.cos(),
                polar_angle.sin() * azimuth.cos(),
            ];
            geometry.positions.push(normal.map(|axis| axis * radius));
            geometry.normals.push(normal);
            geometry.uvs.push([u, v]);
        }
    }

    let row_length = LONGITUDE_SEGMENTS + 1;
    for latitude in 0..LATITUDE_SEGMENTS {
        for longitude in 0..LONGITUDE_SEGMENTS {
            let top_left = latitude * row_length + longitude;
            let top_right = top_left + 1;
            let bottom_left = top_left + row_length;
            let bottom_right = bottom_left + 1;
            // At a pole one edge of the quad collapses to a point; skip the
            // triangle that would have no area.
            if latitude != LATITUDE_SEGMENTS - 1 {
                geometry
                    .indices
                    .extend([top_left, bottom_left, bottom_right]);
            }
            if latitude != 0 {
                geometry.indices.extend([top_left, bottom_right, top_right]);
            }
        }
    }
    geometry
}

/// A square in the XZ plane, facing +Y, `half_size` metres from the origin
/// to each edge. `u` grows along +X and `v` along +Z, both to `uv_repeat`.
pub(crate) fn ground_plane(half_size: f32, uv_repeat: f32) -> Geometry {
    Geometry {
        positions: vec![
            [-half_size, 0.0, -half_size],
            [-half_size, 0.0, half_size],
            [half_size, 0.0, half_size],
            [half_size, 0.0, -half_size],
        ],
        normals: vec![[0.0, 1.0, 0.0]; 4],
        uvs: vec![
            [0.0, 0.0],
            [0.0, uv_repeat],
            [uv_repeat, uv_repeat],
            [uv_repeat, 0.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

/// Side of the grid texture, in texels.
const GRID_TEXTURE_SIZE: u32 = 2048;

/// `GRID_TEXTURE_SIZE`-square RGBA8 sRGB texture of 8 × 8 cells framed by dark
/// lines. Red grows with the cell's column and green with its row, and blue
/// alternates as a checker, so every cell shows where it sits in UV space.
fn grid_texture() -> Vec<u8> {
    const CELLS: u32 = 8;
    const CELL_SIZE: u32 = GRID_TEXTURE_SIZE / CELLS;
    const LINE_WIDTH: u32 = 8;

    let ramp = |cell: u32| (cell * 255 / (CELLS - 1)) as u8;
    let mut pixels = Vec::with_capacity((GRID_TEXTURE_SIZE * GRID_TEXTURE_SIZE * 4) as usize);
    for y in 0..GRID_TEXTURE_SIZE {
        for x in 0..GRID_TEXTURE_SIZE {
            let (column, row) = (x / CELL_SIZE, y / CELL_SIZE);
            let texel = if x % CELL_SIZE < LINE_WIDTH || y % CELL_SIZE < LINE_WIDTH {
                [16, 16, 16, 255]
            } else {
                let blue = if (column + row) % 2 == 0 { 224 } else { 64 };
                [ramp(column), ramp(row), blue, 255]
            };
            pixels.extend(texel);
        }
    }
    pixels
}
