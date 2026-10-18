use serde::Serialize;

use super::{Options, Report, Severity};

/// Findings are indented two spaces under their dimension line.
const INDENT: &str = "  ";

/// `1 error, 2 to do` style counting; `to do` keeps a fixed wording.
fn amount(number: usize, singular: &str, plural: &str) -> String {
    if number == 1 {
        format!("1 {singular}")
    } else {
        format!("{number} {plural}")
    }
}

/// Prints the human report: one line per dimension in canonical order, `✓`
/// on clean dimensions, indented findings, then the verdict on its own line
/// and finally the info line when the filter hides findings.
pub fn human(report: &Report, options: &Options) {
    for dimension in report.selected(options) {
        let shown = report.shown(dimension, options);
        let (errors, todos) = report.counts(dimension);
        if shown.is_empty() {
            if errors == 0 && todos == 0 {
                println!("{} ✓", dimension.id());
            } else {
                // Not clean, but every finding is hidden by the filters.
                println!("{}", dimension.id());
            }
        } else {
            println!("{}", dimension.id());
            for finding in shown {
                let line = match finding.severity {
                    Severity::Error => crate::output::error(&finding.summary),
                    Severity::Todo => crate::output::todo(&finding.summary),
                };
                println!("{INDENT}{line}");
            }
        }
    }

    println!();
    println!("{}", verdict(report));

    let selected = report.selected(options);
    let hidden = report
        .findings
        .iter()
        .filter(|f| !selected.contains(&f.dimension))
        .count();
    if hidden > 0 {
        println!(
            "{}",
            crate::output::info(&format!("{hidden} findings hidden by --dimension"))
        );
    }
}

/// The last report line, on the project as a whole, computed on the full
/// report whatever the display filters say.
fn verdict(report: &Report) -> String {
    let (errors, todos) = report.totals();
    if errors > 0 {
        crate::output::error(&format!(
            "{}, {}",
            amount(errors, "error", "errors"),
            amount(todos, "to do", "to do")
        ))
    } else if todos == 0 {
        crate::output::success("project is conformant")
    } else {
        crate::output::success(&format!(
            "project is conformant ({})",
            amount(todos, "to do", "to do")
        ))
    }
}

// --- JSON ----------------------------------------------------------------

#[derive(Serialize)]
struct JsonDoc<'a> {
    format_version: u8,
    summary: JsonSummary,
    dimensions: Vec<JsonDimension<'a>>,
}

#[derive(Serialize)]
struct JsonSummary {
    errors: usize,
    todos: usize,
    conformant: bool,
}

#[derive(Serialize)]
struct JsonDimension<'a> {
    id: &'static str,
    errors: usize,
    todos: usize,
    findings: Vec<&'a super::Finding>,
}

/// Serializes the report as the schema v1 document: `format_version`, the
/// global (never filtered) summary and the selected dimensions in canonical
/// order. `todo` findings leave `findings` with `--errors-only`, but the
/// counters never change.
pub fn json(report: &Report, options: &Options) -> String {
    let mut dimensions = Vec::new();
    for dimension in report.selected(options) {
        let (errors, todos) = report.counts(dimension);
        dimensions.push(JsonDimension {
            id: dimension.id(),
            errors,
            todos,
            findings: report.shown(dimension, options),
        });
    }
    let (errors, todos) = report.totals();
    let doc = JsonDoc {
        format_version: 1,
        summary: JsonSummary {
            errors,
            todos,
            conformant: errors == 0,
        },
        dimensions,
    };
    serde_json::to_string_pretty(&doc).expect("the schema always serializes")
}
