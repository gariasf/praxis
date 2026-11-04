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

const RULES: [Rule; 7] = [
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
    Rule {
        name: "measured timing or ratio in a shipping comment",
        lines: measured_figures,
    },
];

#[test]
fn the_source_tree_follows_the_conventions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["src", "tests", "scripts", ".github"] {
        collect_sources(root, &root.join(directory), &mut files);
    }
    for file in ["Cargo.toml", ".gitignore"] {
        let source = fs::read_to_string(root.join(file)).expect("read a source file");
        files.push((file.to_owned(), source));
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

/// Every Rust, WGSL, shell, Python, YAML and TOML file under `directory`
/// but this one, whose planted violations are its tests.
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
        let is_source = [".rs", ".wgsl", ".sh", ".py", ".yml", ".toml"]
            .iter()
            .any(|extension| relative.ends_with(extension));
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

/// Comments cite no ticket or PR, as `#<n>`, `PR <n>`, `issue <n>` or an
/// `issues/<n>` or `pull/<n>` link. An upstream bug keeps its repository before the `#`
/// or the link, as in `gfx-rs/wgpu#9215`. Rust and WGSL comment with `//`
/// and `/* */`, the scripts, workflows, `Cargo.toml` and `.gitignore` with
/// `#`. The PR bodies `scripts/pr-lint-test.sh` feeds its lint are
/// fixtures, not comments.
fn ticket_numbers(path: &str, source: &str) -> Vec<usize> {
    if path == "scripts/pr-lint-test.sh" {
        return Vec::new();
    }
    let comments = if path.ends_with(".rs") || path.ends_with(".wgsl") {
        slash_comments(source)
    } else {
        source
            .lines()
            .map(|line| hash_comment(line).to_owned())
            .collect()
    };
    comments
        .iter()
        .enumerate()
        .filter(|(_, comment)| cites_ticket(comment))
        .map(|(index, _)| index + 1)
        .collect()
}

/// Whether `comment` cites a ticket or PR of this repository.
fn cites_ticket(comment: &str) -> bool {
    let bytes = comment.as_bytes();
    let starts_word = |index: usize| index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
    let number_at = |index: usize| bytes.get(index).is_some_and(u8::is_ascii_digit);
    // A link whose path names a repository other than this one.
    let upstream = |before: &[u8]| {
        let repository = before
            .strip_suffix(b"/")
            .and_then(|path| path.rsplit(|&byte| byte == b'/').next());
        repository.is_some_and(|name| !name.is_empty() && name != b"praxis")
    };
    (0..bytes.len()).any(|index| {
        let rest = &bytes[index..];
        let hash = rest[0] == b'#' && number_at(index + 1) && starts_word(index);
        let pull_request = rest.starts_with(b"PR ") && number_at(index + 3) && starts_word(index);
        let issue = rest.len() > 6
            && rest[..6].eq_ignore_ascii_case(b"issue ")
            && number_at(index + 6)
            && starts_word(index);
        let link = ["issues/", "pull/"].iter().any(|path| {
            rest.starts_with(path.as_bytes())
                && number_at(index + path.len())
                && !upstream(&bytes[..index])
        });
        hash || pull_request || issue || link
    })
}

/// Each line's comment text in Rust or WGSL: what follows `//`, and what
/// lies between `/*` and `*/`, which may span lines. String literals are
/// not comments.
fn slash_comments(source: &str) -> Vec<String> {
    let mut in_block = false;
    source
        .lines()
        .map(|line| {
            let (code, line_comment) = split_comment(line);
            // A `//` inside a block comment does not end it, so the line is
            // scanned again whole, its strings left out.
            let scanned = code + line_comment;
            let mut text = String::new();
            let mut rest = scanned.as_str();
            loop {
                if in_block {
                    let Some(end) = rest.find("*/") else {
                        text.push_str(rest);
                        break;
                    };
                    text.push_str(&rest[..end]);
                    rest = &rest[end + 2..];
                    in_block = false;
                    continue;
                }
                let block = rest.find("/*");
                if let Some(line_start) = rest.find("//")
                    && block.is_none_or(|block_start| line_start < block_start)
                {
                    text.push_str(&rest[line_start..]);
                    break;
                }
                let Some(block_start) = block else {
                    break;
                };
                rest = &rest[block_start + 2..];
                in_block = true;
            }
            text
        })
        .collect()
}

/// The line's `#` comment, as a shell, YAML or TOML file writes one: from a
/// `#` that starts a word outside quotes to the line's end.
fn hash_comment(line: &str) -> &str {
    let mut quote = None;
    let mut previous = ' ';
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        match quote {
            _ if escaped => escaped = false,
            Some('"') if character == '\\' => escaped = true,
            Some(open) if character == open => quote = None,
            Some(_) => {}
            None if character == '\\' => escaped = true,
            None if character == '"' || character == '\'' => quote = Some(character),
            None if character == '#' && previous.is_whitespace() => return &line[index..],
            None => {}
        }
        previous = character;
    }
    ""
}

