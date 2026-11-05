//! The field guides in `docs/guides/` are served as a static site, so every
//! link and image between them has to land: on a file that exists, at an id
//! its page has. The hub, `index.html`, links every guide, and each guide's
//! series links reach the guides before and after it in the hub's order.
//! Each page but a volume names the commit of `main` it describes, which
//! must be on `main`, names in code font only identifiers `main` holds, and
//! names no ticket or PR.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

/// The volumes that narrated how each mechanism was built, each kept, and
/// passed over by the page checks, until the page that absorbs it lands.
const VOLUMES: &[&str] = &["crowd-shadows.html"];

/// The values of `attribute` in `html`, in order. Only whole attribute names
/// count: `id` does not match inside `data-id`.
fn attribute_values<'a>(html: &'a str, attribute: &str) -> Vec<&'a str> {
    let marker = format!("{attribute}=\"");
    html.match_indices(&marker)
        .filter(|(start, _)| html[..*start].ends_with(char::is_whitespace))
        .filter_map(|(start, _)| {
            let value = &html[start + marker.len()..];
            value.find('"').map(|end| &value[..end])
        })
        .collect()
}

/// What is wrong with the links between `pages`, keyed by path under
/// `docs/guides/`, given every file there: a link or image to a file that is
/// absent, a link to an id its page lacks, or one out to claude.ai, where the
/// guides lived before; a guide the hub does not link; and a guide whose
/// series links miss its neighbour in the hub's order.
fn link_problems(pages: &BTreeMap<String, String>, files: &BTreeSet<String>) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, html) in pages {
        for href in attribute_values(html, "href") {
            if href.contains("claude.ai/") {
                problems.push(format!("{name} links {href}, outside the site"));
                continue;
            }
            if href.contains("://") {
                continue;
            }
            let (file, id) = href.split_once('#').unwrap_or((href, ""));
            let target = if file.is_empty() { name.as_str() } else { file };
            match pages.get(target) {
                Some(target_html) if !id.is_empty() => {
                    if !attribute_values(target_html, "id").contains(&id) {
                        problems.push(format!("{name} links {href}, whose page has no id {id}"));
                    }
                }
                Some(_) => {}
                None if files.contains(target) => {}
                None => problems.push(format!("{name} links {href}, which is absent")),
            }
        }
        for src in attribute_values(html, "src") {
            if !src.contains("://") && !src.starts_with("data:") && !files.contains(src) {
                problems.push(format!("{name} shows {src}, which is absent"));
            }
        }
    }
    match pages.get("index.html") {
        None => problems.push("there is no index.html".to_owned()),
        Some(hub) => {
            let linked = attribute_values(hub, "href");
            for name in pages.keys().filter(|name| *name != "index.html") {
                if !linked.contains(&name.as_str()) {
                    problems.push(format!("index.html does not link {name}"));
                }
            }
            let mut order: Vec<&str> = Vec::new();
            for href in linked {
                if href != "index.html" && pages.contains_key(href) && !order.contains(&href) {
                    order.push(href);
                }
            }
            for (position, name) in order.iter().enumerate() {
                let html = &pages[*name];
                let series = html.find("class=\"series\"").map_or("", |start| {
                    let rest = &html[start..];
                    &rest[..rest.find("</nav>").unwrap_or(rest.len())]
                });
                let series_links = attribute_values(series, "href");
                let previous = position.checked_sub(1).map(|before| order[before]);
                let next = order.get(position + 1).copied();
                for neighbour in [previous, next].into_iter().flatten() {
                    if !series_links.contains(&neighbour) {
                        problems.push(format!("{name}'s series links miss {neighbour}"));
                    }
                }
            }
        }
    }
    problems
}

fn described_commit(html: &str) -> Option<&str> {
    let paragraph = &html[html.find("class=\"describes\"")?..];
    let paragraph = &paragraph[..paragraph.find("</p>")?];
    let after = paragraph.split_once("<code>main</code> at ")?.1;
    let commit = after
        .split(|character: char| !character.is_ascii_hexdigit())
        .next()?;
    (commit.len() >= 7).then_some(commit)
}

