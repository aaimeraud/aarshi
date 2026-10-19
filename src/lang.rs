use std::env;

use anyhow::{Result, bail};
use inquire::Text;

use crate::{
    config::{self, Config, PROJECT_CONFIG_FILE},
    dictionary, output, project,
    templates::{self, MESSAGES_MARKER},
};

/// Ask for a language code and keep asking until it is valid.
fn prompt_language_code() -> Result<String> {
    loop {
        let code = Text::new("Language code (e.g. de)").prompt()?;
        let code = code.trim().to_ascii_lowercase();
        match config::validate_language(&code) {
            Ok(()) => return Ok(code),
            Err(error) => eprintln!("{}", output::warning(&error.to_string())),
        }
    }
}

/// Adds a language to an existing project: dictionary, routes, layout and config.
pub fn run(code: Option<String>) -> Result<()> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    let config = Config::for_project(&root)?;
    let code = match code {
        Some(code) => code.to_ascii_lowercase(),
        None => prompt_language_code()?,
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
    let mut languages = config.languages.clone();
    languages.push(code.clone());

    project::create_file(
        &dictionary::file(&root, &code),
        &templates::dictionary(&code, &default),
    )?;
    project::update_file(&dictionary::file(&root, &code), |content| {
        keys.iter().try_fold(content.to_string(), |content, key| {
            project::insert_before_marker(&content, MESSAGES_MARKER, &format!("\"{key}\": \"\","))
        })
    })?;
    std::fs::write(
        root.join("lib/i18n/config.ts"),
        templates::i18n_config(&languages),
    )?;
    std::fs::write(
        root.join("lib/i18n/index.ts"),
        templates::i18n_index(&languages),
    )?;
    let mut project_config = config;
    project_config.languages = languages;
    std::fs::write(
        root.join(PROJECT_CONFIG_FILE),
        project_config.to_project_file(),
    )?;

    println!("{}", output::success(&format!("language `{code}` added")));
    println!(
        "{}",
        output::info(&format!(
            "fill the empty values in lib/i18n/dictionaries/{code}.ts, then run `aarshi check`"
        ))
    );
    Ok(())
}
