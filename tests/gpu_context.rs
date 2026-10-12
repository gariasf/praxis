use std::io::Write;

use praxis::gpu::GpuContext;

/// The planned features #25 lists. Written out here, apart from the
/// report's own list, so a feature the report drops turns this test red.
const PLANNED_FEATURES: [&str; 7] = [
    "binding arrays",
    "mesh shaders",
    "ray query",
    "timestamp queries",
    "BC compression",
    "subgroups",
    "immediates",
];

#[test]
fn headless_context_reports_adapter_backend_and_planned_features() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // libtest captures `eprintln!` from passing tests; write to the
            // stderr handle so the skip shows in a normal run.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    let report = context.capability_report();
    assert!(report.lines().count() <= 30, "over 30 lines:\n{report}");

    // The adapter's name is machine-specific: its line only has to exist.
    value_of(&report, "adapter");
    // The desktop members of `wgpu::Backends::PRIMARY` (ADR 0010).
    assert!(
        matches!(value_of(&report, "backend"), "vulkan" | "metal" | "dx12"),
        "not a desktop backend:\n{report}"
    );
    for feature in PLANNED_FEATURES {
        let verdict = value_of(&report, feature).split_whitespace().next();
        assert!(
            matches!(verdict, Some("yes" | "no")),
            "`{feature}` is not marked yes or no:\n{report}"
        );
    }
}

/// The text after `label` on the report's one line that starts with it.
fn value_of<'a>(report: &'a str, label: &str) -> &'a str {
    let values: Vec<&str> = report
        .lines()
        .filter_map(|line| line.strip_prefix(label))
        .collect();
    assert_eq!(values.len(), 1, "expected one `{label}` line:\n{report}");
    values[0].trim()
}
