//! Goldens: each procedural Reference scene's capture matches its committed
//! image in `tests/goldens/<backend>/<scene>.png`.
//!
//! `PRAXIS_UPDATE_GOLDENS=1 cargo test --test goldens` rewrites the current
//! backend's Goldens and prints the scenes whose image changed. Without the
//! variable the test only compares; on a mismatch it writes a diff image and
//! the capture under `target/tmp/golden-diffs/<backend>/`.

use std::io::Write;
use std::path::{Path, PathBuf};

use praxis::ReferenceScene;
use praxis::gpu::GpuContext;

const SCENES: [ReferenceScene; 2] = [ReferenceScene::PbrSweep, ReferenceScene::UvGrid];

/// The capture command's default size; each Golden is about 52 KB as PNG.
const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// The largest channel difference, in 8-bit steps, that still matches.
/// Measured noise on the M4 Pro (Metal): 20 captures per scene, each in its
/// own process, were byte-identical, so the noise is 0. One step above it
/// absorbs a driver rounding a value on the other side of .5; any larger
/// difference in any pixel fails.
const TOLERANCE: u8 = 1;

const UPDATE_VARIABLE: &str = "PRAXIS_UPDATE_GOLDENS";

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
fn diff_image(golden: &image::RgbaImage, actual: &image::RgbaImage) -> image::RgbaImage {
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
    let diff = diff_image(&golden, &actual);
    assert_eq!(diff.get_pixel(0, 0).0, [30, 30, 30, 255]);
    assert_eq!(diff.get_pixel(1, 0).0, [30, 30, 30, 255]);
    assert_eq!(diff.get_pixel(2, 0).0, [255, 0, 0, 255]);
}

#[test]
fn procedural_scenes_match_their_goldens() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };
    let backend = context.backend().to_str();
    let golden_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens")
        .join(backend);

    if std::env::var_os(UPDATE_VARIABLE).is_some_and(|value| value == "1") {
        update_goldens(&context, &golden_directory);
        return;
    }
    if !golden_directory.exists() {
        writeln!(
            std::io::stderr(),
            "skipped: no Goldens for backend {backend}; record them with \
             {UPDATE_VARIABLE}=1 cargo test --test goldens"
        )
        .ok();
        return;
    }

    let diff_directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("golden-diffs")
        .join(backend);
    let mut failures = Vec::new();
    for scene in SCENES {
        let golden_path = golden_path(&golden_directory, scene);
        let golden = image::open(&golden_path)
            .unwrap_or_else(|error| panic!("Golden {} unreadable: {error}", golden_path.display()))
            .into_rgba8();
        let (actual, _) = praxis::capture(&context, scene, WIDTH, HEIGHT).expect("capture failed");
        assert_eq!(
            golden.dimensions(),
            actual.dimensions(),
            "{} Golden size differs from the capture",
            scene.name()
        );

        let difference = compare(&golden, &actual);
        if difference.differing_pixels > 0 {
            std::fs::create_dir_all(&diff_directory).expect("diff directory");
            let diff_path = diff_directory.join(format!("{}-diff.png", scene.name()));
            diff_image(&golden, &actual)
                .save(&diff_path)
                .expect("write diff image");
            actual
                .save(diff_directory.join(format!("{}-actual.png", scene.name())))
                .expect("write capture");
            failures.push(format!(
                "{}: {} pixels differ by more than {TOLERANCE} (max channel delta {}); diff {}",
                scene.name(),
                difference.differing_pixels,
                difference.max_channel_delta,
                diff_path.display()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn golden_path(directory: &Path, scene: ReferenceScene) -> PathBuf {
    directory.join(format!("{}.png", scene.name()))
}

/// Writes each scene's capture as its Golden and prints the scenes whose
/// pixels changed, or that had no Golden.
fn update_goldens(context: &GpuContext, directory: &Path) {
    std::fs::create_dir_all(directory).expect("Golden directory");
    for scene in SCENES {
        let path = golden_path(directory, scene);
        let (actual, _) = praxis::capture(context, scene, WIDTH, HEIGHT).expect("capture failed");
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
