//! `praxis export-scene`: a Reference scene as prepare reads it from its
//! World at a frame of its camera path, for a reference renderer to rebuild
//! from source data: `scene.json`, a glTF binary per generated Mesh and a
//! PNG per generated Texture. Lengths are metres, +Y is up and the camera
//! looks down its −Z, as in glTF; light units are ADR 0023's.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Serialize;
use serde_json::json;

use super::mesh::SourcePrimitive;
use super::{LoadedImage, asset_names, gltf_material, material_textures, read_manifest};
use crate::ReferenceScene;
use crate::assets::{AssetLoader, MaterialChannel, MaterialHandle, MeshHandle};
use crate::camera::{Camera, FIELD_OF_VIEW};
use crate::components::{DirectionalLight, MaterialRef, MeshRef, PointLight};
use crate::cooked::{AssetId, AssetKind, AssetName, AssetSource, CookedMaterial, CookedMesh};
use crate::environment::Environment;
use crate::render::CrowdForm;
use crate::state::Simulation;
use crate::stats::FrameRecorder;
use crate::time::FixedTime;
use crate::transform::{PreviousWorldTransform, WorldTransform};

/// The version of `scene.json`'s format, which a reader that does not know
/// it refuses.
pub const SCENE_FORMAT_VERSION: u32 = 1;

/// Writes `scene` at `frame` of its camera path, or with none at the frame
/// `praxis capture` writes, into `directory`: `scene.json`, and under it
/// `meshes/` and `textures/` for the generated Meshes and Textures its
/// Renderables use. The World runs as a capture runs it, through the fixed
/// schedule, and the manifest sources in `assets` are read for their
/// names; nothing is cooked into the Cooked cache and no GPU is used.
/// `crowd` is refused: its figures' shapes exist only on the GPU.
pub fn export_scene(
    scene: ReferenceScene,
    frame: Option<u32>,
    assets: &Path,
    directory: &Path,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        scene != ReferenceScene::Crowd,
        "crowd is not exported: the GPU bends its figures each frame, so the World holds only their rest shapes"
    );
    let frame = frame.unwrap_or(crate::capture::FRAME_COUNT - 1);
    let catalog = Catalog::new(scene, assets)?;
    let mut recorder = AssetRecorder::default();
    let mut simulation = Simulation::new(
        scene,
        glam::DVec3::ZERO,
        CrowdForm::default(),
        &mut recorder,
        &FrameRecorder::new(),
    )?;
    // Each frame up to `frame` advances the World as a capture's does.
    for drawn in 0..=frame {
        let camera = scene.camera_at(drawn, glam::DVec3::ZERO);
        simulation.world.insert_resource(camera);
        simulation.simulate(scene.frame_delta());
    }
    std::fs::create_dir_all(directory)
        .with_context(|| format!("creating {}", directory.display()))?;
    let mut exported = Exported {
        catalog: &catalog,
        directory,
        meshes: BTreeMap::new(),
        materials: BTreeMap::new(),
        mesh_names: HashMap::new(),
        material_names: HashMap::new(),
    };
    let world = &mut simulation.world;
    let blend_weight = world.resource::<FixedTime>().blend_weight;
    let camera = world.resource::<Camera>();
    let view_rotation = glam::Mat3::from_mat4(camera.view_matrix());
    let camera = CameraEntry {
        position: camera.position.to_array(),
        rotation: glam::Quat::from_mat3(&view_rotation.transpose()).to_array(),
        vertical_field_of_view: FIELD_OF_VIEW,
        ev100: camera.ev100,
    };
    let environment = *world.resource::<Environment>();
    let directional_lights = world
        .query::<(
            &WorldTransform,
            Option<&PreviousWorldTransform>,
            &DirectionalLight,
        )>()
        .iter(world)
        .filter_map(|(current, previous, light)| {
            let transform = current.drawn(previous, blend_weight);
            // As prepare, which draws no light without a direction.
            let direction = (transform.0.matrix3 * glam::DVec3::NEG_Z).try_normalize()?;
            Some(DirectionalLightEntry {
                direction: direction.as_vec3().to_array(),
                color: light.color.to_array(),
                illuminance: light.intensity,
            })
        })
        .collect();
    let point_lights = world
        .query::<(
            &WorldTransform,
            Option<&PreviousWorldTransform>,
            &PointLight,
        )>()
        .iter(world)
        .map(|(current, previous, light)| PointLightEntry {
            position: current
                .drawn(previous, blend_weight)
                .translation()
                .to_array(),
            color: light.color.to_array(),
            intensity: light.intensity,
            range: light.range,
            casts_shadow: light.casts_shadow,
        })
        .collect();
    let mut renderables = Vec::new();
    let mut renderable_query = world.query::<(
        &WorldTransform,
        Option<&PreviousWorldTransform>,
        &MeshRef,
        Option<&MaterialRef>,
    )>();
    for (current, previous, mesh_ref, material_ref) in renderable_query.iter(world) {
        let mesh = exported.mesh(recorder.meshes[mesh_ref.0.0 as usize])?;
        let material = match material_ref {
            Some(material_ref) => {
                let id = recorder.materials[material_ref.0.index() as usize];
                Some(exported.material(id)?)
            }
            // As prepare, which skips it.
            None if exported.meshes[&mesh].materials.contains(&None) => {
                tracing::warn!(
                    mesh,
                    "Renderable has no Material Handle and a Primitive of its Mesh names no Material, skipping it"
                );
                continue;
            }
            None => None,
        };
        let (scale, rotation, translation) = current
            .drawn(previous, blend_weight)
            .0
            .to_scale_rotation_translation();
        renderables.push(RenderableEntry {
            translation: translation.to_array(),
            rotation: rotation.as_quat().to_array(),
            scale: scale.as_vec3().to_array(),
            mesh,
            material,
        });
    }
    let scene_file = SceneFile {
        version: SCENE_FORMAT_VERSION,
        scene: scene.name(),
        frame,
        camera,
        environment: EnvironmentEntry {
            ambient_illuminance: environment.ambient_illuminance.to_array(),
            background_luminance: environment.background_luminance.to_array(),
        },
        directional_lights,
        point_lights,
        meshes: exported.meshes,
        materials: exported.materials,
        renderables,
    };
    let path = directory.join("scene.json");
    let file =
        std::fs::File::create(&path).with_context(|| format!("creating {}", path.display()))?;
    serde_json::to_writer(std::io::BufWriter::new(file), &scene_file)
        .with_context(|| format!("writing {}", path.display()))?;
    tracing::info!(
        scene = scene.name(),
        frame,
        renderables = scene_file.renderables.len(),
        path = %path.display(),
        "scene exported"
    );
    Ok(())
}

