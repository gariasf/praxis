use std::io::Write;

use praxis::ReferenceScene;
use praxis::gpu::GpuContext;

/// Each procedural Reference scene renders the same image twice, from a
/// working directory with no `assets/`, so no scene can read an asset file.
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
        let (first, _) =
            praxis::capture(&context, scene, glam::DVec3::ZERO, 160, 90).expect("first capture");
        let (second, _) =
            praxis::capture(&context, scene, glam::DVec3::ZERO, 160, 90).expect("second capture");
        assert!(
            first.as_raw() == second.as_raw(),
            "{} differs between two captures",
            scene.name()
        );
        assert!(
            first.pixels().any(|pixel| pixel != first.get_pixel(0, 0)),
            "{} is a single colour",
            scene.name()
        );
    }
    std::fs::remove_dir_all(&empty_directory).ok();
}
