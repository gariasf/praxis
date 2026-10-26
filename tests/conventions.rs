//! The conventions a pattern can decide, checked over the source tree so
//! review is left only judgement. Each rule is a function from one file's
//! path and source to the 1-based lines that break it, tested against a
//! planted violation and a near miss it must allow.

use std::fs;
use std::path::Path;

/// A rule: its name, as failures print it, and the lines of a file that
/// break it, given the file's `/`-separated path from the crate root.
struct Rule {
    name: &'static str,
    lines: fn(path: &str, source: &str) -> Vec<usize>,
}

const RULES: [Rule; 6] = [
    Rule {
        name: "print or `log` instead of `tracing`",
        lines: printing,
    },
    Rule {
        name: "ticket number in a comment",
        lines: ticket_numbers,
    },
    Rule {
        name: "abbreviated shading name",
        lines: abbreviated_shading_names,
    },
    Rule {
        name: "sRGB texture format outside sampled textures and the surface",
        lines: srgb_formats,
    },
    Rule {
        name: "depth not cleared to 0 or compared with `Less`",
        lines: reversed_depth,
    },
    Rule {
        name: "`.unwrap()` in the render path",
        lines: render_unwraps,
    },
];

#[test]
fn the_source_tree_follows_the_conventions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["src", "tests"] {
        collect_sources(root, &root.join(directory), &mut files);
    }
    let mut violations = Vec::new();
    for (path, source) in &files {
        for rule in &RULES {
            for line in (rule.lines)(path, source) {
                violations.push(format!("{path}:{line}: {}", rule.name));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "{} convention violations:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

/// Every `.rs` and `.wgsl` file under `directory` but this one, whose
/// planted violations are its tests.
fn collect_sources(root: &Path, directory: &Path, files: &mut Vec<(String, String)>) {
    let entries = fs::read_dir(directory).expect("read a source directory");
    for entry in entries {
        let path = entry.expect("read a directory entry").path();
        if path.is_dir() {
            collect_sources(root, &path, files);
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("a path under the crate root")
            .to_string_lossy()
            .replace('\\', "/");
        let is_source = relative.ends_with(".rs") || relative.ends_with(".wgsl");
        if is_source && relative != "tests/conventions.rs" {
            let source = fs::read_to_string(&path).expect("read a source file");
            files.push((relative, source));
        }
    }
}

/// The line's code and its `//` comment, with each string literal's
/// contents left out of the code. A `//` inside a string is not a comment.
fn split_comment(line: &str) -> (String, &str) {
    let characters: Vec<(usize, char)> = line.char_indices().collect();
    let at = |index: usize| characters.get(index).map(|&(_, character)| character);
    let mut code = String::new();
    let mut in_string = false;
    let mut index = 0;
    while let Some(character) = at(index) {
        match character {
            '\\' if in_string => index += 1,
            // A character literal such as '"' or '\"', which opens no string.
            '\'' if !in_string && at(index + 2) == Some('\'') => index += 2,
            '\'' if !in_string && at(index + 1) == Some('\\') && at(index + 3) == Some('\'') => {
                index += 3
            }
            '"' => {
                in_string = !in_string;
                code.push('"');
            }
            '/' if !in_string && at(index + 1) == Some('/') => {
                return (code, &line[characters[index].0..]);
            }
            _ if !in_string => code.push(character),
            _ => {}
        }
        index += 1;
    }
    (code, "")
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The identifiers in `code`, numeric literals included.
fn identifiers(code: &str) -> impl Iterator<Item = &str> {
    code.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|word| !word.is_empty())
}

/// Whether `code` holds `token` with no identifier character before it.
fn contains_token(code: &str, token: &str) -> bool {
    code.match_indices(token)
        .any(|(start, _)| start == 0 || !is_identifier_byte(code.as_bytes()[start - 1]))
}

/// Lines whose code satisfies `predicate`, comments aside.
fn code_lines(source: &str, predicate: impl Fn(&str) -> bool) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| predicate(&split_comment(line).0))
        .map(|(index, _)| index + 1)
        .collect()
}

/// Output goes through `tracing`, so `src/` prints nothing and uses no
/// `log` macros.
fn printing(path: &str, source: &str) -> Vec<usize> {
    if !path.starts_with("src/") {
        return Vec::new();
    }
    code_lines(source, |code| {
        ["println!", "eprintln!", "print!", "eprint!", "log::"]
            .iter()
            .any(|token| contains_token(code, token))
    })
}

#[test]
fn printing_is_found_in_code_but_not_in_comments() {
    let source = "fn main() {\n    // Not `eprintln!`.\n    println!(\"frame\");\n    log::info!(\"x\");\n    tracing::info!(\"println!\");\n}\n";
    assert_eq!(printing("src/main.rs", source), [3, 4]);
    assert_eq!(printing("tests/main.rs", source), [] as [usize; 0]);
}

/// Comments cite no ticket or PR. An upstream bug keeps its repository
/// before the `#`, as in `gfx-rs/wgpu#9215`.
fn ticket_numbers(_path: &str, source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let comment = split_comment(line).1.as_bytes();
            (0..comment.len()).any(|index| {
                comment[index] == b'#'
                    && comment.get(index + 1).is_some_and(u8::is_ascii_digit)
                    && (index == 0 || !comment[index - 1].is_ascii_alphanumeric())
            })
        })
        .map(|(index, _)| index + 1)
        .collect()
}

