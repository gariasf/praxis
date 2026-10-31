#[cfg(feature = "cook")]
use std::f32::consts::{PI, TAU};
use std::time::Duration;

use bevy_ecs::prelude::*;

use crate::assets::{AssetLoader, MaterialChannel};
use crate::camera::{Camera, ev100_for_exposure};
use crate::components::{Bend, DirectionalLight, MaterialRef, MeshRef, PointLight, Transform};
#[cfg(feature = "cook")]
use crate::cook::LoadedImage;
#[cfg(feature = "cook")]
use crate::cooked::CookedMaterial;
use crate::cooked::{AssetId, AssetKind, AssetName, AssetSource};
use crate::environment::Environment;
use crate::render::CrowdForm;
use crate::time::{FixedTime, SIXTY_HERTZ, TICK_SECONDS};
use crate::transform::PreviousWorldTransform;

mod town;

pub(crate) use town::spawn_town;

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
    /// A glossy checker floor 100 m long, seen at a grazing angle under one
    /// raking point light: its fine checks and high-frequency bumps show
    /// mips, anisotropic filtering and Toksvig.
    Minification,
    /// 10,000 4×4 Textures resident in the albedo Channel, each one colour
    /// that encodes its index, and a 10 × 10 grid of quads, each with its own
    /// Material, sampling Layers across the whole range.
    TextureWall,
    /// A sphere over a ground plane, one Mesh whose two Primitives name
    /// their own Materials, under the sun and one point light: the sphere's
    /// shadow from the sun, traced per pixel, matches its analytic shadow.
    Shadows,
    /// 100 figures of 5,120 triangles, each its own Mesh, swaying in the
    /// fixed schedule as the GPU bends them each frame, a stand-in for
    /// skinned crowd members. Their Meshes have no BLAS, which would
    /// hold their rest shape: each casts its shadow through rigid shadow
    /// proxies, one per bone of its middle, Moving Instances in the dynamic
    /// TLAS.
    Crowd,
    /// A procedural town of 1 × 1 km at M3's target scale (ADR 0013): about
    /// 250,000 Instances, 10,000 of them pigeons circling in flocks, and
    /// 1,024 street lamps, seen from street level, then from over the
    /// rooftops (`town.rs`).
    Town,
}

