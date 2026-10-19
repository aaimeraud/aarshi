use std::{env, fs, process::Command};

use anyhow::{Context, Result};

use crate::output;

const LAYOUT: &str = include_str!("docs/layout.html");
const STYLE: &str = include_str!("docs/style.css");
const REPOSITORY_URL: &str = "https://github.com/aaimeraud/aarshi";
const LOGO: &[u8] = include_bytes!("../assets/aarshi-icon.png");

/// The languages of the documentation. Their order matches `Page::titles` and `Page::bodies`.
const LANGS: &[Lang] = &[
    Lang {
        code: "en",
        name: "English",
        sidebar: "Documentation",
        language: "Language",
        navigation: "Page navigation",
        previous: "Previous",
        next: "Next",
        theme_dark: "Dark mode",
        theme_light: "Light mode",
        repository: "Repository on GitHub",
    },
    Lang {
        code: "fr",
        name: "Français",
        sidebar: "Documentation",
        language: "Langue",
        navigation: "Navigation entre les pages",
        previous: "Précédent",
        next: "Suivant",
        theme_dark: "Mode sombre",
        theme_light: "Mode clair",
        repository: "Dépôt GitHub",
    },
];

/// The pages, in sidebar order. `titles` and `bodies` follow `LANGS`.
const PAGES: &[Page] = &[
    Page {
        key: "index",
        titles: ["Overview", "Vue d'ensemble"],
        bodies: [
            include_str!("docs/en/index.html"),
            include_str!("docs/fr/index.html"),
        ],
    },
    Page {
        key: "config",
        titles: ["The aarshi.toml file", "Le fichier aarshi.toml"],
        bodies: [
            include_str!("docs/en/config.html"),
            include_str!("docs/fr/config.html"),
        ],
    },
    Page {
        key: "commands",
        titles: ["Commands", "Commandes"],
        bodies: [
            include_str!("docs/en/commands.html"),
            include_str!("docs/fr/commands.html"),
        ],
    },
    Page {
        key: "workflow",
        titles: ["Example workflow", "Exemple pas à pas"],
        bodies: [
            include_str!("docs/en/workflow.html"),
            include_str!("docs/fr/workflow.html"),
        ],
    },
    Page {
        key: "status",
        titles: ["Checking a project", "Vérifier un projet"],
        bodies: [
            include_str!("docs/en/status.html"),
            include_str!("docs/fr/status.html"),
        ],
    },
    Page {
        key: "troubleshooting",
        titles: ["Troubleshooting", "Dépannage"],
        bodies: [
            include_str!("docs/en/troubleshooting.html"),
            include_str!("docs/fr/troubleshooting.html"),
        ],
    },
    Page {
        key: "contributing",
        titles: ["Contributing", "Contribuer"],
        bodies: [
            include_str!("docs/en/contributing.html"),
            include_str!("docs/fr/contributing.html"),
        ],
    },
];

struct Lang {
    code: &'static str,
    name: &'static str,
    sidebar: &'static str,
    language: &'static str,
    navigation: &'static str,
    previous: &'static str,
    next: &'static str,
    theme_dark: &'static str,
    theme_light: &'static str,
    repository: &'static str,
}

struct Page {
    key: &'static str,
    titles: [&'static str; 2],
    bodies: [&'static str; 2],
}

/// Writes every page in every language to a temporary folder, emptied on each
/// call, then opens the overview of the system language. `AARSHI_NO_OPEN`
/// skips the opening.
pub fn run() -> Result<()> {
    let root = env::temp_dir().join("aarshi-docs");
    if root.exists() {
        fs::remove_dir_all(&root).with_context(|| format!("cannot clear {}", root.display()))?;
    }
    fs::create_dir_all(&root).with_context(|| format!("cannot create {}", root.display()))?;
    fs::write(root.join("style.css"), STYLE).context("cannot write style.css")?;
    fs::write(root.join("aarshi-icon.png"), LOGO).context("cannot write the logo")?;

    for (lang, labels) in LANGS.iter().enumerate() {
        let dir = root.join(labels.code);
        fs::create_dir_all(&dir).with_context(|| format!("cannot create {}", dir.display()))?;
        for (index, page) in PAGES.iter().enumerate() {
            let file = dir.join(format!("{}.html", page.key));
            fs::write(&file, render(lang, index))
                .with_context(|| format!("cannot write {}", file.display()))?;
        }
    }

    // LANGS lists English first, French second.
    let lang = usize::from(system_language_is_french());
    let page = root.join(LANGS[lang].code).join("index.html");
    println!(
        "{}",
        output::info(&format!("documentation written to {}", root.display()))
    );
    if env::var_os("AARSHI_NO_OPEN").is_none()
        && let Err(error) = open(&page)
    {
        eprintln!(
            "{}",
            output::warning(&format!("cannot open the browser: {error:#}"))
        );
    }
    Ok(())
}

/// One page in one language: the sidebar, the language switch and the
/// previous/next links are filled in around the page body.
fn render(lang: usize, index: usize) -> String {
    let labels = &LANGS[lang];
    let page = &PAGES[index];
    let sidebar = PAGES
        .iter()
        .enumerate()
        .map(|(i, other)| {
            let current = if i == index {
                " aria-current=\"page\""
            } else {
                ""
            };
            format!(
                "        <li><a href=\"{}.html\"{current}>{}</a></li>",
                other.key, other.titles[lang]
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let switch = LANGS
        .iter()
        .enumerate()
        .map(|(other, target)| {
            let current = if other == lang { "true" } else { "false" };
            format!(
                "      <a href=\"../{}/{}.html\" hreflang=\"{}\" aria-current=\"{current}\">{}</a>",
                target.code, page.key, target.code, target.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut pager = String::new();
    if index > 0 {
        let previous = &PAGES[index - 1];
        pager.push_str(&format!(
            "<a class=\"prev\" href=\"{}.html\"><span>{}</span>{}</a>",
            previous.key, labels.previous, previous.titles[lang]
        ));
    }
    if let Some(next) = PAGES.get(index + 1) {
        pager.push_str(&format!(
            "<a class=\"next\" href=\"{}.html\"><span>{}</span>{}</a>",
            next.key, labels.next, next.titles[lang]
        ));
    }

    LAYOUT
        .replace("{{lang}}", labels.code)
        .replace("{{title}}", &format!("{} · aarshi", page.titles[lang]))
        .replace("{{sidebar_label}}", labels.sidebar)
        .replace("{{sidebar}}", &sidebar)
        .replace("{{language_label}}", labels.language)
        .replace("{{switch}}", &switch)
        .replace("{{content}}", page.bodies[lang])
        .replace("{{navigation_label}}", labels.navigation)
        .replace("{{pager}}", &pager)
        .replace("{{repository_url}}", REPOSITORY_URL)
        .replace("{{repository_label}}", labels.repository)
        .replace("{{theme_dark}}", labels.theme_dark)
        .replace("{{theme_light}}", labels.theme_light)
}

fn system_language_is_french() -> bool {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|var| env::var(var).ok().filter(|value| !value.is_empty()))
        .is_some_and(|value| value.starts_with("fr"))
}

/// Opens a file with the system viewer, without waiting for it to close.
fn open(page: &std::path::Path) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(page).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(page)
            .status()
    } else {
        Command::new("xdg-open").arg(page).status()
    };
    let status = status.with_context(|| format!("cannot run the opener for {}", page.display()))?;
    if !status.success() {
        anyhow::bail!("the opener exited with {status}");
    }
    Ok(())
}