/// `scene.json`. A `file` is relative to the directory that holds it, or
/// absolute for a manifest source's.
#[derive(Serialize)]
struct SceneFile {
    version: u32,
    scene: &'static str,
    frame: u32,
    camera: CameraEntry,
    environment: EnvironmentEntry,
    directional_lights: Vec<DirectionalLightEntry>,
    point_lights: Vec<PointLightEntry>,
    /// Each Mesh a Renderable uses, by name.
    meshes: BTreeMap<String, MeshEntry>,
    /// Each Material a Renderable or a Mesh's Primitive names, by name.
    materials: BTreeMap<String, MaterialEntry>,
    renderables: Vec<RenderableEntry>,
}

#[derive(Serialize)]
struct CameraEntry {
    position: [f64; 3],
    /// From the camera's space to the World's, as `[x, y, z, w]`.
    rotation: [f32; 4],
    /// In radians.
    vertical_field_of_view: f32,
    ev100: f32,
}

#[derive(Serialize)]
struct EnvironmentEntry {
    /// Lux, from all around.
    ambient_illuminance: [f32; 3],
    /// cd/m².
    background_luminance: [f32; 3],
}

#[derive(Serialize)]
struct DirectionalLightEntry {
    /// The way the light travels.
    direction: [f32; 3],
    color: [f32; 3],
    /// Lux [ref:khr-lights-punctual-units].
    illuminance: f32,
}

#[derive(Serialize)]
struct PointLightEntry {
    position: [f64; 3],
    color: [f32; 3],
    /// Candela [ref:khr-lights-punctual-units].
    intensity: f32,
    /// Metres past which it adds nothing, KHR_lights_punctual's
    /// [ref:khr-lights-punctual-range].
    range: f32,
    casts_shadow: bool,
}

#[derive(Serialize)]
struct MeshEntry {
    /// A glTF file, and the index of the Mesh in it.
    file: String,
    mesh: usize,
    /// Each Primitive's Material, which a Renderable's Material overrides.
    materials: Vec<Option<String>>,
}

#[derive(Serialize)]
struct MaterialEntry {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    emissive: [f32; 3],
    normal_scale: f32,
    textures: Vec<TextureEntry>,
}

#[derive(Serialize)]
struct TextureEntry {
    /// `albedo`'s and `emissive`'s colour is sRGB, albedo's alpha and the
    /// other Channels linear, as glTF's `baseColorTexture`, `emissiveTexture`,
    /// `normalTexture`, `metallicRoughnessTexture` and `occlusionTexture`.
    channel: &'static str,
    /// An image file, or with `image` the glTF file whose image it is.
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<usize>,
}

#[derive(Serialize)]
struct RenderableEntry {
    translation: [f64; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    mesh: String,
    /// Draws every Primitive when set, else each Primitive's own.
    material: Option<String>,
}

/// Notes the Asset ID behind each Handle a spawn takes, and loads nothing.
#[derive(Default)]
struct AssetRecorder {
    meshes: Vec<AssetId>,
    materials: Vec<AssetId>,
}

impl AssetLoader for AssetRecorder {
    fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle> {
        self.materials.push(id);
        Ok(MaterialHandle(self.materials.len() as u32 - 1))
    }

    /// Every Texture is Layer 0, which no export reads.
    fn texture(&mut self, _channel: MaterialChannel, _id: AssetId) -> anyhow::Result<u32> {
        Ok(0)
    }

    fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.meshes.push(id);
        Ok(MeshHandle(self.meshes.len() as u32 - 1))
    }

    fn bent_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh(id)
    }

    fn refitted_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh(id)
    }

    fn shadow_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh(id)
    }
}

/// Where a Mesh's triangles are.
enum MeshSource {
    /// A Reference scene's generated Primitives, written out as cooked.
    Generated(Vec<SourcePrimitive>),
    /// Mesh `index` of a manifest source's glTF file.
    File { path: PathBuf, index: usize },
}

