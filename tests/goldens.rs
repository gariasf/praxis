//! Goldens: each Reference scene CI can render, procedural or from a
//! committed asset, matches its committed image in
//! `tests/goldens/<key>/<scene>.png`. The key is the backend, plus `-cpu` on
//! a software driver: `metal` is recorded on the M4 Pro, and
//! `vulkan-cpu` on CI's lavapipe (the Mesa pinned in `.github/workflows/ci.yml`).
//! Each scene shifted by 100 km must also match its origin Golden, and each
//! with culling off must match its capture with culling on, so culling
//! removes no visible Instance, as each lit with every light at every pixel
//! must match it lit through Froxel lists, both with shadow rays off, so no
//! list misses a light that reaches its Froxel. `town` takes both checks at
//! eight frames along its camera path, the others at the Golden's frame.
//! The helmet scene has no Golden, since its source is fetched, and takes
//! those two checks where it is fetched.
//! A Golden is a reviewed capture from `praxis::capture`, so it catches
//! regressions, not errors already present when it was recorded.
//!
//! `PRAXIS_UPDATE_GOLDENS=1 cargo test --test goldens` rewrites the current
//! key's Goldens and prints the scenes whose image changed. Without the
//! variable the test only compares; on a mismatch it writes a difference
//! image and the capture under `target/tmp/golden-differences/<key>/`, and a
//! scene with no Golden yet writes its capture there. With
//! `PRAXIS_REQUIRE_GOLDENS=1`, as on CI, a missing adapter or Golden directory
//! fails instead of skipping.
//!
//! `README.md` shows each scene's Metal Golden by linking it, and a test here
//! fails when its links and the scenes checked disagree.

use std::io::Write;
use std::path::{Path, PathBuf};

use praxis::gpu::GpuContext;
use praxis::{ReferenceScene, RenderSwitches};

const SCENES: [ReferenceScene; 9] = [
    ReferenceScene::PbrSweep,
    ReferenceScene::UvGrid,
    ReferenceScene::NormalTangent,
    ReferenceScene::FarCoplanar,
    ReferenceScene::Orrery,
    ReferenceScene::Minification,
    ReferenceScene::TextureWall,
    ReferenceScene::Shadows,
    ReferenceScene::Town,
];

/// The capture command's default size.
const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// The largest channel difference, in 8-bit steps, that still matches.
/// Measured noise on the M4 Pro (Metal) and on CI's lavapipe: 20 captures per
/// scene, each in its own process, were byte-identical, so the noise is 0.
/// One step above it absorbs a driver rounding a value on the other side of
/// .5; any larger difference in any pixel fails.
const TOLERANCE: u8 = 1;

/// The 100 km check (ADR 0014): each scene, shifted with its camera by this
/// much, matches its origin Golden, so no World position reaches the GPU in
/// f32. The shift is whole 64 m anchor cells (1563 of them), so the shifted
/// scene keeps its offsets and composes bit for bit as at the origin; a
/// shift within a cell rounds offsets differently, which the shader-function
/// oracle bounds instead.
const FAR_SHIFT: glam::DVec3 = glam::DVec3::new(100_032.0, 0.0, 100_032.0);

/// Pixels per frame that may differ between culling on and off on a
/// software driver. lavapipe resolves one sample of `town`'s frame 1200
/// differently depending on which triangles the frame's draw holds: drawing
/// more Instances than culling does can bring the difference back, which a
/// missing Instance could not. Metal, the reference, matches exactly.
const SOFTWARE_UNCULLED_PIXELS: usize = 1;

const UPDATE_GOLDENS_VARIABLE: &str = "PRAXIS_UPDATE_GOLDENS";
const REQUIRE_GOLDENS_VARIABLE: &str = "PRAXIS_REQUIRE_GOLDENS";

fn variable_is_set(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| value == "1")
}

/// Skips the test, or fails it when `PRAXIS_REQUIRE_GOLDENS=1`, so a CI
/// runner that lost its driver cannot pass by comparing nothing.
fn skip(reason: &str) {
    assert!(
        !variable_is_set(REQUIRE_GOLDENS_VARIABLE),
        "{reason}, and {REQUIRE_GOLDENS_VARIABLE}=1"
    );
    // Not `eprintln!`, which libtest captures from passing tests.
    writeln!(std::io::stderr(), "skipped: {reason}").ok();
}

/// The Golden directory for the context's adapter. A software rasterizer
/// rounds differently from hardware on the same API, so it gets its own key.
fn golden_key(context: &GpuContext) -> String {
    let backend = context.backend().to_str();
    match context.device_type() {
        wgpu::DeviceType::Cpu => format!("{backend}-cpu"),
        _ => backend.to_owned(),
    }
}

