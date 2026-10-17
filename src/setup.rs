use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use inquire::{MultiSelect, Select, Text};

use crate::{
    add,
    config::{Config, PACKAGE_MANAGERS, PROJECT_CONFIG_FILE},
    project, templates,
};

const DEFAULT_SITE_URL: &str = "http://localhost:3000";
const AGENT_FEEDBACK: &str = "Agent feedback";
const CACHE_COMPONENTS: &str = "Cache Components";

pub struct SetupOptions {
    pub dir: Option<String>,
    pub package_manager: Option<String>,
    pub agent_feedback: bool,
    pub cache_components: bool,
    pub site_url: Option<String>,
    pub yes: bool,
}

pub fn run(options: SetupOptions) -> Result<()> {
    let user = Config::user()?;
    let dir = match options.dir {
        Some(dir) => dir,
        None if options.yes => bail!("a project folder is required with --yes"),
        None => Text::new("Project folder name").prompt()?,
    };
    let root = target_root(&dir)?;
    let site_url = match options.site_url {
        Some(url) => url,
        None if options.yes => DEFAULT_SITE_URL.to_string(),
        None => Text::new("Site URL")
            .with_default(DEFAULT_SITE_URL)
            .prompt()?,
    };
    let site_url = normalize_site_url(&site_url)?;
    let manager = match options.package_manager {
        Some(manager) => manager,
        None if options.yes => user.package_manager.clone(),
        None => select_package_manager(&user.package_manager)?,
    };
    let (agent_feedback, cache_components) =
        if options.yes || options.agent_feedback || options.cache_components {
            (options.agent_feedback, options.cache_components)
        } else {
            select_features()?
        };

    create_next_app(&manager, &dir, agent_feedback, cache_components)?;
    let config = Config {
        languages: user.languages,
        package_manager: manager,
    };
    generate(&root, &config, &site_url, cache_components)
}

fn select_package_manager(default: &str) -> Result<String> {
    let start = PACKAGE_MANAGERS
        .iter()
        .position(|manager| *manager == default)
        .unwrap_or(0);
    let choice = Select::new("Package manager", PACKAGE_MANAGERS.to_vec())
        .with_starting_cursor(start)
        .prompt()?;
    Ok(choice.to_string())
}

fn select_features() -> Result<(bool, bool)> {
    let chosen = MultiSelect::new("Options", vec![AGENT_FEEDBACK, CACHE_COMPONENTS]).prompt()?;
    Ok((
        chosen.contains(&AGENT_FEEDBACK),
        chosen.contains(&CACHE_COMPONENTS),
    ))
}

fn create_next_app(
    manager: &str,
    dir: &str,
    agent_feedback: bool,
    cache_components: bool,
) -> Result<()> {
    let mut command = Command::new(manager);
    command.arg("create").arg("next-app@latest").arg(dir);
    if manager == "npm" {
        command.arg("--");
    }
    command
        .args([
            "--ts",
            "--tailwind",
            "--eslint",
            "--app",
            "--import-alias",
            "@/*",
            "--agents-md",
        ])
        .arg(format!("--use-{manager}"))
        .arg(if agent_feedback {
            "--agent-feedback"
        } else {
            "--no-agent-feedback"
        })
        .env("NEXT_TELEMETRY_DISABLED", "1");
    if cache_components {
        command.arg("--cache-components");
    }
    let status = command
        .status()
        .with_context(|| format!("cannot run `{manager}`; is it installed?"))?;
    if !status.success() {
        bail!("`{manager} create next-app` failed");
    }
    Ok(())
}

/// Returns the project folder. `.` must be empty, any other name must not exist yet.
fn target_root(dir: &str) -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    if dir == "." {
        if fs::read_dir(&cwd)?.next().is_some() {
            bail!("the current folder is not empty");
        }
        return Ok(cwd);
    }
    let root = cwd.join(dir);
    if root.exists() {
        bail!("{} already exists", root.display());
    }
    Ok(root)
}

fn normalize_site_url(url: &str) -> Result<String> {
    let trimmed = url.trim().trim_end_matches('/');
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        bail!("site URL must start with http:// or https://");
    }
    Ok(trimmed.to_string())
}

/// Writes the i18n structure into a project created by create-next-app.
fn generate(root: &Path, config: &Config, site_url: &str, cache_components: bool) -> Result<()> {
    let languages = &config.languages;
    let default = &languages[0];

    for file in ["app/layout.tsx", "app/page.tsx"] {
        let path = root.join(file);
        if path.exists() {
            fs::remove_file(&path)?;
        }
    }
    let build_cache = root.join(".next");
    if build_cache.exists() {
        fs::remove_dir_all(&build_cache)?;
    }
    if !cache_components {
        project::update_file(&root.join("next.config.ts"), |content| {
            if !content.contains("cacheComponents: true") {
                bail!("next.config.ts does not enable cacheComponents as expected");
            }
            Ok(content
                .replace("cacheComponents: true", "cacheComponents: false")
                .replace("  partialPrefetching: true,\n", ""))
        })?;
    }

    let env_file = root.join(".env");
    if env_file.exists() {
        project::update_file(&env_file, |content| {
            Ok(format!(
                "{}\nNEXT_TELEMETRY_DISABLED=1\n",
                content.trim_end()
            ))
        })?;
    } else {
        project::create_file(&env_file, "NEXT_TELEMETRY_DISABLED=1\n")?;
    }
    project::create_file(&root.join(PROJECT_CONFIG_FILE), &config.to_project_file())?;
    project::create_file(&root.join("lib/site.ts"), &templates::site(site_url))?;
    project::create_file(
        &root.join("lib/i18n/config.ts"),
        &templates::i18n_config(languages),
    )?;
    project::create_file(
        &root.join("lib/i18n/index.ts"),
        &templates::i18n_index(languages),
    )?;
    for locale in languages {
        project::create_file(
            &root
                .join("lib/i18n/dictionaries")
                .join(format!("{locale}.ts")),
            &templates::dictionary(locale, default),
        )?;
    }
    project::create_file(&root.join("app/sitemap.ts"), &templates::sitemap())?;
    project::create_file(&root.join("app/robots.ts"), &templates::robots())?;
    for locale in languages {
        project::create_file(
            &add::layout_file(root, locale, default),
            &templates::layout(locale),
        )?;
    }
    add::write_page(root, languages, "home")
}