struct CatalogMesh {
    name: String,
    source: MeshSource,
    /// Each Primitive's Material.
    materials: Vec<Option<AssetId>>,
}

/// Where a Texture's image is.
enum TextureFile {
    /// A Reference scene's generated image, before its cook.
    Generated { name: String, image: LoadedImage },
    /// An image file a glTF file names.
    File(PathBuf),
    /// Image `image` held inside the glTF file at `path`.
    Embedded { path: PathBuf, image: usize },
}

/// Every Mesh, Material and Texture a Reference scene's sources define, by
/// Asset ID, with its name.
#[derive(Default)]
struct Catalog {
    meshes: HashMap<AssetId, CatalogMesh>,
    materials: HashMap<AssetId, (String, CookedMaterial)>,
    textures: HashMap<AssetId, (MaterialChannel, TextureFile)>,
}

impl Catalog {
    /// `scene`'s generated Assets and those of the manifest sources in
    /// `assets` it references, named as the cook names them.
    fn new(scene: ReferenceScene, assets: &Path) -> anyhow::Result<Self> {
        let mut catalog = Self::default();
        for (name, material) in scene.materials() {
            let id = scene.material_id(&name);
            catalog.materials.insert(id, (name, material));
        }
        for (name, channel, image) in scene.textures() {
            let id = scene.texture_id(channel, &name);
            let file = TextureFile::Generated { name, image };
            catalog.textures.insert(id, (channel, file));
        }
        for (name, primitives) in scene.meshes() {
            let primitives: Vec<SourcePrimitive> = primitives
                .into_iter()
                .map(|(geometry, material)| {
                    (geometry, material.map(|name| scene.material_id(name)))
                })
                .collect();
            let mesh = CatalogMesh {
                name: name.into(),
                materials: primitives.iter().map(|(_, material)| *material).collect(),
                source: MeshSource::Generated(primitives),
            };
            catalog.meshes.insert(scene.mesh_id(name), mesh);
        }
        let guids = scene.source_files();
        if !guids.is_empty() {
            let manifest = read_manifest(assets, false)?;
            for &guid in guids {
                let entry = manifest
                    .iter()
                    .find(|entry| entry.guid().is_ok_and(|listed| listed == guid))
                    .with_context(|| format!("no manifest source has GUID {guid:032x}"))?;
                let path = std::path::absolute(assets.join(&entry.path))?;
                catalog
                    .add_gltf(guid, &path)
                    .with_context(|| format!("reading {}", path.display()))?;
            }
        }
        Ok(catalog)
    }

    /// The Meshes, Materials and Textures of the glTF file at `path`, the
    /// manifest source `guid`.
    fn add_gltf(&mut self, guid: u128, path: &Path) -> anyhow::Result<()> {
        let document = gltf::Gltf::open(path)
            .context("run `praxis fetch` if it is missing")?
            .document;
        let source = AssetSource::File(guid);
        let image_names = asset_names(document.images().map(|image| image.name()));
        let material_names = asset_names(document.materials().map(|material| material.name()));
        let mesh_names = asset_names(document.meshes().map(|mesh| mesh.name()));
        let directory = path.parent().unwrap_or(Path::new(""));
        for (material, &name) in document.materials().zip(&material_names) {
            let mut textures = [None; 5];
            let channels = MaterialChannel::ALL.into_iter();
            for (channel, texture) in channels.zip(material_textures(&material)) {
                let Some(texture) = texture else { continue };
                let image = texture.source();
                let kind = AssetKind::Texture(channel);
                let id = AssetId::new(source, kind, image_names[image.index()]);
                textures[channel as usize] = Some(id);
                let file = match image.source() {
                    gltf::image::Source::Uri { uri, .. } if !uri.starts_with("data:") => {
                        TextureFile::File(directory.join(uri))
                    }
                    _ => TextureFile::Embedded {
                        path: path.to_owned(),
                        image: image.index(),
                    },
                };
                self.textures.insert(id, (channel, file));
            }
            let id = AssetId::new(source, AssetKind::Material, name);
            let material = gltf_material(&material, textures);
            self.materials.insert(id, (name_text(name), material));
        }
        for (mesh, name) in document.meshes().zip(mesh_names) {
            let materials = mesh
                .primitives()
                .map(|primitive| {
                    let index = primitive.material().index()?;
                    Some(AssetId::new(
                        source,
                        AssetKind::Material,
                        material_names[index],
                    ))
                })
                .collect();
            let id = AssetId::new(source, AssetKind::Mesh, name);
            let mesh = CatalogMesh {
                name: name_text(name),
                source: MeshSource::File {
                    path: path.to_owned(),
                    index: mesh.index(),
                },
                materials,
            };
            self.meshes.insert(id, mesh);
        }
        Ok(())
    }
}

/// An Asset name as text: its glTF name, or `#` and its index.
fn name_text(name: AssetName) -> String {
    match name {
        AssetName::Name(name) => name.into(),
        AssetName::Index(index) => format!("#{index}"),
    }
}

/// `name` as a file name: each character that is not a letter, a digit,
/// `-`, `_` or `.` becomes `_`.
fn file_name(name: &str) -> String {
    name.chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => character,
            _ => '_',
        })
        .collect()
}

