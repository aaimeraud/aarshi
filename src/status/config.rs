use std::path::Path;

use super::imports::{Target, imports_of};
use crate::{
    config::{Config, ICONS_LUCIDE, UI_BASE},
    status::{Dimension, Finding, Location, Severity},
    templates::PAGES_MARKER,
};

/// The base files of a generated project (catalogue 0008): one
/// `config.file-missing` finding per absent entry.
const BASE_FILES: &[&str] = &[
    "app/[locale]/layout.tsx",
    "app/sitemap.ts",
    "app/robots.ts",
    "lib/site.ts",
    "lib/i18n/config.ts",
    "lib/i18n/index.ts",
    "components/language-switcher.tsx",
    "tsconfig.json",
    "eslint.config.mjs",
    "next.config.ts",
];

/// The PostCSS configs that can wire Tailwind, in the order Next.js looks for them.
const POSTCSS_CONFIGS: &[&str] = &[
    "postcss.config.mjs",
    "postcss.config.js",
    "postcss.config.cjs",
    "postcss.config.ts",
];

/// The base folders; `.gitkeep` is not required, the folder itself is. The
/// trailing slash keeps the finding about a folder, not a file.
const BASE_DIRS: &[&str] = &["components/custom/", "components/icons/"];

/// The keys `setup` writes in `aarshi.toml` since the extended format; a
/// project generated before the extension misses them all.
const PERSISTED_KEYS: &[&str] = &["ui", "icons", "cache_components", "site_url"];

/// The component files every UI flavour ships in `components/ui/`.
const UI_COMPONENTS: &[&str] = &["components/ui/button.tsx", "components/ui/button-group.tsx"];

/// The dependency each option requires to be declared in `package.json`.
const UI_PACKAGE: &str = "@base-ui/react";
const ICONS_PACKAGE: &str = "lucide-react";

/// Files of the base inventory reported through the dimension that owns
/// their content (rule 0013, applied by the 0030 recipe): a missing or
/// unreadable `lib/i18n/config.ts` is reported as `i18n.languages-mismatch`
/// and `lib/i18n/index.ts` as `i18n.index-mismatch`, so `config.file-missing`
/// stays silent about them and each object is reported exactly once. They
/// stay in the inventory above, and the imports of a gone file are still
/// absorbed by the missing list.
const SELF_REPORTED: &[&str] = &["lib/i18n/config.ts", "lib/i18n/index.ts"];

