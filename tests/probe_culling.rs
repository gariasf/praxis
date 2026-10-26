use praxis::gpu::GpuContext;
use praxis::{ReferenceScene, RenderSwitches};

#[test]
fn probe_culling() {
    let context = pollster::block_on(GpuContext::headless()).unwrap();
    let capture = |culling: bool, probe: &str, cold: bool| {
        unsafe { std::env::set_var("PROBE_CULL", probe) };
        if cold {
            unsafe { std::env::set_var("PROBE_NO_PREDECESSOR", "1") };
        }
        let switches = RenderSwitches {
            culling,
            ..Default::default()
        };
        let mut images = praxis::capture_frames(
            &context,
            ReferenceScene::Town,
            glam::DVec3::ZERO,
            640,
            360,
            &[1200],
            switches,
        )
        .unwrap();
        unsafe { std::env::remove_var("PROBE_CULL") };
        unsafe { std::env::remove_var("PROBE_NO_PREDECESSOR") };
        images.remove(0)
    };
    let unculled = capture(false, "", false);
    let culled = capture(true, "", false);
    let variants = [
        ("unculled-again", capture(false, "", false), &unculled),
        ("unculled-cold", capture(false, "", true), &unculled),
        ("culled", culled.clone(), &unculled),
        ("culled-cold-vs-culled", capture(true, "", true), &culled),
        ("culled-cold", capture(true, "", true), &unculled),
        ("no-occlusion-cold", capture(true, "off", true), &unculled),
        ("frustum-margin", capture(true, "margin", false), &unculled),
        (
            "frustum-margin-no-occlusion",
            capture(true, "margin-off", false),
            &unculled,
        ),
    ];
    let mut report = String::new();
    for (name, a, b) in &variants {
        let differing: Vec<(usize, usize)> = (0..a.len() / 4)
            .filter(|&p| (0..3).any(|c| a[4 * p + c].abs_diff(b[4 * p + c]) > 1))
            .map(|p| (p % 640, p / 640))
            .collect();
        report += &format!(
            "{name}: {} px differ {:?}\n",
            differing.len(),
            &differing[..differing.len().min(8)]
        );
    }
    panic!("probe report:\n{report}");
}