/// What an export has written so far: each Mesh and Material, by name, and
/// each one's files.
struct Exported<'a> {
    catalog: &'a Catalog,
    directory: &'a Path,
    meshes: BTreeMap<String, MeshEntry>,
    materials: BTreeMap<String, MaterialEntry>,
    mesh_names: HashMap<AssetId, String>,
    material_names: HashMap<AssetId, String>,
}

impl Exported<'_> {
    /// The name of Mesh `id`, written with its Primitives' Materials the
    /// first time.
    fn mesh(&mut self, id: AssetId) -> anyhow::Result<String> {
        if let Some(name) = self.mesh_names.get(&id) {
            return Ok(name.clone());
        }
        let mesh = self
            .catalog
            .meshes
            .get(&id)
            .with_context(|| format!("Mesh {id} is in none of the scene's sources"))?;
        anyhow::ensure!(
            !self.meshes.contains_key(&mesh.name),
            "two Meshes are named {:?}",
            mesh.name
        );
        let (file, index) = match &mesh.source {
            MeshSource::Generated(primitives) => {
                let file = format!("meshes/{}.glb", file_name(&mesh.name));
                let glb = mesh_glb(&mesh.name, &super::mesh::cook(primitives))?;
                self.write(&file, &glb)?;
                (file, 0)
            }
            MeshSource::File { path, index } => (path.display().to_string(), *index),
        };
        let materials = mesh
            .materials
            .iter()
            .map(|material| material.map(|id| self.material(id)).transpose())
            .collect::<anyhow::Result<_>>()?;
        let entry = MeshEntry {
            file,
            mesh: index,
            materials,
        };
        self.meshes.insert(mesh.name.clone(), entry);
        self.mesh_names.insert(id, mesh.name.clone());
        Ok(mesh.name.clone())
    }

    /// The name of Material `id`, written with its Textures the first time.
    fn material(&mut self, id: AssetId) -> anyhow::Result<String> {
        if let Some(name) = self.material_names.get(&id) {
            return Ok(name.clone());
        }
        let (name, material) = self
            .catalog
            .materials
            .get(&id)
            .with_context(|| format!("Material {id} is in none of the scene's sources"))?;
        anyhow::ensure!(
            !self.materials.contains_key(name),
            "two Materials are named {name:?}"
        );
        let mut textures = Vec::new();
        for texture in material.textures.iter().flatten() {
            let (channel, file) = self.catalog.textures.get(texture).with_context(|| {
                format!("Texture {texture} of Material {name:?} is in none of the scene's sources")
            })?;
            let (file, image) = match file {
                TextureFile::Generated {
                    name,
                    image: generated,
                } => {
                    let channel = channel_name(*channel);
                    let file = format!("textures/{}-{channel}.png", file_name(name));
                    let path = self.directory.join(&file);
                    std::fs::create_dir_all(self.directory.join("textures"))?;
                    image::save_buffer_with_format(
                        &path,
                        &generated.pixels,
                        generated.width,
                        generated.height,
                        image::ExtendedColorType::Rgba8,
                        image::ImageFormat::Png,
                    )
                    .with_context(|| format!("writing {}", path.display()))?;
                    (file, None)
                }
                TextureFile::File(path) => (path.display().to_string(), None),
                TextureFile::Embedded { path, image } => (path.display().to_string(), Some(*image)),
            };
            textures.push(TextureEntry {
                channel: channel_name(*channel),
                file,
                image,
            });
        }
        let entry = MaterialEntry {
            base_color: material.base_color,
            metallic: material.metallic,
            roughness: material.roughness,
            emissive: material.emissive,
            normal_scale: material.normal_scale,
            textures,
        };
        self.materials.insert(name.clone(), entry);
        self.material_names.insert(id, name.clone());
        Ok(name.clone())
    }

    /// Writes `bytes` to `file`, relative to the export's directory.
    fn write(&self, file: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let path = self.directory.join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))
    }
}

fn channel_name(channel: MaterialChannel) -> &'static str {
    match channel {
        MaterialChannel::Albedo => "albedo",
        MaterialChannel::Normal => "normal",
        MaterialChannel::MetallicRoughness => "metallic_roughness",
        MaterialChannel::Occlusion => "occlusion",
        MaterialChannel::Emissive => "emissive",
    }
}

/// The normal and glTF tangent, its w the bitangent sign
/// [ref:gltf-2.0-tangent-bitangent], that a QTangent decodes to as the shaders decode it: each Snorm16 component over 32,767,
/// clamped at −1, the quaternion normalised and applied to +Z and +X.
fn tangent_frame(qtangent: [i16; 4]) -> ([f32; 3], [f32; 4]) {
    let components = qtangent.map(|component| (f32::from(component) / i16::MAX as f32).max(-1.0));
    let rotation = glam::Quat::from_array(components).normalize();
    let bitangent_sign = if rotation.w < 0.0 { -1.0 } else { 1.0 };
    let normal = rotation * glam::Vec3::Z;
    let tangent = rotation * glam::Vec3::X;
    (normal.to_array(), tangent.extend(bitangent_sign).to_array())
}

