use std::path::Path;

use crate::{
    config::Config,
    dictionary,
    project,
    status::{Dimension, Finding, Location, Severity},
};

/// The languages declared in `lib/i18n/config.ts`, in order. An unreadable or
/// absent file yields an empty list: its dimension is `config`, not `i18n`.
fn listed_languages(root: &Path) -> anyhow::Result<Vec<String>> {
    let content = project::read_file(&root.join("lib/i18n/config.ts"))?;
    let line = content
        .lines()
        .find(|line| line.starts_with("export const languages"))
        .unwrap_or_default();
    let inside = line
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .unwrap_or_default();
    Ok(inside
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect())
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

/// Collects the i18n findings: config drift and dictionaries, with the
/// dictionary of `languages[0]` as the reference for keys.
///
/// No early stop: the whole dimension is walked whatever was already found.
pub fn collect(root: &Path, config: &Config, findings: &mut Vec<Finding>) {
    if let Ok(listed) = listed_languages(root) {
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
                    file: Some("lib/i18n/config.ts".to_string()),
                    ..Location::default()
                }),
            });
        }
    }

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
}