#[test]
fn ticket_numbers_are_found_in_comments_but_upstream_bugs_are_allowed() {
    let source = "// Fixed by #123.\nlet x = 1; // (#45)\n/// See gfx-rs/wgpu#9215.\nlet colour = \"#000\"; // C# #pragma\nlet quote = '\\\"'; // #7\n";
    assert_eq!(ticket_numbers("src/a.rs", source), [1, 2, 5]);
}

#[test]
fn tickets_written_out_or_in_block_comments_are_found() {
    let source = "// Fixed in PR 45.\n// See issue 123, or Issue 7.\n// github.com/gariasf/praxis/issues/17\n// issues/12\n/// gfx-rs/wgpu/issues/9215, a PRNG 4, tissue 5\nlet x = 1; /* fixed by #45 */\n/* A block\n   from issue 9\n*/ let y = \"issue 3\";\n/* see https://example.com */\nready(issue 3);\n// github.com/gariasf/praxis/pull/17, gfx-rs/wgpu/pull/8123\n";
    assert_eq!(ticket_numbers("src/a.wgsl", source), [1, 2, 3, 4, 6, 8, 12]);
}

#[test]
fn tickets_are_found_in_hash_comments_outside_the_fixture_bodies() {
    let source = "# Fixed by #45\necho \"#45 issue 3\"\nx=1 # see PR 7\necho ${#array[@]} $#\necho it\\'s # see #4\n";
    assert_eq!(ticket_numbers("scripts/a.sh", source), [1, 3, 5]);
    assert_eq!(
        ticket_numbers(".github/workflows/ci.yml", source),
        [1, 3, 5]
    );
    assert_eq!(ticket_numbers(".gitignore", source), [1, 3, 5]);
    assert_eq!(
        ticket_numbers("scripts/pr-lint-test.sh", source),
        [] as [usize; 0]
    );
}

/// Identifiers are full words: `normal_dot_view`, never `n_dot_v`,
/// `NdotV` or `NoV`, with a suffix (`n_dot_v2`) or without, in code and
/// tests alike. A comment may carry the textbook symbol.
fn abbreviated_shading_names(path: &str, source: &str) -> Vec<usize> {
    if !(path.starts_with("src/") || path.starts_with("tests/")) {
        return Vec::new();
    }
    code_lines(source, |code| {
        identifiers(code).any(is_abbreviated_shading_name)
    })
}

/// Whether `identifier` starts with a dot product of two vectors' initials,
/// as `n_dot_v`, `NdotL` or `NoH`, and any suffix starts a new word: a digit,
/// `_` or, after `dot`, a capital.
fn is_abbreviated_shading_name(identifier: &str) -> bool {
    // The initials of the normal, light, view, half, reflection, tangent and
    // bitangent vectors.
    const VECTOR_INITIALS: &[u8] = b"nlvhrtb";
    let is_initial = |byte: &u8| VECTOR_INITIALS.contains(&byte.to_ascii_lowercase());
    let bytes = identifier.as_bytes();
    let Some((first, after_first)) = bytes.split_first() else {
        return false;
    };
    if !is_initial(first) {
        return false;
    }
    // `NoV2` or `NoV_squared`, but not `NoHeader` or `LoRA`.
    let o_spelling = first.is_ascii_uppercase()
        && after_first.first() == Some(&b'o')
        && after_first
            .get(1)
            .is_some_and(|second| second.is_ascii_uppercase() && is_initial(second))
        && after_first
            .get(2)
            .is_none_or(|next| next.is_ascii_digit() || *next == b'_');
    let before_dot = after_first.strip_prefix(b"_").unwrap_or(after_first);
    let dot_spelling = before_dot.len() >= 4 && before_dot[..3].eq_ignore_ascii_case(b"dot") && {
        let second = before_dot[3..]
            .strip_prefix(b"_")
            .unwrap_or(&before_dot[3..]);
        second.first().is_some_and(is_initial)
            && second.get(1).is_none_or(|next| !next.is_ascii_lowercase())
    };
    o_spelling || dot_spelling
}

