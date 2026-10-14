//! The cook seam: a scratch source tree cooked into a scratch Cooked cache
//! and read back by Asset ID through the runtime's Blob reader.
#![cfg(feature = "cook")]

use std::fs;
use std::path::{Path, PathBuf};

use praxis::cook::cook;
use praxis::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, CookedCache, CookedMaterial, MaterialChannel,
};
use serde_json::json;

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

/// An empty directory for one test in one process.
fn scratch(test: &str) -> PathBuf {
    let directory =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cook-{test}-{}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    directory
}

fn material_id(name: AssetName) -> AssetId {
    AssetId::new(AssetSource::File(GUID), AssetKind::Material, name)
}

const PAINTED: AssetName = AssetName::Name("painted");
const UNNAMED: AssetName = AssetName::Index(1);

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
fn a_changed_source_re_cooks_exactly_its_assets() {
    let directory = scratch("change");
    let (assets, cache) = (directory.join("assets"), directory.join("cooked"));
    let cache = CookedCache::new(&cache);
    write_sources(&assets, "box.gltf", &gltf(0.75));
    cook(&assets, &cache).unwrap();

    write_sources(&assets, "box.gltf", &gltf(0.5));
    let mut cooked = cook(&assets, &cache).unwrap().cooked;
    cooked.sort();
    let mut expected = vec![material_id(PAINTED), material_id(UNNAMED)];
    expected.sort();
    assert_eq!(cooked, expected);
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
        cache.material(id).unwrap();
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
