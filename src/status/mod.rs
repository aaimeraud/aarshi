mod components;
mod config;
mod dependencies;
mod i18n;
mod imports;
mod pages;
mod render;
mod sections;

use std::{env, fmt, str::FromStr};

use anyhow::{Context, Result};

use crate::config::Config;

/// A dimension of the report: an independent family of checks with a stable
/// lowercase identifier, listed in canonical display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dimension {
    #[default]
    Config,
    I18n,
    Pages,
    Sections,
    Components,
    Dependencies,
}

impl Dimension {
    /// All dimensions in canonical order.
    pub const ALL: [Dimension; 6] = [
        Dimension::Config,
        Dimension::I18n,
        Dimension::Pages,
        Dimension::Sections,
        Dimension::Components,
        Dimension::Dependencies,
    ];

    /// The stable identifier used by `--dimension` and the JSON `id` field.
    pub fn id(self) -> &'static str {
        match self {
            Dimension::Config => "config",
            Dimension::I18n => "i18n",
            Dimension::Pages => "pages",
            Dimension::Sections => "sections",
            Dimension::Components => "components",
            Dimension::Dependencies => "dependencies",
        }
    }
}

impl FromStr for Dimension {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|d| d.id() == text)
            .ok_or_else(|| {
                format!(
                    "unknown dimension `{text}` (expected one of: {})",
                    Self::ALL
                        .iter()
                        .map(|d| d.id())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

/// The severity of a finding: the project is broken (`error`) or something is
/// still to be done (`todo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Todo,
}

/// Where a finding comes from. Only the relevant keys are set; `line` is
/// reserved and stays absent until checks become positional.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Location {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

/// One problem found in the project: a stable rule identifier, a one-line
/// summary shown to the human, an optional detail and an optional location.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Finding {
    /// Not serialized: the finding is only published inside its dimension.
    #[serde(skip)]
    pub dimension: Dimension,
    pub rule: &'static str,
    pub severity: Severity,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

/// The collected findings of a whole project, sorted for display.
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    /// (`error`, `todo`) counts for one dimension, computed on the full report
    /// whatever the display filters.
    pub fn counts(&self, dimension: Dimension) -> (usize, usize) {
        let mut errors = 0;
        let mut todos = 0;
        for finding in &self.findings {
            if finding.dimension == dimension {
                match finding.severity {
                    Severity::Error => errors += 1,
                    Severity::Todo => todos += 1,
                }
            }
        }
        (errors, todos)
    }

    /// The dimensions kept for display, always in canonical order.
    pub fn selected(&self, options: &Options) -> Vec<Dimension> {
        Dimension::ALL
            .iter()
            .copied()
            .filter(|d| options.dimensions.is_empty() || options.dimensions.contains(d))
            .collect()
    }

    /// (`error`, `todo`) counts for the whole project, on the full report.
    pub fn totals(&self) -> (usize, usize) {
        let mut errors = 0;
        let mut todos = 0;
        for dimension in Dimension::ALL {
            let (dimension_errors, dimension_todos) = self.counts(dimension);
            errors += dimension_errors;
            todos += dimension_todos;
        }
        (errors, todos)
    }

    /// Findings of one dimension as they are shown with the current filters.
    pub fn shown(&self, dimension: Dimension, options: &Options) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.dimension == dimension)
            .filter(|f| !options.errors_only || f.severity == Severity::Error)
            .collect()
    }
}

/// Command-line options of `aarshi status`.
#[derive(Debug, Default)]
pub struct Options {
    /// Restrict the display to these dimensions; empty means all of them.
    /// Never changes the exit code, the counts or the summary.
    pub dimensions: Vec<Dimension>,
    /// Hide `todo` findings from the display; `error` findings and the
    /// exit code stay unchanged.
    pub errors_only: bool,
    /// Write the report as a single JSON document.
    pub json: bool,
}

/// An error that ends the process with a dedicated exit code. `message` is
/// printed on stderr by `main`; leave it empty when the report was already
/// printed and nothing else should be said.
#[derive(Debug)]
pub struct Exit {
    pub code: i32,
    pub message: String,
}

impl Exit {
    fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Exit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for Exit {}

/// Functions that fill findings for one dimension, in a fixed order. Each
/// dimension ticket adds its collector here; the list must stay free of any
/// early stop so every dimension is always visited.
type Collector = fn(&std::path::Path, &Config, &mut Vec<Finding>);

const COLLECTORS: &[Collector] = &[
    config::collect,
    i18n::collect,
    pages::collect,
    sections::collect,
    components::collect,
    // `dependencies` closes the list, in its canonical last slot.
    dependencies::collect,
];

/// Runs the report. Returns `Ok` with exit code `0` (nothing broken), or
/// `Err(status::Exit { 1 })` when at least one `error` was found, or
/// `Err(status::Exit { 2 })` when the project cannot be analyzed (no
/// `aarshi.toml`, an unreadable or an invalid one): the message lands on
/// stderr, nothing on stdout.
pub fn run(options: Options) -> Result<()> {
    let root = env::current_dir()
        .with_context(|| "cannot read the current folder")
        .map_err(|error| Exit::new(2, &format!("cannot analyze aarshi.toml: {error:#}")))?;
    if !crate::config::is_aarshi_project(&root) {
        return Err(Exit::new(2, "no aarshi.toml in this folder; nothing to check").into());
    }
    // An existing but unreadable or invalid `aarshi.toml` cannot be analyzed
    // either (0011 code contract): it stops like the missing file, code 2.
    let project = Config::for_project(&root)
        .map_err(|error| Exit::new(2, &format!("cannot analyze aarshi.toml: {error:#}")))?;

    // No early stop inside the checks: the whole project is walked first.
    let mut findings = Vec::new();
    for collect in COLLECTORS {
        collect(&root, &project, &mut findings);
    }
    findings.sort_by(|a, b| {
        (
            a.location
                .as_ref()
                .and_then(|l| l.file.as_deref())
                .unwrap_or(""),
            a.rule,
        )
            .cmp(&(
                b.location
                    .as_ref()
                    .and_then(|l| l.file.as_deref())
                    .unwrap_or(""),
                b.rule,
            ))
    });
    let report = Report { findings };

    if options.json {
        println!("{}", render::json(&report, &options));
    } else {
        // The header is skipped in JSON mode so stdout stays a single document.
        crate::output::print_header();
        render::human(&report, &options);
    }

    // The exit code is computed on the full report, never on the filters:
    // a hidden `error` still fails the run.
    let (total_errors, _) = report.totals();
    if total_errors > 0 {
        return Err(Exit::new(1, "").into());
    }
    Ok(())
}
