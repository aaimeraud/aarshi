//! The `components` dimension: icons waiting for their generated component
//! and broken imports inside the files of `components/`. The resolution of
//! the known-file imports is classified in the dimension of the faulty
//! object — `pages` walks the views and routes (0026), `sections` the
//! sections (0027), `config` the base files (0024) — and the unused custom
//! components are not an invariant, so this dimension only examines the
//! imports its own files make.

use std::{fs, path::Path};

use super::imports::imports_of;
use crate::{
    config::Config,
    status::{Dimension, Finding, Location, Severity},
};

/// Every icon of `components/icons/` that still waits for its generated
/// component: one `<name>.svg` whose `<name>.tsx` does not exist, sorted. A
/// `.tsx` without `.svg` is ignored — the SVG left after the conversion —
/// and an absent or unreadable folder yields an empty list, never a stop.
fn unconverted_icons(root: &Path) -> Vec<String> {
    let entries = match fs::read_dir(root.join("components/icons")) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut pending = Vec::new();
    for entry in entries.flatten() {
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_file || !name.ends_with(".svg") {
            continue;
        }
        let stem = name.trim_end_matches(".svg");
        let component = root.join("components/icons").join(format!("{stem}.tsx"));
        if component.exists() {
            continue;
        }
        pending.push(stem.to_string());
    }
    pending.sort();
    pending
}

/// The files whose broken imports belong to this dimension: the custom
/// components and the generated icon components of `components/`, sorted.
/// `components/language-switcher.tsx` is a base file, so `config` (0024)
/// already walks its imports, and the `components/ui/` components ship with
/// the project — neither is taken over here.
fn owned_files(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for dir in ["components/custom", "components/icons"] {
        let entries = match fs::read_dir(root.join(dir)) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_file && name.ends_with(".tsx") {
                files.push(format!("{dir}/{name}"));
            }
        }
    }
    files.sort();
    files
}

/// Collects the `components` findings: one `todo` per unconverted icon and
/// one `error` per import of a custom or icon component no file provides.
/// Bare packages are left to the `dependencies` dimension and nothing stops
/// the walk early.
pub fn collect(root: &Path, _config: &Config, findings: &mut Vec<Finding>) {
    for component in unconverted_icons(root) {
        let file = format!("components/icons/{component}.svg");
        findings.push(Finding {
            dimension: Dimension::Components,
            rule: "components.icon-unconverted",
            severity: Severity::Todo,
            summary: format!("`{file}` has no generated component"),
            detail: Some(format!("expected components/icons/{component}.tsx")),
            location: Some(Location {
                file: Some(file),
                ..Location::default()
            }),
        });
    }

    for file in owned_files(root) {
        let Ok(content) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        for import in imports_of(root, &file, &content) {
            if import.is_bare() || import.is_provided() {
                continue;
            }
            findings.push(Finding {
                dimension: Dimension::Components,
                rule: "components.import-broken",
                severity: Severity::Error,
                summary: format!("broken import `{}` in `{file}`", import.module),
                detail: Some(format!("no file matches `{}`", import.module)),
                location: Some(Location {
                    file: Some(file.clone()),
                    ..Location::default()
                }),
            });
        }
    }
}