#[derive(Debug, PartialEq)]
struct Difference {
    /// Pixels with a channel that differs by more than `TOLERANCE`.
    differing_pixels: usize,
    max_channel_delta: u8,
}

/// Compares two images of equal size channel by channel.
fn compare(golden: &image::RgbaImage, actual: &image::RgbaImage) -> Difference {
    let mut difference = Difference {
        differing_pixels: 0,
        max_channel_delta: 0,
    };
    for (golden_pixel, actual_pixel) in golden.pixels().zip(actual.pixels()) {
        let pixel_delta = channel_delta(golden_pixel, actual_pixel);
        difference.max_channel_delta = difference.max_channel_delta.max(pixel_delta);
        if pixel_delta > TOLERANCE {
            difference.differing_pixels += 1;
        }
    }
    difference
}

fn channel_delta(golden: &image::Rgba<u8>, actual: &image::Rgba<u8>) -> u8 {
    golden
        .0
        .iter()
        .zip(actual.0)
        .map(|(&golden_channel, actual_channel)| golden_channel.abs_diff(actual_channel))
        .max()
        .unwrap_or(0)
}

/// The capture dimmed to a third, with every pixel beyond `TOLERANCE` in
/// opaque red.
fn difference_image(golden: &image::RgbaImage, actual: &image::RgbaImage) -> image::RgbaImage {
    image::RgbaImage::from_fn(actual.width(), actual.height(), |x, y| {
        let actual_pixel = actual.get_pixel(x, y);
        if channel_delta(golden.get_pixel(x, y), actual_pixel) > TOLERANCE {
            image::Rgba([255, 0, 0, 255])
        } else {
            let [red, green, blue, _] = actual_pixel.0;
            image::Rgba([red / 3, green / 3, blue / 3, 255])
        }
    })
}

#[test]
fn compare_counts_pixels_beyond_tolerance_and_marks_them_red() {
    let golden = image::RgbaImage::from_pixel(3, 1, image::Rgba([90, 90, 90, 255]));
    let mut actual = golden.clone();
    actual.put_pixel(1, 0, image::Rgba([90 + TOLERANCE, 90, 90, 255]));
    actual.put_pixel(2, 0, image::Rgba([90, 70, 90, 255]));

    assert_eq!(
        compare(&golden, &actual),
        Difference {
            differing_pixels: 1,
            max_channel_delta: 20,
        }
    );
    let difference_pixels = difference_image(&golden, &actual);
    assert_eq!(difference_pixels.get_pixel(0, 0).0, [30, 30, 30, 255]);
    assert_eq!(difference_pixels.get_pixel(1, 0).0, [30, 30, 30, 255]);
    assert_eq!(difference_pixels.get_pixel(2, 0).0, [255, 0, 0, 255]);
}

