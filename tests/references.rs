//! The registry of sources, `docs/references.toml`, and the citations of it
//! across the tree agree: every cited key has an entry and every entry is
//! cited somewhere.

use std::fs;
use std::path::Path;

const REGISTRY: &str = "docs/references.toml";

#[test]
fn every_citation_has_an_entry_and_every_entry_a_citation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry = fs::read_to_string(root.join(REGISTRY)).expect("read the registry");
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    let problems = mismatches(&entries(&registry), &files);
    assert!(
        problems.is_empty(),
        "{} registry mismatches:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn a_dangling_key_and_an_unused_entry_are_found() {
    let registry = "# [\"commented-out\"]\n[\"higham-3.1\"]\nurl = \"x\"\n\n[\"unused\"]\nquote = '''[ref:inside-a-quote]'''\n";
    let files = [
        (
            "src/a.rs".to_owned(),
            "// Higham, §3.1 [ref:higham-3.1].\n".to_owned(),
        ),
        (
            "docs/b.md".to_owned(),
            "Prose.\nSee [ref:missing] and the form, [ref:<key>].\n".to_owned(),
        ),
    ];
    assert_eq!(entries(registry), ["higham-3.1", "unused"]);
    assert_eq!(
        mismatches(&entries(registry), &files),
        [
            "docs/b.md:2: `missing` is cited but has no entry",
            "docs/references.toml: `unused` is cited nowhere",
        ]
    );
}

#[test]
fn malformed_citations_and_keys_are_found() {
    let entries = ["higham-3.1", "Higham_3.1", "higham-3.1"].map(str::to_owned);
    let files = [(
        "src/a.rs".to_owned(),
        "// [ref:higham-3.1] [ref:Higham-3.1] [ref:higham-3.1 ]\n// [ref:higham-\n// 3.1]\n"
            .to_owned(),
    )];
    assert_eq!(
        mismatches(&entries, &files),
        [
            "src/a.rs:1: `[ref:Higham-3.1]` cites no key",
            "src/a.rs:1: `[ref:higham-3.1 ]` cites no key",
            "src/a.rs:2: `[ref:higham-` cites no key",
            "docs/references.toml: `Higham_3.1` is not a key",
            "docs/references.toml: `higham-3.1` has more than one entry",
        ]
    );
}

/// The registry's keys, from its `["key"]` table headers, in order.
fn entries(registry: &str) -> Vec<String> {
    registry
        .lines()
        .filter_map(|line| line.trim().strip_prefix("[\"")?.strip_suffix("\"]"))
        .map(str::to_owned)
        .collect()
}

/// Whether `text` is a key: lowercase letters, digits, dots and hyphens.
fn is_key(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || ".-".contains(character)
        })
}

/// Each `[ref:` in `source`, with its 1-based line, as written: up to and
/// including its `]`, or to the line's end when no `]` follows.
fn citations(source: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    for (index, line) in source.lines().enumerate() {
        for (start, _) in line.match_indices("[ref:") {
            let length = line[start..]
                .find(']')
                .map_or(line.len() - start, |end| end + 1);
            found.push((index + 1, &line[start..start + length]));
        }
    }
    found
}

/// Malformed and dangling citations, then malformed, repeated and unused
/// entries, given each file's `/`-separated path from the crate root and its
/// text. `[ref:<key>]` names the form and cites nothing.
fn mismatches(entries: &[String], files: &[(String, String)]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut cited = Vec::new();
    for (path, source) in files {
        for (line, citation) in citations(source) {
            let key = citation
                .strip_prefix("[ref:")
                .and_then(|rest| rest.strip_suffix(']'));
            match key {
                Some("<key>") => {}
                Some(key) if is_key(key) => {
                    if !entries.iter().any(|entry| entry == key) {
                        problems.push(format!("{path}:{line}: `{key}` is cited but has no entry"));
                    }
                    cited.push(key);
                }
                _ => problems.push(format!("{path}:{line}: `{citation}` cites no key")),
            }
        }
    }
    for (index, entry) in entries.iter().enumerate() {
        if !is_key(entry) {
            problems.push(format!("{REGISTRY}: `{entry}` is not a key"));
        } else if entries[..index].contains(entry) {
            problems.push(format!("{REGISTRY}: `{entry}` has more than one entry"));
        } else if !cited.contains(&entry.as_str()) {
            problems.push(format!("{REGISTRY}: `{entry}` is cited nowhere"));
        }
    }
    problems
}

/// The text files that may cite the registry: code, shaders, scripts and
/// Markdown, outside hidden directories, build output and cooked data. The
/// registry itself and this file, whose planted citations are its tests,
/// are left out.
fn collect_files(root: &Path, directory: &Path, files: &mut Vec<(String, String)>) {
    let listing = fs::read_dir(directory).expect("read a directory");
    for entry in listing {
        let path = entry.expect("read a directory entry").path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if path.is_dir() {
            if !(name.starts_with('.') || name == "target" || name == "cooked") {
                collect_files(root, &path, files);
            }
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("a path under the crate root")
            .to_string_lossy()
            .replace('\\', "/");
        let is_text = ["rs", "wgsl", "md", "sh", "toml"]
            .iter()
            .any(|extension| path.extension().is_some_and(|found| found == *extension));
        if is_text && relative != REGISTRY && relative != "tests/references.rs" {
            let text = fs::read_to_string(&path).expect("read a text file");
            files.push((relative, text));
        }
    }
}
