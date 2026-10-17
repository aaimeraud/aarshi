use std::env;

use anyhow::{Result, bail};

use crate::{
    add,
    config::{self, Config},
    dictionary, project,
};

/// Reports empty translations, missing keys, missing routes and config drift. Changes nothing.
pub fn run() -> Result<()> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    let config = Config::for_project(&root)?;
    let default = config.languages[0].clone();
    let mut problems = Vec::new();

    let default_keys: Vec<String> =
        dictionary::entries(&project::read_file(&dictionary::file(&root, &default))?)
            .into_iter()
            .map(|entry| entry.key)
            .collect();

    let listed = listed_languages(&root)?;
    if listed != config.languages {
        problems.push(format!(
            "lib/i18n/config.ts lists {listed:?}, but aarshi.toml lists {:?}",
            config.languages
        ));
    }

    for locale in &config.languages {
        let path = dictionary::file(&root, locale);
        if !path.exists() {
            problems.push(format!(
                "{locale}: missing lib/i18n/dictionaries/{locale}.ts"
            ));
            continue;
        }
        let entries = dictionary::entries(&project::read_file(&path)?);
        for entry in entries.iter().filter(|entry| dictionary::is_empty(entry)) {
            problems.push(format!("{locale}: empty translation for `{}`", entry.key));
        }
        for key in &default_keys {
            if !entries.iter().any(|entry| &entry.key == key) {
                problems.push(format!("{locale}: missing key `{key}`"));
            }
        }
    }

    for page in dictionary::pages(&root)? {
        let route_path = add::route_path(&page);
        if !add::route_file(&root, &route_path).exists() {
            problems.push(format!("missing route for `{route_path}`"));
        }
    }

    if problems.is_empty() {
        println!("All translations are filled.");
        return Ok(());
    }
    for problem in &problems {
        println!("- {problem}");
    }
    bail!("{} problem(s) found", problems.len())
}

/// The languages declared in `lib/i18n/config.ts`, in order.
fn listed_languages(root: &std::path::Path) -> Result<Vec<String>> {
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
