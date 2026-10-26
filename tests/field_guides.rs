//! The field guides in `docs/guides/` are served as a static site, so every
//! link and image between them has to land: on a file that exists, at an id
//! its page has. The hub, `index.html`, links every guide, and each guide's
//! series links reach the guides before and after it in the hub's order.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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

#[test]
fn every_link_between_the_field_guides_lands() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/guides");
    let mut files = BTreeSet::new();
    files_under(&directory, "", &mut files);
    let pages: BTreeMap<String, String> = files
        .iter()
        .filter(|name| name.ends_with(".html"))
        .map(|name| {
            let html = std::fs::read_to_string(directory.join(name)).expect("read a field guide");
            (name.clone(), html)
        })
        .collect();
    let problems = link_problems(&pages, &files);
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