#[test]
fn abbreviated_shading_names_are_found_in_three_spellings() {
    let source = "let n_dot_v = 1.0;\nlet NdotL = 1.0;\nlet NoH = 1.0;\nlet normal_dot_view = 1.0; // n_dot_v\nlet node = Nov;\n";
    assert_eq!(abbreviated_shading_names("src/a.wgsl", source), [1, 2, 3]);
}

#[test]
fn abbreviated_shading_names_are_found_with_suffixes_tangents_and_in_tests() {
    let source = "let n_dot_v2 = 1.0;\nlet n_dot_l_squared = 1.0;\nlet NoV2 = 1.0;\nlet t_dot_v = 1.0;\nlet BdotL = 1.0;\nlet NoHeader = 1.0;\nlet ToBytes = 1.0;\nlet tangent_dot_view = 1.0;\nlet LoRA = VoTE;\n";
    assert_eq!(
        abbreviated_shading_names("tests/shadows.rs", source),
        [1, 2, 3, 4, 5]
    );
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
/// `Equal` against the pre-pass), never with a `Less` comparison. A clear
/// that is not a colour passes only as a zero literal, `0` or `0.0` with any
/// suffix, so neither a nonzero literal nor a named constant can clear depth
/// to the far plane.
fn reversed_depth(path: &str, source: &str) -> Vec<usize> {
    if !path.starts_with("src/") {
        return Vec::new();
    }
    code_lines(source, |code| {
        let less = code.contains("CompareFunction::Less");
        let nonzero_clear = code.match_indices("LoadOp::Clear(").any(|(start, token)| {
            let argument = code[start + token.len()..].trim_start();
            let is_colour = argument.starts_with("wgpu::Color") || argument.starts_with("Color");
            !is_colour && !is_zero_literal(argument)
        });
        less || nonzero_clear
    })
}

/// Whether `text` starts with a zero literal that ends at `)` or `,`: `0`,
/// `0.0` or either with a type suffix, as `0f32`, `0.0_f32` or `0_u8`.
fn is_zero_literal(text: &str) -> bool {
    let end = text.find([')', ',']).unwrap_or(text.len());
    let Some(rest) = text[..end].trim_end().strip_prefix('0') else {
        return false;
    };
    let rest = rest
        .strip_prefix('.')
        .map_or(rest, |fraction| fraction.trim_start_matches('0'));
    let suffix = rest.strip_prefix('_').unwrap_or(rest);
    let type_suffix = suffix
        .strip_prefix(['f', 'u', 'i'])
        .is_some_and(|bits| !bits.is_empty() && bits.chars().all(|digit| digit.is_ascii_digit()));
    suffix.is_empty() || type_suffix
}

#[test]
fn forward_depth_is_found() {
    let source = "depth_compare: Some(wgpu::CompareFunction::Less),\ndepth_compare: Some(wgpu::CompareFunction::LessEqual),\nload: wgpu::LoadOp::Clear(1.0),\nload: wgpu::LoadOp::Clear(0.0),\ndepth_compare: Some(wgpu::CompareFunction::Greater),\nload: wgpu::LoadOp::Clear(wgpu::Color::BLACK),\nstencil_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1),\n";
    assert_eq!(
        reversed_depth("src/render/targets.rs", source),
        [1, 2, 3, 7]
    );
}

#[test]
fn a_depth_clear_passes_only_as_a_zero_literal() {
    let source = "load: wgpu::LoadOp::Clear(1f32),\nload: wgpu::LoadOp::Clear(FAR_DEPTH),\nload: wgpu::LoadOp::Clear(0f32),\nload: wgpu::LoadOp::Clear(0.0_f32),\nload: wgpu::LoadOp::Clear(0),\nload: wgpu::LoadOp::Clear(0.5),\nload: wgpu::LoadOp::Clear(Color::BLACK),\nload: wgpu::LoadOp::Clear(0_u8),\n";
    assert_eq!(reversed_depth("src/render/targets.rs", source), [1, 2, 6]);
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
    shipping_lines(source, |index| predicate(&split_comment(lines[index]).0))
}

/// Lines whose 0-based index satisfies `predicate`, skipping each item
/// compiled only for tests.
fn shipping_lines(source: &str, predicate: impl Fn(usize) -> bool) -> Vec<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        let test_only =
            line.starts_with("#[cfg(") && line.contains("test") && !line.contains("not(test");
        if test_only {
            index = end_of_item(&lines, index);
        } else if predicate(index) {
            found.push(index + 1);
        }
        index += 1;
    }
    found
}

