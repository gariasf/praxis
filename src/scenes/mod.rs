use std::f32::consts::{PI, TAU};

use anyhow::Context;
use bevy_ecs::prelude::*;

use crate::assets::loader::generate_tangents;
use crate::assets::material::TEXTURE_DIM;
use crate::assets::{
    LoadedImage, MaterialChannel, MaterialData, MaterialHandle, MaterialPool, Mesh, MeshHandle,
    MeshPool, Primitive, load_model,
};
use crate::camera::Camera;
use crate::components::{MaterialRef, MeshRef, Transform};
use crate::render::Vertex;

/// The Scenes `capture` renders, each with a camera path in frames.
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
}

impl ReferenceScene {
    pub const ALL: [Self; 5] = [
        Self::Helmet,
        Self::PbrSweep,
        Self::UvGrid,
        Self::NormalTangent,
        Self::FarCoplanar,
    ];

    /// The name the capture command selects the scene by.
    pub fn name(self) -> &'static str {
        match self {
            Self::Helmet => "helmet",
            Self::PbrSweep => "pbr-sweep",
            Self::UvGrid => "uv-grid",
            Self::NormalTangent => "normal-tangent",
            Self::FarCoplanar => "far-coplanar",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scene| scene.name() == name)
    }

    /// The camera at `frame`. The path is a function of the frame number, not
    /// of time, so it does not depend on machine speed.
    pub(crate) fn camera_at(self, frame: u32) -> Camera {
        match self {
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
const SWEEP_ROUGHNESS: [f32; 5] = [0.1, 0.3, 0.5, 0.7, 0.9];
const SPHERE_RADIUS: f32 = 0.5;
const SPHERE_SPACING: f32 = 1.3;

pub(crate) fn spawn_pbr_sweep(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) {
    let sphere = insert_mesh(device, queue, mesh_pool, &uv_sphere(SPHERE_RADIUS));
    for (row, metallic) in [0.0_f32, 1.0].into_iter().enumerate() {
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
}

pub(crate) fn spawn_uv_grid(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) {
    let plane = insert_mesh(device, queue, mesh_pool, &ground_plane(4.0, 2.0));
    let albedo_layer = material_pool.upload(
        queue,
        MaterialChannel::Albedo,
        &grid_texture(),
        TEXTURE_DIM,
        TEXTURE_DIM,
    );
    // Rough so the highlight does not wash out the texture.
    let material = material_pool.insert(queue, factor_material([1.0; 4], 0.0, 0.8, albedo_layer));
    world.spawn((Transform::IDENTITY, MeshRef(plane), MaterialRef(material)));
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
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) -> anyhow::Result<()> {
    let (mesh, material) = insert_gltf_model(
        device,
        queue,
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

pub(crate) fn spawn_far_coplanar(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
) {
    let plane = insert_mesh(
        device,
        queue,
        mesh_pool,
        &ground_plane(FAR_COPLANAR_DISTANCE as f32, 1.0),
    );
    let centre = glam::dvec3(0.0, 0.0, -FAR_COPLANAR_DISTANCE);
    for (height, base_color) in [
        (0.0, [0.8, 0.1, 0.1, 1.0]),
        (-FAR_COPLANAR_GAP, [0.1, 0.8, 0.1, 1.0]),
    ] {
        let material = material_pool.insert(
            queue,
            factor_material(base_color, 0.0, 0.8, MaterialPool::DEFAULT_LAYER),
        );
        world.spawn((
            Transform::from_translation(centre + glam::DVec3::Y * height),
            MeshRef(plane),
            MaterialRef(material),
        ));
    }
}

/// Loads the glTF file at `path`, relative to the working directory, as one
/// Mesh, and inserts the Material of its first Primitive with each of its
/// textures uploaded to its Channel's array.
pub(crate) fn insert_gltf_model(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh_pool: &mut MeshPool,
    material_pool: &mut MaterialPool,
    path: &str,
) -> anyhow::Result<(MeshHandle, MaterialHandle)> {
    let model = load_model(path).with_context(|| format!("load {path}"))?;

    let mut primitives = Vec::new();
    for (primitive_index, primitive) in model.primitives.iter().enumerate() {
        let _primitive_span = tracing::debug_span!("upload_primitive", primitive_index).entered();

        let (vertex_offset, index_offset, index_count) =
            mesh_pool.push_primitive(device, queue, &primitive.vertices, &primitive.indices);

        tracing::debug!(
            vertices = primitive.vertices.len(),
            indices = primitive.indices.len(),
            "primitive uploaded"
        );

        primitives.push(Primitive {
            vertex_offset,
            index_offset,
            index_count,
        });
    }
    let mesh = Mesh { primitives };
    let mesh_handle = mesh_pool.insert(mesh);

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
            Some(image) => {
                material_pool.upload(queue, channel, &image.pixels, image.width, image.height)
            }
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
#[derive(Default)]
struct Geometry {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

/// Uploads `geometry` as a one-Primitive Mesh, with MikkTSpace tangents as a
/// glTF file without tangents gets them.
fn insert_mesh(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh_pool: &mut MeshPool,
    geometry: &Geometry,
) -> MeshHandle {
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
    let (vertex_offset, index_offset, index_count) =
        mesh_pool.push_primitive(device, queue, &vertices, &geometry.indices);
    mesh_pool.insert(Mesh {
        primitives: vec![Primitive {
            vertex_offset,
            index_offset,
            index_count,
        }],
    })
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
fn ground_plane(half_size: f32, uv_repeat: f32) -> Geometry {
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

/// `TEXTURE_DIM`-square RGBA8 sRGB texture of 8 × 8 cells framed by dark
/// lines. Red grows with the cell's column and green with its row, and blue
/// alternates as a checker, so every cell shows where it sits in UV space.
fn grid_texture() -> Vec<u8> {
    const CELLS: u32 = 8;
    const CELL_SIZE: u32 = TEXTURE_DIM / CELLS;
    const LINE_WIDTH: u32 = 8;

    let ramp = |cell: u32| (cell * 255 / (CELLS - 1)) as u8;
    let mut pixels = Vec::with_capacity((TEXTURE_DIM * TEXTURE_DIM * 4) as usize);
    for y in 0..TEXTURE_DIM {
        for x in 0..TEXTURE_DIM {
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
