use std::io::Write;

use praxis::ReferenceScene;
use praxis::gpu::GpuContext;

/// The clear colour, exposed (0.1, 0.2, 0.3), as GT7 shows it: Polyphony's
/// sample implementation, between Rec. 709 and Rec. 2020, gives (0.0967,
/// 0.1993, 0.2988), which an 8-bit sRGB target stores as 87.6, 123.4 and
/// 148.6. The tolerance of 1 absorbs a GPU's rounding.
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
            "skipped: {} is not fetched; run scripts/fetch-assets.sh",
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