/// `mesh`, named `name`, as a glTF binary of one Mesh on one node: its
/// vertices once, each Primitive's indices into them, and each vertex's
/// tangent frame as its QTangent decodes.
fn mesh_glb(name: &str, mesh: &CookedMesh) -> anyhow::Result<Vec<u8>> {
    // glTF's component types and buffer view targets.
    const FLOAT: u32 = 5126;
    const UNSIGNED_INT: u32 = 5125;
    const ARRAY_BUFFER: u32 = 34962;
    const ELEMENT_ARRAY_BUFFER: u32 = 34963;

    let (mut normals, mut tangents) = (Vec::new(), Vec::new());
    for vertex in &mesh.vertices {
        let (normal, tangent) = tangent_frame(vertex.qtangent);
        normals.push(normal);
        tangents.push(tangent);
    }
    let positions: Vec<[f32; 3]> = mesh.vertices.iter().map(|vertex| vertex.position).collect();
    let uvs: Vec<[f32; 2]> = mesh.vertices.iter().map(|vertex| vertex.uv).collect();
    let lowest = positions
        .iter()
        .fold([f32::INFINITY; 3], |lowest, position| {
            std::array::from_fn(|axis| lowest[axis].min(position[axis]))
        });
    let highest = positions
        .iter()
        .fold([f32::NEG_INFINITY; 3], |highest, position| {
            std::array::from_fn(|axis| highest[axis].max(position[axis]))
        });

    let mut bin = Vec::new();
    let mut views = Vec::new();
    let mut view = |bytes: &[u8], target: u32| {
        views.push(json!({
            "buffer": 0,
            "byteOffset": bin.len(),
            "byteLength": bytes.len(),
            "target": target,
        }));
        bin.extend_from_slice(bytes);
        views.len() - 1
    };
    let count = mesh.vertices.len();
    let mut accessors = vec![
        json!({
            "bufferView": view(bytemuck::cast_slice(&positions), ARRAY_BUFFER),
            "componentType": FLOAT, "count": count, "type": "VEC3",
            "min": lowest, "max": highest,
        }),
        json!({
            "bufferView": view(bytemuck::cast_slice(&normals), ARRAY_BUFFER),
            "componentType": FLOAT, "count": count, "type": "VEC3",
        }),
        json!({
            "bufferView": view(bytemuck::cast_slice(&tangents), ARRAY_BUFFER),
            "componentType": FLOAT, "count": count, "type": "VEC4",
        }),
        json!({
            "bufferView": view(bytemuck::cast_slice(&uvs), ARRAY_BUFFER),
            "componentType": FLOAT, "count": count, "type": "VEC2",
        }),
    ];
    let indices = view(bytemuck::cast_slice(&mesh.indices), ELEMENT_ARRAY_BUFFER);
    let mut primitives = Vec::new();
    for primitive in &mesh.primitives {
        accessors.push(json!({
            "bufferView": indices,
            "byteOffset": primitive.first_index as usize * size_of::<u32>(),
            "componentType": UNSIGNED_INT, "count": primitive.index_count, "type": "SCALAR",
        }));
        primitives.push(json!({
            "attributes": { "POSITION": 0, "NORMAL": 1, "TANGENT": 2, "TEXCOORD_0": 3 },
            "indices": accessors.len() - 1,
        }));
    }
    let document = json!({
        "asset": { "version": "2.0", "generator": "praxis export-scene" },
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": views,
        "accessors": accessors,
        "meshes": [{ "name": name, "primitives": primitives }],
        "nodes": [{ "name": name, "mesh": 0 }],
        "scenes": [{ "nodes": [0] }],
        "scene": 0,
    });
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: 0,
        },
        json: Cow::Owned(serde_json::to_vec(&document)?),
        bin: Some(Cow::Owned(bin)),
    };
    Ok(glb.to_vec()?)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use super::*;
    use crate::cooked::{AssetId, CookedVertex};

    /// A scratch directory for one test's export, emptied first.
    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("praxis-export-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&directory).ok();
        directory
    }

    fn assets() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    fn read_scene(directory: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(directory.join("scene.json")).unwrap()).unwrap()
    }

    fn floats(value: &Value) -> Vec<f64> {
        value
            .as_array()
            .unwrap_or_else(|| panic!("{value} is not an array"))
            .iter()
            .map(|number| number.as_f64().unwrap())
            .collect()
    }

    fn assert_near(actual: &[f64], expected: &[f64], tolerance: f64, what: &str) {
        assert_eq!(actual.len(), expected.len(), "{what}: {actual:?}");
        for (actual_component, expected_component) in actual.iter().zip(expected) {
            assert!(
                (actual_component - expected_component).abs() <= tolerance,
                "{what}: {actual:?}, expected {expected:?}"
            );
        }
    }

    /// helmet, when SciFiHelmet is fetched: the textured helmet drawn with
    /// its glTF Materials, named from its source file, and the ten sweep
    /// copies with their transforms and factors, under the four coloured
    /// point lights, as `src/scenes/mod.rs` and `src/state.rs` place them.
    #[test]
    fn helmet_exports_the_textured_helmet_its_sweep_and_lights() {
        if !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(crate::DEFAULT_MODEL)
            .exists()
        {
            // Not `eprintln!`, which libtest captures from passing tests.
            use std::io::Write;
            writeln!(std::io::stderr(), "skipped: SciFiHelmet is not fetched").ok();
            return;
        }
        let directory = scratch("helmet");
        export_scene(ReferenceScene::Helmet, None, &assets(), &directory).unwrap();
        let scene = read_scene(&directory);

        assert_eq!(scene["frame"], 59);
        let renderables = scene["renderables"].as_array().unwrap();
        assert_eq!(renderables.len(), 11);
        for renderable in renderables {
            assert_eq!(renderable["mesh"], "SciFiHelmet");
        }
        let textured: Vec<_> = renderables
            .iter()
            .filter(|renderable| renderable["material"].is_null())
            .collect();
        assert_eq!(textured.len(), 1);
        assert_eq!(floats(&textured[0]["translation"]), [0.0, 0.0, -8.0]);
        assert_eq!(floats(&textured[0]["scale"]), [1.0; 3]);
        for (row, metallic) in [0.0, 1.0].into_iter().enumerate() {
            for (column, roughness) in [0.1, 0.3, 0.5, 0.7, 0.9].into_iter().enumerate() {
                let name = format!("metallic-{metallic}-roughness-{roughness}");
                let copies: Vec<_> = renderables
                    .iter()
                    .filter(|renderable| renderable["material"] == name.as_str())
                    .collect();
                assert_eq!(copies.len(), 1, "{name}");
                let translation = [(column as f64 - 2.0) * 2.0, 2.6 - row as f64 * 5.2, -8.0];
                assert_near(
                    &floats(&copies[0]["translation"]),
                    &translation,
                    1e-6,
                    &name,
                );
                assert_eq!(floats(&copies[0]["rotation"]), [0.0, 0.0, 0.0, 1.0]);
                assert_eq!(floats(&copies[0]["scale"]), [0.75; 3]);
                let material = &scene["materials"][&name];
                assert_eq!(floats(&material["base_color"]), [1.0; 4]);
                assert_eq!(material["metallic"], metallic);
                assert_near(
                    &[material["roughness"].as_f64().unwrap()],
                    &[roughness],
                    1e-7,
                    &name,
                );
            }
        }

        let mesh = &scene["meshes"]["SciFiHelmet"];
        let file = mesh["file"].as_str().unwrap();
        assert!(file.ends_with("SciFiHelmet.gltf"), "{file}");
        assert!(Path::new(file).is_absolute(), "{file}");
        assert_eq!(mesh["mesh"], 0);
        let materials = mesh["materials"].as_array().unwrap();
        assert_eq!(materials.len(), 1);
        let material = &scene["materials"][materials[0].as_str().unwrap()];
        let textures = material["textures"].as_array().unwrap();
        let channels: Vec<_> = textures.iter().map(|texture| &texture["channel"]).collect();
        assert_eq!(
            channels,
            ["albedo", "normal", "metallic_roughness", "occlusion"]
        );
        for (texture, suffix) in textures.iter().zip([
            "SciFiHelmet_BaseColor.png",
            "SciFiHelmet_Normal.png",
            "SciFiHelmet_MetallicRoughness.png",
            "SciFiHelmet_AmbientOcclusion.png",
        ]) {
            let file = texture["file"].as_str().unwrap();
            assert!(file.ends_with(suffix) && Path::new(file).exists(), "{file}");
        }

        let lights = scene["point_lights"].as_array().unwrap();
        let expected = [
            ([2.0, 2.0, 2.0], [1.0, 0.0, 0.0]),
            ([-2.0, 2.0, 2.0], [0.0, 1.0, 0.0]),
            ([2.0, 2.0, -2.0], [0.0, 0.0, 1.0]),
            ([-2.0, 2.0, -2.0], [1.0, 1.0, 0.0]),
        ];
        assert_eq!(lights.len(), expected.len());
        for (position, color) in expected {
            let light = lights
                .iter()
                .find(|light| floats(&light["position"]) == position)
                .unwrap_or_else(|| panic!("no light at {position:?}"));
            assert_eq!(floats(&light["color"]), color);
            assert_eq!(light["intensity"], 900_000.0);
            assert_eq!(light["range"], 20.0);
        }
        let suns = scene["directional_lights"].as_array().unwrap();
        assert_eq!(suns.len(), 1);
        let direction = glam::dvec3(0.5, -1.0, -0.3).normalize();
        assert_near(
            &floats(&suns[0]["direction"]),
            &direction.to_array(),
            1e-6,
            "sun direction",
        );
        assert_eq!(floats(&suns[0]["color"]), [1.0; 3]);
        assert_eq!(suns[0]["illuminance"], 100_000.0);
        let environment = &scene["environment"];
        assert_eq!(floats(&environment["ambient_illuminance"]), [10_000.0; 3]);
        assert_near(
            &floats(&environment["background_luminance"]),
            &[10_000.0, 20_000.0, 30_000.0],
            0.01,
            "background",
        );
        assert!(!directory.join("meshes").exists());
        std::fs::remove_dir_all(&directory).ok();
    }

    /// Checks that each name `scene.json` uses is defined in it, and that
    /// each file it names exists.
    fn check_references(directory: &Path, scene: &Value) {
        let exists = |file: &Value| {
            let file = file.as_str().unwrap();
            assert!(directory.join(file).exists(), "{file} is not written");
        };
        let (meshes, materials) = (&scene["meshes"], &scene["materials"]);
        for renderable in scene["renderables"].as_array().unwrap() {
            assert!(meshes.get(renderable["mesh"].as_str().unwrap()).is_some());
            if let Some(material) = renderable["material"].as_str() {
                assert!(materials.get(material).is_some(), "{material}");
            }
        }
        for mesh in meshes.as_object().unwrap().values() {
            exists(&mesh["file"]);
            for material in mesh["materials"].as_array().unwrap() {
                if let Some(material) = material.as_str() {
                    assert!(materials.get(material).is_some(), "{material}");
                }
            }
        }
        for material in materials.as_object().unwrap().values() {
            for texture in material["textures"].as_array().unwrap() {
                exists(&texture["file"]);
            }
        }
    }

    /// Every Reference scene but crowd, and town, which its own test
    /// exports: each Renderable's Mesh and Material, and each file they name.
    #[test]
    fn every_reference_scene_but_crowd_exports_whole() {
        let helmet_fetched = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(crate::DEFAULT_MODEL)
            .exists();
        for scene in ReferenceScene::ALL {
            if matches!(scene, ReferenceScene::Crowd | ReferenceScene::Town)
                || (scene == ReferenceScene::Helmet && !helmet_fetched)
            {
                continue;
            }
            let directory = scratch(scene.name());
            export_scene(scene, Some(0), &assets(), &directory).unwrap();
            let exported = read_scene(&directory);
            assert_eq!(exported["scene"], scene.name());
            assert!(!exported["renderables"].as_array().unwrap().is_empty());
            check_references(&directory, &exported);
            std::fs::remove_dir_all(&directory).ok();
        }
    }

    /// town at a frame of its camera path: its camera there, its 1,024 lamp
    /// lights, and its pigeons moved by the fixed schedule since frame 0,
    /// while nothing else moved.
    #[test]
    fn town_exports_its_pigeons_where_the_fixed_schedule_placed_them() {
        let renderables_at = |frame| {
            let directory = scratch(&format!("town-{frame}"));
            export_scene(ReferenceScene::Town, Some(frame), &assets(), &directory).unwrap();
            let scene = read_scene(&directory);
            check_references(&directory, &scene);
            std::fs::remove_dir_all(&directory).ok();
            scene
        };
        let (start, later) = (renderables_at(0), renderables_at(30));
        // Up the avenue from (2, 1.7, 445) at 4 m/s for half a second.
        assert_near(
            &floats(&later["camera"]["position"]),
            &[2.0, 1.7, 443.0],
            1e-9,
            "camera",
        );
        assert_eq!(later["point_lights"].as_array().unwrap().len(), 1_024);
        let (start, later) = (
            start["renderables"].as_array().unwrap(),
            later["renderables"].as_array().unwrap(),
        );
        assert_eq!(start.len(), later.len());
        let mut birds = 0;
        for (before, after) in start.iter().zip(later) {
            assert_eq!(before["mesh"], after["mesh"]);
            let moved = before["translation"] != after["translation"];
            assert_eq!(moved, before["mesh"] == "bird", "{before} then {after}");
            birds += usize::from(moved);
        }
        assert_eq!(birds, 10_000);
    }

    #[test]
    fn crowd_is_refused_with_why() {
        let directory = scratch("crowd");
        let error = export_scene(ReferenceScene::Crowd, Some(0), &assets(), &directory)
            .unwrap_err()
            .to_string();
        assert!(error.contains("the GPU bends its figures"), "{error}");
        assert!(!directory.exists());
    }

    /// One Snorm16 step, the QTangent's precision.
    const STEP: f64 = 1.0 / i16::MAX as f64;

    /// shadows, which needs no fetched source: its camera, lights,
    /// Environment, Renderable and Materials are `src/scenes/mod.rs`'s, and its
    /// generated Mesh, read back by the `gltf` crate, is the one the cooker
    /// cooks, its normals and tangents the analytic sphere's and plane's
    /// within the QTangent's Snorm16 steps.
    #[test]
    fn shadows_exports_its_constants_and_its_cooked_mesh() {
        let directory = scratch("shadows");
        export_scene(ReferenceScene::Shadows, Some(0), &assets(), &directory).unwrap();
        let scene = read_scene(&directory);

        assert_eq!(scene["version"], 1);
        assert_eq!(scene["scene"], "shadows");
        assert_eq!(scene["frame"], 0);
        let camera = &scene["camera"];
        let position = floats(&camera["position"]);
        assert_eq!(position, [0.0, 2.4, 3.0]);
        let rotation = floats(&camera["rotation"]);
        let rotation = glam::DQuat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);
        let towards = (glam::dvec3(0.0, 0.45, -0.4) - glam::dvec3(0.0, 2.4, 3.0)).normalize();
        assert_near(
            &(rotation * glam::DVec3::NEG_Z).to_array(),
            &towards.to_array(),
            1e-6,
            "camera forward",
        );
        assert_near(
            &[camera["vertical_field_of_view"].as_f64().unwrap()],
            &[std::f64::consts::FRAC_PI_4],
            1e-7,
            "field of view",
        );
        // Exposes 100,000 lux to 1: log2(1 / (1.2 · 10^-5)).
        assert_near(
            &[camera["ev100"].as_f64().unwrap()],
            &[16.3466],
            1e-4,
            "EV100",
        );

        let environment = &scene["environment"];
        assert_eq!(floats(&environment["ambient_illuminance"]), [10_000.0; 3]);
        // 0.3 · 100,000 in f32 rounds to 30,000.002.
        assert_near(
            &floats(&environment["background_luminance"]),
            &[10_000.0, 20_000.0, 30_000.0],
            0.01,
            "background",
        );
        let suns = scene["directional_lights"].as_array().unwrap();
        assert_eq!(suns.len(), 1);
        let direction = glam::dvec3(0.5, -1.0, -0.3).normalize();
        assert_near(
            &floats(&suns[0]["direction"]),
            &direction.to_array(),
            1e-6,
            "sun direction",
        );
        assert_eq!(floats(&suns[0]["color"]), [1.0; 3]);
        assert_eq!(suns[0]["illuminance"], 100_000.0);
        let lights = scene["point_lights"].as_array().unwrap();
        assert_eq!(lights.len(), 1);
        assert_eq!(floats(&lights[0]["position"]), [1.5, 2.5, 1.0]);
        assert_eq!(floats(&lights[0]["color"]), [1.0; 3]);
        assert_eq!(lights[0]["intensity"], 900_000.0);
        assert_eq!(lights[0]["range"], 20.0);
        assert_eq!(lights[0]["casts_shadow"], true);

        let renderables = scene["renderables"].as_array().unwrap();
        assert_eq!(renderables.len(), 1);
        assert_eq!(renderables[0]["mesh"], "sphere-over-plane");
        assert_eq!(renderables[0]["material"], Value::Null);
        assert_eq!(floats(&renderables[0]["translation"]), [0.0; 3]);
        assert_eq!(floats(&renderables[0]["rotation"]), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(floats(&renderables[0]["scale"]), [1.0; 3]);

        for (name, base_color) in [
            ("ground", [0.8, 0.8, 0.8, 1.0]),
            ("sphere", [0.8, 0.25, 0.1, 1.0]),
        ] {
            let material = &scene["materials"][name];
            assert_near(&floats(&material["base_color"]), &base_color, 1e-7, name);
            assert_eq!(material["metallic"], 0.0);
            assert_near(
                &[material["roughness"].as_f64().unwrap()],
                &[0.6],
                1e-7,
                name,
            );
            assert_eq!(floats(&material["emissive"]), [0.0; 3]);
            assert_eq!(material["normal_scale"], 1.0);
            assert_eq!(material["textures"], serde_json::json!([]));
        }

        let mesh = &scene["meshes"]["sphere-over-plane"];
        assert_eq!(mesh["materials"], serde_json::json!(["ground", "sphere"]));
        assert_eq!(mesh["mesh"], 0);
        let file = directory.join(mesh["file"].as_str().unwrap());
        let (document, buffers, _) = gltf::import(&file).unwrap();
        let read_back = document.meshes().next().unwrap();
        let cooked = crate::cook::mesh::cook(
            &ReferenceScene::Shadows.meshes()[0]
                .1
                .iter()
                .map(|(geometry, _)| (geometry.clone(), Some(AssetId(1))))
                .collect::<Vec<_>>(),
        );
        assert_eq!(read_back.primitives().len(), cooked.primitives.len());
        for (index, (primitive, cooked_primitive)) in
            read_back.primitives().zip(&cooked.primitives).enumerate()
        {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
            let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).unwrap().into_f32().collect();
            let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
            let tangents: Vec<[f32; 4]> = reader.read_tangents().unwrap().collect();
            let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
            let first_index = cooked_primitive.first_index as usize;
            assert_eq!(
                indices,
                cooked.indices[first_index..][..cooked_primitive.index_count as usize]
            );
            let cooked_positions: Vec<[f32; 3]> = cooked
                .vertices
                .iter()
                .map(|vertex| vertex.position)
                .collect();
            assert_eq!(positions, cooked_positions);
            let cooked_uvs: Vec<[f32; 2]> = cooked
                .vertices
                .iter()
                .map(|vertex: &CookedVertex| vertex.uv)
                .collect();
            assert_eq!(uvs, cooked_uvs);
            for &vertex in &indices {
                let vertex = vertex as usize;
                let position = glam::Vec3::from(positions[vertex]).as_dvec3();
                let normal = glam::Vec3::from(normals[vertex]).as_dvec3();
                let [x, y, z, sign] = tangents[vertex].map(f64::from);
                let tangent = glam::dvec3(x, y, z);
                // The ground's normal is +Y and its tangent +X, u growing
                // along it; the sphere's normal points out from its centre,
                // 1 m up, and its tangent lies across it.
                let expected_normal = if index == 0 {
                    glam::DVec3::Y
                } else {
                    (position - glam::DVec3::Y).normalize()
                };
                assert_near(
                    &normal.to_array(),
                    &expected_normal.to_array(),
                    4.0 * STEP,
                    "normal",
                );
                assert!(
                    (tangent.length() - 1.0).abs() <= 4.0 * STEP,
                    "tangent {tangent}"
                );
                assert!(normal.dot(tangent).abs() <= 4.0 * STEP, "tangent {tangent}");
                assert_eq!(sign.abs(), 1.0);
                if index == 0 {
                    assert_near(&tangent.to_array(), &[1.0, 0.0, 0.0], 4.0 * STEP, "tangent");
                }
            }
        }
        std::fs::remove_dir_all(&directory).ok();
    }
}