impl ReferenceScene {
    pub const ALL: [Self; 11] = [
        Self::Helmet,
        Self::PbrSweep,
        Self::UvGrid,
        Self::NormalTangent,
        Self::FarCoplanar,
        Self::Orrery,
        Self::Minification,
        Self::TextureWall,
        Self::Shadows,
        Self::Crowd,
        Self::Town,
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
            Self::Minification => "minification",
            Self::TextureWall => "texture-wall",
            Self::Shadows => "shadows",
            Self::Crowd => "crowd",
            Self::Town => "town",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scene| scene.name() == name)
    }

    /// The Asset ID of the scene's procedural Material `name`.
    pub(crate) fn material_id(self, name: &str) -> AssetId {
        let source = AssetSource::Scene(self.name());
        AssetId::new(source, AssetKind::Material, AssetName::Name(name))
    }

    /// The Asset ID of the scene's procedural Mesh `name`.
    pub(crate) fn mesh_id(self, name: &str) -> AssetId {
        let source = AssetSource::Scene(self.name());
        AssetId::new(source, AssetKind::Mesh, AssetName::Name(name))
    }

    /// The Asset ID of the scene's procedural Texture `name` in `channel`.
    pub(crate) fn texture_id(self, channel: MaterialChannel, name: &str) -> AssetId {
        let source = AssetSource::Scene(self.name());
        AssetId::new(source, AssetKind::Texture(channel), AssetName::Name(name))
    }

    /// The manifest sources the scene references, by GUID.
    #[cfg(feature = "cook")]
    pub(crate) fn source_files(self) -> &'static [u128] {
        match self {
            Self::Helmet => &[SCI_FI_HELMET],
            Self::NormalTangent => &[NORMAL_TANGENT_TEST],
            Self::PbrSweep
            | Self::UvGrid
            | Self::FarCoplanar
            | Self::Orrery
            | Self::Minification
            | Self::TextureWall
            | Self::Shadows
            | Self::Crowd
            | Self::Town => &[],
        }
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
            normal_scale: 1.0,
            textures: [
                albedo.map(|name| self.texture_id(MaterialChannel::Albedo, name)),
                None,
                None,
                None,
                None,
            ],
        };
        let sweep = |base_color| {
            SWEEP_METALLIC.into_iter().flat_map(move |metallic| {
                SWEEP_ROUGHNESS.into_iter().map(move |roughness| {
                    let name = sweep_material_name(metallic, roughness);
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
            Self::Minification => vec![(
                "ground".into(),
                CookedMaterial {
                    // Factors of 1 pass the Textures' values through.
                    base_color: [1.0; 4],
                    metallic: 1.0,
                    roughness: 1.0,
                    emissive: [0.0; 3],
                    normal_scale: 1.0,
                    textures: [
                        Some(self.texture_id(MaterialChannel::Albedo, "checker")),
                        Some(self.texture_id(MaterialChannel::Normal, "bumps")),
                        Some(self.texture_id(MaterialChannel::MetallicRoughness, "gloss")),
                        None,
                        None,
                    ],
                },
            )],
            Self::TextureWall => (0..TEXTURE_WALL_QUADS)
                .map(|quad| {
                    let texture = wall_texture_name(quad * TEXTURE_WALL_TEXTURE_STRIDE);
                    let material = factors([1.0; 4], 0.0, TEXTURE_WALL_ROUGHNESS, Some(&texture));
                    (wall_material_name(quad), material)
                })
                .collect(),
            Self::Shadows => ["ground", "sphere"]
                .into_iter()
                .zip(SHADOWS_COLORS)
                .map(|(name, base_color)| {
                    let material = factors(base_color, 0.0, SHADOWS_ROUGHNESS, None);
                    (name.into(), material)
                })
                .collect(),
            Self::Crowd => ["ground", "figure"]
                .into_iter()
                .zip(CROWD_COLORS)
                .map(|(name, base_color)| {
                    let material = factors(base_color, 0.0, CROWD_ROUGHNESS, None);
                    (name.into(), material)
                })
                .collect(),
            Self::Town => town::materials(),
        }
    }

    /// The scene's generated Textures by name, with their Channel. A
    /// metallic-roughness Texture cooks widened by the normal Texture of the
    /// Material that samples it (Toksvig).
    #[cfg(feature = "cook")]
    pub(crate) fn textures(self) -> Vec<(String, MaterialChannel, LoadedImage)> {
        match self {
            Self::UvGrid | Self::Orrery => {
                let grid = LoadedImage {
                    pixels: grid_texture(),
                    width: GRID_TEXTURE_SIZE,
                    height: GRID_TEXTURE_SIZE,
                };
                vec![("grid".into(), MaterialChannel::Albedo, grid)]
            }
            Self::Minification => vec![
                ("checker".into(), MaterialChannel::Albedo, checker_texture()),
                ("bumps".into(), MaterialChannel::Normal, bump_texture()),
                (
                    "gloss".into(),
                    MaterialChannel::MetallicRoughness,
                    gloss_texture(),
                ),
            ],
            Self::TextureWall => (0..TEXTURE_WALL_TEXTURES)
                .map(|index| {
                    let texture = wall_texture(index);
                    (wall_texture_name(index), MaterialChannel::Albedo, texture)
                })
                .collect(),
            Self::Helmet
            | Self::PbrSweep
            | Self::NormalTangent
            | Self::FarCoplanar
            | Self::Shadows
            | Self::Crowd
            | Self::Town => Vec::new(),
        }
    }

    /// The scene's procedural Meshes by name, each with its Primitives; glTF
    /// Meshes cook from their manifest source.
    #[cfg(feature = "cook")]
    pub(crate) fn meshes(self) -> Vec<(&'static str, Vec<ScenePrimitive>)> {
        match self {
            Self::Helmet | Self::NormalTangent => Vec::new(),
            Self::PbrSweep => vec![("sphere", vec![(scene_sphere(SPHERE_RADIUS), None)])],
            Self::UvGrid => vec![("plane", vec![(ground_plane(4.0, 2.0), Some("grid"))])],
            Self::FarCoplanar => {
                let plane = ground_plane(FAR_COPLANAR_DISTANCE as f32, 1.0);
                vec![("plane", vec![(plane, None)])]
            }
            Self::Orrery => vec![("sphere", vec![(scene_sphere(SPHERE_RADIUS), Some("grid"))])],
            Self::Minification => {
                let ground = ground_plane(MINIFICATION_HALF_SIZE, MINIFICATION_UV_REPEAT);
                vec![("ground", vec![(ground, Some("ground"))])]
            }
            Self::TextureWall => {
                let quad = ground_plane(TEXTURE_WALL_QUAD_HALF_SIZE, 1.0);
                vec![("quad", vec![(quad, None)])]
            }
            Self::Shadows => {
                let mut sphere = scene_sphere(SHADOWS_SPHERE_RADIUS);
                for position in &mut sphere.positions {
                    position[1] += SHADOWS_SPHERE_HEIGHT;
                }
                let ground = ground_plane(SHADOWS_GROUND_HALF_SIZE, 1.0);
                let primitives = vec![(ground, Some("ground")), (sphere, Some("sphere"))];
                vec![("sphere-over-plane", primitives)]
            }
            // The figures' Material is their Renderables', so each Mesh
            // loaded for a figure shares it.
            Self::Crowd => {
                let ground = ground_plane(CROWD_GROUND_HALF_SIZE, 1.0);
                let [longitudes, latitudes] = CROWD_PROXY_SEGMENTS;
                vec![
                    ("ground", vec![(ground, Some("ground"))]),
                    ("figure", vec![(crowd_figure(), None)]),
                    ("proxy", vec![(uv_sphere(1.0, longitudes, latitudes), None)]),
                ]
            }
            Self::Town => town::meshes(),
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
            // github.com/KhronosGroup/glTF-Sample-Assets/tree/edc7c9e67c639d230715049ee31f9a96a6babbbe/Models/NormalTangentTest
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
            Self::Minification => {
                Camera::looking_at(MINIFICATION_CAMERA, MINIFICATION_CAMERA_TARGET)
            }
            Self::TextureWall => Camera::looking_at(TEXTURE_WALL_CAMERA, glam::DVec3::ZERO),
            Self::Shadows => Camera::looking_at(SHADOWS_CAMERA, SHADOWS_CAMERA_TARGET),
            Self::Crowd => Camera::looking_at(CROWD_CAMERA, CROWD_CAMERA_TARGET),
            Self::Town => town::camera_at(frame),
        };
        camera.position += shift;
        camera.ev100 = self.ev100();
        camera
    }

    /// The scene's exposure, an EV100 set by hand (ADR 0023). Each scene
    /// exposes the sun's illuminance to 1, as it was lit before its lights
    /// took photometric units: EV100 16.35.
    fn ev100(self) -> f32 {
        match self {
            Self::Helmet
            | Self::PbrSweep
            | Self::UvGrid
            | Self::NormalTangent
            | Self::FarCoplanar
            | Self::Orrery
            | Self::Minification
            | Self::TextureWall
            | Self::Shadows
            | Self::Crowd
            | Self::Town => ev100_for_exposure(1.0 / SUN_ILLUMINANCE),
        }
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
            | Self::FarCoplanar
            | Self::Minification
            | Self::TextureWall
            | Self::Shadows
            | Self::Crowd
            | Self::Town => SIXTY_HERTZ,
        }
    }

    /// The scene's point lights: position, colour, intensity and range.
    /// town's stand in its lamps, which its spawn places.
    pub(crate) fn point_lights(self) -> &'static [(glam::DVec3, PointLight)] {
        match self {
            Self::Minification => &[(
                MINIFICATION_LIGHT,
                PointLight {
                    color: glam::Vec3::ONE,
                    intensity: MINIFICATION_LIGHT_INTENSITY,
                    range: MINIFICATION_LIGHT_RANGE,
                    casts_shadow: true,
                },
            )],
            // The directional light alone, white, so each quad shows its
            // Texture's colour; and so the crowd's figures cast the sun's
            // shadows alone.
            Self::TextureWall | Self::Crowd => &[],
            // town's spawn places its lamps' lights, where it places the lamps.
            Self::Town => &[],
            Self::Shadows => &SHADOWS_LIGHTS,
            Self::Helmet
            | Self::PbrSweep
            | Self::UvGrid
            | Self::NormalTangent
            | Self::FarCoplanar
            | Self::Orrery => &COLOURED_LIGHTS,
        }
    }

    /// The scene's sun, white, of `SUN_ILLUMINANCE`: a directional light
    /// whose forward axis is turned to shine along the scene's direction.
    /// town's stands 33° up in the south-southwest, behind the camera's
    /// street, so the street takes the sun between long shadows.
    pub(crate) fn sun(self) -> (Transform, DirectionalLight) {
        let direction = match self {
            Self::Helmet
            | Self::PbrSweep
            | Self::UvGrid
            | Self::NormalTangent
            | Self::FarCoplanar
            | Self::Orrery
            | Self::Minification
            | Self::TextureWall
            | Self::Shadows
            | Self::Crowd => glam::vec3(0.5, -1.0, -0.3),
            Self::Town => glam::vec3(0.25, -0.55, -0.8),
        };
        let transform = Transform {
            rotation: glam::Quat::from_rotation_arc(glam::Vec3::NEG_Z, direction.normalize()),
            ..Transform::IDENTITY
        };
        let light = DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: SUN_ILLUMINANCE,
        };
        (transform, light)
    }

    /// The scene's surroundings: from all around, a tenth of the sun's
    /// illuminance, and a blue background whose luminance in cd/m² is a tenth
    /// to three tenths of the sun's illuminance in lux.
    pub(crate) fn environment(self) -> Environment {
        match self {
            Self::Helmet
            | Self::PbrSweep
            | Self::UvGrid
            | Self::NormalTangent
            | Self::FarCoplanar
            | Self::Orrery
            | Self::Minification
            | Self::TextureWall
            | Self::Shadows
            | Self::Crowd
            | Self::Town => Environment {
                ambient_illuminance: glam::Vec3::splat(0.1 * SUN_ILLUMINANCE),
                background_luminance: glam::vec3(0.1, 0.2, 0.3) * SUN_ILLUMINANCE,
            },
        }
    }
}

