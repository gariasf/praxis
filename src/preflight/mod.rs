//! `praxis preflight`: a session's starting state, one line per check, so
//! the session reads it rather than working it out. A check that would make
//! `scripts/verify.sh` fail is a `FAIL`, with the command that fixes it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use std::process::Command;

use crate::bench::BASELINE_DIRECTORY;
use crate::capture::{GOLDEN_DIRECTORY, golden_key};

/// GPU utilisation, in percent, above which a bench's times would be
/// someone else's.
const BUSY_GPU_PERCENT: u32 = 10;

/// Names listed per line before the rest are counted.
const LISTED_NAMES: usize = 4;

/// The machines whose results the repository records or compares against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Machine {
    /// The Apple M4 Pro on Metal: ADR 0013's reference, whose Baselines
    /// the Budgets are set against.
    M4Pro,
    /// Mesa's lavapipe on Vulkan, as CI's jobs run it.
    Lavapipe,
    /// The owner's NVIDIA RTX 2070 SUPER, a comparison adapter only.
    RtxComparison,
    Unknown,
}

impl Machine {
    /// Which machine an adapter is, from what it reports of itself.
    pub fn classify(name: &str, backend: wgpu::Backend, device_type: wgpu::DeviceType) -> Self {
        match (backend, device_type) {
            (wgpu::Backend::Metal, _) if name == "Apple M4 Pro" => Self::M4Pro,
            (wgpu::Backend::Vulkan, wgpu::DeviceType::Cpu) if name.starts_with("llvmpipe") => {
                Self::Lavapipe
            }
            (_, wgpu::DeviceType::DiscreteGpu) if name == "NVIDIA GeForce RTX 2070 SUPER" => {
                Self::RtxComparison
            }
            _ => Self::Unknown,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::M4Pro => "the M4 Pro, the reference machine",
            Self::Lavapipe => "lavapipe, as on CI",
            Self::RtxComparison => "the RTX comparison adapter",
            Self::Unknown => "no known machine",
        }
    }
}

/// Whether a check passed, and whether its result blocks `verify`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verdict {
    Ok,
    /// Information, such as work in flight, that passes or fails nothing.
    Note,
    /// Worth acting on before a bench, but `verify` still passes.
    Warn,
    /// `scripts/verify.sh` fails until it is fixed.
    Fail,
}

/// One line of the preflight: what was checked and what was found.
#[derive(Debug)]
pub struct Check {
    pub name: &'static str,
    pub verdict: Verdict,
    pub detail: String,
}

