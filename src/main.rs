mod add;
mod config;
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
    /// Add a section to a page, or a page
    Add(AddArgs),
}

#[derive(Args)]
#[command(group(ArgGroup::new("package-manager").args(["npm", "pnpm", "yarn", "bun"])))]
struct SetupArgs {
    /// Project folder, or `.` for the current folder. Asked when omitted.
    dir: Option<String>,
    /// Use npm (default)
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
    /// Enable agent feedback
    #[arg(long)]
    agent_feedback: bool,
    /// Enable Cache Components
    #[arg(long)]
    cache_components: bool,
    /// Site URL, asked when omitted
    #[arg(long, value_name = "URL")]
    site_url: Option<String>,
}

#[derive(Args)]
struct AddArgs {
    /// Section to add to the page
    #[arg(short = 's', long, value_name = "SECTION")]
    section: Option<String>,
    /// Add a page
    #[arg(short = 'p', long, conflicts_with = "section")]
    page_only: bool,
    /// Page name
    page: String,
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
        }),
        Command::Add(args) => add::run(add::AddOptions {
            section: args.section,
            page: args.page,
            page_only: args.page_only,
        }),
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
