use std::path::Path;

use crate::{
    config::{self, Config},
    dictionary,
    status::{Dimension, Finding, Location, Severity},
};

/// The languages declared in `lib/i18n/config.ts`, in order. An absent or
/// unparsable `export const languages` yields an empty list.
fn listed_languages(content: &str) -> Vec<String> {
    let line = content
        .lines()
        .find(|line| line.starts_with("export const languages"))
        .unwrap_or_default();
    let inside = line
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .unwrap_or_default();
    inside
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// The `defaultLocale` literal declared in `lib/i18n/config.ts`, when present.
fn default_locale(content: &str) -> Option<String> {
    let line = content
        .lines()
        .find(|line| line.starts_with("export const defaultLocale"))?;
    let (_, rest) = line.split_once('"')?;
    let (value, _) = rest.split_once('"')?;
    Some(value.to_string())
}

/// The `direction` entries declared in `lib/i18n/config.ts` as (locale,
/// direction) pairs, in written order. Only `<locale>: "<direction>",` lines
/// inside the `export const direction` object are read.
fn directions(content: &str) -> Vec<(String, String)> {
    let mut lines = content
        .lines()
        .skip_while(|line| !line.starts_with("export const direction"));
    lines.next();
    let mut entries = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.starts_with('}') {
            break;
        }
        let Some((locale, rest)) = line.split_once(':') else {
            continue;
        };
        let locale = locale.trim().trim_matches('"').to_string();
        let direction = rest
            .trim()
            .trim_end_matches(',')
            .trim()
            .trim_matches('"')
            .to_string();
        if !locale.is_empty() && !direction.is_empty() {
            entries.push((locale, direction));
        }
    }
    entries
}

/// The locales imported from `./dictionaries/` in `lib/i18n/index.ts`, in
/// written order.
fn imported_locales(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.starts_with("import") && line.contains("./dictionaries/"))
        .filter_map(|line| {
            let (_, rest) = line.split_once("./dictionaries/")?;
            let (locale, _) = rest.split_once('"')?;
            if locale.is_empty() {
                None
            } else {
                Some(locale.to_string())
            }
        })
        .collect()
}

/// The entries of the `const messages = { ... }` map in `lib/i18n/index.ts`,
/// in written order.
fn mapped_locales(content: &str) -> Vec<String> {
    let Some((_, rest)) = content.split_once("const messages = {") else {
        return Vec::new();
    };
    let Some((map, _)) = rest.split_once('}') else {
        return Vec::new();
    };
    map.split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// The language the `MessageKey` type is derived from in
/// `lib/i18n/index.ts`, read as the identifier after `keyof typeof`.
fn message_key_source(content: &str) -> Option<String> {
    let line = content
        .lines()
        .find(|line| line.starts_with("export type MessageKey"))?;
    let (_, rest) = line.split_once("keyof typeof")?;
    let name = rest
        .trim_start()
        .split(|c: char| c == ';' || c.is_whitespace())
        .find(|part| !part.is_empty())?;
    Some(name.to_string())
}

/// The keys that appear more than once in the parsed entries, each once, in
/// first-occurrence order.
fn duplicate_keys(entries: &[dictionary::Entry]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut duplicates: Vec<String> = Vec::new();
    for entry in entries {
        if seen.contains(&entry.key) && !duplicates.contains(&entry.key) {
            duplicates.push(entry.key.clone());
        }
        if !seen.contains(&entry.key) {
            seen.push(entry.key.clone());
        }
    }
    duplicates
}

/// The direction `aarshi.toml`-side expects for a language: `rtl` or `ltr`.
fn expected_direction(language: &str) -> &'static str {
    if config::is_rtl(language) {
        "rtl"
    } else {
        "ltr"
    }
}