/// Collects the `config` findings: base files and folders, the pages marker
/// of `app/sitemap.ts`, the root redirect, the persisted keys of
/// `aarshi.toml` and the options checked against the project.
///
/// No early stop: the whole dimension is walked whatever was already found.
pub fn collect(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    let mut missing: Vec<&'static str> = Vec::new();
    for entry in BASE_FILES.iter().copied().chain(BASE_DIRS.iter().copied()) {
        let path = entry.trim_end_matches('/');
        if !root.join(path).is_dir() && !root.join(path).is_file() {
            missing.push(entry);
        }
    }
    for file in &missing {
        if SELF_REPORTED.contains(file) {
            continue;
        }
        findings.push(Finding {
            dimension: Dimension::Config,
            rule: "config.file-missing",
            severity: Severity::Error,
            summary: format!("missing `{file}`"),
            detail: None,
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        });
    }

    // An unreadable file absorbs its checks: an `error` says checked then wrong.
    let next_config = read(root.join("next.config.ts"));
    let site = read(root.join("lib/site.ts"));
    let sitemap = read(root.join("app/sitemap.ts"));
    let package = read(root.join("package.json"));
    let project_file = read(root.join("aarshi.toml"));

    if let Some(content) = &project_file {
        for key in PERSISTED_KEYS {
            if !has_key(content, key) {
                findings.push(Finding {
                    dimension: Dimension::Config,
                    rule: "config.project-file-incomplete",
                    severity: Severity::Todo,
                    summary: format!("aarshi.toml is missing the `{key}` key"),
                    detail: None,
                    location: Some(Location {
                        file: Some("aarshi.toml".to_string()),
                        key: Some((*key).to_string()),
                        ..Location::default()
                    }),
                });
            }
        }
    }

    if let Some(content) = &sitemap
        && !content.contains(PAGES_MARKER)
    {
        findings.push(Finding {
            dimension: Dimension::Config,
            rule: "config.marker-missing",
            severity: Severity::Error,
            summary: format!("app/sitemap.ts has no `// {PAGES_MARKER}` marker"),
            detail: Some(format!("expected // {PAGES_MARKER}")),
            location: Some(Location {
                file: Some("app/sitemap.ts".to_string()),
                ..Location::default()
            }),
        });
    }

    if let Some(content) = &next_config {
        check_redirect(content, config, findings);
        check_cache_components(content, config, findings);
        check_tailwind(root, content, findings);
    }

    if let Some(content) = &site {
        check_site_url(content, config, findings);
    }

    for file in UI_COMPONENTS {
        if !root.join(file).exists() {
            findings.push(Finding {
                dimension: Dimension::Config,
                rule: "config.option-ui",
                severity: Severity::Error,
                summary: format!("missing `{file}`"),
                detail: Some(format!("required when ui = `{}`", config.ui)),
                location: Some(Location {
                    file: Some((*file).to_string()),
                    key: Some("ui".to_string()),
                    ..Location::default()
                }),
            });
        }
    }
    if let Some(content) = &package {
        if config.ui == UI_BASE && !content.contains(&format!("\"{UI_PACKAGE}\"")) {
            findings.push(Finding {
                dimension: Dimension::Config,
                rule: "config.option-ui",
                severity: Severity::Error,
                summary: format!("package.json does not declare `{UI_PACKAGE}`"),
                detail: Some(format!("required when ui = `{}`", config.ui)),
                location: Some(Location {
                    file: Some("package.json".to_string()),
                    key: Some("ui".to_string()),
                    ..Location::default()
                }),
            });
        }
        if config.icons == ICONS_LUCIDE && !content.contains(&format!("\"{ICONS_PACKAGE}\"")) {
            findings.push(Finding {
                dimension: Dimension::Config,
                rule: "config.option-icons",
                severity: Severity::Error,
                summary: format!("package.json does not declare `{ICONS_PACKAGE}`"),
                detail: Some(format!("required when icons = `{ICONS_LUCIDE}`")),
                location: Some(Location {
                    file: Some("package.json".to_string()),
                    key: Some("icons".to_string()),
                    ..Location::default()
                }),
            });
        }
    }

    for file in BASE_FILES {
        let Ok(content) = std::fs::read_to_string(root.join(file)) else {
            continue;
        };
        check_imports(root, file, &content, &missing, findings);
    }
}

/// Tailwind is wired either by the Turbopack loader in `next.config.ts` or by a
/// PostCSS config that names `@tailwindcss/postcss`.
fn check_tailwind(root: &Path, next_config: &str, findings: &mut Vec<Finding>) {
    if next_config.contains("@tailwindcss/turbopack") {
        return;
    }
    let postcss = POSTCSS_CONFIGS.iter().any(|file| {
        read(root.join(file)).is_some_and(|content| content.contains("@tailwindcss/postcss"))
    });
    if postcss {
        return;
    }
    findings.push(Finding {
        dimension: Dimension::Config,
        rule: "config.tailwind-missing",
        severity: Severity::Error,
        summary: "Tailwind is not wired".to_string(),
        detail: Some(
            "add `@tailwindcss/turbopack` to next.config.ts, or a postcss.config.* that uses `@tailwindcss/postcss`"
                .to_string(),
        ),
        location: Some(Location {
            file: Some("next.config.ts".to_string()),
            ..Location::default()
        }),
    });
}

