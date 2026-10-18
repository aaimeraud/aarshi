//! Shared read primitives and the `pages` dimension: a view without a route,
//! a hand-added route without a view, the `<page>.title` key in every
//! dictionary, sitemap entries, the view import and broken imports.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::Config,
    dictionary,
    status::{Dimension, Finding, Location, Severity},
};

/// One `page.tsx` found under `app/[locale]/`, with the URL path it serves.
pub struct Route {
    /// Project-relative file, e.g. `app/[locale]/about/page.tsx`.
    pub file: String,
    /// URL inside a locale: `/` for the root page, `/<segment>` after.
    pub path: String,
    /// The page aarshi derives from the route: `home` at the root, else the
    /// first path segment.
    pub page: String,
}

// --- Shared read primitives ----------------------------------------------

/// Every `page.tsx` under `app/[locale]/`, recursively, sorted by file path.
/// An absent or unreadable folder yields an empty list, never a stop.
pub fn routes(root: &Path) -> Vec<Route> {
    let mut found = Vec::new();
    walk_page_files(&root.join("app/[locale]"), &mut Vec::new(), &mut found);
    found.sort_unstable_by(|a, b| a.file.cmp(&b.file));
    found
}

fn walk_page_files(dir: &Path, segments: &mut Vec<String>, found: &mut Vec<Route>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = dir.join(&name);
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            segments.push(name);
            walk_page_files(&path, segments, found);
            segments.pop();
        } else if name == "page.tsx" {
            let joined = segments.join("/");
            found.push(Route {
                file: if joined.is_empty() {
                    "app/[locale]/page.tsx".to_string()
                } else {
                    format!("app/[locale]/{joined}/page.tsx")
                },
                path: if joined.is_empty() {
                    "/".to_string()
                } else {
                    format!("/{joined}")
                },
                page: segments
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "home".to_string()),
            });
        }
    }
}

/// Page names: the folders of `views/` that hold a `<name>.view.tsx`, sorted.
/// A missing or unreadable folder yields an empty list.
pub fn views(root: &Path) -> Vec<String> {
    let entries = match fs::read_dir(root.join("views")) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut pages = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().join(format!("{name}.view.tsx")).exists() {
            pages.push(name);
        }
    }
    pages.sort();
    pages
}

/// Section names: the `<name>.tsx` files of `views/<page>/sections/`, sorted.
/// A missing or unreadable folder yields an empty list.
#[allow(dead_code)] // reused by the `sections` dimension (ticket 0027) next.
pub fn sections(root: &Path, page: &str) -> Vec<String> {
    let dir = root.join("views").join(page).join("sections");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        if is_file && path.extension().map(|e| e == "tsx").unwrap_or(false) {
            names.push(name.trim_end_matches(".tsx").to_string());
        }
    }
    names.sort();
    names
}

/// One import read as text: the module between the quotes and the specifiers
/// it brings in.
pub struct Import {
    /// The text between the quotes, without the trailing `;`.
    pub module: String,
    /// Specifiers: the default name and the `{ A, B }` members, with `type `
    /// markers dropped.
    #[allow(dead_code)] // read by the `sections` dimension (ticket 0027) next.
    pub names: Vec<String>,
}

/// The imports of a TypeScript file, one per line, in order. Textual only, in
/// the style of `dictionary::entries`: multi-line imports are not understood.
pub fn imports(content: &str) -> Vec<Import> {
    content.lines().filter_map(parse_import).collect()
}

fn parse_import(line: &str) -> Option<Import> {
    let line = line.trim();
    let rest = line.strip_prefix("import ")?.trim_start();
    let (specifiers, module) = match rest.split_once(" from ") {
        Some((specifiers, module)) => (Some(specifiers.trim()), module),
        None => (None, rest),
    };
    // Import assertions (`import "x" with { ... }`) are not part of the path.
    let module = module
        .split_once(" with ")
        .map(|(path, _)| path)
        .unwrap_or(module);
    let module = module
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_matches(['"', '\'']);
    if module.is_empty() {
        return None;
    }
    Some(Import {
        module: module.to_string(),
        names: specifiers.map(names_of).unwrap_or_default(),
    })
}