/// Collects the i18n findings: config drift and dictionaries, with the
/// dictionary of `languages[0]` as the reference for keys.
///
/// No early stop: the whole dimension is walked whatever was already found.
pub fn collect(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    collect_config(root, config, findings);
    collect_index(root, config, findings);

    let default = &config.languages[0];
    // The default dictionary is the reference; unreadable, it becomes an
    // empty reference and slips to the `missing dictionary` case below.
    let default_keys: Vec<String> = std::fs::read_to_string(dictionary::file(root, default))
        .ok()
        .map(|content| {
            dictionary::entries(&content)
                .into_iter()
                .map(|entry| entry.key)
                .collect()
        })
        .unwrap_or_default();

    for locale in &config.languages {
        let path = dictionary::file(root, locale);
        let file = format!("lib/i18n/dictionaries/{locale}.ts");
        let Some(content) = std::fs::read_to_string(&path).ok() else {
            // An absent (or unreadable) dictionary: one error, no key checks.
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.dictionary-missing",
                severity: Severity::Error,
                summary: format!("no dictionary for `{locale}`"),
                detail: Some(format!("expected {file}")),
                location: Some(Location {
                    file: Some(file),
                    locale: Some(locale.clone()),
                    key: None,
                }),
            });
            continue;
        };
        let entries = dictionary::entries(&content);

        for key in duplicate_keys(&entries) {
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.key-duplicate",
                severity: Severity::Error,
                summary: format!("{locale} has a duplicate entry for `{key}`"),
                detail: None,
                location: Some(Location {
                    file: Some(file.clone()),
                    locale: Some(locale.clone()),
                    key: Some(key),
                }),
            });
        }

        for entry in entries.iter().filter(|entry| dictionary::is_empty(entry)) {
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.key-empty",
                severity: Severity::Todo,
                summary: format!("{locale} has an empty translation for `{}`", entry.key),
                detail: None,
                location: Some(Location {
                    file: Some(file.clone()),
                    locale: Some(locale.clone()),
                    key: Some(entry.key.clone()),
                }),
            });
        }

        if locale != default {
            let mut reported_extras: Vec<String> = Vec::new();
            for entry in &entries {
                if default_keys.contains(&entry.key) || reported_extras.contains(&entry.key) {
                    continue;
                }
                reported_extras.push(entry.key.clone());
                findings.push(Finding {
                    dimension: Dimension::I18n,
                    rule: "i18n.key-extra",
                    severity: Severity::Todo,
                    summary: format!("{locale} has an extra translation for `{}`", entry.key),
                    detail: Some(format!("reference dictionary `{default}`")),
                    location: Some(Location {
                        file: Some(file.clone()),
                        locale: Some(locale.clone()),
                        key: Some(entry.key.clone()),
                    }),
                });
            }
            for key in &default_keys {
                if !entries.iter().any(|entry| &entry.key == key) {
                    findings.push(Finding {
                        dimension: Dimension::I18n,
                        rule: "i18n.key-missing",
                        severity: Severity::Error,
                        summary: format!("{locale} is missing a translation for `{key}`"),
                        detail: Some(format!("reference dictionary `{default}`")),
                        location: Some(Location {
                            file: Some(file.clone()),
                            locale: Some(locale.clone()),
                            key: Some(key.clone()),
                        }),
                    });
                }
            }
        }
    }

    collect_orphans(root, config, findings);
}

/// Checks `lib/i18n/config.ts` against `aarshi.toml`: the languages list,
/// `defaultLocale` and the `direction` table, all compared textually. An
/// unreadable file counts as one error and never stops the other checks.
fn collect_config(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    let file = "lib/i18n/config.ts";
    let Ok(content) = std::fs::read_to_string(root.join(file)) else {
        findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.languages-mismatch",
            severity: Severity::Error,
            summary: format!("{file} is missing or unreadable"),
            detail: Some(format!("expected {}", quoted(&config.languages))),
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        });
        return;
    };

    let listed = listed_languages(&content);
    if !listed.is_empty() && listed != config.languages {
        findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.languages-mismatch",
            severity: Severity::Error,
            summary: format!(
                "lib/i18n/config.ts lists {}, but aarshi.toml lists {}",
                quoted(&listed),
                quoted(&config.languages)
            ),
            detail: Some(format!("expected {}", quoted(&config.languages))),
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        });
    }

    match default_locale(&content) {
        None => findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.default-locale-mismatch",
            severity: Severity::Error,
            summary: format!("{file} has no defaultLocale"),
            detail: Some(format!("expected `{}`", config.languages[0])),
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        }),
        Some(locale) if locale != config.languages[0] => findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.default-locale-mismatch",
            severity: Severity::Error,
            summary: format!(
                "{file} defaults to `{locale}`, but aarshi.toml defaults to `{}`",
                config.languages[0]
            ),
            detail: Some(format!("expected `{}`", config.languages[0])),
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        }),
        _ => {}
    }

    let table = directions(&content);
    for language in &config.languages {
        let expected = expected_direction(language);
        match table.iter().find(|(locale, _)| locale == language) {
            None => findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.direction-mismatch",
                severity: Severity::Error,
                summary: format!("{language} is missing a direction entry"),
                detail: Some(format!("expected `{expected}`")),
                location: Some(Location {
                    file: Some(file.to_string()),
                    locale: Some(language.clone()),
                    key: None,
                }),
            }),
            Some((_, declared)) if declared != expected => findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.direction-mismatch",
                severity: Severity::Error,
                summary: format!("{language} has direction `{declared}` instead of `{expected}`"),
                detail: None,
                location: Some(Location {
                    file: Some(file.to_string()),
                    locale: Some(language.clone()),
                    key: None,
                }),
            }),
            _ => {}
        }
    }
}

