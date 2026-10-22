use std::io::Write;

/// The planned features #25 lists, and the Capability floor's features #93
/// found unprinted. Written out here, apart from the report's own list, so a
/// feature the report drops turns this test red.
const PLANNED_FEATURES: [&str; 8] = [
    "binding arrays",
    "mesh shaders",
    "ray query",
    "timestamp queries",
    "BC compression",
    "indirect first instance",
    "subgroups",
    "immediates",
];

#[test]
fn capability_report_names_adapter_backend_planned_features_and_floor_items() {
    let report = match pollster::block_on(praxis::gpu::capability_report()) {
        Ok(report) => report,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // libtest captures `eprintln!` from passing tests; write to the
            // stderr handle so the skip shows in a normal run.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("capability report failed: {error:#}"),
    };

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
    // The floor's limits and 4x MSAA per main-pass format, #93's other three.
    assert!(value_of(&report, "max buffer size").ends_with(" bytes"));
    assert!(value_of(&report, "acceleration structures").ends_with(" per stage"));
    for format in ["Depth32Float", "Rgba16Float"] {
        let msaa_x4 = value_of(&report, "4x MSAA");
        assert!(
            msaa_x4.contains(&format!("{format} yes")) || msaa_x4.contains(&format!("{format} no")),
            "4x MSAA not reported for {format}:\n{report}"
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
