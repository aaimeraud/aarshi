//! The `dependencies` dimension: an npm package imported in the known files
//! but not declared in `package.json`, a lockfile inconsistent with
//! `package_manager`, and a missing `node_modules/`.

use std::{fs, path::Path};

use serde_json::Value;

use super::pages::{imports, is_bare, routes, sections, views};
use crate::{
    config::Config,
    status::{Dimension, Finding, Location, Severity},
};

/// The `package.json` sections whose keys count as a declaration.
const DECLARING_SECTIONS: [&str; 2] = ["dependencies", "devDependencies"];

/// The lockfile(s) each manager owns, in scan order; `bun` may carry the
/// legacy binary file or the text one.
const LOCKFILES: &[(&str, &str)] = &[
    ("package-lock.json", "npm"),
    ("pnpm-lock.yaml", "pnpm"),
    ("yarn.lock", "yarn"),
    ("bun.lockb", "bun"),
    ("bun.lock", "bun"),
];

/// One imported npm package kept with the known files that import it.
struct Package {
    name: String,
    importers: Vec<String>,
}

/// Collects the `dependencies` findings. The known-file inventory covers the
/// dimensions implemented so far (`config` base files, the dictionaries of
/// the declared languages, routes, views and their sections) and widens with
/// each ticket. A bare import always lands here, whatever dimension owns the
/// file: the object is the dependency, not the file. Nothing stops early.
pub fn collect(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    // Without a readable `package.json` there is nothing to compare imports
    // or lockfiles with: the whole dimension is absorbed, never a stop.
    let Some(declared) = declared_packages(root) else {
        return;
    };
    report_packages(root, config, &declared, findings);
    report_lockfiles(root, config, findings);
    if !root.join("node_modules").is_dir() {
        findings.push(Finding {
            dimension: Dimension::Dependencies,
            rule: "dependencies.modules-missing",
            severity: Severity::Todo,
            summary: "`node_modules/` is missing".to_string(),
            detail: Some(format!("run {} install", config.package_manager)),
            location: Some(Location {
                file: Some("node_modules/".to_string()),
                ..Location::default()
            }),
        });
    }
}

/// The packages declared in `package.json` (`dependencies` and
/// `devDependencies`). `None` when the file is absent or not valid JSON.
fn declared_packages(root: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(root.join("package.json")).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let mut names = Vec::new();
    for section in DECLARING_SECTIONS {
        if let Some(entries) = value.get(section).and_then(Value::as_object) {
            names.extend(entries.keys().map(|name| name.to_string()));
        }
    }
    Some(names)
}

/// Checks the files of every dimension for bare imports, then one `error`
/// per undeclared package (sorted by name), with the importers as detail.
/// The declared set covers both sections.
fn report_packages(root: &Path, config: &Config, declared: &[String], findings: &mut Vec<Finding>) {
    let mut packages: Vec<Package> = Vec::new();
    for file in known_files(root, config) {
        let Ok(content) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        for import in imports(&content) {
            // `@/x`, `./x` and `../x` name a file another dimension checks;
            // anything else names an npm package.
            if !is_bare(&import.module) {
                continue;
            }
            let name = package_name(&import.module);
            if declared.iter().any(|declared| declared == &name) {
                continue;
            }
            let Some(package) = packages.iter_mut().find(|p| p.name == name) else {
                packages.push(Package {
                    name,
                    importers: vec![file.clone()],
                });
                continue;
            };
            if !package.importers.contains(&file) {
                package.importers.push(file.clone());
            }
        }
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    for package in &mut packages {
        package.importers.sort();
        findings.push(Finding {
            dimension: Dimension::Dependencies,
            rule: "dependencies.package-missing",
            severity: Severity::Error,
            summary: format!("`{}` is not declared in package.json", package.name),
            detail: Some(format!("imported in {}", package.importers.join(", "))),
            location: Some(Location {
                file: Some("package.json".to_string()),
                ..Location::default()
            }),
        });
    }
}

/// The known files whose imports the scan covers, extended with each
/// implemented dimension: the `config` base files, the dictionaries of the
/// declared languages, then routes, views and their sections.
fn known_files(root: &Path, config: &Config) -> Vec<String> {
    const BASE: [&str; 8] = [
        "app/[locale]/layout.tsx",
        "app/sitemap.ts",
        "app/robots.ts",
        "lib/site.ts",
        "lib/i18n/config.ts",
        "lib/i18n/index.ts",
        "components/language-switcher.tsx",
        "next.config.ts",
    ];
    let mut files: Vec<String> = BASE.iter().map(|file| file.to_string()).collect();
    for locale in &config.languages {
        files.push(format!("lib/i18n/dictionaries/{locale}.ts"));
    }
    for route in routes(root) {
        files.push(route.file);
    }
    for page in views(root) {
        files.push(format!("views/{page}/{page}.view.tsx"));
        for section in sections(root, &page) {
            files.push(format!("views/{page}/sections/{section}.tsx"));
        }
    }
    files
}

/// The npm package a bare module belongs to: `next/navigation` comes from
/// `next`, `@base-ui/react/button` from `@base-ui/react`.
fn package_name(module: &str) -> String {
    let mut segments = module.split('/');
    let first = segments.next().unwrap_or(module);
    if first.starts_with('@') {
        match segments.next() {
            Some(name) => format!("{first}/{name}"),
            None => first.to_string(),
        }
    } else {
        first.to_string()
    }
}

/// One `error` per lockfile whose manager disagrees with `package_manager`.
/// A missing lockfile is not a finding of this dimension.
fn report_lockfiles(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    for (lockfile, manager) in LOCKFILES {
        let lockfile = *lockfile;
        if !root.join(lockfile).is_file() || *manager == config.package_manager {
            continue;
        }
        findings.push(Finding {
            dimension: Dimension::Dependencies,
            rule: "dependencies.lockfile-mismatch",
            severity: Severity::Error,
            summary: format!(
                "`{lockfile}` is present, but aarshi.toml declares the `{}` package manager",
                config.package_manager
            ),
            detail: Some(format!("expected {}", expected_lockfile(&config.package_manager))),
            location: Some(Location {
                file: Some(lockfile.to_string()),
                ..Location::default()
            }),
        });
    }
}

/// The lockfile(s) `package_manager` should carry, for the `detail`.
fn expected_lockfile(manager: &str) -> &'static str {
    match manager {
        "pnpm" => "pnpm-lock.yaml",
        "yarn" => "yarn.lock",
        "bun" => "bun.lockb or bun.lock",
        _ => "package-lock.json",
    }
}
