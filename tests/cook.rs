//! The cook seam: a scratch source tree cooked into a scratch Cooked cache
//! and read back by Asset ID through the runtime's Blob reader.
#![cfg(feature = "cook")]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use praxis::ReferenceScene;
use praxis::cook::{CookSettings, texture_sources};
use praxis::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, CookedCache, CookedMaterial, CookedTexture,
    MaterialChannel,
};
use serde_json::json;

/// The Reference scenes this seam cooks: all but texture-wall, whose 10,000
/// Textures are 20,000 files, each flushed to the drive as it is written
/// (84 s cold on the M4 Pro). Its Goldens cover its cook, not its PSNR: each
/// of its Textures is one colour.
fn scenes() -> Vec<ReferenceScene> {
    ReferenceScene::ALL
        .into_iter()
        .filter(|&scene| scene != ReferenceScene::TextureWall)
        .collect()
}

fn cook(assets: &Path, cache: &CookedCache) -> anyhow::Result<praxis::cook::CookReport> {
    praxis::cook::cook(assets, &scenes(), cache, &CookSettings::default())
}

const GUID: u128 = 0x5ca1ab1e_0000_4000_8000_00000000c00c;

/// A glTF file with a named Material sampling a named image, and an
/// unnamed Material, which is named by its index.
fn gltf(roughness: f32) -> String {
    json!({
        "asset": {"version": "2.0"},
        "images": [{"name": "paint", "uri": "paint.png"}],
        "textures": [{"source": 0}],
        "materials": [
            {
                "name": "painted",
                "pbrMetallicRoughness": {
                    "baseColorFactor": [0.5, 0.25, 1.0, 1.0],
                    "baseColorTexture": {"index": 0},
                    "metallicFactor": 0.0,
                    "roughnessFactor": roughness,
                },
            },
            {"emissiveFactor": [1.0, 0.5, 0.0]},
        ],
    })
    .to_string()
}

/// A source tree holding `gltf` at `path`, listed in its manifest under
/// `GUID`.
fn write_sources(assets: &Path, path: &str, gltf: &str) {
    if assets.exists() {
        fs::remove_dir_all(assets).unwrap();
    }
    fs::create_dir_all(assets.join(path).parent().unwrap()).unwrap();
    fs::write(assets.join(path), gltf).unwrap();
    let manifest = json!([{"path": path, "guid": format!("{GUID:032x}")}]);
    fs::write(assets.join("manifest.json"), manifest.to_string()).unwrap();
}

/// A directory for one test in one process, holding only `cooked/`: a copy
/// of `scene_cache`, so a test cooks only its own sources.
fn scratch(test: &str) -> PathBuf {
    let directory =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cook-{test}-{}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    for part in ["blobs", "index"] {
        let (from, to) = (
            scene_cache().join(part),
            directory.join("cooked").join(part),
        );
        fs::create_dir_all(&to).unwrap();
        for file in fs::read_dir(from).unwrap() {
            let file = file.unwrap();
            fs::copy(file.path(), to.join(file.file_name())).unwrap();
        }
    }
    directory
}

/// A Cooked cache of the procedural Reference scenes alone, kept across
/// runs: cooks are safe to run concurrently, and one whose scenes changed
/// re-cooks them.
fn scene_cache() -> &'static Path {
    static CACHE: OnceLock<PathBuf> = OnceLock::new();
    CACHE.get_or_init(|| {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cook-scenes");
        let assets = directory.join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("manifest.json"), "[]").unwrap();
        let cache = directory.join("cooked");
        cook(&assets, &CookedCache::new(&cache)).unwrap();
        cache
    })
}

fn material_id(name: AssetName) -> AssetId {
    AssetId::new(AssetSource::File(GUID), AssetKind::Material, name)
}

const PAINTED: AssetName = AssetName::Name("painted");
const UNNAMED: AssetName = AssetName::Index(1);

/// The Asset ID of the image `name` in `channel`.
fn texture_id(channel: MaterialChannel, name: &str) -> AssetId {
    AssetId::new(
        AssetSource::File(GUID),
        AssetKind::Texture(channel),
        AssetName::Name(name),
    )
}

/// The Asset ID of the image `paint` in the albedo Channel.
fn paint_id() -> AssetId {
    texture_id(MaterialChannel::Albedo, "paint")
}

