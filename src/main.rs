mod add;
mod config;
mod dictionary;
mod docs;
mod icons;
mod lang;
mod output;
mod project;
mod setup;
mod status;
mod templates;

use anyhow::Result;
use clap::{ArgGroup, Args, Parser, Subcommand};

/// Scaffold Next.js sites.
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
    /// Add a section to a page, a page, or a component. Asks for missing names.
    Add(AddArgs),
    /// Add a language to the project
    AddLang {
        /// Two-letter language code, e.g. `de`. Asked when omitted.
        code: Option<String>,
    },
    /// Turn the SVG files of `components/icons` into React components
    Icons {
        /// Convert every SVG without asking
        #[arg(long)]
        all: bool,
    },
    /// Report what is missing or wrong in the Next.js project
    #[command(visible_alias = "check")]
    Status(StatusArgs),
    /// Open the aarshi documentation in the browser
    Docs,
}

#[derive(Args)]
struct StatusArgs {
    /// Restrict the report to these dimensions, repeatable, commas accepted
    #[arg(
        long,
        value_name = "NAME",
        value_delimiter = ',',
        value_parser = clap::value_parser!(status::Dimension)
    )]
    dimension: Vec<status::Dimension>,
    /// Show broken rules only; kept `todo`s still count in the verdict
    #[arg(long)]
    errors_only: bool,
    /// Write the report as JSON on stdout
    #[arg(long)]
    json: bool,
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
    /// Comma-separated language codes, e.g. `fr,en,ja`. Asked when omitted.
    #[arg(long, value_name = "CODES", value_delimiter = ',')]
    languages: Option<Vec<String>>,
    /// Default language code, served at the root. Asked when omitted.
    #[arg(long, value_name = "CODE")]
    default_language: Option<String>,
    /// UI components: `shadcn` or `base-ui`. Asked when omitted.
    #[arg(long, value_name = "NAME")]
    ui: Option<String>,
    /// Icon library: `lucide` or `none`. Asked when omitted.
    #[arg(long, value_name = "NAME")]
    icons: Option<String>,
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
        conflicts_with_all = ["page_only", "component"]
    )]
    section: Option<String>,
    /// Add a component to `components/custom/`. Without a value, the name is asked.
    #[arg(
        short = 'c',
        long,
        value_name = "NAME",
        num_args = 0..=1,
        default_missing_value = "",
        conflicts_with_all = ["section", "page_only"]
    )]
    component: Option<String>,
    /// Add a page
    #[arg(short = 'p', long, conflicts_with_all = ["section", "component"])]
    page_only: bool,
    /// Page name. Asked when omitted.
    page: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        if let Some(exit) = error.downcast_ref::<status::Exit>() {
            if !exit.message.is_empty() {
                eprintln!("{}", output::error(&exit.message));
            }
            std::process::exit(exit.code);
        }
        eprintln!("{}", output::error(&format!("{error:#}")));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    // `status` keeps the header to itself: it must print nothing on stdout
    // when it stops (JSON strictness, exit code 2 outside a project).
    if !matches!(cli.command, Command::Status(_)) {
        output::print_header();
    }
    match cli.command {
        Command::Setup(args) => setup::run(setup::SetupOptions {
            package_manager: package_manager(&args),
            dir: args.dir,
            languages: args.languages,
            default_language: args.default_language,
            ui: args.ui,
            icons: args.icons,
            agent_feedback: args.agent_feedback,
            cache_components: args.cache_components,
            site_url: args.site_url,
            yes: args.yes,
        }),
        Command::Add(args) => add::run(add::AddOptions {
            section: args.section,
            component: args.component,
            page: args.page,
            page_only: args.page_only,
        }),
        Command::AddLang { code } => lang::run(code),
        Command::Icons { all } => icons::run(all),
        Command::Docs => docs::run(),
        Command::Status(args) => status::run(status::Options {
            dimensions: args.dimension,
            errors_only: args.errors_only,
            json: args.json,
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
