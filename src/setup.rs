use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use inquire::{MultiSelect, Select, Text};

use crate::{
    add,
    config::{self, Config, PACKAGE_MANAGERS, PROJECT_CONFIG_FILE, SUPPORTED_LANGUAGES},
    project, templates,
};

const DEFAULT_SITE_URL: &str = "http://localhost:3000";
const AGENT_FEEDBACK: &str = "Agent feedback";
const CACHE_COMPONENTS: &str = "Cache Components";
const UI_SHADCN: &str = "shadcn";
const UI_BASE: &str = "base-ui";
const ICONS_LUCIDE: &str = "lucide";
const ICONS_NONE: &str = "none";
/// Class helper, used by the switcher, the icons and the custom components.
const CN_PACKAGE: &str = "@sglara/cn";

pub struct SetupOptions {
    pub dir: Option<String>,
    pub package_manager: Option<String>,
    pub languages: Option<Vec<String>>,
    pub default_language: Option<String>,
    pub ui: Option<String>,
    pub icons: Option<String>,
    pub agent_feedback: bool,
    pub cache_components: bool,
    pub site_url: Option<String>,
    pub yes: bool,
}

pub fn run(options: SetupOptions) -> Result<()> {
    let user = Config::user()?;
    let mut dir = match &options.dir {
        Some(dir) => dir.clone(),
        None if options.yes => bail!("a project folder is required with --yes"),
        None => Text::new("Project folder name").prompt()?,
    };
    let root = loop {
        match target_root(&dir) {
            Ok(root) => break root,
            Err(error) => {
                if options.dir.is_some() || options.yes {
                    return Err(error);
                }
                eprintln!("{error}");
                dir = Text::new("Project folder name").prompt()?;
            }
        }
    };
    let languages = select_languages(&options, &user)?;
    let default = select_default_language(&options, &languages)?;
    // The default language comes first: it is the one the dictionaries are typed from.
    let mut languages = languages;
    languages.sort_by_key(|language| language != &default);
    let mut site_url_input = match &options.site_url {
        Some(url) => url.clone(),
        None if options.yes => DEFAULT_SITE_URL.to_string(),
        None => Text::new("Site URL")
            .with_default(DEFAULT_SITE_URL)
            .prompt()?,
    };
    let site_url = loop {
        match normalize_site_url(&site_url_input) {
            Ok(url) => break url,
            Err(error) => {
                if options.site_url.is_some() || options.yes {
                    return Err(error);
                }
                eprintln!("{error}");
                site_url_input = Text::new("Site URL")
                    .with_default(DEFAULT_SITE_URL)
                    .prompt()?;
            }
        }
    };
    let manager = match &options.package_manager {
        Some(manager) => manager.clone(),
        None if options.yes => user.package_manager.clone(),
        None => select_package_manager(&user.package_manager)?,
    };
    let ui = select_ui(&options)?;
    let icons = select_icons(&options)?;
    let (agent_feedback, cache_components) =
        if options.yes || options.agent_feedback || options.cache_components {
            (options.agent_feedback, options.cache_components)
        } else {
            select_features()?
        };

    create_next_app(&manager, &dir, agent_feedback, cache_components)?;
    install_dependencies(&root, &manager, &ui, &icons)?;
    create_component_folders(&root)?;
    let config = Config {
        languages,
        package_manager: manager,
    };
    generate(&root, &config, &site_url, cache_components)
}

/// The offered languages, preselected with the user config when it has some.
fn select_languages(options: &SetupOptions, user: &Config) -> Result<Vec<String>> {
    if let Some(languages) = &options.languages {
        let languages: Vec<String> = languages.iter().map(|l| l.to_ascii_lowercase()).collect();
        config::validate_languages(&languages)?;
        return Ok(languages);
    }
    if options.yes {
        return Ok(user.languages.clone());
    }
    let choices: Vec<(&str, String)> = SUPPORTED_LANGUAGES
        .iter()
        .map(|(code, name)| (*code, format!("{name} ({code})")))
        .collect();
    let labels: Vec<String> = choices.iter().map(|(_, label)| label.clone()).collect();
    let preselected: Vec<usize> = choices
        .iter()
        .enumerate()
        .filter_map(|(index, (code, _))| user.languages.iter().any(|l| l == code).then_some(index))
        .collect();
    let selected = loop {
        let selected = MultiSelect::new("Languages", labels.clone())
            .with_default(&preselected)
            .prompt()?;
        if !selected.is_empty() {
            break selected;
        }
        eprintln!("error: select at least one language");
    };
    let languages: Vec<String> = choices
        .into_iter()
        .filter(|(_, label)| selected.contains(label))
        .map(|(code, _)| code.to_string())
        .collect();
    config::validate_languages(&languages)?;
    Ok(languages)
}