/// The sun's illuminance in every scene, in lux: direct sunlight, as Bevy's
/// `lux::DIRECT_SUNLIGHT` gives it [ref:bevy-lux-constants] (ADR 0023).
/// The scenes' other lights are given against it.
const SUN_ILLUMINANCE: f32 = 100_000.0;

/// Red, green, blue and yellow, 2 m up and 2 m from the origin along X and Z,
/// each giving the sun's illuminance 3 m away and reaching 20 m.
const COLOURED_LIGHTS: [(glam::DVec3, PointLight); 4] = [
    (
        glam::DVec3::new(2.0, 2.0, 2.0),
        coloured_light(glam::Vec3::X),
    ),
    (
        glam::DVec3::new(-2.0, 2.0, 2.0),
        coloured_light(glam::Vec3::Y),
    ),
    (
        glam::DVec3::new(2.0, 2.0, -2.0),
        coloured_light(glam::Vec3::Z),
    ),
    (
        glam::DVec3::new(-2.0, 2.0, -2.0),
        coloured_light(glam::vec3(1.0, 1.0, 0.0)),
    ),
];

const fn coloured_light(color: glam::Vec3) -> PointLight {
    PointLight {
        color,
        intensity: 9.0 * SUN_ILLUMINANCE,
        range: 20.0,
        casts_shadow: true,
    }
}

/// SciFiHelmet's glTF file, by its manifest GUID.
pub(crate) const SCI_FI_HELMET: u128 = 0x3b67bb0196a34daa9d4389cb09aa9a27;
/// NormalTangentTest's glTF file, by its manifest GUID.
const NORMAL_TANGENT_TEST: u128 = 0xc750a32dc879478eaca4748dbc82645a;

/// A camera on a circle of `radius` at `height` around the origin, looking at
/// it, starting a little left of +Z and moving a quarter degree per frame.
fn orbit(frame: u32, radius: f32, height: f32) -> Camera {
    let angle = (-7.5 + 0.25 * frame as f32).to_radians();
    let position = glam::vec3(radius * angle.sin(), height, radius * angle.cos());
    Camera::looking_at(position.as_dvec3(), glam::DVec3::ZERO)
}

/// Linear sRGB reflectance of gold at normal incidence, F0 (Hoffman 2013,
/// Table 1 [ref:hoffman-2013-f0]): metals tint their highlight with it and
/// dielectrics their diffuse, so the two rows differ.
#[cfg(feature = "cook")]
const SWEEP_BASE_COLOR: [f32; 4] = [1.0, 0.71, 0.29, 1.0];
/// The helmet scene's sweep is white, so the factors alone show.
#[cfg(feature = "cook")]
const HELMET_SWEEP_BASE_COLOR: [f32; 4] = [1.0; 4];
pub(crate) const SWEEP_METALLIC: [f32; 2] = [0.0, 1.0];
pub(crate) const SWEEP_ROUGHNESS: [f32; 5] = [0.1, 0.3, 0.5, 0.7, 0.9];
#[cfg(feature = "cook")]
const SPHERE_RADIUS: f32 = 0.5;
const SPHERE_SPACING: f32 = 1.3;

/// The name of a sweep's Material of `metallic` and `roughness`.
pub(crate) fn sweep_material_name(metallic: f32, roughness: f32) -> String {
    format!("metallic-{metallic}-roughness-{roughness}")
}