#[test]
fn ticket_numbers_are_found_in_comments_but_upstream_bugs_are_allowed() {
    let source = "// Fixed by #123.\nlet x = 1; // (#45)\n/// See gfx-rs/wgpu#9215.\nlet colour = \"#000\"; // C# #pragma\nlet quote = '\\\"'; // #7\n";
    assert_eq!(ticket_numbers("src/a.rs", source), [1, 2, 5]);
}

/// Identifiers are full words: `normal_dot_view`, never `n_dot_v`,
/// `NdotV` or `NoV`. A comment may carry the textbook symbol.
fn abbreviated_shading_names(path: &str, source: &str) -> Vec<usize> {
    if !path.starts_with("src/") {
        return Vec::new();
    }
    // The initials of the normal, light, view, half and reflection vectors.
    const VECTOR_INITIALS: &[u8] = b"nlvhr";
    let is_abbreviation = |identifier: &str| {
        let folded = identifier.to_ascii_lowercase().replace('_', "");
        let bytes = folded.as_bytes();
        let dot_spelling = bytes.len() == 5
            && &folded[1..4] == "dot"
            && VECTOR_INITIALS.contains(&bytes[0])
            && VECTOR_INITIALS.contains(&bytes[4]);
        let o_spelling = identifier.len() == 3
            && identifier.as_bytes()[1] == b'o'
            && [0, 2].iter().all(|&index| {
                let byte = identifier.as_bytes()[index];
                byte.is_ascii_uppercase() && VECTOR_INITIALS.contains(&byte.to_ascii_lowercase())
            });
        dot_spelling || o_spelling
    };
    code_lines(source, |code| identifiers(code).any(is_abbreviation))
}

#[test]
fn abbreviated_shading_names_are_found_in_three_spellings() {
    let source = "let n_dot_v = 1.0;\nlet NdotL = 1.0;\nlet NoH = 1.0;\nlet normal_dot_view = 1.0; // n_dot_v\nlet node = Nov;\n";
    assert_eq!(abbreviated_shading_names("src/a.wgsl", source), [1, 2, 3]);
}

/// No target is an sRGB format: the tonemapping pass encodes sRGB once,
/// in its shader. Colour textures that assets and the cooker make are
/// sRGB formats, which decode to linear when read, so those two modules
/// may name them; tests elsewhere may too.
fn srgb_formats(path: &str, source: &str) -> Vec<usize> {
    let allowed = ["src/assets/", "src/cook/"];
    if !path.starts_with("src/") || allowed.iter().any(|prefix| path.starts_with(prefix)) {
        return Vec::new();
    }
    shipping_code_lines(source, |code| {
        identifiers(code)
            .any(|identifier| identifier.ends_with("Srgb") || identifier == "add_srgb_suffix")
    })
}

#[test]
fn srgb_formats_are_found_outside_textures_and_tests() {
    let source = "format: wgpu::TextureFormat::Rgba8UnormSrgb,\nformat.add_srgb_suffix()\nformat: wgpu::TextureFormat::Rgba8Unorm, // not Rgba8UnormSrgb\nencode_srgb(colour)\n#[cfg(test)]\nuse wgpu::TextureFormat::Bgra8UnormSrgb;\n";
    assert_eq!(srgb_formats("src/render/targets.rs", source), [1, 2]);
    assert_eq!(
        srgb_formats("src/assets/material.rs", source),
        [] as [usize; 0]
    );
}

