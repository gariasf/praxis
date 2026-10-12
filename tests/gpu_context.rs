use std::io::Write;

use praxis::gpu::GpuContext;

/// The planned features the capability report must name (M0 spec, #25).
/// Written out here, apart from the report's own list, so a feature the
/// report drops turns this test red.
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
    let lines: Vec<&str> = report.lines().collect();
    assert!(lines.len() <= 30, "{} lines:\n{report}", lines.len());

    for label in ["adapter", "backend"] {
        assert!(
            !value_of(&lines, label, &report).is_empty(),
            "`{label}` has no value:\n{report}"
        );
    }
    for feature in PLANNED_FEATURES {
        let verdict = value_of(&lines, feature, &report).split_whitespace().next();
        assert!(
            matches!(verdict, Some("yes" | "no")),
            "`{feature}` is not marked yes or no:\n{report}"
        );
    }
}

/// The text after `label` on the one line that starts with it.
fn value_of<'a>(lines: &[&'a str], label: &str, report: &str) -> &'a str {
    let values: Vec<&str> = lines
        .iter()
        .filter_map(|line| line.strip_prefix(label))
        .collect();
    assert_eq!(values.len(), 1, "expected one `{label}` line:\n{report}");
    values[0].trim()
}