fn select_default_language(options: &SetupOptions, languages: &[String]) -> Result<String> {
    match &options.default_language {
        Some(code) => {
            let code = code.to_ascii_lowercase();
            if languages.contains(&code) {
                Ok(code)
            } else {
                bail!("default language `{code}` is not in the selected languages")
            }
        }
        None if options.yes => Ok(languages[0].clone()),
        None => Ok(Select::new("Default language", languages.to_vec()).prompt()?),
    }
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

fn select_ui(options: &SetupOptions) -> Result<String> {
    if let Some(ui) = &options.ui {
        let ui = ui.to_ascii_lowercase();
        if ui != UI_SHADCN && ui != UI_BASE {
            bail!("unknown UI `{ui}` (expected `shadcn` or `base-ui`)");
        }
        return Ok(ui);
    }
    if options.yes {
        return Ok(UI_SHADCN.to_string());
    }
    let choice = Select::new(
        "UI components",
        vec!["shadcn (styled, Base UI based)", "Base UI (unstyled)"],
    )
    .prompt()?;
    Ok(if choice.starts_with("shadcn") {
        UI_SHADCN
    } else {
        UI_BASE
    }
    .to_string())
}

fn select_icons(options: &SetupOptions) -> Result<String> {
    if let Some(icons) = &options.icons {
        let icons = icons.to_ascii_lowercase();
        if icons != ICONS_LUCIDE && icons != ICONS_NONE {
            bail!("unknown icon library `{icons}` (expected `lucide` or `none`)");
        }
        return Ok(icons);
    }
    if options.yes {
        return Ok(ICONS_LUCIDE.to_string());
    }
    let choice = Select::new("Icon library", vec!["lucide-react", "None"]).prompt()?;
    Ok(if choice == "lucide-react" {
        ICONS_LUCIDE
    } else {
        ICONS_NONE
    }
    .to_string())
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
    // Yarn 1 turns `next-app@latest` into a binary name (`create-next-app@latest`) and fails.
    let package = if manager == "yarn" {
        "next-app"
    } else {
        "next-app@latest"
    };
    command.arg("create").arg(package).arg(dir);
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

/// Installs the class helper, the icon library and the UI components.
fn install_dependencies(root: &Path, manager: &str, ui: &str, icons: &str) -> Result<()> {
    let mut packages = vec![CN_PACKAGE.to_string()];
    if icons == ICONS_LUCIDE {
        packages.push("lucide-react".to_string());
    }
    if ui == UI_BASE {
        packages.push("@base-ui/react".to_string());
    }
    let mut command = Command::new(manager);
    command
        .current_dir(root)
        .arg("add")
        .args(&packages)
        .env("NEXT_TELEMETRY_DISABLED", "1");
    let status = command
        .status()
        .with_context(|| format!("cannot run `{manager}`; is it installed?"))?;
    if !status.success() {
        bail!("`{manager} add {}` failed", packages.join(" "));
    }

    if ui == UI_SHADCN {
        run_shadcn(root, manager, &["init", "--defaults", "--yes"])?;
        run_shadcn(root, manager, &["add", "button", "button-group", "--yes"])?;
    } else {
        project::create_file(&root.join("components/ui/button.tsx"), &templates::button())?;
        project::create_file(
            &root.join("components/ui/button-group.tsx"),
            &templates::button_group(),
        )?;
    }
    Ok(())
}

fn run_shadcn(root: &Path, manager: &str, args: &[&str]) -> Result<()> {
    // Yarn 1 has no `dlx`, and `npx` ships with Node, which every manager here needs.
    let (bin, prefix): (&str, &[&str]) = match manager {
        "pnpm" => ("pnpm", &["dlx"]),
        "bun" => ("bunx", &[]),
        _ => ("npx", &["--yes"]),
    };
    let status = Command::new(bin)
        .current_dir(root)
        .args(prefix)
        .arg("shadcn@latest")
        .args(args)
        .env("NEXT_TELEMETRY_DISABLED", "1")
        .status()
        .with_context(|| format!("cannot run `{bin}`; is it installed?"))?;
    if !status.success() {
        bail!("`shadcn {}` failed", args.join(" "));
    }
    Ok(())
}

/// `components/custom` and `components/icons`, kept in git with a `.gitkeep`.
fn create_component_folders(root: &Path) -> Result<()> {
    for dir in ["components/custom", "components/icons"] {
        let path = root.join(dir);
        fs::create_dir_all(&path).with_context(|| format!("cannot create {}", path.display()))?;
        let keep = path.join(".gitkeep");
        if !keep.exists() {
            fs::write(&keep, "")?;
        }
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
    project::update_file(&root.join("next.config.ts"), |content| {
        if !content.contains("const nextConfig: NextConfig = {") {
            bail!("next.config.ts has an unexpected shape; cannot add the root redirect");
        }
        let redirect = format!(
            "const nextConfig: NextConfig = {{\n  async redirects() {{\n    return [{{ source: \"/\", destination: \"/{}\", permanent: false }}];\n  }},",
            config.languages[0]
        );
        let content = content.replace("const nextConfig: NextConfig = {", &redirect);
        if cache_components {
            return Ok(content);
        }
        if !content.contains("cacheComponents: true") {
            bail!("next.config.ts does not enable cacheComponents as expected");
        }
        Ok(content
            .replace("cacheComponents: true", "cacheComponents: false")
            .replace("  partialPrefetching: true,\n", ""))
    })?;

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
    project::create_file(&root.join("app/[locale]/layout.tsx"), &templates::layout())?;
    project::create_file(
        &root.join("components/language-switcher.tsx"),
        &templates::language_switcher(),
    )?;
    add::write_page(root, languages, "home")
}