pub(crate) fn spawn_pbr_sweep(
    world: &mut World,
    loader: &mut dyn AssetLoader,
) -> anyhow::Result<()> {
    let sphere = loader.mesh(ReferenceScene::PbrSweep.mesh_id("sphere"))?;
    for (row, metallic) in SWEEP_METALLIC.into_iter().enumerate() {
        for (column, roughness) in SWEEP_ROUGHNESS.into_iter().enumerate() {
            let name = sweep_material_name(metallic, roughness);
            let material = loader.material(ReferenceScene::PbrSweep.material_id(&name))?;
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
#[cfg(feature = "cook")]
const UV_GRID_ROUGHNESS: f32 = 0.8;

/// The plane draws with the grid Material its Primitive names.
pub(crate) fn spawn_uv_grid(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    let plane = loader.mesh(ReferenceScene::UvGrid.mesh_id("plane"))?;
    world.spawn((Transform::IDENTITY, MeshRef(plane)));
    Ok(())
}

/// The middle of the model's bounds, which run from -1.2 to 0.95 in Y.
const NORMAL_TANGENT_CENTRE_HEIGHT: f64 = -0.125;

/// The Mesh draws with the Material its Primitive names.
pub(crate) fn spawn_normal_tangent(
    world: &mut World,
    loader: &mut dyn AssetLoader,
) -> anyhow::Result<()> {
    let source = AssetSource::File(NORMAL_TANGENT_TEST);
    let name = AssetName::Name("NormalTangentTest_low");
    let mesh = loader.mesh(AssetId::new(source, AssetKind::Mesh, name))?;
    world.spawn((Transform::IDENTITY, MeshRef(mesh)));
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
#[cfg(feature = "cook")]
const FAR_COPLANAR_COLORS: [[f32; 4]; 2] = [[0.8, 0.1, 0.1, 1.0], [0.1, 0.8, 0.1, 1.0]];
#[cfg(feature = "cook")]
const FAR_COPLANAR_ROUGHNESS: f32 = 0.8;

pub(crate) fn spawn_far_coplanar(
    world: &mut World,
    loader: &mut dyn AssetLoader,
) -> anyhow::Result<()> {
    let plane = loader.mesh(ReferenceScene::FarCoplanar.mesh_id("plane"))?;
    let centre = glam::dvec3(0.0, 0.0, -FAR_COPLANAR_DISTANCE);
    for (height, name) in [(0.0, "upper"), (-FAR_COPLANAR_GAP, "lower")] {
        let material = loader.material(ReferenceScene::FarCoplanar.material_id(name))?;
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
#[cfg(feature = "cook")]
const ORRERY_ROUGHNESS: f32 = 0.5;

/// A chain of five spheres from a root at the origin, each a child of the
/// one before, on its parent's +X axis, and all spinning about their own +Y
/// in the fixed schedule, so each circles its parent. The spheres draw with
/// the grid Material their Primitive names, which shows each one's spin.
pub(crate) fn spawn_orrery(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    let sphere = loader.mesh(ReferenceScene::Orrery.mesh_id("sphere"))?;
    let body = |radians_per_second| {
        (
            MeshRef(sphere),
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

/// minification's floor: half its side, in metres, and its UV range, so
/// each Texture repeats every metre.
#[cfg(feature = "cook")]
const MINIFICATION_HALF_SIZE: f32 = 50.0;
#[cfg(feature = "cook")]
const MINIFICATION_UV_REPEAT: f32 = 100.0;
/// 1.2 m above the floor, 4 m in from its near edge, looking down about
/// 8.5° along it.
const MINIFICATION_CAMERA: glam::DVec3 = glam::DVec3::new(0.0, 1.2, 46.0);
const MINIFICATION_CAMERA_TARGET: glam::DVec3 = glam::DVec3::new(0.0, 0.0, 38.0);
/// 1 m over the floor 30 m ahead: the floor mirrors it about 16 m out, in
/// the minified distance, so Toksvig shapes its reflection.
const MINIFICATION_LIGHT: glam::DVec3 = glam::DVec3::new(0.0, 1.0, 16.0);
/// The sun's illuminance about 17 m away.
const MINIFICATION_LIGHT_INTENSITY: f32 = 300.0 * SUN_ILLUMINANCE;
/// Past the floor's far corners, 83 m from the light.
const MINIFICATION_LIGHT_RANGE: f32 = 200.0;
/// Side of minification's Textures, in texels.
#[cfg(feature = "cook")]
const MINIFICATION_TEXTURE_SIZE: u32 = 256;
/// Checks of 16 texels, so each 4×4 block is one colour.
#[cfg(feature = "cook")]
const MINIFICATION_CHECK_SIZE: u32 = 16;
/// A bump every 8 texels, with slopes up to 0.5 (27°).
#[cfg(feature = "cook")]
const MINIFICATION_BUMP_PERIOD: u32 = 8;
#[cfg(feature = "cook")]
const MINIFICATION_BUMP_SLOPE: f32 = 0.5;
/// The gloss before Toksvig widens it.
#[cfg(feature = "cook")]
const MINIFICATION_GLOSS_ROUGHNESS: f32 = 0.2;

/// The floor draws with the Material its Primitive names.
pub(crate) fn spawn_minification(
    world: &mut World,
    loader: &mut dyn AssetLoader,
) -> anyhow::Result<()> {
    let ground = loader.mesh(ReferenceScene::Minification.mesh_id("ground"))?;
    world.spawn((Transform::IDENTITY, MeshRef(ground)));
    Ok(())
}

/// A `MINIFICATION_TEXTURE_SIZE`-square image of `texel(x, y)`, top row
/// first.
#[cfg(feature = "cook")]
fn minification_image(texel: impl Fn(u32, u32) -> [u8; 4]) -> LoadedImage {
    let side = MINIFICATION_TEXTURE_SIZE;
    LoadedImage {
        pixels: (0..side * side)
            .flat_map(|index| texel(index % side, index / side))
            .collect(),
        width: side,
        height: side,
    }
}

/// Light and dark grey checks of `MINIFICATION_CHECK_SIZE` texels, sRGB.
#[cfg(feature = "cook")]
fn checker_texture() -> LoadedImage {
    minification_image(|x, y| {
        let check = x / MINIFICATION_CHECK_SIZE + y / MINIFICATION_CHECK_SIZE;
        let value = if check.is_multiple_of(2) { 200 } else { 40 };
        [value, value, value, 255]
    })
}

/// A tangent-space normal Texture of the height sin(x) sin(y), one bump every
/// `MINIFICATION_BUMP_PERIOD` texels each way with slopes up to
/// `MINIFICATION_BUMP_SLOPE`: a box of a period or more holds normals tilted
/// every way, whose variance Toksvig widens the gloss by.
#[cfg(feature = "cook")]
fn bump_texture() -> LoadedImage {
    let frequency = TAU / MINIFICATION_BUMP_PERIOD as f32;
    // The height's slope along one axis, at a texel's centre.
    let slope = |along: f32, across: f32| {
        MINIFICATION_BUMP_SLOPE * (frequency * along).cos() * (frequency * across).sin()
    };
    minification_image(|x, y| {
        let (x, y) = (x as f32 + 0.5, y as f32 + 0.5);
        // The image's y runs down and tangent space's +Y up (glTF), so the
        // y slope's sign flips.
        let normal = glam::vec3(-slope(x, y), slope(y, x), 1.0).normalize();
        let [red, green, blue] = normal
            .to_array()
            .map(|component| ((component * 0.5 + 0.5) * 255.0).round() as u8);
        [red, green, blue, 255]
    })
}

/// Roughness `MINIFICATION_GLOSS_ROUGHNESS` and metallic 0 at every texel
/// (glTF's green and blue), the size of the normal Texture, so Toksvig
/// widens every mip.
#[cfg(feature = "cook")]
fn gloss_texture() -> LoadedImage {
    let roughness = (MINIFICATION_GLOSS_ROUGHNESS * 255.0).round() as u8;
    minification_image(|_, _| [0, roughness, 0, 255])
}

/// texture-wall's Textures, all in the albedo Channel.
const TEXTURE_WALL_TEXTURES: u32 = 10_000;
/// The quads: 10 rows of 10, 1 m apart.
const TEXTURE_WALL_QUADS: u32 = 100;
const TEXTURE_WALL_COLUMNS: u32 = 10;
const TEXTURE_WALL_SPACING: f64 = 1.0;
#[cfg(feature = "cook")]
const TEXTURE_WALL_QUAD_HALF_SIZE: f32 = 0.4;
/// Quad k samples Texture 101 k, 0 to 9,999: both ends of the range, and
/// a colour whose red grows with the quad's column, green with its row and
/// blue with k.
#[cfg(feature = "cook")]
const TEXTURE_WALL_TEXTURE_STRIDE: u32 = 101;
/// Matte, so no highlight hides a colour.
#[cfg(feature = "cook")]
const TEXTURE_WALL_ROUGHNESS: f32 = 1.0;
/// 12 m up and 7 m back from the grid's centre, looking down about 60° at
/// it.
const TEXTURE_WALL_CAMERA: glam::DVec3 = glam::DVec3::new(0.0, 12.0, 7.0);

fn wall_texture_name(index: u32) -> String {
    format!("texture-{index}")
}

fn wall_material_name(quad: u32) -> String {
    format!("quad-{quad}")
}

/// Loads every Texture in index order, so Texture i is Layer i + 1 of the
/// albedo Channel, then the quads, row 0 farthest from the camera and
/// column 0 on its left, each with its own Material.
pub(crate) fn spawn_texture_wall(
    world: &mut World,
    loader: &mut dyn AssetLoader,
) -> anyhow::Result<()> {
    let (scene, channel) = (ReferenceScene::TextureWall, MaterialChannel::Albedo);
    for index in 0..TEXTURE_WALL_TEXTURES {
        loader.texture(
            channel,
            scene.texture_id(channel, &wall_texture_name(index)),
        )?;
    }
    let mesh = loader.mesh(scene.mesh_id("quad"))?;
    let centre = f64::from(TEXTURE_WALL_COLUMNS - 1) / 2.0;
    for quad_index in 0..TEXTURE_WALL_QUADS {
        let material = loader.material(scene.material_id(&wall_material_name(quad_index)))?;
        let (row, column) = (
            quad_index / TEXTURE_WALL_COLUMNS,
            quad_index % TEXTURE_WALL_COLUMNS,
        );
        let position = glam::dvec3(f64::from(column) - centre, 0.0, f64::from(row) - centre)
            * TEXTURE_WALL_SPACING;
        world.spawn((
            Transform::from_translation(position),
            MeshRef(mesh),
            MaterialRef(material),
        ));
    }
    Ok(())
}

/// texture-wall's Texture `index`, 4×4 texels of one colour that encodes
/// it: red is its units digit and green its tens, 28 steps each, and blue
/// its hundreds, 0 to 99 scaled to 255. Neighbouring indices differ by at
/// least 28 steps in red.
#[cfg(feature = "cook")]
fn wall_texture(index: u32) -> LoadedImage {
    let digit_steps = |place: u32| (index / place % 10 * 28) as u8;
    let blue = (index / 100 * 255 / 99) as u8;
    LoadedImage {
        pixels: [digit_steps(1), digit_steps(10), blue, 255].repeat(16),
        width: 4,
        height: 4,
    }
}

/// shadows' sphere: its radius and its centre's height over the ground
/// plane, which reaches `SHADOWS_GROUND_HALF_SIZE` from the origin each way.
#[cfg(feature = "cook")]
const SHADOWS_SPHERE_RADIUS: f32 = 0.5;
#[cfg(feature = "cook")]
const SHADOWS_SPHERE_HEIGHT: f32 = 1.0;
#[cfg(feature = "cook")]
const SHADOWS_GROUND_HALF_SIZE: f32 = 8.0;
/// The ground's base colour, light grey, then the sphere's, orange, so the
/// Golden shows each Primitive's own Material.
#[cfg(feature = "cook")]
const SHADOWS_COLORS: [[f32; 4]; 2] = [[0.8, 0.8, 0.8, 1.0], [0.8, 0.25, 0.1, 1.0]];
#[cfg(feature = "cook")]
const SHADOWS_ROUGHNESS: f32 = 0.6;
/// In front of the sphere and above it, looking down at the ground its
/// shadows fall on: the sun's behind it to the right, the point light's to
/// the left.
const SHADOWS_CAMERA: glam::DVec3 = glam::DVec3::new(0.0, 2.4, 3.0);
const SHADOWS_CAMERA_TARGET: glam::DVec3 = glam::DVec3::new(0.0, 0.45, -0.4);
/// White, up and to the right of the sphere, in front of it, giving the
/// sun's illuminance 3 m away and reaching 20 m, as each coloured light does.
const SHADOWS_LIGHTS: [(glam::DVec3, PointLight); 1] = [(
    glam::DVec3::new(1.5, 2.5, 1.0),
    coloured_light(glam::Vec3::ONE),
)];

/// The Mesh draws each of its Primitives with the Material it names.
pub(crate) fn spawn_shadows(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    let mesh = loader.mesh(ReferenceScene::Shadows.mesh_id("sphere-over-plane"))?;
    world.spawn((Transform::IDENTITY, MeshRef(mesh)));
    Ok(())
}

/// crowd's figures: rows of `CROWD_COLUMNS`, `CROWD_SPACING` metres apart,
/// on a ground plane reaching `CROWD_GROUND_HALF_SIZE` from the origin each
/// way.
const CROWD_FIGURES: u32 = 100;
const CROWD_COLUMNS: u32 = 10;
const CROWD_SPACING: f64 = 1.6;
#[cfg(feature = "cook")]
const CROWD_GROUND_HALF_SIZE: f32 = 12.0;
/// A figure is a spheroid `CROWD_FIGURE_HEIGHT` tall and twice
/// `CROWD_FIGURE_RADIUS` across, standing on its lowest point with its
/// middle up +Y: 40 segments around and 65 from pole to pole make 5,120
/// triangles.
const CROWD_FIGURE_HEIGHT: f32 = 1.8;
const CROWD_FIGURE_RADIUS: f32 = 0.25;
#[cfg(feature = "cook")]
const CROWD_FIGURE_SEGMENTS: [u32; 2] = [40, 65];
/// The bones each figure's middle is cut into, each with a shadow proxy:
/// about as many as the limb and torso bones of a humanoid skeleton.
const CROWD_BONES: u32 = 16;
/// A shadow proxy is a sphere of 12 segments around and 8 from pole to
/// pole, 168 triangles, stretched over its bone: across, to
/// `CROWD_PROXY_WIDTH` of the figure there, so it stays inside the figure's
/// surface; along, to `CROWD_PROXY_LENGTH` bones, so neighbours overlap,
/// but short of the figure's poles, so a figure's shadow rays leave it
/// clear of its own proxies.
#[cfg(feature = "cook")]
const CROWD_PROXY_SEGMENTS: [u32; 2] = [12, 8];
const CROWD_PROXY_WIDTH: f64 = 0.85;
const CROWD_PROXY_LENGTH: f64 = 1.5;
/// Each figure's middle bends towards its azimuth and back, to
/// `CROWD_SWAY_CURVATURE` radians per metre at most, which turns its top by
/// 0.63 rad (36°), once every `CROWD_SWAY_PERIOD` seconds.
const CROWD_SWAY_CURVATURE: f64 = 0.35;
const CROWD_SWAY_PERIOD: f64 = 2.5;
/// From one figure to the next, the sway's phase and azimuth both step by
/// the golden angle, π (3 − √5), so no two neighbours sway alike.
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
/// The ground's base colour, light grey, then the figures', blue.
#[cfg(feature = "cook")]
const CROWD_COLORS: [[f32; 4]; 2] = [[0.6, 0.6, 0.6, 1.0], [0.2, 0.35, 0.7, 1.0]];
#[cfg(feature = "cook")]
const CROWD_ROUGHNESS: f32 = 0.6;
/// Above the crowd and back from it, looking down about 28° at its middle,
/// so every figure stays in view and none hides another whole.
const CROWD_CAMERA: glam::DVec3 = glam::DVec3::new(0.0, 11.0, 20.0);
const CROWD_CAMERA_TARGET: glam::DVec3 = glam::DVec3::new(0.0, 0.6, -0.5);

/// crowd's figure, standing on its lowest point at the origin with its
/// middle up +Y: a sphere stretched to `CROWD_FIGURE_RADIUS` in radius and
/// `CROWD_FIGURE_HEIGHT` tall, its normals by the inverse stretch, and
/// culled by the box that holds every shape it sways to.
#[cfg(feature = "cook")]
fn crowd_figure() -> Geometry {
    let [longitudes, latitudes] = CROWD_FIGURE_SEGMENTS;
    let mut figure = uv_sphere(1.0, longitudes, latitudes);
    let half_height = 0.5 * CROWD_FIGURE_HEIGHT;
    let stretch = glam::vec3(CROWD_FIGURE_RADIUS, half_height, CROWD_FIGURE_RADIUS);
    for (position, normal) in figure.positions.iter_mut().zip(&mut figure.normals) {
        let stretched = glam::Vec3::from(*position) * stretch + glam::Vec3::Y * half_height;
        *position = stretched.into();
        *normal = (glam::Vec3::from(*normal) / stretch).normalize().into();
    }
    Geometry {
        bounds: Some(crowd_figure_bounds()),
        ..figure
    }
}

/// The box that holds a crowd figure however it bends: its middle's top
/// reaches (1 − cos κh) / κ sideways at the most curvature κ, h its height,
/// and the figure lies within its radius of its middle; no point rises
/// above the figure's height.
#[cfg(feature = "cook")]
fn crowd_figure_bounds() -> [[f32; 3]; 2] {
    let height = f64::from(CROWD_FIGURE_HEIGHT);
    let sideways = (1.0 - (CROWD_SWAY_CURVATURE * height).cos()) / CROWD_SWAY_CURVATURE;
    let reach = (f64::from(CROWD_FIGURE_RADIUS) + sideways) as f32;
    [[-reach, 0.0, -reach], [reach, CROWD_FIGURE_HEIGHT, reach]]
}

/// Spawns the ground, then the figures in rows of `CROWD_COLUMNS` centred
/// on the origin, each drawing its own Mesh, bent from one rest Mesh, with
/// its bones' shadow proxies as children (`CrowdForm::Proxies`).
pub(crate) fn spawn_crowd(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
    spawn_crowd_as(world, loader, CrowdForm::Proxies)
}

/// Spawns the crowd as `spawn_crowd` does, each figure traced through its
/// own Mesh's BLAS, updated each frame, and no shadow proxies
/// (`CrowdForm::Refit`, `CrowdForm::Rebuild`).
pub(crate) fn spawn_traced_crowd(
    world: &mut World,
    loader: &mut dyn AssetLoader,
) -> anyhow::Result<()> {
    spawn_crowd_as(world, loader, CrowdForm::Refit)
}

/// Spawns the crowd as `form` traces it: through shadow proxies, or through
/// the figures' own BLASes.
fn spawn_crowd_as(
    world: &mut World,
    loader: &mut dyn AssetLoader,
    form: CrowdForm,
) -> anyhow::Result<()> {
    let scene = ReferenceScene::Crowd;
    let ground = loader.mesh(scene.mesh_id("ground"))?;
    world.spawn((Transform::IDENTITY, MeshRef(ground)));
    let material = loader.material(scene.material_id("figure"))?;
    let figure = scene.mesh_id("figure");
    let rest = loader.bent_mesh(figure)?;
    let proxy = match form {
        CrowdForm::Proxies => Some(loader.shadow_proxy(scene.mesh_id("proxy"))?),
        CrowdForm::Refit | CrowdForm::Rebuild => None,
    };
    let centre = f64::from(CROWD_COLUMNS - 1) / 2.0;
    for index in 0..CROWD_FIGURES {
        let (row, column) = (index / CROWD_COLUMNS, index % CROWD_COLUMNS);
        let position =
            glam::dvec3(f64::from(column) - centre, 0.0, f64::from(row) - centre) * CROWD_SPACING;
        let sway = Sway {
            phase: (f64::from(index) * GOLDEN_ANGLE).rem_euclid(std::f64::consts::TAU),
        };
        let curvature = sway.curvature(0.0);
        let bend = Bend {
            rest,
            azimuth: sway.phase as f32,
            curvature,
            previous_curvature: curvature,
        };
        let mut figure_entity = world.spawn((
            Transform::from_translation(position),
            MaterialRef(material),
            bend,
            sway,
        ));
        let Some(proxy) = proxy else {
            // A Moving Instance, so it joins the dynamic TLAS, rebuilt each
            // frame over its BLAS's bent bounds; the static TLAS would keep
            // the bounds of its load.
            figure_entity.insert((
                MeshRef(loader.refitted_mesh(figure)?),
                PreviousWorldTransform::default(),
            ));
            continue;
        };
        figure_entity.insert(MeshRef(loader.bent_mesh(figure)?));
        let figure_entity = figure_entity.id();
        for bone in 0..CROWD_BONES {
            world.spawn((
                bone_transform(bone, &bend),
                ChildOf(figure_entity),
                MeshRef(proxy),
                Bone(bone),
                PreviousWorldTransform::default(),
            ));
        }
    }
    Ok(())
}

/// Sways a crowd figure: its middle bends towards its azimuth and back,
/// from its phase at Tick 0.
#[derive(Component)]
pub(crate) struct Sway {
    phase: f64,
}

impl Sway {
    /// Its curvature `seconds` after Tick 0.
    fn curvature(&self, seconds: f64) -> f32 {
        let tau = std::f64::consts::TAU;
        let angle = (tau * seconds / CROWD_SWAY_PERIOD + self.phase).rem_euclid(tau);
        (CROWD_SWAY_CURVATURE * angle.sin()) as f32
    }
}

/// One of a crowd figure's bones, numbered up its middle from its foot,
/// whose shadow proxy the figure's sway places.
#[derive(Component)]
pub(crate) struct Bone(u32);

/// Sets each swaying figure's curvature for the end of the running Tick,
/// from the Tick count, so no rounding accumulates over Ticks, and places
/// its bones' shadow proxies on its bent middle.
pub(crate) fn sway(
    fixed_time: Res<FixedTime>,
    mut figures: Query<(&Sway, &mut Bend, Option<&Children>)>,
    mut bones: Query<(&Bone, &mut Transform)>,
) {
    let seconds = (fixed_time.tick + 1) as f64 * TICK_SECONDS;
    for (sway, mut bend, children) in &mut figures {
        bend.curvature = sway.curvature(seconds);
        for &child in children.into_iter().flatten() {
            if let Ok((bone, mut transform)) = bones.get_mut(child) {
                *transform = bone_transform(bone.0, &bend);
            }
        }
    }
}

/// Where bone `bone`'s shadow proxy sits in a figure bent by `bend`:
/// centred on the bent middle halfway up the bone and turned as the middle
/// turns there, stretched over the bone, short of the poles, and to
/// `CROWD_PROXY_WIDTH` of the figure's width there, a spheroid's.
fn bone_transform(bone: u32, bend: &Bend) -> Transform {
    let figure_height = f64::from(CROWD_FIGURE_HEIGHT);
    let length = figure_height / f64::from(CROWD_BONES);
    let height = (f64::from(bone) + 0.5) * length;
    let (centre, turn) = bent_middle(height, bend);
    let from_equator = 2.0 * height / figure_height - 1.0;
    let radius = f64::from(CROWD_FIGURE_RADIUS) * (1.0 - from_equator * from_equator).sqrt();
    let across = CROWD_PROXY_WIDTH * radius;
    let to_pole = height.min(figure_height - height);
    let along = (0.5 * CROWD_PROXY_LENGTH * length).min(CROWD_PROXY_WIDTH * to_pole);
    Transform {
        translation: centre,
        rotation: turn.as_quat(),
        scale: glam::dvec3(across, along, across).as_vec3(),
    }
}

/// Where `bend` takes the point of the middle `height` up it, and how it
/// turns the middle there, as the bend pass bends a vertex there
/// (src/render/shaders/bend.wgsl): the arc's chord from the foot runs at
/// half the angle the middle turns through, its length the height times
/// sin(half) / half.
fn bent_middle(height: f64, bend: &Bend) -> (glam::DVec3, glam::DQuat) {
    let half_angle = 0.5 * f64::from(bend.curvature) * height;
    let azimuth = f64::from(bend.azimuth);
    let toward = glam::dvec3(azimuth.cos(), 0.0, azimuth.sin());
    let axis = glam::dvec3(toward.z, 0.0, -toward.x);
    let turn = glam::DQuat::from_axis_angle(axis, 2.0 * half_angle);
    let chord = if half_angle == 0.0 {
        height
    } else {
        height * half_angle.sin() / half_angle
    };
    (
        chord * (glam::DVec3::Y * half_angle.cos() + toward * half_angle.sin()),
        turn,
    )
}

/// A procedural Primitive: its triangles, and the name of its Material, if
/// its Renderables do not each override it.
#[cfg(feature = "cook")]
pub(crate) type ScenePrimitive = (Geometry, Option<&'static str>);

/// An indexed triangle list, counter-clockwise when seen from outside.
#[cfg(feature = "cook")]
#[derive(Clone, Default)]
pub(crate) struct Geometry {
    pub(crate) positions: Vec<[f32; 3]>,
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) uvs: Vec<[f32; 2]>,
    pub(crate) indices: Vec<u32>,
    /// A box its Mesh's cooked bounds must hold besides its vertices, for a
    /// Mesh the GPU bends: every shape it bends to.
    pub(crate) bounds: Option<[[f32; 3]; 2]>,
}

/// A sphere of `radius` around the origin, 48 segments around and 24 from
/// pole to pole, as the scenes' spheres are.
#[cfg(feature = "cook")]
fn scene_sphere(radius: f32) -> Geometry {
    uv_sphere(radius, 48, 24)
}

/// A sphere of `radius` around the origin, in `longitude_segments` around
/// and `latitude_segments` from pole to pole. `u` runs once around from +Z
/// towards +X, `v` from the north pole to the south pole, as a glTF texture
/// maps top row first.
#[cfg(feature = "cook")]
fn uv_sphere(radius: f32, longitude_segments: u32, latitude_segments: u32) -> Geometry {
    let mut geometry = Geometry::default();
    for latitude in 0..=latitude_segments {
        let v = latitude as f32 / latitude_segments as f32;
        let polar_angle = PI * v;
        for longitude in 0..=longitude_segments {
            let u = longitude as f32 / longitude_segments as f32;
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

    let row_length = longitude_segments + 1;
    for latitude in 0..latitude_segments {
        for longitude in 0..longitude_segments {
            let top_left = latitude * row_length + longitude;
            let top_right = top_left + 1;
            let bottom_left = top_left + row_length;
            let bottom_right = bottom_left + 1;
            // At a pole one edge of the quad collapses to a point; skip the
            // triangle that would have no area.
            if latitude != latitude_segments - 1 {
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
#[cfg(feature = "cook")]
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
        bounds: None,
    }
}

/// Side of the grid texture, in texels.
#[cfg(feature = "cook")]
const GRID_TEXTURE_SIZE: u32 = 2048;

/// `GRID_TEXTURE_SIZE`-square RGBA8 sRGB texture of 8 × 8 cells framed by dark
/// lines. Red grows with the cell's column and green with its row, and blue
/// alternates as a checker, so every cell shows where it sits in UV space.
#[cfg(feature = "cook")]
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

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::assets::MeshHandle;
    use crate::render::bend_pass::tests::bent;

    /// A figure bent by `curvature` towards azimuth 2.2.
    fn bend(curvature: f32) -> Bend {
        Bend {
            rest: MeshHandle(0),
            azimuth: 2.2,
            curvature,
            previous_curvature: curvature,
        }
    }

    /// Each bone's shadow proxy sits on the middle as the bend pass bends
    /// it, centred halfway up the bone, every 0.1125 m (1.8 m over 16
    /// bones) from 0.05625 m, and turned as the middle turns there, unbent
    /// and bent either way to the sway's most curvature, within the
    /// oracle's sum of 1,024 steps, about 3e-8 m.
    #[test]
    fn each_bones_shadow_proxy_sits_on_the_bent_middle() {
        let most = CROWD_SWAY_CURVATURE as f32;
        for bend in [bend(-most), bend(0.0), bend(most)] {
            for bone in 0..CROWD_BONES {
                let transform = bone_transform(bone, &bend);
                let middle = glam::dvec3(0.0, 0.05625 + 0.1125 * f64::from(bone), 0.0);
                let (centre, turn) = bent(middle, bend.azimuth.into(), bend.curvature.into());
                let off_centre = (transform.translation - centre).length();
                let rotation = transform.rotation.as_dquat();
                let off_turn = [glam::DVec3::X, glam::DVec3::Y]
                    .map(|axis| (rotation * axis - turn * axis).length())
                    .into_iter()
                    .fold(0.0, f64::max);
                assert!(
                    off_centre <= 1e-7 && off_turn <= 1e-6,
                    "bone {bone} at curvature {}: {off_centre} m off, axes {off_turn} off",
                    bend.curvature
                );
            }
        }
    }

    /// Unbent, every point of every bone's shadow proxy lies inside its
    /// figure, the spheroid 0.25 m in radius and 1.8 m tall standing on its
    /// lowest point, so the figure's own shadow rays leave it clear of them.
    #[test]
    fn each_bones_shadow_proxy_lies_inside_its_figure() {
        for bone in 0..CROWD_BONES {
            let transform = bone_transform(bone, &bend(0.0));
            for latitude in 0..=16 {
                for longitude in 0..32 {
                    let polar = std::f32::consts::PI * latitude as f32 / 16.0;
                    let around = std::f32::consts::TAU * longitude as f32 / 32.0;
                    let on_unit_sphere = glam::vec3(
                        polar.sin() * around.cos(),
                        polar.cos(),
                        polar.sin() * around.sin(),
                    );
                    let point = transform
                        .to_affine()
                        .transform_point3(on_unit_sphere.as_dvec3());
                    let from_middle = (point.x * point.x + point.z * point.z) / (0.25 * 0.25)
                        + (point.y - 0.9) * (point.y - 0.9) / (0.9 * 0.9);
                    assert!(from_middle < 1.0, "bone {bone}'s proxy reaches {point}");
                }
            }
        }
    }

    /// Every vertex of a crowd figure, bent by curvatures from barely any to
    /// the sway's most, lies in the box its Mesh is culled by: sideways the
    /// most curvature reaches furthest, but the ring below the top rises
    /// highest at about 0.011 rad/m. A negative curvature bends the other
    /// way, so the azimuths, one along a column of vertices and one between
    /// two, cover it; the figure is round about its middle.
    #[test]
    fn a_crowd_figure_bends_within_its_bounds() {
        let [lowest, highest] = crowd_figure_bounds().map(glam::Vec3::from);
        let figure = crowd_figure();
        let curvatures = [
            0.002,
            0.006,
            0.011,
            0.02,
            0.05,
            0.1,
            0.2,
            CROWD_SWAY_CURVATURE,
        ];
        for curvature in curvatures {
            for azimuth in [0.0, std::f64::consts::PI / 40.0] {
                for &position in &figure.positions {
                    let rest = glam::Vec3::from(position).as_dvec3();
                    let (bent, _) = bent(rest, azimuth, curvature);
                    let bent = bent.as_vec3();
                    assert!(
                        bent.cmpge(lowest).all() && bent.cmple(highest).all(),
                        "{rest} bends to {bent} at curvature {curvature}, azimuth {azimuth}"
                    );
                }
            }
        }
    }
}