fn names_of(specifiers: &str) -> Vec<String> {
    let mut names = Vec::new();
    match specifiers.find('{') {
        Some(brace) => {
            // An optional default import stands before the braces.
            let default = specifiers[..brace].trim().trim_matches(',').trim();
            if let Some(name) = default.split_whitespace().next()
                && name != "type" && name != "*" {
                names.push(name.to_string());
            }
            if let Some(close) = specifiers[brace..].find('}') {
                for member in specifiers[brace + 1..brace + close].split(',') {
                    let member = member.trim();
                    let member = member.strip_prefix("type ").unwrap_or(member).trim();
                    let member = member.split_whitespace().next().unwrap_or(member);
                    if !member.is_empty() {
                        names.push(member.to_string());
                    }
                }
            }
        }
        None => {
            if let Some(name) = specifiers.split_whitespace().next()
                && name != "type" && name != "*" {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// The file an import module points to: `@/x` is the project root, `./x` and
/// `../x` sit next to `importer` (a project-relative file). Any other target
/// is a bare package, the business of the `dependencies` dimension.
/// TypeScript's suffixes are tried in turn: exact, `.ts`, `.tsx`, then an
/// `index` file.
pub fn resolve_module(root: &Path, importer: &str, module: &str) -> Option<PathBuf> {
    if module.is_empty() {
        return None;
    }
    // The alias resets the path to the project root; a relative walk starts
    // next to the importer, so `..` cannot escape the project root between
    // here and the candidate files.
    let (aliased, under) = if let Some(under) = module.strip_prefix("@/") {
        (true, under)
    } else if module.starts_with("./")
        || module.starts_with("../")
        || module.strip_prefix('.') == Some("")
    {
        (false, module)
    } else {
        // A bare package (`next`, `@sglara/cn`, …) never resolves from the
        // filesystem of the project; it belongs to `dependencies`.
        return None;
    };
    let mut parts: Vec<String> = if aliased {
        Vec::new()
    } else {
        importer
            .rsplit_once('/')
            .map(|(dir, _)| dir)
            .unwrap_or("")
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .map(String::from)
            .collect()
    };
    for segment in under.split('/') {
        match segment {
            "" | "." => continue,
            ".." => drop(parts.pop()),
            _ => parts.push(segment.to_string()),
        }
    }
    let path = root.join(parts.join("/"));
    [
        path.clone(),
        with_suffix(&path, ".ts"),
        with_suffix(&path, ".tsx"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
    .or_else(|| {
        ["index.ts", "index.tsx"]
            .into_iter()
            .map(|index| path.join(index))
            .find(|candidate| candidate.is_file())
    })
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let file = path.as_os_str().to_string_lossy().into_owned();
    PathBuf::from(format!("{file}{suffix}"))
}

/// The `"/path"` entries listed in `app/sitemap.ts`, in order, quotes
/// removed. `None` when the file cannot be read: its missing file is a finding
/// of the `config` dimension, but nothing depends on the entries here.
pub fn sitemap_entries(root: &Path) -> Option<Vec<String>> {
    let content = fs::read_to_string(root.join("app/sitemap.ts")).ok()?;
    Some(
        content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                let rest = line.strip_prefix('"')?;
                let (entry, _) = rest.split_once('"')?;
                if entry.is_empty() {
                    None
                } else {
                    Some(entry.to_string())
                }
            })
            .collect(),
    )
}

// --- The pages dimension --------------------------------------------------

/// A module that is neither `@/`-aliased nor relative: it names an npm
/// package, whose declaration is the `dependencies` dimension's business.
pub fn is_bare(module: &str) -> bool {
    !(module.starts_with("@/") || module.starts_with("./") || module.starts_with("../"))
}

/// Collects the `pages` findings: routes ↔ views both ways, `<page>.title`
/// keys, sitemap entries, the view import and broken imports in views and
/// routes. Bare packages and dangling section imports are left to their own
/// dimensions; nothing stops the walk early.
pub fn collect(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    let route_files = routes(root);
    let page_files = views(root);
    let expected_paths: Vec<String> = page_files.iter().map(|p| route_path(p)).collect();

    for page in &page_files {
        let route_path = route_path(page);
        let route_file = route_file(&route_path);
        // view -> route: every view waits for its route file.
        let Some(route) = route_files.iter().find(|r| r.path == route_path) else {
            findings.push(Finding {
                dimension: Dimension::Pages,
                rule: "pages.route-missing",
                severity: Severity::Error,
                summary: format!("no route for the page `{page}`"),
                detail: Some(format!("expected {route_file}")),
                location: Some(Location {
                    file: Some(route_file),
                    ..Location::default()
                }),
            });
            continue;
        };
        // The route exists: it must import the view through its alias.
        if let Ok(content) = fs::read_to_string(root.join(&route.file)) {
            let view_module = format!("@/views/{page}/{page}.view");
            if !imports(&content)
                .iter()
                .any(|import| import.module == view_module)
            {
                findings.push(Finding {
                    dimension: Dimension::Pages,
                    rule: "pages.view-import-missing",
                    severity: Severity::Error,
                    summary: format!("`{route_path}` does not import its view"),
                    detail: Some(format!("expected {view_module}")),
                    location: Some(Location {
                        file: Some(route.file.clone()),
                        ..Location::default()
                    }),
                });
            }
        }
    }

    // route -> view: a hand-added route may serve an unknown page.
    for route in &route_files {
        if expected_paths.iter().any(|serve| serve == &route.path) {
            continue;
        }
        // A single-segment route names its view; nested ones have no
        // conventional view to offer as a fix.
        let single_segment = route.path == "/" || !route.path[1..].contains('/');
        findings.push(Finding {
            dimension: Dimension::Pages,
            rule: "pages.route-orphan",
            severity: Severity::Error,
            summary: format!("no view for the route `{}`", route.path),
            detail: single_segment
                .then(|| format!("expected views/{}/{}.view.tsx", route.page, route.page)),
            location: Some(Location {
                file: Some(route.file.clone()),
                ..Location::default()
            }),
        });
    }

    // The `<page>.title` key is expected in every dictionary, seeded by the
    // filesystem. An unreadable dictionary is already an `i18n` finding; the
    // locale is simply skipped here so nothing is reported twice.
    for locale in &config.languages {
        let file = format!("lib/i18n/dictionaries/{locale}.ts");
        let Ok(content) = fs::read_to_string(dictionary::file(root, locale)) else {
            continue;
        };
        let keys: HashSet<String> = dictionary::entries(&content)
            .into_iter()
            .map(|entry| entry.key)
            .collect();
        for page in &page_files {
            let key = format!("{page}.title");
            if !keys.contains(&key) {
                findings.push(Finding {
                    dimension: Dimension::Pages,
                    rule: "pages.title-key-missing",
                    severity: Severity::Error,
                    summary: format!("`{key}` is missing from the dictionary of `{locale}`"),
                    detail: None,
                    location: Some(Location {
                        file: Some(file.clone()),
                        locale: Some(locale.clone()),
                        key: Some(key.clone()),
                    }),
                });
            }
        }
    }

    // Sitemap entries ↔ views, both ways. `/` stays hard-coded in the sitemap
    // template (home), so it is never an orphan.
    if let Some(entries) = sitemap_entries(root) {
        for page in &page_files {
            let entry = route_path(page);
            if !entries.contains(&entry) {
                findings.push(Finding {
                    dimension: Dimension::Pages,
                    rule: "pages.sitemap-entry-missing",
                    severity: Severity::Error,
                    summary: format!("no sitemap entry for `{entry}`"),
                    detail: Some(format!("expected \"{entry}\",")),
                    location: Some(Location {
                        file: Some("app/sitemap.ts".to_string()),
                        key: Some(entry.clone()),
                        ..Location::default()
                    }),
                });
            }
        }
        for entry in &entries {
            if entry == "/" || expected_paths.iter().any(|serve| serve == entry) {
                continue;
            }
            findings.push(Finding {
                dimension: Dimension::Pages,
                rule: "pages.sitemap-entry-orphan",
                severity: Severity::Error,
                summary: format!("sitemap entry `{entry}` has no page"),
                detail: None,
                location: Some(Location {
                    file: Some("app/sitemap.ts".to_string()),
                    key: Some(entry.clone()),
                    ..Location::default()
                }),
            });
        }
    }

    // Broken imports in the files this dimension owns: views and routes.
    // Bare packages belong to `dependencies`.
    for route in &route_files {
        if let Ok(content) = fs::read_to_string(root.join(&route.file)) {
            let orphan_view = !page_files.contains(&route.page);
            for import in imports(&content) {
                if is_bare(&import.module) {
                    continue;
                }
                if resolve_module(root, &route.file, &import.module).is_some() {
                    continue;
                }
                // The view a hand-added route waits for is covered by
                // `pages.route-orphan`, never reported as a broken import.
                if orphan_view
                    && import.module == format!("@/views/{}/{}.view", route.page, route.page)
                {
                    continue;
                }
                raise_broken_import(findings, &route.file, &import);
            }
        }
    }
    for page in &page_files {
        let file = format!("views/{page}/{page}.view.tsx");
        if let Ok(content) = fs::read_to_string(root.join(&file)) {
            for import in imports(&content) {
                if is_bare(&import.module) {
                    continue;
                }
                if resolve_module(root, &file, &import.module).is_some() {
                    continue;
                }
                // A dangling view -> section import is the `sections`
                // dimension's `sections.file-missing` (ticket 0027).
                if import.module.starts_with("./sections/") {
                    continue;
                }
                raise_broken_import(findings, &file, &import);
            }
        }
    }
}

/// Adds one broken-import finding for an unresolved module.
fn raise_broken_import(findings: &mut Vec<Finding>, file: &str, import: &Import) {
    findings.push(Finding {
        dimension: Dimension::Pages,
        rule: "pages.import-broken",
        severity: Severity::Error,
        summary: format!("broken import `{}` in `{file}`", import.module),
        detail: Some(format!("no file matches `{}`", import.module)),
        location: Some(Location {
            file: Some(file.to_string()),
            ..Location::default()
        }),
    });
}

/// The URL path a view waits for: `/` for the home page, else `/<page>`.
fn route_path(page: &str) -> String {
    crate::add::route_path(page)
}

/// The route file a view waits for: `app/[locale]/page.tsx` at the root,
/// `app/[locale]/<page>/page.tsx` after.
fn route_file(route_path: &str) -> String {
    if route_path == "/" {
        "app/[locale]/page.tsx".to_string()
    } else {
        format!("app/[locale]{route_path}/page.tsx")
    }
}
