use std::{
    collections::BTreeSet,
    env, fmt, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use inquire::{Confirm, MultiSelect};

use crate::{
    add,
    config::{self, Config},
    dictionary, output, project,
};

/// One folder per removal, named by its timestamp, holds the copies `restore` needs.
const REMOVED_DIR: &str = ".aarshi/removed";
/// `/` redirects to the home page, so it is never offered for removal.
const HOME: &str = "home";

pub struct RemoveOptions {
    /// Remove everything that can be removed, without picking.
    pub all: bool,
    /// Skip the confirmation.
    pub yes: bool,
}

#[derive(Clone, PartialEq)]
enum Item {
    Page(String),
    Section { page: String, name: String },
    Component(String),
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Item::Page(page) => write!(f, "page `{page}`"),
            Item::Section { page, name } => write!(f, "section `{name}` in page `{page}`"),
            Item::Component(name) => write!(f, "component `{name}`"),
        }
    }
}

/// Removes the chosen pages, sections and components. Every file it touches is
/// copied first, so `restore` can put the project back as it was.
pub fn run(options: RemoveOptions) -> Result<()> {
    let root = project_root()?;
    let config = Config::for_project(&root)?;
    let items = list_items(&root)?;
    if items.is_empty() {
        println!("{}", output::info("nothing to remove"));
        return Ok(());
    }
    let mut selected = if options.all {
        items
    } else {
        MultiSelect::new("What do you want to remove?", items).prompt()?
    };
    // A section of a removed page goes with the page.
    let removed_pages: Vec<String> = selected
        .iter()
        .filter_map(|item| match item {
            Item::Page(page) => Some(page.clone()),
            _ => None,
        })
        .collect();
    selected
        .retain(|item| !matches!(item, Item::Section { page, .. } if removed_pages.contains(page)));
    if selected.is_empty() {
        println!("{}", output::info("nothing selected"));
        return Ok(());
    }
    if !options.yes {
        let confirmed = Confirm::new(&format!(
            "Remove {} item(s)? `aarshi restore` brings them back.",
            selected.len()
        ))
        .with_default(false)
        .prompt()?;
        if !confirmed {
            println!("{}", output::info("nothing removed"));
            return Ok(());
        }
    }

    let touched = touched_files(&root, &config, &selected);
    snapshot(&root, &touched, &selected)?;
    for item in &selected {
        if let Item::Page(page) = item {
            remove_page(&root, &config, page)?;
        }
    }
    for item in &selected {
        match item {
            Item::Section { page, name } => remove_section(&root, &config, page, name)?,
            Item::Component(name) => remove_file(&component_file(&root, name))?,
            Item::Page(_) => {}
        }
    }
    done(&format!(
        "removed {} item(s), `aarshi restore` brings them back",
        selected.len()
    ));
    Ok(())
}

/// Puts back the files of the last removal, then forgets that removal.
pub fn restore(yes: bool) -> Result<()> {
    let root = project_root()?;
    let dir = latest_removal(&root)?;
    let items = fs::read_to_string(dir.join("items"))?;
    let manifest = fs::read_to_string(dir.join("manifest"))?;
    println!("{}", output::info("the last removal holds:"));
    for line in items.lines() {
        println!("  {line}");
    }
    if !yes {
        let confirmed =
            Confirm::new("Restore them? Files changed since the removal are overwritten.")
                .with_default(false)
                .prompt()?;
        if !confirmed {
            println!("{}", output::info("nothing restored"));
            return Ok(());
        }
    }

    let mut restored = 0;
    for rel in manifest.lines().filter(|line| !line.is_empty()) {
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        fs::copy(dir.join("files").join(rel), &target)
            .with_context(|| format!("cannot restore {rel}"))?;
        restored += 1;
    }
    fs::remove_dir_all(&dir)?;
    prune_empty(&root.join(REMOVED_DIR));
    done(&format!("restored {restored} file(s)"));
    Ok(())
}

fn project_root() -> Result<PathBuf> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    Ok(root)
}

fn list_items(root: &Path) -> Result<Vec<Item>> {
    let pages = dictionary::pages(root)?;
    let mut items: Vec<Item> = pages
        .iter()
        .filter(|page| page.as_str() != HOME)
        .map(|page| Item::Page(page.clone()))
        .collect();
    for page in &pages {
        for name in stems(&root.join("views").join(page).join("sections"), "tsx") {
            items.push(Item::Section {
                page: page.clone(),
                name,
            });
        }
    }
    for name in stems(&root.join("components/custom"), "tsx") {
        items.push(Item::Component(name));
    }
    Ok(items)
}