/// Depth is reversed: cleared to 0 and compared with `Greater` (or
/// `Equal` against the pre-pass), never with a `Less` comparison.
fn reversed_depth(path: &str, source: &str) -> Vec<usize> {
    if !path.starts_with("src/") {
        return Vec::new();
    }
    code_lines(source, |code| {
        let less = code.contains("CompareFunction::Less");
        // Depth clears take a float; a stencil clear's integer is not depth.
        let nonzero_clear = code.match_indices("LoadOp::Clear(").any(|(start, token)| {
            let literal: String = code[start + token.len()..]
                .trim_start()
                .chars()
                .take_while(|character| character.is_ascii_digit() || *character == '.')
                .collect();
            literal.contains('.') && literal.chars().any(|digit| ('1'..='9').contains(&digit))
        });
        less || nonzero_clear
    })
}

#[test]
fn forward_depth_is_found() {
    let source = "depth_compare: Some(wgpu::CompareFunction::Less),\ndepth_compare: Some(wgpu::CompareFunction::LessEqual),\nload: wgpu::LoadOp::Clear(1.0),\nload: wgpu::LoadOp::Clear(0.0),\ndepth_compare: Some(wgpu::CompareFunction::Greater),\nload: wgpu::LoadOp::Clear(wgpu::Color::BLACK),\nstencil_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1),\n";
    assert_eq!(reversed_depth("src/render/targets.rs", source), [1, 2, 3]);
}

/// The render path handles failure instead of panicking: no `.unwrap()`
/// in `src/render/` outside `#[cfg(test)]` items and test-only files.
fn render_unwraps(path: &str, source: &str) -> Vec<usize> {
    // The files `src/render/mod.rs` declares under `#[cfg(test)]`.
    let test_files = [
        "src/render/shader_function_tests.rs",
        "src/render/shader_tests.rs",
    ];
    if !path.starts_with("src/render/") || test_files.contains(&path) {
        return Vec::new();
    }
    shipping_code_lines(source, |code| code.contains(".unwrap()"))
}

/// Like `code_lines`, but skipping each item compiled only for tests.
fn shipping_code_lines(source: &str, predicate: impl Fn(&str) -> bool) -> Vec<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        let test_only =
            line.starts_with("#[cfg(") && line.contains("test") && !line.contains("not(test");
        if test_only {
            index = end_of_item(&lines, index);
        } else if predicate(&split_comment(lines[index]).0) {
            found.push(index + 1);
        }
        index += 1;
    }
    found
}

/// The index of the last line of the item whose attributes start at
/// `attribute`: its first line, if it ends there with `;`, or the first
/// later line that closes it at its own indentation, as `rustfmt` lays
/// it out.
fn end_of_item(lines: &[&str], attribute: usize) -> usize {
    let mut item = attribute;
    while lines
        .get(item)
        .is_some_and(|line| line.trim_start().starts_with("#["))
    {
        item += 1;
    }
    let Some(first) = lines.get(item) else {
        return lines.len();
    };
    if first.trim_end().ends_with(';') {
        return item;
    }
    let indentation = &first[..first.len() - first.trim_start().len()];
    (item + 1..lines.len())
        .find(|&index| {
            lines[index]
                .strip_prefix(indentation)
                .is_some_and(|rest| rest.starts_with('}'))
        })
        .unwrap_or(lines.len())
}

#[test]
fn render_unwraps_are_found_outside_tests() {
    let source = "fn draw() {\n    buffer.get().unwrap();\n    // .unwrap() is not here\n}\n\n#[cfg(test)]\nfn helper() {\n    x.unwrap();\n}\n\n#[cfg(test)]\nmod tests;\nfn after() {\n    y.unwrap();\n}\n\n#[cfg(test)]\n#[allow(dead_code)]\nmod tests {\n    fn inner() {\n        z.unwrap();\n    }\n}\n#[cfg(all(test, feature = \"cook\"))]\nfn cooked() {\n    w.unwrap();\n}\n#[cfg(not(test))]\nfn shipped() {\n    v.unwrap();\n}\n";
    assert_eq!(render_unwraps("src/render/pass.rs", source), [2, 14, 30]);
    assert_eq!(
        render_unwraps("src/assets/pools.rs", source),
        [] as [usize; 0]
    );
    assert_eq!(
        render_unwraps("src/render/shader_tests.rs", source),
        [] as [usize; 0]
    );
}
