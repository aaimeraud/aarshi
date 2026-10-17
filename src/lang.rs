use std::env;

use anyhow::{Result, bail};
use inquire::Text;

use crate::{
    add,
    config::{self, Config, PROJECT_CONFIG_FILE},
    dictionary, project,
    templates::{self, MESSAGES_MARKER},
};

/// Adds a language to an existing project: dictionary, routes, layout and config.
pub fn run(code: Option<String>) -> Result<()> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    let config = Config::for_project(&root)?;
    let code = match code {
        Some(code) => code,
        None => Text::new("Language code (e.g. de)").prompt()?,
    };
    config::validate_language(&code)?;
    if config.languages.contains(&code) {
        bail!("language `{code}` already exists");
    }

    let default = config.languages[0].clone();
    let default_dictionary = project::read_file(&dictionary::file(&root, &default))?;
    let keys: Vec<String> = dictionary::entries(&default_dictionary)
        .into_iter()
        .map(|entry| entry.key)
        .collect();
    let pages = dictionary::pages(&root)?;
    let mut languages = config.languages.clone();
    languages.push(code.clone());

    project::create_file(
        &dictionary::file(&root, &code),
        &templates::dictionary(&code, &default),
    )?;
    for key in &keys {
        project::update_file(&dictionary::file(&root, &code), |content| {
            project::insert_before_marker(content, MESSAGES_MARKER, &format!("\"{key}\": \"\","))
        })?;
    }
    for page in &pages {
        let route_path = add::route_path(page);
        project::create_file(
            &add::route_file(&root, &code, &default, &route_path),
            &templates::route(&code, page, &route_path),
        )?;
    }
    project::create_file(
        &add::layout_file(&root, &code, &default),
        &templates::layout(&code),
    )?;

    std::fs::write(
        root.join("lib/i18n/config.ts"),
        templates::i18n_config(&languages),
    )?;
    std::fs::write(
        root.join("lib/i18n/index.ts"),
        templates::i18n_index(&languages),
    )?;
    let project_config = Config {
        languages,
        package_manager: config.package_manager,
    };
    std::fs::write(
        root.join(PROJECT_CONFIG_FILE),
        project_config.to_project_file(),
    )?;

    println!(
        "Added `{code}`. Fill the empty values in lib/i18n/dictionaries/{code}.ts, then run `aarshi check`."
    );
    Ok(())
}
