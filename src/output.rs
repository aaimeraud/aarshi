use colored::Colorize;

/// Release date of the current version, shown in the header.
const RELEASE_DATE: &str = "2026-10-18";

/// GitHub repository URL, shown in the header.
const REPOSITORY_URL: &str = "https://github.com/aaimeraud/aarshi";

/// Prints the aarshi header unless the environment asks to keep output quiet.
///
/// The header is skipped when `AARSHI_NO_HEADER` is set so tests and scripts
/// are not flooded with banner lines.
pub fn print_header() {
    if std::env::var("AARSHI_NO_HEADER").is_ok() {
        return;
    }

    let banner = r#"
                      _     _ 
  __ _  __ _ _ __ ___| |__ (_)
 / _` |/ _` | '__/ __| '_ \| |
| (_| | (_| | |  \__ \ | | | |
 \__,_|\__,_|_|  |___/_| |_|_| "#;

    println!("{}", banner.bright_cyan().bold());
    println!();
    println!(
        "  {}  {}",
        "version".bright_black().bold(),
        env!("CARGO_PKG_VERSION").cyan()
    );
    println!(
        "  {}  {}",
        "release".bright_black().bold(),
        RELEASE_DATE.cyan()
    );
    println!(
        "  {}   {}",
        "author".bright_black().bold(),
        "aaimeraud".cyan()
    );
    println!(
        "  {}   {}",
        "github".bright_black().bold(),
        REPOSITORY_URL.bright_blue().underline()
    );
    println!();
}

/// Formats a fatal error message in red.
pub fn error(message: &str) -> String {
    format!("{} {}", "error:".red().bold(), message)
}

/// Formats a warning message in yellow.
pub fn warning(message: &str) -> String {
    format!("{} {}", "warning:".yellow().bold(), message)
}

/// Formats a to-do finding in yellow, symmetric of `warning()`.
pub fn todo(message: &str) -> String {
    format!("{} {}", "todo:".yellow().bold(), message)
}

/// Formats a success message in green.
pub fn success(message: &str) -> String {
    format!("{} {}", "✓".green().bold(), message.green())
}

/// Formats an informational message in bright blue.
pub fn info(message: &str) -> String {
    format!("{} {}", "info:".bright_blue().bold(), message)
}
