//! The `sections` dimension: dangling view imports of `./sections/<section>`,
//! the `<page>.<section>.title` keys, the `add -s` markers of every view,
//! never-imported sections and broken imports inside sections.

use std::{collections::HashSet, fs, path::Path};

use super::imports::imports_of;
use crate::{
    config::Config,
    dictionary,
    status::{Dimension, Finding, Location, Severity, pages},
};

/// The marker that closes the import block of a view, exactly as the view
/// template writes it.
const IMPORTS_MARKER: &str = "// aarshi:imports:end";

/// The marker that closes the section block of a view, exactly as the view
/// template writes it.
const SECTIONS_MARKER: &str = "{/* aarshi:sections:end */}";

/// Collects the `sections` findings: a dangling `./sections/<x>` view import,
/// the `<page>.<section>.title` keys derived from the filesystem, a missing
/// `add -s` marker in a view, a section no view imports, and broken imports
/// in sections. Nothing stops the walk early.
pub fn collect(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    // The sections of every page, read once: `vector of (page, [sections])`
    // shared by the key, orphan and broken-import checks.
    let mut listed: Vec<(String, Vec<String>)> = Vec::new();
    for page in pages::views(root) {
        let sections = pages::sections(root, &page);
        listed.push((page, sections));
    }

    // A view `./sections/<x>` import that matches no file was deliberately
    // kept silent by `pages.collect` (0016 addendum): this rule reports it.
    for (page, _) in &listed {
        let file = format!("views/{page}/{page}.view.tsx");
        let Ok(content) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        for import in imports_of(root, &file, &content) {
            let Some(rest) = import.module.strip_prefix("./sections/") else {
                continue;
            };
            let target = rest.strip_suffix(".tsx").unwrap_or(rest);
            if target.is_empty() || import.is_provided() {
                continue;
            }
            findings.push(
                Finding::new(
                    Dimension::Sections,
                    "sections.file-missing",
                    Severity::Error,
                    format!("dangling import `{}` in `{file}`", import.module),
                )
                .detail(format!("expected views/{page}/sections/{target}.tsx"))
                .at(Location {
                    file: Some(file.clone()),
                    ..Location::default()
                }),
            );
        }
    }

    // The `<page>.<section>.title` key is expected in every dictionary,
    // derived from the filesystem. An unreadable dictionary is already an
    // `i18n` finding; the locale is simply skipped here so nothing is
    // reported twice.
    for locale in &config.languages {
        let file = format!("lib/i18n/dictionaries/{locale}.ts");
        let Ok(content) = fs::read_to_string(dictionary::file(root, locale)) else {
            continue;
        };
        let keys: HashSet<String> = dictionary::entries(&content)
            .into_iter()
            .map(|entry| entry.key)
            .collect();
        for (page, sections) in &listed {
            for section in sections {
                let key = format!("{page}.{section}.title");
                if !keys.contains(&key) {
                    findings.push(
                        Finding::new(
                            Dimension::Sections,
                            "sections.title-key-missing",
                            Severity::Error,
                            format!("`{key}` is missing from the dictionary of `{locale}`"),
                        )
                        .at(Location {
                            file: Some(file.clone()),
                            locale: Some(locale.clone()),
                            key: Some(key.clone()),
                        }),
                    );
                }
            }
        }
    }

    // The two markers `add -s` needs to install imports and sections: one
    // finding per marker actually missing, none for a complete view.
    for (page, _) in &listed {
        let file = format!("views/{page}/{page}.view.tsx");
        let Ok(content) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        for marker in [IMPORTS_MARKER, SECTIONS_MARKER] {
            if !content.contains(marker) {
                findings.push(
                    Finding::new(
                        Dimension::Sections,
                        "sections.marker-missing",
                        Severity::Error,
                        format!("`{file}` is missing the marker `{marker}`"),
                    )
                    .at(Location {
                        file: Some(file.clone()),
                        ..Location::default()
                    }),
                );
            }
        }
    }

    // A section no view ever imports: files nobody wires into a page. Only
    // the view of the section's own page is consulted; when it cannot be
    // read, its imports are unknown and the page is left alone.
    for (page, sections) in &listed {
        if sections.is_empty() {
            continue;
        }
        let file = format!("views/{page}/{page}.view.tsx");
        let Ok(content) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        let alias = format!("@/views/{page}/sections/");
        let imported: HashSet<String> = imports_of(root, &file, &content)
            .into_iter()
            .filter_map(|import| {
                let rest = import
                    .module
                    .strip_prefix("./sections/")
                    .or_else(|| import.module.strip_prefix(&alias))?;
                Some(rest.strip_suffix(".tsx").unwrap_or(rest).to_string())
            })
            .collect();
        for section in sections {
            if imported.contains(section) {
                continue;
            }
            let section_file = format!("views/{page}/sections/{section}.tsx");
            findings.push(
                Finding::new(
                    Dimension::Sections,
                    "sections.orphan",
                    Severity::Todo,
                    format!("section `{section}` of `{page}` is never imported"),
                )
                .at(Location {
                    file: Some(section_file),
                    ..Location::default()
                }),
            );
        }
    }

    // The files this dimension owns are the sections: every import they
    // make must resolve. Bare packages belong to `dependencies`.
    for (page, sections) in &listed {
        for section in sections {
            let file = format!("views/{page}/sections/{section}.tsx");
            let Ok(content) = fs::read_to_string(root.join(&file)) else {
                continue;
            };
            for import in imports_of(root, &file, &content) {
                if import.is_bare() || import.is_provided() {
                    continue;
                }
                findings.push(
                    Finding::new(
                        Dimension::Sections,
                        "sections.import-broken",
                        Severity::Error,
                        format!("broken import `{}` in `{file}`", import.module),
                    )
                    .detail(format!("no file matches `{}`", import.module))
                    .at(Location {
                        file: Some(file.clone()),
                        ..Location::default()
                    }),
                );
            }
        }
    }
}