fn text_of(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        text.push_str(&rest[..start]);
        rest = rest[start..]
            .find('>')
            .map_or("", |end| &rest[start + end + 1..]);
    }
    text.push_str(rest);
    [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&amp;", "&"),
    ]
    .iter()
    .fold(text, |text, (entity, character)| {
        text.replace(entity, character)
    })
}

fn without_element(html: &str, tag: &str) -> String {
    let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
    let mut kept = String::new();
    let mut rest = html;
    while let Some(start) = rest.find(&open) {
        kept.push_str(&rest[..start]);
        rest = rest[start..]
            .find(&close)
            .map_or("", |end| &rest[start + end + close.len()..]);
    }
    kept + rest
}

/// Each ticket or PR `prose` names as this repository's tests find them in
/// comments: a number behind `#`, `PR ` or `issue ` at the start of a word.
fn ticket_citations(prose: &str) -> Vec<String> {
    let bytes = prose.as_bytes();
    let starts_word = |index: usize| {
        index == 0 || !bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b'&'
    };
    let digits_from = |index: usize| {
        bytes[index..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count()
    };
    let mut citations = Vec::new();
    for index in (0..bytes.len()).filter(|&index| starts_word(index)) {
        let rest = &bytes[index..];
        let prefix = [&b"#"[..], b"PR ", b"issue ", b"Issue "]
            .into_iter()
            .find(|prefix| rest.starts_with(prefix));
        if let Some(prefix) = prefix {
            let digits = digits_from(index + prefix.len());
            if digits > 0 {
                citations.push(prose[index..index + prefix.len() + digits].to_owned());
            }
        }
    }
    citations
}

fn code_font_text(html: &str) -> String {
    html.match_indices("<code")
        .filter(|(start, marker)| {
            matches!(html.as_bytes().get(start + marker.len()), Some(b'>' | b' '))
        })
        .filter_map(|(start, _)| {
            let element = &html[start..];
            let body = &element[element.find('>')? + 1..];
            Some(text_of(&body[..body.find("</code>")?]) + "\n")
        })
        .collect()
}

/// The identifiers in `text`, as `tests/conventions.rs` splits code into
/// tokens, numeric literals left out.
fn identifiers_in(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|word| word.starts_with(|first: char| !first.is_ascii_digit()))
}

/// What is wrong with the pages among `pages`, every guide but the
/// `volumes`: a page whose Describes line names no commit, or one `on_main`
/// rejects; an identifier in code font that `identifiers`, every one on
/// `main`, lacks; and a volume listed that is gone.
fn page_problems(
    pages: &BTreeMap<String, String>,
    volumes: &[&str],
    identifiers: &BTreeSet<String>,
    on_main: impl Fn(&str) -> bool,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, html) in pages
        .iter()
        .filter(|(name, _)| !volumes.contains(&name.as_str()))
    {
        match described_commit(html) {
            None => problems.push(format!(
                "{name} has no Describes line naming a commit of main"
            )),
            Some(commit) if !on_main(commit) => {
                problems.push(format!("{name} describes {commit}, which is not on main"))
            }
            Some(_) => {}
        }
        let code_font = code_font_text(html);
        let missing: BTreeSet<&str> = identifiers_in(&code_font)
            .filter(|identifier| !identifiers.contains(*identifier))
            .collect();
        for identifier in missing {
            problems.push(format!(
                "{name} names {identifier} in code font, which main lacks"
            ));
        }
        let links = attribute_values(html, "href")
            .into_iter()
            .filter(|href| {
                ["/gariasf/praxis/issues/", "/gariasf/praxis/pull/"]
                    .iter()
                    .any(|path| href.contains(path))
            })
            .map(str::to_owned);
        for citation in ticket_citations(&text_of(&without_element(
            &without_element(html, "style"),
            "script",
        )))
        .into_iter()
        .chain(links)
        {
            problems.push(format!("{name} names a ticket or PR: {citation}"));
        }
    }
    for volume in volumes
        .iter()
        .filter(|volume| !pages.contains_key(**volume))
    {
        problems.push(format!("the volumes list {volume}, which is absent"));
    }
    problems
}