/// Writes `path`: `width` × `height` RGBA8 from `texel(x, y)`.
fn write_png(path: &Path, width: u32, height: u32, texel: impl Fn(u32, u32) -> [u8; 4]) {
    let image = image::RgbaImage::from_fn(width, height, |x, y| image::Rgba(texel(x, y)));
    image.save(path).unwrap();
}

/// Writes `paint.png` beside `box.gltf`: `width` × `height` RGBA8 from
/// `texel(x, y)`.
fn write_paint(assets: &Path, width: u32, height: u32, texel: impl Fn(u32, u32) -> [u8; 4]) {
    write_png(&assets.join("paint.png"), width, height, texel);
}

/// Every texel of `texture`'s mip `level`, decoded, as RGBA8.
fn decode(texture: &CookedTexture, level: usize) -> Vec<[u8; 4]> {
    let width = (texture.width >> level).max(1) as usize;
    let height = (texture.height >> level).max(1) as usize;
    let blocks = &texture.levels[level];
    let mut texels = vec![0u32; width * height];
    match texture.format {
        wgpu::TextureFormat::Bc7RgbaUnormSrgb => {
            texture2ddecoder::decode_bc7(blocks, width, height, &mut texels)
        }
        wgpu::TextureFormat::Bc5RgUnorm => {
            texture2ddecoder::decode_bc5(blocks, width, height, &mut texels)
        }
        wgpu::TextureFormat::Bc4RUnorm => {
            texture2ddecoder::decode_bc4(blocks, width, height, &mut texels)
        }
        other => panic!("unexpected format {other:?}"),
    }
    .unwrap();
    // The decoder packs B, G, R, A from the low byte up.
    texels
        .into_iter()
        .map(|texel| {
            let [blue, green, red, alpha] = texel.to_le_bytes();
            [red, green, blue, alpha]
        })
        .collect()
}

/// Peak signal-to-noise ratio in dB of 8-bit samples against their source.
fn psnr(pairs: impl Iterator<Item = (u8, u8)>) -> f64 {
    let (mut squared_error, mut count) = (0.0, 0.0);
    for (decoded, source) in pairs {
        squared_error += (decoded as f64 - source as f64).powi(2);
        count += 1.0;
    }
    let mean_squared_error = squared_error / count;
    10.0 * (255.0f64.powi(2) / mean_squared_error).log10()
}

fn blob_count(cache: &Path) -> usize {
    fs::read_dir(cache.join("blobs")).unwrap().count()
}