#[test]
fn reference_scenes_match_their_goldens() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            skip(&format!("no GPU adapter: {error}"));
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };
    let key = golden_key(&context);
    let software = context.device_type() == wgpu::DeviceType::Cpu;
    let unculled_pixels_allowed = if software {
        SOFTWARE_UNCULLED_PIXELS
    } else {
        0
    };
    let golden_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens")
        .join(&key);

    if variable_is_set(UPDATE_GOLDENS_VARIABLE) {
        update_goldens(&context, &golden_directory);
        return;
    }
    if !golden_directory.exists() {
        skip(&format!(
            "no Goldens for {key}; record them with \
             {UPDATE_GOLDENS_VARIABLE}=1 cargo test --test goldens"
        ));
        return;
    }

    let difference_directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("golden-differences")
        .join(&key);
    let mut failures = Vec::new();
    let helmet_is_fetched = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(praxis::DEFAULT_MODEL)
        .exists();
    if !helmet_is_fetched {
        writeln!(std::io::stderr(), "skipped helmet: assets not fetched").ok();
    }
    let helmet = helmet_is_fetched.then_some(ReferenceScene::Helmet);
    for scene in SCENES.into_iter().chain(helmet) {
        let golden_path = golden_path(&golden_directory, scene);
        let frames = equivalence_frames(scene);
        let culled = capture_frames(&context, scene, frames, RenderSwitches::default());
        let unculled = capture_frames(
            &context,
            scene,
            frames,
            RenderSwitches {
                culling: false,
                ..RenderSwitches::default()
            },
        );
        // With shadow rays off, as M3 judges the Froxel lists.
        let unshadowed = RenderSwitches {
            shadow_rays: false,
            ..RenderSwitches::default()
        };
        let froxel_lit = capture_frames(&context, scene, frames, unshadowed);
        let brute_force = capture_frames(
            &context,
            scene,
            frames,
            RenderSwitches {
                froxel_lights: false,
                ..unshadowed
            },
        );
        for (index, frame) in frames.iter().enumerate() {
            let suffix = if frames.len() == 1 {
                String::new()
            } else {
                format!("-frame-{frame}")
            };
            failures.extend(mismatch(
                &culled[index],
                &unculled[index],
                &format!("{}-unculled{suffix}", scene.name()),
                unculled_pixels_allowed,
                &difference_directory,
            ));
            failures.extend(mismatch(
                &froxel_lit[index],
                &brute_force[index],
                &format!("{}-brute-force{suffix}", scene.name()),
                0,
                &difference_directory,
            ));
        }
        let culled = &culled[frames
            .iter()
            .position(|&frame| frame == GOLDEN_FRAME)
            .expect("the frames checked hold the Golden's")];
        if scene == ReferenceScene::Helmet {
            continue;
        }
        if !golden_path.exists() {
            // Saved so a Golden for a machine one cannot run locally (CI's
            // lavapipe) can be taken from the job's artifact.
            std::fs::create_dir_all(&difference_directory).expect("difference directory");
            let actual_path = difference_directory.join(format!("{}-actual.png", scene.name()));
            culled.save(&actual_path).expect("write capture");
            failures.push(format!(
                "{}: no Golden at {}; capture {}",
                scene.name(),
                golden_path.display(),
                actual_path.display()
            ));
            continue;
        }
        let golden = image::open(&golden_path)
            .unwrap_or_else(|error| panic!("Golden {} unreadable: {error}", golden_path.display()))
            .into_rgba8();
        failures.extend(mismatch(
            &golden,
            culled,
            scene.name(),
            0,
            &difference_directory,
        ));

        let shifted = capture(&context, scene, FAR_SHIFT, RenderSwitches::default());
        failures.extend(mismatch(
            &golden,
            &shifted,
            &format!("{}-100km", scene.name()),
            0,
            &difference_directory,
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Compares `actual` with `golden`; on more than `allowed_pixels` differing,
/// writes the difference image and the capture under `directory` as
/// `<label>-difference.png` and `<label>-actual.png`, and describes it.
fn mismatch(
    golden: &image::RgbaImage,
    actual: &image::RgbaImage,
    label: &str,
    allowed_pixels: usize,
    directory: &Path,
) -> Option<String> {
    assert_eq!(
        golden.dimensions(),
        actual.dimensions(),
        "{label} Golden size differs from the capture"
    );
    let difference = compare(golden, actual);
    if difference.differing_pixels <= allowed_pixels {
        return None;
    }
    std::fs::create_dir_all(directory).expect("difference directory");
    let difference_path = directory.join(format!("{label}-difference.png"));
    difference_image(golden, actual)
        .save(&difference_path)
        .expect("write difference image");
    actual
        .save(directory.join(format!("{label}-actual.png")))
        .expect("write capture");
    Some(format!(
        "{label}: {} pixels differ by more than {TOLERANCE} (max channel delta {}); difference image {}",
        difference.differing_pixels,
        difference.max_channel_delta,
        difference_path.display()
    ))
}

/// The last frame `praxis::capture` renders of `scene`, moved by `shift`, at
/// the Goldens' size, as `switches` set.
fn capture(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    switches: RenderSwitches,
) -> image::RgbaImage {
    let (pixels, _) =
        praxis::capture(context, scene, shift, WIDTH, HEIGHT, switches).expect("capture failed");
    image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels).expect("WIDTH × HEIGHT pixels")
}

/// The frame a Golden is: `praxis::capture`'s last.
const GOLDEN_FRAME: u32 = 59;

/// `town`'s frames whose culling and Froxel lists the test checks, spread
/// along its camera path: two on the street, two on the rise over the
/// rooftops, four round the circle.
const TOWN_FRAMES: [u32; 8] = [GOLDEN_FRAME, 300, 500, 700, 900, 1200, 1500, 1800];

/// The frames of `scene` whose culling and Froxel lists the test checks,
/// the Golden's among them.
fn equivalence_frames(scene: ReferenceScene) -> &'static [u32] {
    match scene {
        ReferenceScene::Town => &TOWN_FRAMES,
        _ => &[GOLDEN_FRAME],
    }
}

/// The pixels `praxis::capture_frames` renders of `scene` at `frames`, at
/// the Goldens' size, as `switches` set.
fn capture_frames(
    context: &GpuContext,
    scene: ReferenceScene,
    frames: &[u32],
    switches: RenderSwitches,
) -> Vec<image::RgbaImage> {
    praxis::capture_frames(
        context,
        scene,
        glam::DVec3::ZERO,
        WIDTH,
        HEIGHT,
        frames,
        switches,
    )
    .expect("capture failed")
    .into_iter()
    .map(|pixels| image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels).expect("WIDTH × HEIGHT pixels"))
    .collect()
}

