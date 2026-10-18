use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use inquire::{Confirm, Select, Text};

use crate::{
    config::{self, Config},
    dictionary, output,
    project::{self, to_pascal_case, validate_name},
    templates::{self, IMPORTS_MARKER, MESSAGES_MARKER, PAGES_MARKER, SECTIONS_MARKER},
};

/// First entry of the page list, so a new page can be created from `add -s`.
const NEW_PAGE: &str = "+ Create a new page";

pub struct AddOptions {
    /// `Some("")` when `-s` is given without a value, so the name is asked.
    pub section: Option<String>,
    /// `Some("")` when `-c` is given without a value, so the name is asked.
    pub component: Option<String>,
    pub page: Option<String>,
    pub page_only: bool,
}

pub fn run(options: AddOptions) -> Result<()> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    let config = Config::for_project(&root)?;
    let languages = &config.languages;

    if let Some(section) = options.section {
        let section = if section.is_empty() {
            prompt_name("section")?
        } else {
            section.to_ascii_lowercase()
        };
        validate_name("section", &section)?;
        let page = ask_target_page(&root, languages, options.page)?;
        add_section(&root, languages, &page, &section)?;
        println!(
            "{}",
            output::success(&format!("section `{section}` added to page `{page}`"))
        );
        return Ok(());
    }
    if let Some(component) = options.component {
        let component = if component.is_empty() {
            prompt_name("component")?
        } else {
            component.to_ascii_lowercase()
        };
        validate_name("component", &component)?;
        create_custom_component(&root, &component)?;
        println!(
            "{}",
            output::success(&format!("custom component `{component}` created"))
        );
        return Ok(());
    }
    if options.page_only {
        let page = ask_new_page(options.page)?;
        create_page(&root, languages, &page)?;
        println!("{}", output::success(&format!("page `{page}` created")));
        return Ok(());
    }
    if options.page.is_some() {
        bail!(
            "use `-s <section> <page>` for a section, `-p <page>` for a page, or `-c <name>` for a component"
        );
    }
    match Select::new(
        "What do you want to add?",
        vec!["Section", "Page", "Custom component"],
    )
    .prompt()?
    {
        "Section" => {
            let section = prompt_name("section")?;
            let page = ask_target_page(&root, languages, None)?;
            add_section(&root, languages, &page, &section)?;
            println!(
                "{}",
                output::success(&format!("section `{section}` added to page `{page}`"))
            );
            Ok(())
        }
        "Page" => {
            let page = ask_new_page(None)?;
            create_page(&root, languages, &page)?;
            println!("{}", output::success(&format!("page `{page}` created")));
            Ok(())
        }
        _ => {
            let component = prompt_name("component")?;
            create_custom_component(&root, &component)?;
            println!(
                "{}",
                output::success(&format!("custom component `{component}` created"))
            );
            Ok(())
        }
    }
}

/// The new page name, from the flag or the prompt.
fn ask_new_page(page: Option<String>) -> Result<String> {
    let page = match page {
        Some(page) => page.to_ascii_lowercase(),
        None => prompt_name("page")?,
    };
    validate_name("page", &page)?;
    Ok(page)
}

/// The page a section goes into. Lists the pages of `views/` so the name cannot be misspelled.
fn ask_target_page(root: &Path, languages: &[String], page: Option<String>) -> Result<String> {
    if let Some(page) = page {
        let page = page.to_ascii_lowercase();
        validate_name("page", &page)?;
        return Ok(page);
    }
    let mut choices = vec![NEW_PAGE.to_string()];
    choices.extend(dictionary::pages(root)?);
    let choice = Select::new("Page", choices).prompt()?;
    if choice != NEW_PAGE {
        return Ok(choice);
    }
    let page = prompt_name("page")?;
    create_page(root, languages, &page)?;
    Ok(page)
}

/// Ask for a lowercase identifier and keep asking until it is valid.
fn prompt_name(kind: &str) -> Result<String> {
    loop {
        let value = Text::new(&format!("{} name", kind)).prompt()?;
        let value = value.trim().to_ascii_lowercase();
        match validate_name(kind, &value) {
            Ok(()) => return Ok(value),
            Err(error) => eprintln!("{}", output::warning(&error.to_string())),
        }
    }
}

/// A starter component in `components/custom/`.
pub fn create_custom_component(root: &Path, name: &str) -> Result<()> {
    validate_name("component", name)?;
    let path = root.join("components/custom").join(format!("{name}.tsx"));
    project::create_file(&path, &templates::custom_component(name))
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
        eprintln!(
            "{}",
            output::warning(&format!("page `{page}` does not exist"))
        );
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