#[test]
fn moving_and_renaming_a_source_keeps_its_asset_ids() {
    let directory = scratch("move");
    let (assets, cache_path) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache_path);
    write_sources(&assets, "models/box.gltf", &gltf(0.75));
    cook(&assets, &cache).unwrap();

    let painted = cache.material(material_id(PAINTED)).unwrap();
    let paint = AssetId::new(
        AssetSource::File(GUID),
        AssetKind::Texture(MaterialChannel::Albedo),
        AssetName::Name("paint"),
    );
    assert_eq!(
        painted,
        CookedMaterial {
            base_color: [0.5, 0.25, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.75,
            emissive: [0.0; 3],
            textures: [Some(paint), None, None, None, None],
        }
    );
    let unnamed = cache.material(material_id(UNNAMED)).unwrap();
    assert_eq!(unnamed.emissive, [1.0, 0.5, 0.0]);

    write_sources(&assets, "moved/renamed.gltf", &gltf(0.75));
    let blobs = blob_count(&cache_path);
    let report = cook(&assets, &cache).unwrap();
    assert_eq!(
        report.cooked,
        [],
        "a move with the GUID kept re-cooks nothing"
    );
    assert_eq!(blob_count(&cache_path), blobs);
    assert_eq!(cache.material(material_id(PAINTED)).unwrap(), painted);
    assert_eq!(cache.material(material_id(UNNAMED)).unwrap(), unnamed);
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_changed_source_re_cooks_exactly_the_assets_it_changes() {
    let directory = scratch("change");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    write_sources(&assets, "box.gltf", &gltf(0.75));
    cook(&assets, &cache).unwrap();

    write_sources(&assets, "box.gltf", &gltf(0.5));
    assert_eq!(
        cook(&assets, &cache).unwrap().cooked,
        [material_id(PAINTED)]
    );
    assert_eq!(cache.material(material_id(PAINTED)).unwrap().roughness, 0.5);
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn deleting_the_cooked_cache_is_safe() {
    let directory = scratch("delete");
    let (assets, cache_path) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache_path);
    write_sources(&assets, "box.gltf", &gltf(0.75));
    cook(&assets, &cache).unwrap();

    fs::remove_dir_all(&cache_path).unwrap();
    let error = cache.material(material_id(PAINTED)).unwrap_err();
    assert!(
        format!("{error:#}").contains("run `praxis cook`"),
        "{error:#}"
    );
    cook(&assets, &cache).unwrap();
    assert_eq!(
        cache.material(material_id(PAINTED)).unwrap().roughness,
        0.75
    );
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn concurrent_cooks_leave_a_valid_cache() {
    let directory = scratch("concurrent");
    let (assets, cache_path) = (directory.join("assets"), directory.join("cooked"));
    write_sources(&assets, "box.gltf", &gltf(0.75));
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| cook(&assets, &CookedCache::new(&cache_path)).unwrap());
        }
    });

    let cache = CookedCache::new(&cache_path);
    let report = cook(&assets, &cache).unwrap();
    assert_eq!(report.cooked, [], "the concurrent cooks left every entry");
    for id in report.unchanged {
        cache.read(id).unwrap();
    }
    let temporary_files = fs::read_dir(cache_path.join("index"))
        .unwrap()
        .chain(fs::read_dir(cache_path.join("blobs")).unwrap())
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        })
        .count();
    assert_eq!(temporary_files, 0);
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn reference_scene_textures_cook_every_mip_and_keep_their_quality() {
    let directory = scratch("reference");
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let cache = CookedCache::new(directory.join("cooked"));
    cook(&assets, &cache).unwrap();

    let sources = texture_sources(&assets, &scenes()).unwrap();
    assert!(!sources.is_empty());
    for source in sources {
        let texture = cache.texture(source.id).unwrap();
        let mips = texture.width.max(texture.height).ilog2() as usize + 1;
        assert_eq!(texture.levels.len(), mips, "{}", source.name);

        let image = source.decode().unwrap();
        let decoded = decode(&texture, 0);
        let texels = decoded.iter().zip(image.pixels.as_chunks::<4>().0);
        // The source channels each stored channel holds (ADR 0018).
        let stored: &[(usize, usize)] = match source.channel {
            MaterialChannel::Albedo | MaterialChannel::Emissive => {
                &[(0, 0), (1, 1), (2, 2), (3, 3)]
            }
            MaterialChannel::Normal => &[(0, 0), (1, 1)],
            MaterialChannel::MetallicRoughness => &[(0, 1), (1, 2)],
            MaterialChannel::Occlusion => &[(0, 0)],
        };
        let decibels = psnr(texels.flat_map(|(decoded, source)| {
            stored
                .iter()
                .map(|&(stored, from)| (decoded[stored], source[from]))
        }));
        // Not `eprintln!`, which libtest captures from passing tests.
        writeln!(
            std::io::stderr(),
            "{} {:?}: {decibels:.2} dB",
            source.name,
            source.channel
        )
        .ok();
        if texture.format == wgpu::TextureFormat::Bc7RgbaUnormSrgb {
            assert!(decibels >= 44.0, "{}: {decibels:.2} dB", source.name);
        }
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_black_and_white_checker_filters_in_linear_light() {
    let directory = scratch("checker");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    write_sources(&assets, "box.gltf", &gltf(0.75));
    write_paint(&assets, 8, 8, |x, y| {
        let value = if (x + y) % 2 == 0 { 0 } else { 255 };
        [value, value, value, 255]
    });
    cook(&assets, &cache).unwrap();

    let texture = cache.texture(paint_id()).unwrap();
    assert_eq!(texture.levels.len(), 4);
    // Half of full intensity in linear light is 187.5 in sRGB; an average
    // of the encoded values would be 128.
    for [red, green, blue, alpha] in decode(&texture, 1) {
        for value in [red, green, blue] {
            assert!(value.abs_diff(188) <= 2, "{value}");
        }
        assert_eq!(alpha, 255);
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_texture_size_not_a_multiple_of_4_or_above_8192_is_refused() {
    let directory = scratch("refuse");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    for ((width, height), reason) in [((6, 4), "multiple of 4"), ((8196, 4), "8192")] {
        write_sources(&assets, "box.gltf", &gltf(0.75));
        write_paint(&assets, width, height, |_, _| [255; 4]);
        let error = format!("{:#}", cook(&assets, &cache).unwrap_err());
        assert!(error.contains(reason), "{width}x{height}: {error}");
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_changed_cook_setting_re_cooks_exactly_the_assets_it_covers() {
    let directory = scratch("setting");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    write_sources(&assets, "box.gltf", &gltf(0.75));
    write_paint(&assets, 8, 8, |x, y| [x as u8 * 32, y as u8 * 32, 0, 255]);
    let settings = CookSettings::default();
    praxis::cook::cook(&assets, &scenes(), &cache, &settings).unwrap();
    let levels = cache.texture(paint_id()).unwrap().levels;

    let changed = CookSettings {
        texture_zstd_level: settings.texture_zstd_level + 1,
    };
    let mut cooked = praxis::cook::cook(&assets, &scenes(), &cache, &changed)
        .unwrap()
        .cooked;
    cooked.sort();
    let mut textures: Vec<_> = texture_sources(&assets, &scenes())
        .unwrap()
        .into_iter()
        .map(|source| source.id)
        .collect();
    textures.sort();
    assert!(textures.contains(&paint_id()));
    assert_eq!(cooked, textures, "every Texture and no Material");
    assert_eq!(cache.texture(paint_id()).unwrap().levels, levels);
    fs::remove_dir_all(&directory).unwrap();
}

/// The side and mip count of the Toksvig sources' images.
const TOKSVIG_SIDE: u32 = 16;
const TOKSVIG_MIPS: usize = 5;
/// In the normal map's top half, each low bit of a texel's x tilts its
/// normal about y by this much, one way when the bit is clear and the other
/// when it is set; the bottom half is flat. A tilted box 2^m texels wide
/// then holds every sign combination of m tilts, so its normals' mean has
/// length cos(tilt)^m.
const TILT_DEGREES: f32 = 10.0;
/// The roughness both metallic-roughness images hold at every texel.
const ROUGHNESS_BYTE: u8 = 77;
/// Covers BC5's error on these blocks of one or two values, and the 8-bit
/// rounding of the source normals.
const ROUGHNESS_TOLERANCE: f32 = 0.01;

/// `bumpy` samples a tilted normal map and a metallic-roughness image;
/// `smooth` samples another metallic-roughness image and no normal map.
fn write_toksvig_sources(assets: &Path) {
    let gltf = json!({
        "asset": {"version": "2.0"},
        "images": [
            {"name": "bumps", "uri": "bumps.png"},
            {"name": "bumpy-roughness", "uri": "bumpy-roughness.png"},
            {"name": "smooth-roughness", "uri": "smooth-roughness.png"},
        ],
        "textures": [{"source": 0}, {"source": 1}, {"source": 2}],
        "materials": [
            {
                "name": "bumpy",
                "normalTexture": {"index": 0},
                "pbrMetallicRoughness": {"metallicRoughnessTexture": {"index": 1}},
            },
            {
                "name": "smooth",
                "pbrMetallicRoughness": {"metallicRoughnessTexture": {"index": 2}},
            },
        ],
    });
    write_sources(assets, "toksvig.gltf", &gltf.to_string());
    let tilt = TILT_DEGREES.to_radians();
    write_png(
        &assets.join("bumps.png"),
        TOKSVIG_SIDE,
        TOKSVIG_SIDE,
        |x, y| {
            let angle: f32 = (0..TOKSVIG_MIPS - 1)
                .map(|bit| if x >> bit & 1 == 0 { tilt } else { -tilt })
                .sum();
            let angle = if y < TOKSVIG_SIDE / 2 { angle } else { 0.0 };
            let byte = |value: f32| ((value + 1.0) / 2.0 * 255.0).round() as u8;
            [byte(angle.sin()), byte(0.0), byte(angle.cos()), 255]
        },
    );
    for name in ["bumpy-roughness", "smooth-roughness"] {
        write_png(
            &assets.join(format!("{name}.png")),
            TOKSVIG_SIDE,
            TOKSVIG_SIDE,
            |_, _| [0, ROUGHNESS_BYTE, 0, 255],
        );
    }
}

#[test]
fn toksvig_widens_each_metallic_roughness_mip_by_its_normal_maps_variance() {
    let directory = scratch("toksvig");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    write_toksvig_sources(&assets);
    cook(&assets, &cache).unwrap();

    let id = texture_id(MaterialChannel::MetallicRoughness, "bumpy-roughness");
    let texture = cache.texture(id).unwrap();
    assert_eq!(texture.levels.len(), TOKSVIG_MIPS);
    let (roughness, tilt) = (ROUGHNESS_BYTE as f32 / 255.0, TILT_DEGREES.to_radians());
    for mip in 0..TOKSVIG_MIPS {
        let side = TOKSVIG_SIDE >> mip;
        // Roughness is the stored red channel (ADR 0018).
        for (texel, [cooked, ..]) in decode(&texture, mip).into_iter().enumerate() {
            // The tilted share of the rows this texel covers. Only the last
            // mip covers both halves, where the tilted mean points straight
            // up, so the two lengths average by share.
            let row = texel as u32 / side;
            let rows = (row << mip)..((row + 1) << mip);
            let tilted =
                rows.filter(|row| *row < TOKSVIG_SIDE / 2).count() as f32 / (1 << mip) as f32;
            let mean_length = tilted * tilt.cos().powi(mip as i32) + 1.0 - tilted;
            // Toksvig's variance from the mean's length, then GGX's alpha
            // (roughness squared) widened by it, as src/cook/texture.rs cites.
            let angular_variance = (1.0 - mean_length) / mean_length;
            let expected = (roughness.powi(4) + 2.0 * angular_variance).powf(0.25);
            if mip > 0 && tilted > 0.0 {
                assert!(expected - roughness > 10.0 * ROUGHNESS_TOLERANCE);
            }
            let cooked = cooked as f32 / 255.0;
            assert!(
                (cooked - expected).abs() <= ROUGHNESS_TOLERANCE,
                "mip {mip}, texel {texel}: roughness {cooked}, Toksvig {expected}"
            );
        }
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_material_without_a_normal_map_cooks_its_roughness_unchanged() {
    let directory = scratch("no-normal-map");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    write_toksvig_sources(&assets);
    cook(&assets, &cache).unwrap();

    let id = texture_id(MaterialChannel::MetallicRoughness, "smooth-roughness");
    let texture = cache.texture(id).unwrap();
    assert_eq!(texture.levels.len(), TOKSVIG_MIPS);
    for mip in 0..TOKSVIG_MIPS {
        for [roughness, ..] in decode(&texture, mip) {
            assert_eq!(roughness, ROUGHNESS_BYTE, "mip {mip}");
        }
    }
    fs::remove_dir_all(&directory).unwrap();
}

const NORMAL_TANGENT_TEST_GUID: u128 = 0xc750a32dc879478eaca4748dbc82645a;

/// Changes one texel of the normal map embedded in NormalTangentTest's .glb
/// at `path` (its image 2): the edited image is appended to the binary
/// chunk, and the image's buffer view moves onto it.
fn touch_embedded_normal_map(path: &Path) {
    let glb = fs::read(path).unwrap();
    let gltf::Glb {
        header,
        json,
        bin: binary_chunk,
    } = gltf::Glb::from_slice(&glb).unwrap();
    let mut binary_chunk = binary_chunk.unwrap().into_owned();
    let mut document: serde_json::Value = serde_json::from_slice(&json).unwrap();
    let view = document["images"][2]["bufferView"].as_u64().unwrap() as usize;
    let offset = document["bufferViews"][view]["byteOffset"]
        .as_u64()
        .unwrap_or(0) as usize;
    let length = document["bufferViews"][view]["byteLength"]
        .as_u64()
        .unwrap() as usize;
    let mut normal_map = image::load_from_memory(&binary_chunk[offset..][..length])
        .unwrap()
        .into_rgba8();
    normal_map.get_pixel_mut(0, 0).0[0] ^= 0x10;
    let mut png = std::io::Cursor::new(Vec::new());
    normal_map
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let appended_offset = binary_chunk.len().next_multiple_of(4);
    binary_chunk.resize(appended_offset, 0);
    binary_chunk.extend(png.get_ref());
    document["bufferViews"][view] = json!({
        "buffer": 0,
        "byteOffset": appended_offset,
        "byteLength": png.get_ref().len(),
    });
    document["buffers"][0]["byteLength"] = json!(binary_chunk.len());
    let glb = gltf::Glb {
        header,
        json: document.to_string().into_bytes().into(),
        bin: Some(binary_chunk.into()),
    };
    fs::write(path, glb.to_vec().unwrap()).unwrap();
}

/// A dev launch of the normal-tangent scene cooks its source and nothing
/// else, cooks nothing when nothing changed, re-cooks exactly a touched
/// normal map and the metallic-roughness Texture widened by it, and keeps
/// the Blobs of a source, or a manifest, gone absent.
#[test]
fn a_dev_launch_cooks_exactly_what_its_scene_references_and_changed() {
    let directory = scratch("touch-embedded-image");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    let glb = assets.join("NormalTangentTest/NormalTangentTest.glb");
    fs::create_dir_all(glb.parent().unwrap()).unwrap();
    let repository_assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    fs::copy(
        repository_assets.join("NormalTangentTest/NormalTangentTest.glb"),
        &glb,
    )
    .unwrap();
    // Another source beside it, which the scene does not reference.
    fs::write(assets.join("box.gltf"), gltf(0.75)).unwrap();
    let manifest = json!([
        {
            "path": "NormalTangentTest/NormalTangentTest.glb",
            "guid": format!("{NORMAL_TANGENT_TEST_GUID:032x}"),
        },
        {"path": "box.gltf", "guid": format!("{GUID:032x}")},
    ]);
    fs::write(assets.join("manifest.json"), manifest.to_string()).unwrap();
    let launch = || {
        let settings = CookSettings::default();
        praxis::cook::cook_scene(ReferenceScene::NormalTangent, &assets, &cache, &settings).unwrap()
    };
    let first = launch();
    // The scratch cache holds every procedural scene: a full cook would
    // find this one unchanged.
    let other_scene = AssetId::new(
        AssetSource::Scene("pbr-sweep"),
        AssetKind::Mesh,
        AssetName::Name("sphere"),
    );
    assert!(!first.cooked.is_empty());
    for other in [other_scene, material_id(PAINTED)] {
        assert!(!first.cooked.contains(&other) && !first.unchanged.contains(&other));
    }
    assert_eq!(launch().cooked, [], "nothing changed, yet a launch cooked");

    touch_embedded_normal_map(&glb);
    let mut cooked = launch().cooked;
    cooked.sort();
    // Unnamed, NormalTangentTest's images are named by their index; image 1
    // is its metallic-roughness and its occlusion.
    let texture = |channel, image| {
        let source = AssetSource::File(NORMAL_TANGENT_TEST_GUID);
        AssetId::new(source, AssetKind::Texture(channel), AssetName::Index(image))
    };
    let mut expected = vec![
        texture(MaterialChannel::Normal, 2),
        texture(MaterialChannel::MetallicRoughness, 1),
    ];
    expected.sort();
    assert_eq!(cooked, expected);

    // A Blob stays as it is once its source, then the manifest, is gone.
    let kept = cache.texture(expected[0]).unwrap();
    for absent in [glb, assets.join("manifest.json")] {
        fs::remove_file(absent).unwrap();
        assert_eq!(launch().cooked, []);
        assert_eq!(cache.texture(expected[0]).unwrap(), kept);
    }
    fs::remove_dir_all(&directory).unwrap();
}

/// SciFiHelmet's cooked Mesh Blob bytes per triangle, Meshlets included, at
/// most: the M3 milestone's mesh-size ceiling (#192). A lower figure
/// re-records it.
const HELMET_MESH_BYTES_PER_TRIANGLE: f64 = 13.83;
const HELMET_TRIANGLES: usize = 23_358;

#[test]
fn every_reference_scene_mesh_cooks_and_reads_back() {
    let directory = scratch("meshes");
    let cache = CookedCache::new(directory.join("cooked"));
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let report = cook(&assets, &cache).unwrap();
    assert_eq!(report.unresolved_materials, []);

    let mesh_id = |source, name| AssetId::new(source, AssetKind::Mesh, AssetName::Name(name));
    let scene = |name| AssetSource::Scene(name);
    let normal_tangent = AssetSource::File(NORMAL_TANGENT_TEST_GUID);
    for id in [
        mesh_id(scene("pbr-sweep"), "sphere"),
        mesh_id(scene("uv-grid"), "plane"),
        mesh_id(scene("far-coplanar"), "plane"),
        mesh_id(scene("orrery"), "sphere"),
        mesh_id(scene("minification"), "ground"),
        mesh_id(normal_tangent, "NormalTangentTest_low"),
    ] {
        let mesh = cache.mesh(id).unwrap();
        assert!(!mesh.primitives.is_empty(), "Mesh {id} has no Primitives");
    }

    let helmet = mesh_id(
        AssetSource::File(0x3b67bb0196a34daa9d4389cb09aa9a27),
        "SciFiHelmet",
    );
    if assets.join("SciFiHelmet/SciFiHelmet.gltf").exists() {
        assert_eq!(
            cache.mesh(helmet).unwrap().indices.len(),
            HELMET_TRIANGLES * 3
        );
        let bytes_per_triangle = cache.read(helmet).unwrap().len() as f64 / HELMET_TRIANGLES as f64;
        assert!(
            bytes_per_triangle <= HELMET_MESH_BYTES_PER_TRIANGLE,
            "SciFiHelmet's Mesh Blob is {bytes_per_triangle:.2} bytes per triangle, over the {HELMET_MESH_BYTES_PER_TRIANGLE} ceiling"
        );
    }
    fs::remove_dir_all(&directory).unwrap();
}

/// The identity criterion's render half: NormalTangentTest moved and renamed
/// in a scratch copy, its GUID kept and its Cooked cache deleted, re-cooks
/// to the same Asset IDs and renders its Reference scene identically. Each
/// capture is a `praxis capture` process in the scratch directory, so its
/// launch cooks from that directory's `assets/`.
#[test]
fn a_moved_and_renamed_source_renders_its_scene_identically() {
    if let Err(error) = pollster::block_on(praxis::gpu::GpuContext::headless()) {
        assert!(error.is::<wgpu::RequestAdapterError>(), "{error:#}");
        // Not `eprintln!`, which libtest captures from passing tests.
        writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
        return;
    }
    let directory =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cook-render-{}", std::process::id()));
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/NormalTangentTest/NormalTangentTest.glb");
    let render = |path: &str| {
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        let assets = directory.join("assets");
        fs::create_dir_all(assets.join(path).parent().unwrap()).unwrap();
        fs::copy(&source, assets.join(path)).unwrap();
        let manifest = json!([{"path": path, "guid": format!("{NORMAL_TANGENT_TEST_GUID:032x}")}]);
        fs::write(assets.join("manifest.json"), manifest.to_string()).unwrap();
        let capture = std::process::Command::new(env!("CARGO_BIN_EXE_praxis"))
            .args(["capture", "normal-tangent", "capture.png", "160x90"])
            .current_dir(&directory)
            .output()
            .unwrap();
        assert!(
            capture.status.success(),
            "capturing from {path} failed: {}{}",
            String::from_utf8_lossy(&capture.stdout),
            String::from_utf8_lossy(&capture.stderr)
        );
        let mut ids: Vec<_> = fs::read_dir(directory.join("cooked/index"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        ids.sort();
        let pixels = image::open(directory.join("capture.png"))
            .unwrap()
            .into_rgba8();
        (ids, pixels)
    };

    let (ids, pixels) = render("NormalTangentTest/NormalTangentTest.glb");
    let (moved_ids, moved_pixels) = render("moved/renamed.glb");
    assert!(!ids.is_empty(), "the launch cooked nothing");
    assert_eq!(moved_ids, ids, "the moved source cooked other Asset IDs");
    assert!(
        moved_pixels == pixels,
        "the moved source renders differently"
    );
    fs::remove_dir_all(&directory).unwrap();
}