/// Checks `lib/i18n/index.ts` against `aarshi.toml`: one import and one map
/// entry per language, and `MessageKey` derived from the default language.
fn collect_index(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    let file = "lib/i18n/index.ts";
    let Ok(content) = std::fs::read_to_string(root.join(file)) else {
        findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.index-mismatch",
            severity: Severity::Error,
            summary: format!("{file} is missing or unreadable"),
            detail: None,
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        });
        return;
    };

    let imported = imported_locales(&content);
    for locale in &config.languages {
        if !imported.iter().any(|name| name == locale) {
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.index-mismatch",
                severity: Severity::Error,
                summary: format!("{file} has no import for `{locale}`"),
                detail: Some(format!(
                    "expected import {{ messages as {locale} }} from \"./dictionaries/{locale}\";"
                )),
                location: Some(Location {
                    file: Some(file.to_string()),
                    locale: Some(locale.clone()),
                    key: None,
                }),
            });
        }
    }
    for locale in &imported {
        if !config.languages.contains(locale) {
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.index-mismatch",
                severity: Severity::Error,
                summary: format!("{file} has an extra import `{locale}`"),
                detail: None,
                location: Some(Location {
                    file: Some(file.to_string()),
                    locale: Some(locale.clone()),
                    key: None,
                }),
            });
        }
    }

    let mapped = mapped_locales(&content);
    for locale in &config.languages {
        if !mapped.iter().any(|name| name == locale) {
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.index-mismatch",
                severity: Severity::Error,
                summary: format!("{file} has no map entry for `{locale}`"),
                detail: None,
                location: Some(Location {
                    file: Some(file.to_string()),
                    locale: Some(locale.clone()),
                    key: None,
                }),
            });
        }
    }
    for locale in &mapped {
        if !config.languages.contains(locale) {
            findings.push(Finding {
                dimension: Dimension::I18n,
                rule: "i18n.index-mismatch",
                severity: Severity::Error,
                summary: format!("{file} has an extra map entry `{locale}`"),
                detail: None,
                location: Some(Location {
                    file: Some(file.to_string()),
                    locale: Some(locale.clone()),
                    key: None,
                }),
            });
        }
    }

    let default = &config.languages[0];
    match message_key_source(&content) {
        None => findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.index-mismatch",
            severity: Severity::Error,
            summary: format!("{file} has no MessageKey type"),
            detail: Some(format!("expected `keyof typeof {default}`")),
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        }),
        Some(source) if &source != default => findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.index-mismatch",
            severity: Severity::Error,
            summary: format!(
                "{file} derives MessageKey from `{source}`, but the default language is `{default}`"
            ),
            detail: None,
            location: Some(Location {
                file: Some(file.to_string()),
                ..Location::default()
            }),
        }),
        _ => {}
    }
}

/// Finds `<locale>.ts` dictionary files for languages outside `aarshi.toml`.
fn collect_orphans(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    let Ok(directory) = std::fs::read_dir(root.join("lib/i18n/dictionaries")) else {
        return;
    };
    let mut paths: Vec<_> = directory
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    for path in paths {
        if path.extension().and_then(|extension| extension.to_str()) != Some("ts") {
            continue;
        }
        let Some(locale) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if config.languages.iter().any(|language| language == locale) {
            continue;
        }
        findings.push(Finding {
            dimension: Dimension::I18n,
            rule: "i18n.dictionary-orphan",
            severity: Severity::Todo,
            summary: format!("no language for the `{locale}` dictionary"),
            detail: None,
            location: Some(Location {
                file: Some(format!("lib/i18n/dictionaries/{locale}.ts")),
                locale: Some(locale.to_string()),
                key: None,
            }),
        });
    }
}

/// Formats a language list as a bare array, e.g. `["fr", "de"]`.
fn quoted(languages: &[String]) -> String {
    format!(
        "[{}]",
        languages
            .iter()
            .map(|l| format!("\"{l}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
