use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use inquire::{Confirm, Select, Text};

use crate::{
    config::{self, Config},
    dictionary,
    project::{self, to_pascal_case, validate_name},
    templates::{self, IMPORTS_MARKER, MESSAGES_MARKER, PAGES_MARKER, SECTIONS_MARKER},
};

pub struct AddOptions {
    /// `Some("")` when `-s` is given without a value, so the name is asked.
    pub section: Option<String>,
    pub page: Option<String>,
    pub page_only: bool,
}

pub fn run(options: AddOptions) -> Result<()> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    let config = Config::for_project(&root)?;

    match (options.section, options.page_only, options.page) {
        (Some(section), _, page) => {
            let section = if section.is_empty() {
                Text::new("Section name").prompt()?
            } else {
                section
            };
            let page = ask_page(page)?;
            validate_name("section", &section)?;
            add_section(&root, &config.languages, &page, &section)
        }
        (None, true, page) => create_page(&root, &config.languages, &ask_page(page)?),
        (None, false, Some(_)) => {
            bail!("use `-s <section> <page>` to add a section or `-p <page>` to add a page")
        }
        (None, false, None) => {
            match Select::new("What do you want to add?", vec!["Section", "Page"]).prompt()? {
                "Section" => {
                    let section = Text::new("Section name").prompt()?;
                    let page = ask_page(None)?;
                    validate_name("section", &section)?;
                    add_section(&root, &config.languages, &page, &section)
                }
                _ => create_page(&root, &config.languages, &ask_page(None)?),
            }
        }
    }
}

fn ask_page(page: Option<String>) -> Result<String> {
    let page = match page {
        Some(page) => page,
        None => Text::new("Page name").prompt()?,
    };
    validate_name("page", &page)?;
    Ok(page)
}

/// Creates a page and registers it in the sitemap.
pub fn create_page(root: &Path, languages: &[String], page: &str) -> Result<()> {
    validate_name("page", page)?;
    write_page(root, languages, page)?;
    let route_path = route_path(page);
    project::update_file(&root.join("app/sitemap.ts"), |content| {
        project::insert_before_marker(content, PAGES_MARKER, &format!("\"{route_path}\","))
    })
}

/// Writes the view and one route per language. Keys are added to every dictionary.
pub fn write_page(root: &Path, languages: &[String], page: &str) -> Result<()> {
    let route_path = route_path(page);
    let view = view_file(root, page);
    let route = route_file(root, &route_path);
    for path in [&view, &route] {
        if path.exists() {
            bail!("{} already exists", path.display());
        }
    }

    project::create_file(&view, &templates::view(page))?;
    project::create_file(&route, &templates::route(page, &route_path))?;
    for locale in languages {
        let key = format!("\"{page}.title\": \"\",");
        project::update_file(&dictionary::file(root, locale), |content| {
            project::insert_before_marker(content, MESSAGES_MARKER, &key)
        })?;
    }
    Ok(())
}

pub fn add_section(root: &Path, languages: &[String], page: &str, section: &str) -> Result<()> {
    if !view_file(root, page).exists() {
        eprintln!("error: page `{page}` does not exist");
        let create = Confirm::new(&format!("Create page `{page}`?"))
            .with_default(false)
            .prompt()?;
        if !create {
            bail!("page `{page}` was not created");
        }
        create_page(root, languages, page)?;
    }

    let section_file = root
        .join("views")
        .join(page)
        .join("sections")
        .join(format!("{section}.tsx"));
    let component = format!("{}Section", to_pascal_case(section));
    let view = view_file(root, page);

    project::create_file(&section_file, &templates::section(page, section))?;
    project::update_file(&view, |content| {
        let with_import = project::insert_before_marker(
            content,
            IMPORTS_MARKER,
            &format!("import {{ {component} }} from \"./sections/{section}\";"),
        )?;
        project::insert_before_marker(
            &with_import,
            SECTIONS_MARKER,
            &format!("<{component} locale={{locale}} />"),
        )
    })?;
    for locale in languages {
        let key = format!("\"{page}.{section}.title\": \"\",");
        project::update_file(&dictionary::file(root, locale), |content| {
            project::insert_before_marker(content, MESSAGES_MARKER, &key)
        })?;
    }
    Ok(())
}

/// The sitemap lists `/` for home and `/<page>` for other pages.
pub fn route_path(page: &str) -> String {
    if page == "home" {
        "/".to_string()
    } else {
        format!("/{page}")
    }
}

fn view_file(root: &Path, page: &str) -> PathBuf {
    root.join("views")
        .join(page)
        .join(format!("{page}.view.tsx"))
}

/// Every language shares one route tree, under the `[locale]` segment.
pub fn route_file(root: &Path, route_path: &str) -> PathBuf {
    let mut path = root.join("app/[locale]");
    for segment in route_path.split('/').filter(|s| !s.is_empty()) {
        path.push(segment);
    }
    path.push("page.tsx");
    path
}