/// Reads a file, swallowing any error so the report never stops.
fn read(path: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Whether `aarshi.toml` sets the key, parsed loosely: a line starts with
/// the key and carries whatever spaces around its `=`.
fn has_key(content: &str, key: &str) -> bool {
    content.lines().any(|line| {
        line.trim()
            .strip_prefix(key)
            .is_some_and(|rest| rest.trim_start().starts_with('='))
    })
}

/// The destination of the redirect whose `source` is `"/"`, as written in
/// `next.config.ts`; `None` when there is no such redirect.
fn slash_redirect(content: &str) -> Option<String> {
    let entry = content.find("source: \"/\"")?;
    let head = content[entry..].find('}')?;
    let body = &content[entry..entry + head];
    let end = body.find("destination: \"")? + "destination: \"".len();
    let rest = &body[end..];
    let value = rest.find('"')?;
    Some(rest[..value].to_string())
}

/// The redirect `"/"` must send to the first language of `aarshi.toml`.
fn check_redirect(content: &str, config: &Config, findings: &mut Vec<Finding>) {
    let expected = format!("/{}", config.languages[0]);
    let location = Location {
        file: Some("next.config.ts".to_string()),
        ..Location::default()
    };
    match slash_redirect(content) {
        Some(destination) if destination == expected => {}
        Some(destination) => findings.push(Finding {
            dimension: Dimension::Config,
            rule: "config.next-config-redirect",
            severity: Severity::Error,
            summary: format!(
                "next.config.ts redirects \"/\" to \"{destination}\", expected \"{expected}\""
            ),
            detail: None,
            location: Some(location),
        }),
        None => findings.push(Finding {
            dimension: Dimension::Config,
            rule: "config.next-config-redirect",
            severity: Severity::Error,
            summary: "next.config.ts has no redirect for \"/\"".to_string(),
            detail: Some(format!("expected \"{expected}\"")),
            location: Some(location),
        }),
    }
}

/// `next.config.ts` must reflect the `cache_components` of `aarshi.toml`.
fn check_cache_components(content: &str, config: &Config, findings: &mut Vec<Finding>) {
    let enabled = content.contains("cacheComponents: true");
    if enabled == config.cache_components {
        return;
    }
    let (summary, expected) = if config.cache_components {
        ("next.config.ts does not enable `cacheComponents`", "true")
    } else {
        ("next.config.ts enables `cacheComponents`", "false")
    };
    findings.push(Finding {
        dimension: Dimension::Config,
        rule: "config.option-cache-components",
        severity: Severity::Error,
        summary: summary.to_string(),
        detail: Some(format!("expected cacheComponents = {expected}")),
        location: Some(Location {
            file: Some("next.config.ts".to_string()),
            key: Some("cache_components".to_string()),
            ..Location::default()
        }),
    });
}

/// The `SITE_URL` of `lib/site.ts` must match `aarshi.toml`.
fn check_site_url(content: &str, config: &Config, findings: &mut Vec<Finding>) {
    let Some(line) = content.lines().find(|line| line.contains("SITE_URL")) else {
        return;
    };
    let quoted = line
        .split_once('"')
        .and_then(|(_, rest)| rest.split_once('"'));
    let Some((declared, _)) = quoted else {
        return;
    };
    if declared == config.site_url {
        return;
    }
    findings.push(Finding {
        dimension: Dimension::Config,
        rule: "config.option-site-url",
        severity: Severity::Error,
        summary: format!(
            "lib/site.ts sets SITE_URL to \"{declared}\", expected \"{}\"",
            config.site_url
        ),
        detail: None,
        location: Some(Location {
            file: Some("lib/site.ts".to_string()),
            key: Some("site_url".to_string()),
            ..Location::default()
        }),
    });
}

/// One `config.import-broken` finding per import of a base file that no
/// local file provides. Package imports belong to the `dependencies`
/// dimension, `components/` imports to the `components` one, asset imports
/// stay out of the textual reach of v1, and an import reaching a missing
/// base file is already reported by `config.file-missing`.
fn check_imports(
    root: &Path,
    file: &str,
    content: &str,
    missing: &[&str],
    findings: &mut Vec<Finding>,
) {
    for import in imports_of(root, file, content) {
        let broken = match &import.target {
            Target::Local { candidates, found } => {
                // The `components` masters resolve the imports of their files.
                let owned = candidates[0].starts_with("components");
                // A missing base file is already reported by `config.file-missing`.
                let already_missing = candidates
                    .iter()
                    .any(|path| path.to_str().is_some_and(|path| missing.contains(&path)));
                found.is_none() && !owned && !already_missing
            }
            Target::Escapes => true,
            Target::Asset { .. } | Target::Bare => false,
        };
        if !broken {
            continue;
        }
        findings.push(Finding {
            dimension: Dimension::Config,
            rule: "config.import-broken",
            severity: Severity::Error,
            summary: format!(
                "{file} imports `{}`, but the file does not exist",
                import.module
            ),
            detail: None,
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        });
    }
}
