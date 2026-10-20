use std::io::Write;

use praxis::ReferenceScene;
use praxis::gpu::GpuContext;

/// Each procedural Reference scene renders the same image twice, from a
/// working directory with no `assets/`, so no scene can read an asset file;
/// a scene whose source is absent there, with no Blob, fails naming it.
#[test]
fn procedural_scenes_render_identically_without_asset_files() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    // The only test in this binary, so changing the process's directory
    // races with nothing.
    let empty_directory =
        std::env::temp_dir().join(format!("praxis-reference-scenes-{}", std::process::id()));
    std::fs::create_dir_all(&empty_directory).expect("temporary directory");
    std::env::set_current_dir(&empty_directory).expect("change directory");

    for scene in [
        ReferenceScene::PbrSweep,
        ReferenceScene::UvGrid,
        ReferenceScene::Orrery,
    ] {
        let capture = || {
            praxis::capture(
                &context,
                scene,
                glam::DVec3::ZERO,
                160,
                90,
                praxis::RenderSwitches::default(),
            )
        };
        let (first, _) = capture().expect("first capture");
        let (second, _) = capture().expect("second capture");
        assert!(
            first == second,
            "{} differs between two captures",
            scene.name()
        );
        let pixels = first.as_chunks::<4>().0;
        assert!(
            pixels.iter().any(|pixel| pixel != &pixels[0]),
            "{} is a single colour",
            scene.name()
        );
    }
    // Each launch cooked its scene into the directory's Cooked cache.
    if cfg!(feature = "cook") {
        let index = std::fs::read_dir("cooked/index").expect("a launch cooked");
        assert!(index.count() > 0, "a launch cooked an empty index");
        let error = praxis::capture(
            &context,
            ReferenceScene::NormalTangent,
            glam::DVec3::ZERO,
            160,
            90,
            praxis::RenderSwitches::default(),
        )
        .expect_err("NormalTangentTest is absent");
        // Its manifest GUID: no manifest lists it here.
        let message = format!("{error:#}");
        assert!(
            message.contains("c750a32dc879478eaca4748dbc82645a"),
            "{message}"
        );
    }
    std::fs::remove_dir_all(&empty_directory).ok();
}