/// The names of the files with this extension in `dir`, sorted; empty when `dir` is absent.
fn stems(dir: &Path, extension: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .filter_map(|path| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

/// Every existing file a removal of `items` may change or delete.
fn touched_files(root: &Path, config: &Config, items: &[Item]) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    for item in items {
        match item {
            Item::Page(page) => {
                files.extend(files_under(&root.join("views").join(page)));
                files.extend(files_under(&root.join("app/[locale]").join(page)));
                files.insert(root.join("app/sitemap.ts"));
            }
            Item::Section { page, name } => {
                files.insert(section_file(root, page, name));
                files.insert(view_file(root, page));
            }
            Item::Component(name) => {
                files.insert(component_file(root, name));
            }
        }
        if !matches!(item, Item::Component(_)) {
            for locale in &config.languages {
                files.insert(dictionary::file(root, locale));
            }
        }
    }
    files.retain(|file| file.is_file());
    files
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .flat_map(|path| {
            if path.is_dir() {
                files_under(&path)
            } else {
                vec![path]
            }
        })
        .collect()
}

fn snapshot(root: &Path, files: &BTreeSet<PathBuf>, items: &[Item]) -> Result<()> {
    let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let dir = root.join(REMOVED_DIR).join(id.to_string());
    let mut manifest = String::new();
    for file in files {
        let rel = file.strip_prefix(root)?;
        let copy = dir.join("files").join(rel);
        if let Some(parent) = copy.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, &copy).with_context(|| format!("cannot copy {}", file.display()))?;
        manifest.push_str(&format!("{}\n", rel.display()));
    }
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("manifest"), manifest)?;
    let labels: String = items.iter().map(|item| format!("{item}\n")).collect();
    fs::write(dir.join("items"), labels)?;
    ignore_removed_dir(root)
}

/// Removes `dir` and its parents while they stay empty, up to the project root.
fn prune_empty(dir: &Path) {
    let mut current = Some(dir);
    while let Some(path) = current {
        if fs::remove_dir(path).is_err() {
            break;
        }
        current = path.parent();
    }
}

/// Keeps the copies out of version control when the project has a `.gitignore`.
fn ignore_removed_dir(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(());
    };
    if content.lines().any(|line| line.trim() == ".aarshi/") {
        return Ok(());
    }
    fs::write(&path, format!("{}\n.aarshi/\n", content.trim_end()))
        .with_context(|| format!("cannot update {}", path.display()))
}

fn latest_removal(root: &Path) -> Result<PathBuf> {
    let Ok(entries) = fs::read_dir(root.join(REMOVED_DIR)) else {
        bail!("nothing to restore: no removal recorded");
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter_map(|path| {
            let id = path.file_name()?.to_str()?.parse::<u128>().ok()?;
            Some((id, path))
        })
        .max_by_key(|(id, _)| *id)
        .map(|(_, path)| path)
        .context("nothing to restore: no removal recorded")
}

fn remove_page(root: &Path, config: &Config, page: &str) -> Result<()> {
    remove_dir(&root.join("views").join(page))?;
    remove_dir(&root.join("app/[locale]").join(page))?;
    let entry = format!("\"{}\",", add::route_path(page));
    keep_lines(&root.join("app/sitemap.ts"), |line| line != entry)?;
    for locale in &config.languages {
        let prefix = format!("\"{page}.");
        keep_lines(&dictionary::file(root, locale), |line| {
            !line.starts_with(&prefix)
        })?;
    }
    Ok(())
}

fn remove_section(root: &Path, config: &Config, page: &str, name: &str) -> Result<()> {
    let file = section_file(root, page, name);
    remove_file(&file)?;
    let sections = root.join("views").join(page).join("sections");
    if sections.is_dir() && fs::read_dir(&sections)?.next().is_none() {
        fs::remove_dir(&sections)?;
    }
    let component = format!("{}Section", project::to_pascal_case(name));
    let import = format!("import {{ {component} }} from \"./sections/{name}\";");
    let mount = format!("<{component} locale={{locale}} />");
    keep_lines(&view_file(root, page), |line| {
        line != import && line != mount
    })?;
    let key = format!("\"{page}.{name}.title\":");
    for locale in &config.languages {
        keep_lines(&dictionary::file(root, locale), |line| {
            !line.starts_with(&key)
        })?;
    }
    Ok(())
}

fn section_file(root: &Path, page: &str, name: &str) -> PathBuf {
    root.join("views")
        .join(page)
        .join("sections")
        .join(format!("{name}.tsx"))
}

fn view_file(root: &Path, page: &str) -> PathBuf {
    root.join("views")
        .join(page)
        .join(format!("{page}.view.tsx"))
}

fn component_file(root: &Path, name: &str) -> PathBuf {
    root.join("components/custom").join(format!("{name}.tsx"))
}

/// Rewrites a file keeping the lines `keep` accepts, matched on the trimmed line.
/// Does nothing when the file is absent or unchanged.
fn keep_lines(path: &Path, keep: impl Fn(&str) -> bool) -> Result<()> {
    let Ok(content) = fs::read_to_string(path) else {
        return Ok(());
    };
    let kept: String = content
        .split_inclusive('\n')
        .filter(|line| keep(line.trim()))
        .collect();
    if kept != content {
        fs::write(path, kept).with_context(|| format!("cannot write {}", path.display()))?;
    }
    Ok(())
}

fn remove_file(path: &Path) -> Result<()> {
    if path.is_file() {
        fs::remove_file(path).with_context(|| format!("cannot remove {}", path.display()))?;
    }
    Ok(())
}

fn remove_dir(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("cannot remove {}", path.display()))?;
    }
    Ok(())
}

fn done(message: &str) {
    println!("{}", output::success(message));
}