fn golden_path(directory: &Path, scene: ReferenceScene) -> PathBuf {
    directory.join(format!("{}.png", scene.name()))
}

/// The README's link to a scene's Golden, relative to the repository root.
const README_GOLDEN_PREFIX: &str = "tests/goldens/metal/";

/// What disagrees between `readme` and the scenes this test checks: a scene
/// the README does not show, or shows more than once, a Golden it shows of a
/// scene not checked, and a linked Golden absent from `root`. A link counts
/// only as the prefix, a scene name and `.png`.
fn readme_disagreements(readme: &str, root: &Path) -> Vec<String> {
    let linked: Vec<&str> = readme
        .split(README_GOLDEN_PREFIX)
        .skip(1)
        .filter_map(|after_prefix| {
            let name_length = after_prefix
                .find(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
                .unwrap_or(after_prefix.len());
            let (scene, rest) = after_prefix.split_at(name_length);
            (!scene.is_empty() && rest.starts_with(".png")).then_some(scene)
        })
        .collect();
    let checked: Vec<&str> = SCENES.iter().map(|scene| scene.name()).collect();
    let mut disagreements = Vec::new();
    for scene in &checked {
        match linked
            .iter()
            .filter(|linked_scene| *linked_scene == scene)
            .count()
        {
            0 => disagreements.push(format!("README.md does not show the Golden of {scene}")),
            1 => {}
            _ => disagreements.push(format!("README.md shows {scene} more than once")),
        }
    }
    for scene in &linked {
        if !checked.contains(scene) {
            disagreements.push(format!(
                "README.md shows {scene}, which the goldens test does not check"
            ));
        }
        let path = root.join(format!("{README_GOLDEN_PREFIX}{scene}.png"));
        if !path.exists() {
            disagreements.push(format!(
                "README.md links {}, which is absent",
                path.display()
            ));
        }
    }
    disagreements
}

#[test]
fn readme_shows_each_scene_golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README.md");
    let disagreements = readme_disagreements(&readme, root);
    assert!(disagreements.is_empty(), "{}", disagreements.join("\n"));
}

#[test]
fn readme_disagreements_name_missing_repeated_unchecked_and_absent_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README.md");
    let town_link = format!("{README_GOLDEN_PREFIX}town.png");
    let without_town: String = readme
        .lines()
        .filter(|line| !line.contains(&town_link))
        .map(|line| format!("{line}\n"))
        .collect();
    let with_crowd = format!("{readme}\n![crowd]({README_GOLDEN_PREFIX}crowd.png)\n");
    let with_town_twice = format!("{readme}\n![town]({README_GOLDEN_PREFIX}town.png)\n");

    assert_eq!(
        readme_disagreements(&without_town, root),
        ["README.md does not show the Golden of town"]
    );
    assert_eq!(
        readme_disagreements(&with_town_twice, root),
        ["README.md shows town more than once"]
    );
    assert_eq!(
        readme_disagreements(&with_crowd, root),
        [
            "README.md shows crowd, which the goldens test does not check".to_owned(),
            format!(
                "README.md links {}, which is absent",
                root.join(format!("{README_GOLDEN_PREFIX}crowd.png"))
                    .display()
            ),
        ]
    );
}

/// Writes each scene's capture as its Golden and prints the scenes whose
/// pixels changed, or that had no Golden.
fn update_goldens(context: &GpuContext, directory: &Path) {
    std::fs::create_dir_all(directory).expect("Golden directory");
    for scene in SCENES {
        let path = golden_path(directory, scene);
        let actual = capture(context, scene, glam::DVec3::ZERO, RenderSwitches::default());
        let unchanged =
            image::open(&path).is_ok_and(|golden| golden.into_rgba8().as_raw() == actual.as_raw());
        if unchanged {
            continue;
        }
        actual.save(&path).expect("write Golden");
        writeln!(
            std::io::stderr(),
            "updated Golden {}: {}",
            scene.name(),
            path.display()
        )
        .ok();
    }
}
