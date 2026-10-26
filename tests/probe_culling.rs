use praxis::gpu::GpuContext;
use praxis::{ReferenceScene, RenderSwitches};

#[test]
fn probe_culling() {
    let context = pollster::block_on(GpuContext::headless()).unwrap();
    let frames = [59, 300, 500, 700, 900, 1200, 1500, 1800];
    let capture = |switches: RenderSwitches, probe: &str| {
        unsafe { std::env::set_var("PROBE_CULL", probe) };
        let images = praxis::capture_frames(&context, ReferenceScene::Town, glam::DVec3::ZERO, 640, 360, &frames, switches).unwrap();
        unsafe { std::env::remove_var("PROBE_CULL") };
        images
    };
    let unculled = capture(RenderSwitches { culling: false, ..Default::default() }, "");
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("golden-differences/probe");
    std::fs::create_dir_all(&dir).unwrap();
    let mut report = String::new();
    for (name, probe) in [("culled", ""), ("no-occlusion", "off"), ("uniform-level", "uniform")] {
        let images = capture(RenderSwitches::default(), probe);
        for (index, frame) in frames.iter().enumerate() {
            let (a, b) = (&images[index], &unculled[index]);
            let differing: Vec<usize> = (0..a.len() / 4)
                .filter(|&p| (0..3).any(|c| a[4 * p + c].abs_diff(b[4 * p + c]) > 1))
                .collect();
            report += &format!("{name} frame {frame}: {} px differ {:?}\n", differing.len(), differing.iter().take(5).map(|p| (p % 640, p / 640)).collect::<Vec<_>>());
            if !differing.is_empty() {
                image::save_buffer(dir.join(format!("{name}-{frame}.png")), a, 640, 360, image::ExtendedColorType::Rgba8).unwrap();
                image::save_buffer(dir.join(format!("unculled-{frame}.png")), b, 640, 360, image::ExtendedColorType::Rgba8).unwrap();
            }
        }
    }
    std::fs::write(dir.join("report.txt"), &report).unwrap();
    panic!("probe report:\n{report}");
}