/// Every file under `directory`, as a path relative to it behind `prefix`.
fn files_under(directory: &Path, prefix: &str, files: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(directory).expect("read a directory of docs/guides") {
        let path = entry.expect("read a docs/guides entry").path();
        let name = format!(
            "{prefix}{}",
            path.file_name().expect("a file name").to_string_lossy()
        );
        if path.is_dir() {
            files_under(&path, &format!("{name}/"), files);
        } else {
            files.insert(name);
        }
    }
}

fn guides() -> (BTreeSet<String>, BTreeMap<String, String>) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/guides");
    let mut files = BTreeSet::new();
    files_under(&directory, "", &mut files);
    let pages = files
        .iter()
        .filter(|name| name.ends_with(".html"))
        .map(|name| {
            let html = std::fs::read_to_string(directory.join(name)).expect("read a field guide");
            (name.clone(), html)
        })
        .collect();
    (files, pages)
}

fn git(arguments: &[&str]) -> (bool, Vec<u8>) {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run git");
    (output.status.success(), output.stdout)
}

/// Every identifier the tree holds, as `main` will once it merges: in the
/// path of each file git tracks, and in the text of each outside `docs/` and
/// Markdown, whose prose may name what
/// the code no longer holds. This file is left out, as its fixtures plant
/// names that are not.
fn identifiers_in_the_tree() -> BTreeSet<String> {
    let (listed, paths) = git(&["ls-files", "-z"]);
    assert!(listed, "git ls-files failed");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut identifiers = BTreeSet::new();
    for path in String::from_utf8_lossy(&paths)
        .split('\0')
        .filter(|path| !path.is_empty())
    {
        identifiers.extend(identifiers_in(path).map(str::to_owned));
        let prose = path.starts_with("docs/") || path.ends_with(".md");
        if prose || path == "tests/field_guides.rs" {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(root.join(path)) {
            identifiers.extend(identifiers_in(&text).map(str::to_owned));
        }
    }
    identifiers
}

#[test]
fn every_link_between_the_field_guides_lands() {
    let (files, pages) = guides();
    let problems = link_problems(&pages, &files);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_page_describes_main_names_what_it_holds_and_no_ticket() {
    let (_, pages) = guides();
    let on_main = |commit: &str| git(&["merge-base", "--is-ancestor", commit, "origin/main"]).0;
    let problems = page_problems(&pages, VOLUMES, &identifiers_in_the_tree(), on_main);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn link_check_names_absent_files_missing_ids_outside_links_unlisted_guides_and_broken_series() {
    let pages = BTreeMap::from([
        (
            "index.html".to_owned(),
            r#"<a href="a.html">A</a> <a href="c.html">C</a> <a href="a.html">Read</a>"#.to_owned(),
        ),
        (
            "a.html".to_owned(),
            concat!(
                r##"<h2 id="one">One</h2> <a href="#one">here</a> <a href="index.html">hub</a> "##,
                r##"<a href="#two">there</a> <a href="gone.html">gone</a> "##,
                r#"<a href="https://claude.ai/artifact/abc">old</a> <p data-id="two"></p> "#,
                r#"<img src="img/here.png"> <img src="img/gone.png"> <img src="data:image/png;base64,AA"> "#,
                r#"<nav class="series"><a href="index.html">All</a></nav>"#,
            )
            .to_owned(),
        ),
        ("b.html".to_owned(), String::new()),
        (
            "c.html".to_owned(),
            r#"<nav class="series"><a href="a.html">Previous</a> <a href="index.html">All</a></nav>"#
                .to_owned(),
        ),
    ]);
    let files = ["index.html", "a.html", "b.html", "c.html", "img/here.png"]
        .map(str::to_owned)
        .into();

    assert_eq!(
        link_problems(&pages, &files),
        [
            "a.html links #two, whose page has no id two",
            "a.html links gone.html, which is absent",
            "a.html links https://claude.ai/artifact/abc, outside the site",
            "a.html shows img/gone.png, which is absent",
            "index.html does not link b.html",
            "a.html's series links miss c.html",
        ]
    );
}

#[test]
fn page_check_names_a_page_that_describes_no_commit_or_one_off_main() {
    let pages = BTreeMap::from([
        (
            "index.html".to_owned(),
            r#"<p class="describes"><b>Describes</b> <code>main</code> at 1a2b3c4d (2026-11-04).</p>"#
                .to_owned(),
        ),
        (
            "future.html".to_owned(),
            r#"<p class="describes"><b>Describes</b> <code>main</code> at 9f8e7d6c.</p>"#.to_owned(),
        ),
        ("undated.html".to_owned(), "<p>A page.</p>".to_owned()),
        ("volume.html".to_owned(), "<p>A volume.</p>".to_owned()),
    ]);

    assert_eq!(
        page_problems(
            &pages,
            &["volume.html", "dropped.html"],
            &BTreeSet::from(["main".to_owned()]),
            |commit| { commit == "1a2b3c4d" }
        ),
        [
            "future.html describes 9f8e7d6c, which is not on main",
            "undated.html has no Describes line naming a commit of main",
            "the volumes list dropped.html, which is absent",
        ]
    );
}

#[test]
fn page_check_names_an_identifier_in_code_font_that_main_lacks() {
    let describes = r#"<p class="describes"><code>main</code> at 1a2b3c4d.</p>"#;
    let pages = BTreeMap::from([
        (
            "page.html".to_owned(),
            format!(
                "{describes} <code>build_shadow_mask</code> <code>trace_edges</code> \
                 <pre class=\"code\"><code>let mask = sample_mask(&amp;pixel, 2u);\n\
                 <span class=\"comment\">// 4 pixels</span> Vec&lt;u32&gt;</code></pre>"
            ),
        ),
        (
            "volume.html".to_owned(),
            "<code>gone_long_ago</code>".to_owned(),
        ),
    ]);
    let identifiers = [
        "main",
        "build_shadow_mask",
        "let",
        "mask",
        "sample_mask",
        "pixel",
        "pixels",
        "Vec",
        "u32",
    ]
    .map(str::to_owned)
    .into();

    assert_eq!(
        page_problems(&pages, &["volume.html"], &identifiers, |_| true),
        ["page.html names trace_edges in code font, which main lacks"]
    );
}

#[test]
fn page_check_names_a_ticket_or_pr_in_prose_or_a_link() {
    let describes = r#"<p class="describes"><code>main</code> at 1a2b3c4d.</p>"#;
    let page = |body: &str| format!("<style>a {{ color: #333; }}</style>{describes} {body}");
    let pages = BTreeMap::from([
        (
            "hash.html".to_owned(),
            page("<p>Since #123, the mask holds two words.</p>"),
        ),
        ("pull.html".to_owned(), page("<p>As PR 487 left it.</p>")),
        (
            "link.html".to_owned(),
            page("<a href=\"https://github.com/gariasf/praxis/issues/17\">the map</a>"),
        ),
        (
            "clean.html".to_owned(),
            page(concat!(
                r##"<a href="#frame">Vidim&#269;e</a> <span style="color:#555">x</span> "##,
                r#"<a href="https://github.com/gfx-rs/wgpu/issues/9215">wgpu's bug</a>"#,
                "<script>const tile = '#000';</script>",
            )),
        ),
        (
            "volume.html".to_owned(),
            "<p>Built from ticket #435.</p>".to_owned(),
        ),
    ]);
    let identifiers = BTreeSet::from(["main".to_owned()]);

    assert_eq!(
        page_problems(&pages, &["volume.html"], &identifiers, |_| true),
        [
            "hash.html names a ticket or PR: #123",
            "link.html names a ticket or PR: https://github.com/gariasf/praxis/issues/17",
            "pull.html names a ticket or PR: PR 487",
        ]
    );
}
