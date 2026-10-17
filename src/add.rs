use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use inquire::Confirm;

use crate::{
    config::Config,
    project::{self, to_pascal_case, validate_name},
    templates::{self, IMPORTS_MARKER, MESSAGES_MARKER, PAGES_MARKER, SECTIONS_MARKER},
};

pub struct AddOptions {
    pub section: Option<String>,
    pub page: String,
    pub page_only: bool,
}

pub fn run(options: AddOptions) -> Result<()> {
    let root = env::current_dir()?;
    validate_name("page", &options.page)?;
    let config = Config::for_project(&root)?;

    match options.section {
        Some(section) => {
            validate_name("section", &section)?;
            if !view_file(&root, &options.page).exists() {
                eprintln!("error: page `{}` does not exist", options.page);
                let create = Confirm::new(&format!("Create page `{}`?", options.page))
                    .with_default(false)
                    .prompt()?;
                if !create {
                    bail!("page `{}` was not created", options.page);
                }
                create_page(&root, &config.languages, &options.page)?;
            }
            add_section(&root, &config.languages, &options.page, &section)
        }
        None if options.page_only => create_page(&root, &config.languages, &options.page),
        None => bail!("use `-s <section> <page>` to add a section or `-p <page>` to add a page"),
    }
}

/// Creates a page and registers it in the sitemap.
pub fn create_page(root: &Path, languages: &[String], page: &str) -> Result<()> {
    write_page(root, languages, page)?;
    let route_path = route_path(page);
    project::update_file(&root.join("app/sitemap.ts"), |content| {
        project::insert_before_marker(content, PAGES_MARKER, &format!("\"{route_path}\","))
    })
}

/// Writes the view and one route per language. Keys are added to every dictionary.
pub fn write_page(root: &Path, languages: &[String], page: &str) -> Result<()> {
    let route_path = route_path(page);
    let default = &languages[0];
    let view = view_file(root, page);
    let routes: Vec<PathBuf> = languages
        .iter()
        .map(|locale| route_file(root, locale, default, &route_path))
        .collect();
    for path in std::iter::once(&view).chain(routes.iter()) {
        if path.exists() {
            bail!("{} already exists", path.display());
        }
    }

    project::create_file(&view, &templates::view(page))?;
    for (locale, route) in languages.iter().zip(&routes) {
        project::create_file(route, &templates::route(locale, page, &route_path))?;
    }
    for locale in languages {
        let key = format!("\"{page}.title\": \"\",");
        project::update_file(&messages_file(root, locale), |content| {
            project::insert_before_marker(content, MESSAGES_MARKER, &key)
        })?;
    }
    Ok(())
}

fn add_section(root: &Path, languages: &[String], page: &str, section: &str) -> Result<()> {
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
        project::update_file(&messages_file(root, locale), |content| {
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

fn messages_file(root: &Path, locale: &str) -> PathBuf {
    root.join("lib/i18n/dictionaries")
        .join(format!("{locale}.ts"))
}

/// The default language lives in the root group, others under `/<locale>`.
fn route_file(root: &Path, locale: &str, default: &str, route_path: &str) -> PathBuf {
    let mut path = root.join("app").join(format!("({locale})"));
    if locale != default {
        path.push(locale);
    }
    for segment in route_path.split('/').filter(|s| !s.is_empty()) {
        path.push(segment);
    }
    path.push("page.tsx");
    path
}
