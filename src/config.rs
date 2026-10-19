use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::project;

pub const PROJECT_CONFIG_FILE: &str = "aarshi.toml";
pub const PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];
pub const UI_SHADCN: &str = "shadcn";
pub const UI_BASE: &str = "base-ui";
pub const ICONS_LUCIDE: &str = "lucide";
pub const ICONS_NONE: &str = "none";
pub const DEFAULT_SITE_URL: &str = "http://localhost:3000";

/// Languages offered by `setup`, as (code, name in that language).
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("fr", "Français"),
    ("en", "English"),
    ("es", "Español"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("de", "Deutsch"),
];
const DEFAULT_LANGUAGES: &[&str] = &["fr", "en"];
const DEFAULT_PACKAGE_MANAGER: &str = "npm";
const RTL_LANGUAGES: &[&str] = &["ar", "he", "fa"];

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    languages: Option<Vec<String>>,
    package_manager: Option<String>,
    ui: Option<String>,
    icons: Option<String>,
    cache_components: Option<bool>,
    site_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub languages: Vec<String>,
    pub package_manager: String,
    pub ui: String,
    pub icons: String,
    pub cache_components: bool,
    pub site_url: String,
}

impl Config {
    pub fn defaults() -> Self {
        Self {
            languages: DEFAULT_LANGUAGES.iter().map(|l| l.to_string()).collect(),
            package_manager: DEFAULT_PACKAGE_MANAGER.to_string(),
            ui: UI_SHADCN.to_string(),
            icons: ICONS_LUCIDE.to_string(),
            cache_components: false,
            site_url: DEFAULT_SITE_URL.to_string(),
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
        if let Some(ui) = file.ui {
            validate_ui(&ui)?;
            self.ui = ui;
        }
        if let Some(icons) = file.icons {
            validate_icons(&icons)?;
            self.icons = icons;
        }
        if let Some(cache_components) = file.cache_components {
            self.cache_components = cache_components;
        }
        if let Some(site_url) = file.site_url {
            validate_site_url(&site_url)?;
            self.site_url = site_url;
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
            "languages = [{languages}]\npackage_manager = \"{}\"\nui = \"{}\"\nicons = \"{}\"\ncache_components = {}\nsite_url = \"{}\"\n",
            self.package_manager, self.ui, self.icons, self.cache_components, self.site_url
        )
    }
}

pub fn is_aarshi_project(root: &Path) -> bool {
    root.join(PROJECT_CONFIG_FILE).exists()
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

pub fn validate_ui(ui: &str) -> Result<()> {
    if ui != UI_SHADCN && ui != UI_BASE {
        bail!("unknown UI `{ui}` (expected `shadcn` or `base-ui`)");
    }
    Ok(())
}

pub fn validate_icons(icons: &str) -> Result<()> {
    if icons != ICONS_LUCIDE && icons != ICONS_NONE {
        bail!("unknown icon library `{icons}` (expected `lucide` or `none`)");
    }
    Ok(())
}

pub fn validate_site_url(url: &str) -> Result<()> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        bail!("site URL must start with http:// or https://");
    }
    // The URL is written into `aarshi.toml` and `lib/site.ts`, so quotes and spaces would break both.
    if url
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '\\' | '`'))
    {
        bail!("site URL must not contain spaces or quotes");
    }
    Ok(())
}

pub fn validate_languages(languages: &[String]) -> Result<()> {
    if languages.is_empty() {
        bail!("`languages` must contain at least one language");
    }
    for (index, language) in languages.iter().enumerate() {
        validate_language(language)?;
        if languages[..index].contains(language) {
            bail!("language `{language}` is listed more than once");
        }
    }
    Ok(())
}

pub fn validate_language(language: &str) -> Result<()> {
    let valid = language.len() == 2 && language.chars().all(|c| c.is_ascii_lowercase());
    if !valid {
        bail!("invalid language `{language}` (expected a lowercase two-letter code)");
    }
    Ok(())
}

fn user_config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(Path::new(&home).join(".config/aarshi/config.toml"))
}

fn read_file(path: &Path) -> Result<FileConfig> {
    let content = project::read_file(path)?;
    toml::from_str(&content).with_context(|| format!("invalid config in {}", path.display()))
}
