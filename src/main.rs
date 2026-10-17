mod add;
mod check;
mod config;
mod dictionary;
mod lang;
mod project;
mod setup;
mod templates;

use anyhow::Result;
use clap::{ArgGroup, Args, Parser, Subcommand};

/// Scaffold multilingual Next.js sites.
#[derive(Parser)]
#[command(name = "aarshi", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a Next.js project
    Setup(SetupArgs),
    /// Add a section to a page, or a page. Asks for missing names.
    Add(AddArgs),
    /// Add a language to the project
    AddLang {
        /// Two-letter language code, e.g. `de`. Asked when omitted.
        code: Option<String>,
    },
    /// Report empty translations and missing routes
    Check,
}

#[derive(Args)]
#[command(group(ArgGroup::new("package-manager").args(["npm", "pnpm", "yarn", "bun"])))]
struct SetupArgs {
    /// Project folder, or `.` for the current folder. Asked when omitted.
    dir: Option<String>,
    /// Use npm
    #[arg(long)]
    npm: bool,
    /// Use pnpm
    #[arg(long)]
    pnpm: bool,
    /// Use yarn
    #[arg(long)]
    yarn: bool,
    /// Use bun
    #[arg(long)]
    bun: bool,
    /// Enable agent feedback. Asked when no option flag is given.
    #[arg(long)]
    agent_feedback: bool,
    /// Enable Cache Components. Asked when no option flag is given.
    #[arg(long)]
    cache_components: bool,
    /// Site URL, asked when omitted
    #[arg(long, value_name = "URL")]
    site_url: Option<String>,
    /// Accept defaults for every question. Requires the folder argument.
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
struct AddArgs {
    /// Add a section with this name. Without a value, the name is asked.
    #[arg(
        short = 's',
        long,
        value_name = "SECTION",
        num_args = 0..=1,
        default_missing_value = "",
        conflicts_with = "page_only"
    )]
    section: Option<String>,
    /// Add a page
    #[arg(short = 'p', long, conflicts_with = "section")]
    page_only: bool,
    /// Page name. Asked when omitted.
    page: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Setup(args) => setup::run(setup::SetupOptions {
            package_manager: package_manager(&args),
            dir: args.dir,
            agent_feedback: args.agent_feedback,
            cache_components: args.cache_components,
            site_url: args.site_url,
            yes: args.yes,
        }),
        Command::Add(args) => add::run(add::AddOptions {
            section: args.section,
            page: args.page,
            page_only: args.page_only,
        }),
        Command::AddLang { code } => lang::run(code),
        Command::Check => check::run(),
    }
}

fn package_manager(args: &SetupArgs) -> Option<String> {
    [
        (args.npm, "npm"),
        (args.pnpm, "pnpm"),
        (args.yarn, "yarn"),
        (args.bun, "bun"),
    ]
    .into_iter()
    .find(|(selected, _)| *selected)
    .map(|(_, name)| name.to_string())
}
