//! The crowd's BLAS refit: on Metal, refitting each figure's BLAS to
//! the shape the GPU bends it to each frame casts the shadows that
//! rebuilding it each frame casts; other backends refuse the refit.

use std::io::Write;

use praxis::gpu::GpuContext;
use praxis::{CrowdForm, ReferenceScene, RenderSwitches};

/// The capture command's default size, as the Goldens'.
const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// Devices that capture the refit at once: a refit the TLAS build is not
/// ordered after shows only sometimes, and more often under load.
const DEVICES: usize = 4;

/// A headless GPU context, or None, after saying so, when there is no GPU.
fn gpu_context() -> Option<GpuContext> {
    match pollster::block_on(GpuContext::headless()) {
        Ok(context) => Some(context),
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            None
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    }
}

/// The last frame `praxis::capture` renders of the crowd in `form`.
fn capture(context: &GpuContext, form: CrowdForm) -> anyhow::Result<image::RgbaImage> {
    let switches = RenderSwitches {
        crowd_form: form,
        ..RenderSwitches::default()
    };
    let (pixels, _) = praxis::capture(
        context,
        ReferenceScene::Crowd,
        glam::DVec3::ZERO,
        WIDTH,
        HEIGHT,
        switches,
    )?;
    Ok(image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels).expect("WIDTH × HEIGHT pixels"))
}

#[test]
fn refitted_blases_cast_the_shadows_rebuilt_ones_cast() {
    let Some(context) = gpu_context() else {
        return;
    };
    if context.backend() != wgpu::Backend::Metal {
        let refused = capture(&context, CrowdForm::Refit).expect_err("refit off Metal");
        assert!(
            format!("{refused:#}").contains("unavailable"),
            "{refused:#}"
        );
        return;
    }
    let rebuilt = capture(&context, CrowdForm::Rebuild).expect("capture failed");
    let refitted: Vec<image::RgbaImage> = std::thread::scope(|scope| {
        let captures: Vec<_> = (0..DEVICES)
            .map(|_| {
                scope.spawn(|| {
                    let context = gpu_context().expect("a GPU, as the first context found");
                    capture(&context, CrowdForm::Refit).expect("capture failed")
                })
            })
            .collect();
        captures
            .into_iter()
            .map(|capture| capture.join().expect("capture thread panicked"))
            .collect()
    });
    for (device, refitted) in refitted.iter().enumerate() {
        let differing = rebuilt
            .pixels()
            .zip(refitted.pixels())
            .filter(|(rebuilt, refitted)| rebuilt != refitted)
            .count();
        assert_eq!(
            differing, 0,
            "device {device}: pixels differing from the rebuild"
        );
    }
}
