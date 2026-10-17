use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

pub const PROJECT_CONFIG_FILE: &str = "aarshi.toml";
pub const PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];

const DEFAULT_LANGUAGES: &[&str] = &["fr", "en", "es", "ko"];
const DEFAULT_PACKAGE_MANAGER: &str = "npm";
const RTL_LANGUAGES: &[&str] = &["ar", "he", "fa"];

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    languages: Option<Vec<String>>,
    package_manager: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub languages: Vec<String>,
    pub package_manager: String,
}

impl Config {
    pub fn defaults() -> Self {
        Self {
            languages: DEFAULT_LANGUAGES.iter().map(|l| l.to_string()).collect(),
            package_manager: DEFAULT_PACKAGE_MANAGER.to_string(),
        }
    }

    /// Defaults, then the user config.
    pub fn user() -> Result<Self> {
        let mut config = Self::defaults();
        if let Some(path) = user_config_path().filter(|path| path.exists()) {
            config = config.with(read_file(&path)?)?;
        }
        Ok(config)
    }

    /// User config, then the project config when it exists.
    /// A project `languages` list replaces the user list entirely.
    pub fn for_project(project_root: &Path) -> Result<Self> {
        let mut config = Self::user()?;
        let project_file = project_root.join(PROJECT_CONFIG_FILE);
        if project_file.exists() {
            config = config.with(read_file(&project_file)?)?;
        }
        Ok(config)
    }

    fn with(mut self, file: FileConfig) -> Result<Self> {
        if let Some(languages) = file.languages {
            validate_languages(&languages)?;
            self.languages = languages;
        }
        if let Some(manager) = file.package_manager {
            validate_package_manager(&manager)?;
            self.package_manager = manager;
        }
        Ok(self)
    }

    /// Contents of `aarshi.toml` for a generated project.
    pub fn to_project_file(&self) -> String {
        let languages = self
            .languages
            .iter()
            .map(|l| format!("\"{l}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "languages = [{languages}]\npackage_manager = \"{}\"\n",
            self.package_manager
        )
    }
}

pub fn is_rtl(language: &str) -> bool {
    RTL_LANGUAGES.contains(&language)
}

pub fn validate_package_manager(manager: &str) -> Result<()> {
    if !PACKAGE_MANAGERS.contains(&manager) {
        bail!("unknown package manager `{manager}` (expected one of: npm, pnpm, yarn, bun)");
    }
    Ok(())
}

fn validate_languages(languages: &[String]) -> Result<()> {
    if languages.is_empty() {
        bail!("`languages` must contain at least one language");
    }
    for (index, language) in languages.iter().enumerate() {
        let valid = language.len() == 2 && language.chars().all(|c| c.is_ascii_lowercase());
        if !valid {
            bail!("invalid language `{language}` (expected a lowercase two-letter code)");
        }
        if languages[..index].contains(language) {
            bail!("language `{language}` is listed more than once");
        }
    }
    Ok(())
}

fn user_config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(Path::new(&home).join(".config/aarshi/config.toml"))
}

fn read_file(path: &Path) -> Result<FileConfig> {
    let content =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("invalid config in {}", path.display()))
}
