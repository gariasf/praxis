//! The visibility buffer Render switch, a prototype of option D of ADR
//! 0019: `town` at the eight frames the Goldens test checks, shaded by the
//! material pass from the depth passes' visibility IDs, matches the same
//! frames drawn by the lit pass within the Goldens' tolerance but at a few
//! isolated pixels per frame. With `PRAXIS_VISIBILITY_IMAGES=<directory>`,
//! the test writes each frame's capture with the switch off and on and their
//! difference image there. It skips where the adapter lacks
//! `PRIMITIVE_INDEX`.

use std::io::Write;
use std::path::Path;

use praxis::gpu::GpuContext;
use praxis::{ReferenceScene, RenderSwitches};

/// The Goldens' size.
const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// The Goldens' tolerance: the largest channel difference, in 8-bit steps,
/// that still matches.
const TOLERANCE: u8 = 1;

/// `town`'s frames the Goldens test checks, spread along its camera path.
const TOWN_FRAMES: [u32; 8] = [59, 300, 500, 700, 900, 1200, 1500, 1800];

fn gpu_context() -> Option<GpuContext> {
    match pollster::block_on(GpuContext::headless()) {
        Ok(context) => Some(context),
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            None
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    }
}

/// The most pixels of a frame, of 230,400, that may differ past the
/// tolerance: 0.3%. On the M4 Pro, 5 to 617 do, isolated pixels of
/// sub-pixel geometry, most in the frames that look down on the whole town.
const MOST_DIFFERING_PIXELS: usize = 700;

/// The pixels of `town` at `TOWN_FRAMES`, as `switches` set.
fn capture_frames(context: &GpuContext, switches: RenderSwitches) -> Vec<image::RgbaImage> {
    praxis::capture_frames(
        context,
        ReferenceScene::Town,
        glam::DVec3::ZERO,
        WIDTH,
        HEIGHT,
        &TOWN_FRAMES,
        switches,
    )
    .expect("capture failed")
    .into_iter()
    .map(|pixels| image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels).expect("WIDTH × HEIGHT pixels"))
    .collect()
}

/// The largest channel difference between the pixels at the same place.
fn channel_delta(a: &image::Rgba<u8>, b: &image::Rgba<u8>) -> u8 {
    a.0.iter()
        .zip(b.0)
        .map(|(&a, b)| a.abs_diff(b))
        .max()
        .unwrap_or(0)
}

/// The pixels of `a` and `b` that differ by more than `TOLERANCE`.
fn differing_pixels(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    a.pixels()
        .zip(b.pixels())
        .filter(|(a, b)| channel_delta(a, b) > TOLERANCE)
        .count()
}

/// `a` and `b`'s difference, each pixel's scaled up so a step shows, and red
/// where it passes the tolerance.
fn difference_image(a: &image::RgbaImage, b: &image::RgbaImage) -> image::RgbaImage {
    image::RgbaImage::from_fn(a.width(), a.height(), |x, y| {
        let delta = channel_delta(a.get_pixel(x, y), b.get_pixel(x, y));
        if delta > TOLERANCE {
            image::Rgba([255, 0, 0, 255])
        } else {
            let scaled = delta.saturating_mul(64);
            image::Rgba([scaled, scaled, scaled, 255])
        }
    })
}

#[test]
fn town_with_the_visibility_buffer_matches_town_without_it_at_eight_frames() {
    let Some(context) = gpu_context() else {
        return;
    };
    if !context.has_primitive_index() {
        writeln!(std::io::stderr(), "skipped: no PRIMITIVE_INDEX").ok();
        return;
    }
    let lit = capture_frames(&context, RenderSwitches::default());
    let shaded = capture_frames(
        &context,
        RenderSwitches {
            visibility_buffer: true,
            ..RenderSwitches::default()
        },
    );
    let images = std::env::var_os("PRAXIS_VISIBILITY_IMAGES").map(|directory| {
        let directory = Path::new(&directory).to_path_buf();
        std::fs::create_dir_all(&directory).expect("the images directory");
        directory
    });
    let mut counts = Vec::new();
    for ((frame, lit), shaded) in TOWN_FRAMES.iter().zip(&lit).zip(&shaded) {
        if let Some(directory) = &images {
            lit.save(directory.join(format!("town-{frame}-a.png")))
                .expect("saved");
            shaded
                .save(directory.join(format!("town-{frame}-d.png")))
                .expect("saved");
            difference_image(lit, shaded)
                .save(directory.join(format!("town-{frame}-difference.png")))
                .expect("saved");
        }
        counts.push(differing_pixels(lit, shaded));
    }
    writeln!(
        std::io::stderr(),
        "pixels past the tolerance per frame: {counts:?}"
    )
    .ok();
    assert!(
        counts.iter().all(|&count| count <= MOST_DIFFERING_PIXELS),
        "pixels past the tolerance per frame: {counts:?}"
    );
}
