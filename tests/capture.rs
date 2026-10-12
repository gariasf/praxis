use std::io::Write;

use praxis::gpu::GpuContext;

/// The clear colour, linear (0.1, 0.2, 0.3), as an 8-bit sRGB target stores
/// it: 255 × (1.055 × c^(1/2.4) − 0.055) gives 89.0, 123.6 and 148.9. The GPU
/// may round 123.6 either way, hence the tolerance of 1.
const BACKGROUND: [u8; 4] = [89, 124, 149, 255];

#[test]
fn capture_draws_the_scene_over_the_srgb_encoded_background() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    let image = praxis::capture(&context, 160, 90).expect("capture failed");

    assert_eq!(image.dimensions(), (160, 90));
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