/// The index of the last line of the item whose attributes start at
/// `attribute`: its first line, if it ends there with `;`, or the first
/// later line that closes it at its own indentation, a `}` followed by no
/// more than `)`, `]`, `;` or `,` as `rustfmt` lays it out, so a string
/// literal's line such as `}"#;` is not taken for it.
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
                .and_then(|rest| rest.trim_end().strip_prefix('}'))
                .is_some_and(|after| after.chars().all(|character| ")];,".contains(character)))
        })
        .unwrap_or(lines.len())
}

#[test]
fn render_unwraps_are_found_outside_tests() {
    let source = "fn draw() {\n    buffer.get().unwrap();\n    // .unwrap() is not here\n}\n\n#[cfg(test)]\nfn helper() {\n    x.unwrap();\n}\n\n#[cfg(test)]\nmod tests;\nfn after() {\n    y.unwrap();\n}\n\n#[cfg(test)]\n#[allow(dead_code)]\nmod tests {\n    fn inner() {\n        z.unwrap();\n    }\n}\n#[cfg(all(test, feature = \"cook\"))]\nfn cooked() {\n    w.unwrap();\n}\n#[cfg(not(test))]\nfn shipped() {\n    v.unwrap();\n}\n#[cfg(test)]\nmod raw {\n    const JSON: &str = r#\"{\n}\"#;\n    fn inner() {\n        u.unwrap();\n    }\n}\n#[cfg(test)]\nstatic LOCK: Mutex<u8> = Mutex::new({\n    0\n});\nfn after_static() {\n    t.unwrap();\n}\n";
    assert_eq!(
        render_unwraps("src/render/pass.rs", source),
        [2, 14, 30, 45]
    );
    assert_eq!(
        render_unwraps("src/assets/pools.rs", source),
        [] as [usize; 0]
    );
    assert_eq!(
        render_unwraps("src/render/shader_tests.rs", source),
        [] as [usize; 0]
    );
}

/// A timing or a ratio measured on one machine at one commit goes stale with
/// the next change to its pass, so shipping comments give the reason in words
/// and the figure lives in `docs/performance.md`'s "Measurements behind the
/// code". Comments in tests keep their figures.
fn measured_figures(path: &str, source: &str) -> Vec<usize> {
    if !path.starts_with("src/") || path.ends_with("_tests.rs") {
        return Vec::new();
    }
    let comments = slash_comments(source);
    shipping_lines(source, |index| has_measured_figure(&comments[index]))
}

/// Whether `text` holds a digit followed, after at most one space, by `%` or
/// by a unit of time that ends its word: `38 µs`, `5ms`, `30%`.
fn has_measured_figure(text: &str) -> bool {
    text.char_indices().any(|(start, character)| {
        if !character.is_ascii_digit() {
            return false;
        }
        let after = &text[start + 1..];
        let after = after.strip_prefix(char::is_whitespace).unwrap_or(after);
        after.starts_with('%')
            // The micro sign and the Greek letter mu look alike.
            || ["ms", "µs", "μs", "us", "ns"].iter().any(|unit| {
                after.strip_prefix(unit).is_some_and(|rest| {
                    !rest.starts_with(|next: char| next.is_alphanumeric() || next == '_')
                })
            })
    })
}

#[test]
fn measured_figures_are_found_in_shipping_comments() {
    let source = "// The pass took 38 µs at 1440p.\n// About 30% slower.\n/// 5ms, 2 ns and 4us.\nlet budget = 12; // 12 ms\n// 64 KiB, 0.09 rad, 2 msaa samples, f32 steps\nlet percent = \"5%\";\n#[cfg(test)]\nmod tests {\n    // 10% slower.\n}\nlet a = 1; /* 38 µs */\n// 38 μs, in Greek\n";
    assert_eq!(
        measured_figures("src/render/pass.wgsl", source),
        [1, 2, 3, 4, 11, 12]
    );
    assert_eq!(
        measured_figures("src/render/shader_tests.rs", source),
        [] as [usize; 0]
    );
    assert_eq!(measured_figures("tests/bench.rs", source), [] as [usize; 0]);
}