impl Check {
    fn new(name: &'static str, verdict: Verdict, detail: impl Into<String>) -> Self {
        Self {
            name,
            verdict,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Check {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let verdict = match self.verdict {
            Verdict::Ok => "ok",
            Verdict::Note => "note",
            Verdict::Warn => "warn",
            Verdict::Fail => "FAIL",
        };
        write!(formatter, "{verdict:<5} {:<9} {}", self.name, self.detail)
    }
}

/// Every check, run from the repository root.
pub fn run() -> Vec<Check> {
    let mut checks = vec![gpu_load(), other_processes()];
    match pollster::block_on(crate::gpu::adapter_info()) {
        Ok(info) => {
            let machine = Machine::classify(&info.name, info.backend, info.device_type);
            checks.push(Check::new(
                "adapter",
                Verdict::Ok,
                format!(
                    "{}, {}, {} {}: {}",
                    info.name,
                    info.backend.to_str(),
                    info.driver,
                    info.driver_info,
                    machine.description()
                ),
            ));
            checks.push(goldens(
                &golden_key(info.backend, info.device_type),
                machine,
            ));
            checks.push(baselines(&info.name, machine));
        }
        Err(error) => checks.push(Check::new(
            "adapter",
            Verdict::Fail,
            format!("{error:#}; `praxis capabilities` diagnoses it"),
        )),
    }
    checks.push(assets());
    checks.extend(work_in_flight());
    checks
}

/// The file stems with `extension` in each subdirectory of `root`, by the
/// subdirectory's name. An absent `root` has none.
fn stems_by_directory(root: &Path, extension: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut stems = BTreeMap::new();
    let Ok(directories) = std::fs::read_dir(root) else {
        return stems;
    };
    for directory in directories.flatten() {
        let Ok(files) = std::fs::read_dir(directory.path()) else {
            continue;
        };
        let names = files
            .flatten()
            .map(|file| file.path())
            .filter(|path| path.extension().is_some_and(|found| found == extension))
            .filter_map(|path| Some(path.file_stem()?.to_str()?.to_owned()))
            .collect();
        stems.insert(directory.file_name().to_string_lossy().into_owned(), names);
    }
    stems
}

/// What `key`'s directory lacks of what any directory holds: a scene with a
/// Golden or Baseline on one adapter is checked on every adapter.
pub fn missing(stems_by_directory: &BTreeMap<String, BTreeSet<String>>, key: &str) -> Vec<String> {
    let empty = BTreeSet::new();
    let present = stems_by_directory.get(key).unwrap_or(&empty);
    let expected: BTreeSet<_> = stems_by_directory.values().flatten().collect();
    expected
        .into_iter()
        .filter(|stem| !present.contains(*stem))
        .cloned()
        .collect()
}

fn goldens(key: &str, machine: Machine) -> Check {
    let directory = format!("{GOLDEN_DIRECTORY}/{key}");
    let stems = stems_by_directory(Path::new(GOLDEN_DIRECTORY), "png");
    let absent = missing(&stems, key);
    if absent.is_empty() {
        return Check::new(
            "goldens",
            Verdict::Ok,
            format!("{directory}: {}", stems.get(key).map_or(0, BTreeSet::len)),
        );
    }
    let fix = match machine {
        Machine::Lavapipe => "take them from CI's golden-differences artifact",
        _ => {
            "record them with `PRAXIS_UPDATE_GOLDENS=1 cargo test --test goldens` for the owner to judge"
        }
    };
    Check::new(
        "goldens",
        Verdict::Fail,
        format!("{directory} lacks {}; {fix}", listed(&absent)),
    )
}

fn baselines(adapter: &str, machine: Machine) -> Check {
    let key = crate::bench::baseline_directory(adapter)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let directory = format!("{BASELINE_DIRECTORY}/{key}");
    let stems = stems_by_directory(Path::new(BASELINE_DIRECTORY), "json");
    let absent = missing(&stems, &key);
    if absent.is_empty() {
        return Check::new(
            "baselines",
            Verdict::Ok,
            format!("{directory}: {}", stems.get(&key).map_or(0, BTreeSet::len)),
        );
    }
    // With no scene named, bench records every scene whose assets are present.
    let scenes = if absent.len() <= LISTED_NAMES {
        format!(" {}", absent.join(" "))
    } else {
        String::new()
    };
    let fix = match machine {
        Machine::Lavapipe => "copy them from CI's bench-results artifact".to_owned(),
        _ => format!("on a quiet machine, `praxis bench {directory}{scenes}`"),
    };
    Check::new(
        "baselines",
        Verdict::Fail,
        format!("{directory} lacks {}; {fix}", listed(&absent)),
    )
}

/// The fetched sources, which `verify` benches the helmet from, and the
/// Cooked cache, which a launch fills with what it loads.
#[cfg(feature = "cook")]
fn assets() -> Check {
    let (absent, fetched) =
        match crate::cook::absent_fetched(Path::new(crate::cook::ASSETS_DIRECTORY)) {
            Ok(found) => found,
            Err(error) => return Check::new("assets", Verdict::Fail, format!("{error:#}")),
        };
    if !absent.is_empty() {
        return Check::new(
            "assets",
            Verdict::Fail,
            format!(
                "{} of {fetched} fetched files absent; run `praxis fetch`",
                absent.len()
            ),
        );
    }
    let index = Path::new(crate::cooked::COOKED_CACHE_DIRECTORY).join("index");
    let cooked = std::fs::read_dir(index).map_or(0, |entries| entries.count());
    if cooked == 0 {
        return Check::new(
            "assets",
            Verdict::Warn,
            format!(
                "{fetched} fetched; Cooked cache empty, so each launch cooks first: run `praxis cook`"
            ),
        );
    }
    Check::new(
        "assets",
        Verdict::Ok,
        format!("{fetched} fetched; {cooked} Assets cooked"),
    )
}

#[cfg(not(feature = "cook"))]
fn assets() -> Check {
    Check::new("assets", Verdict::Note, "not checked: built without `cook`")
}

/// The output of `program` with `arguments`, when it runs and succeeds.
fn output(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn gpu_load() -> Check {
    let load = if cfg!(target_os = "macos") {
        output(
            "ioreg",
            &["-r", "-d", "1", "-w", "0", "-c", "IOAccelerator"],
        )
        .and_then(|report| parse_ioreg_utilization(&report))
    } else {
        output(
            "nvidia-smi",
            &[
                "--query-gpu=utilization.gpu",
                "--format=csv,noheader,nounits",
            ],
        )
        .and_then(|report| parse_nvidia_smi_utilization(&report))
    };
    match load {
        Some(percent) if percent > BUSY_GPU_PERCENT => Check::new(
            "gpu load",
            Verdict::Warn,
            format!("{percent}%: bench only once it is quiet"),
        ),
        Some(percent) => Check::new("gpu load", Verdict::Ok, format!("{percent}%")),
        None => Check::new(
            "gpu load",
            Verdict::Note,
            "unknown: no nvidia-smi or ioreg reading",
        ),
    }
}

/// The busiest GPU's utilisation from `nvidia-smi
/// --query-gpu=utilization.gpu --format=csv,noheader,nounits`: one
/// percentage per line, one line per GPU.
pub fn parse_nvidia_smi_utilization(report: &str) -> Option<u32> {
    report
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .max()
}

/// The busiest GPU's utilisation from `ioreg -r -d 1 -w 0 -c IOAccelerator`,
/// whose performance statistics hold `"Device Utilization %"=<percent>`.
pub fn parse_ioreg_utilization(report: &str) -> Option<u32> {
    const KEY: &str = "\"Device Utilization %\"=";
    report
        .match_indices(KEY)
        .filter_map(|(start, _)| {
            let value = &report[start + KEY.len()..];
            let digits = value
                .find(|character: char| !character.is_ascii_digit())
                .unwrap_or(value.len());
            value[..digits].parse().ok()
        })
        .max()
}

fn other_processes() -> Check {
    let processes = if cfg!(windows) {
        output("tasklist", &["/FO", "CSV", "/NH"]).map(|listing| parse_tasklist(&listing))
    } else {
        output("ps", &["-A", "-o", "pid=,comm="]).map(|listing| parse_ps(&listing))
    };
    let Some(processes) = processes else {
        return Check::new("processes", Verdict::Note, "unknown: no process listing");
    };
    let others = praxis_processes(&processes, std::process::id());
    if others.is_empty() {
        return Check::new("processes", Verdict::Ok, "no other praxis process");
    }
    let names: Vec<_> = others
        .iter()
        .map(|(id, name)| format!("{name} {id}"))
        .collect();
    Check::new(
        "processes",
        Verdict::Warn,
        format!("{}: bench only once they end", listed(&names)),
    )
}

/// Process IDs and image names from `tasklist /FO CSV /NH`.
pub fn parse_tasklist(listing: &str) -> Vec<(u32, String)> {
    listing
        .lines()
        .filter_map(|line| {
            let mut fields = line.split("\",\"");
            let name = fields.next()?.trim_start_matches('"');
            let id = fields.next()?.parse().ok()?;
            Some((id, name.to_owned()))
        })
        .collect()
}

/// Process IDs and command names from `ps -A -o pid=,comm=`.
pub fn parse_ps(listing: &str) -> Vec<(u32, String)> {
    listing
        .lines()
        .filter_map(|line| {
            let (id, command) = line.trim_start().split_once(char::is_whitespace)?;
            Some((id.parse().ok()?, command.trim().to_owned()))
        })
        .collect()
}

/// The processes that are praxis, its binary or one of its test binaries
/// (`praxis-<hash>`), other than `own_id`.
pub fn praxis_processes(processes: &[(u32, String)], own_id: u32) -> Vec<(u32, String)> {
    processes
        .iter()
        .filter(|(id, name)| {
            let file_name = name.rsplit(['/', '\\']).next().unwrap_or(name);
            let stem = file_name.strip_suffix(".exe").unwrap_or(file_name);
            *id != own_id && (stem == "praxis" || stem.starts_with("praxis-"))
        })
        .cloned()
        .collect()
}

/// The open PRs and the live milestone's assigned tickets, through `gh`.
fn work_in_flight() -> Vec<Check> {
    let skipped = |name| Check::new(name, Verdict::Note, "skipped: `gh` absent or signed out");
    let Some(pulls) = gh_json(&[
        "pr",
        "list",
        "--state",
        "open",
        "--json",
        "number,headRefName,isDraft",
    ]) else {
        return vec![skipped("prs"), skipped("tickets")];
    };
    let pulls: Vec<_> = pulls
        .iter()
        .filter_map(|pull| {
            let draft = if pull["isDraft"].as_bool()? {
                " (draft)"
            } else {
                ""
            };
            Some(format!(
                "#{} {}{draft}",
                pull["number"].as_u64()?,
                pull["headRefName"].as_str()?
            ))
        })
        .collect();
    let pulls = if pulls.is_empty() {
        "none open".to_owned()
    } else {
        pulls.join(", ")
    };
    vec![Check::new("prs", Verdict::Note, pulls), assigned_tickets()]
}

fn assigned_tickets() -> Check {
    let milestones = gh_json(&["api", "repos/{owner}/{repo}/milestones"]).unwrap_or_default();
    let titles: Vec<_> = milestones
        .iter()
        .filter_map(|milestone| milestone["title"].as_str())
        .collect();
    let Some(live) = live_milestone(&titles) else {
        return Check::new("tickets", Verdict::Note, "no open milestone named M<n>");
    };
    let issues = gh_json(&[
        "issue",
        "list",
        "--milestone",
        live,
        "--state",
        "open",
        "--limit",
        "200",
        "--json",
        "number,assignees",
    ])
    .unwrap_or_default();
    let mut assigned: Vec<_> = issues
        .iter()
        .filter(|issue| {
            issue["assignees"]
                .as_array()
                .is_some_and(|assignees| !assignees.is_empty())
        })
        .filter_map(|issue| issue["number"].as_u64())
        .collect();
    assigned.sort_unstable();
    let assigned: Vec<_> = assigned.iter().map(|number| format!("#{number}")).collect();
    let assigned = if assigned.is_empty() {
        "none".to_owned()
    } else {
        assigned.join(", ")
    };
    Check::new(
        "tickets",
        Verdict::Note,
        format!("{live} assigned: {assigned}"),
    )
}

fn gh_json(arguments: &[&str]) -> Option<Vec<serde_json::Value>> {
    serde_json::from_str(&output("gh", arguments)?).ok()
}

/// The live milestone: the open one with the lowest number in its `M<n>`
/// title.
pub fn live_milestone<'title>(titles: &[&'title str]) -> Option<&'title str> {
    titles
        .iter()
        .filter_map(|&title| Some((title.strip_prefix('M')?.parse::<u32>().ok()?, title)))
        .min()
        .map(|(_, title)| title)
}

