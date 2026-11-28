use std::io::Write;

use praxis::ReferenceScene;
use praxis::gpu::GpuContext;

/// The clear colour, exposed (0.1, 0.2, 0.3), as GT7 shows it: Polyphony's
/// sample implementation, between Rec. 709 and Rec. 2020, gives (0.0967,
/// 0.1993, 0.2988), which sRGB encodes as 87.6, 123.4 and 148.6. The
/// tolerance of 1 absorbs the dither's step.
const BACKGROUND: [u8; 4] = [88, 123, 149, 255];

#[test]
fn capture_draws_the_scene_over_the_tonemapped_background() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    // A fetched file, so absent from a fresh clone and from CI.
    let model = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(praxis::DEFAULT_MODEL);
    if !model.exists() {
        writeln!(
            std::io::stderr(),
            "skipped: {} is not fetched; run `praxis fetch`",
            praxis::DEFAULT_MODEL
        )
        .ok();
        return;
    }

    let (pixels, _) = praxis::capture(
        &context,
        ReferenceScene::Helmet,
        glam::DVec3::ZERO,
        160,
        90,
        praxis::RenderSwitches::default(),
    )
    .expect("capture failed");

    assert_eq!(pixels.len(), 160 * 90 * 4);
    let image = image::RgbaImage::from_raw(160, 90, pixels).expect("160 × 90 pixels");
    let corner = image.get_pixel(0, 0).0;
    assert!(
        corner
            .iter()
            .zip(BACKGROUND)
            .all(|(&channel, expected)| channel.abs_diff(expected) <= 1),
        "background is {corner:?}, expected {BACKGROUND:?} within 1"
    );
    // The camera looks at the helmet, so the centre is not background.
    assert_ne!(
        image.get_pixel(80, 45).0,
        corner,
        "nothing drawn at the centre"
    );
}

/// `praxis capture --frame` lands on the frame it names: `town`'s camera
/// moves along its path, so its first frame and its Golden's differ.
#[cfg(feature = "cook")]
#[test]
fn capture_command_renders_the_frame_it_names() {
    if let Err(error) = pollster::block_on(GpuContext::headless()) {
        assert!(error.is::<wgpu::RequestAdapterError>(), "{error:#}");
        writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
        return;
    }
    let capture = |frame: u32| {
        let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "capture-town-frame-{frame}-{}.png",
            std::process::id()
        ));
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_praxis"))
            .args(["capture", "town"])
            .arg(&path)
            .args(["64x36", "--frame", &frame.to_string()])
            .output()
            .expect("praxis runs");
        assert!(
            output.status.success(),
            "capturing frame {frame} failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let pixels = image::open(&path).expect("capture written").into_rgba8();
        std::fs::remove_file(&path).ok();
        pixels
    };

    let first = capture(0);
    let golden_frame = capture(59);
    assert_eq!(first.dimensions(), (64, 36));
    assert!(first != golden_frame, "frames 0 and 59 render alike");
}