/// `names` joined, the first `LISTED_NAMES` of them, then a count of the rest.
fn listed(names: &[String]) -> String {
    let shown = names[..names.len().min(LISTED_NAMES)].join(", ");
    match names.len().saturating_sub(LISTED_NAMES) {
        0 => shown,
        rest => format!("{shown} and {rest} more"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapters_classify_as_the_machines_the_repository_records() {
        use wgpu::{Backend, DeviceType};
        let cases = [
            (
                "Apple M4 Pro",
                Backend::Metal,
                DeviceType::IntegratedGpu,
                Machine::M4Pro,
            ),
            (
                "Apple M4 Max",
                Backend::Metal,
                DeviceType::IntegratedGpu,
                Machine::Unknown,
            ),
            (
                "llvmpipe (LLVM 20.1.2, 256 bits)",
                Backend::Vulkan,
                DeviceType::Cpu,
                Machine::Lavapipe,
            ),
            (
                "NVIDIA GeForce RTX 2070 SUPER",
                Backend::Vulkan,
                DeviceType::DiscreteGpu,
                Machine::RtxComparison,
            ),
            (
                "NVIDIA GeForce RTX 2070 SUPER",
                Backend::Dx12,
                DeviceType::DiscreteGpu,
                Machine::RtxComparison,
            ),
            (
                "Microsoft Basic Render Driver",
                Backend::Dx12,
                DeviceType::Cpu,
                Machine::Unknown,
            ),
        ];
        for (name, backend, device_type, machine) in cases {
            assert_eq!(
                Machine::classify(name, backend, device_type),
                machine,
                "{name} on {backend:?}"
            );
        }
    }

    fn directories(entries: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
        entries
            .iter()
            .map(|(key, stems)| {
                let stems = stems.iter().map(|&stem| stem.to_owned()).collect();
                ((*key).to_owned(), stems)
            })
            .collect()
    }

    #[test]
    fn missing_lists_what_any_other_directory_holds() {
        let stems = directories(&[
            ("metal", &["helmet", "town"]),
            ("vulkan-cpu", &["town", "crowd"]),
        ]);
        assert_eq!(missing(&stems, "metal"), ["crowd"]);
        assert_eq!(missing(&stems, "vulkan-cpu"), ["helmet"]);
        assert_eq!(missing(&stems, "vulkan"), ["crowd", "helmet", "town"]);
        assert!(missing(&directories(&[("metal", &["town"])]), "metal").is_empty());
    }

    #[test]
    fn the_repositorys_goldens_and_baselines_are_complete_for_their_adapters() {
        let goldens = stems_by_directory(Path::new(GOLDEN_DIRECTORY), "png");
        assert!(missing(&goldens, "metal").is_empty());
        assert!(!missing(&goldens, "dx12").is_empty());
        let baselines = stems_by_directory(Path::new(BASELINE_DIRECTORY), "json");
        assert!(missing(&baselines, "apple-m4-pro").is_empty());
    }

    #[test]
    fn utilisation_is_the_busiest_gpus() {
        assert_eq!(parse_nvidia_smi_utilization("7\n43\n"), Some(43));
        assert_eq!(parse_nvidia_smi_utilization("[N/A]\n"), None);
        let ioreg = r#"| "PerformanceStatistics" = {"In use system memory"=123,"Device Utilization %"=27,"Renderer Utilization %"=25}"#;
        assert_eq!(parse_ioreg_utilization(ioreg), Some(27));
        assert_eq!(parse_ioreg_utilization("no statistics"), None);
    }

    #[test]
    fn praxis_processes_come_from_either_listing_without_this_one() {
        let tasklist = "\"praxis.exe\",\"4120\",\"Console\",\"1\",\"9.436 K\"\n\
                        \"praxis-0a1b2c3d4e5f.exe\",\"77\",\"Console\",\"1\",\"8 K\"\n\
                        \"praxisd.exe\",\"78\",\"Console\",\"1\",\"8 K\"\n\
                        \"cargo.exe\",\"90\",\"Console\",\"1\",\"8 K\"";
        assert_eq!(
            praxis_processes(&parse_tasklist(tasklist), 4120),
            [(77, "praxis-0a1b2c3d4e5f.exe".to_owned())]
        );
        let ps = "  412 /Users/owner/praxis/target/release/praxis\n 9001 /bin/zsh\n";
        assert_eq!(
            praxis_processes(&parse_ps(ps), 1),
            [(412, "/Users/owner/praxis/target/release/praxis".to_owned())]
        );
    }

    #[test]
    fn the_live_milestone_is_the_lowest_numbered_open_one() {
        assert_eq!(live_milestone(&["M10", "M3", "Backlog", "M4"]), Some("M3"));
        assert_eq!(live_milestone(&["Backlog"]), None);
    }

    #[test]
    fn long_lists_count_what_they_leave_out() {
        let names: Vec<_> = (1..=6).map(|number| format!("#{number}")).collect();
        assert_eq!(listed(&names[..2]), "#1, #2");
        assert_eq!(listed(&names), "#1, #2, #3, #4 and 2 more");
    }
}
