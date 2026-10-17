use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Fake package manager: records its arguments and environment, then creates a bare Next.js project.
const FAKE_NPM: &str = r#"#!/bin/sh
echo "$@" > "$AARSHI_TEST_LOG"
echo "$NEXT_TELEMETRY_DISABLED" >> "$AARSHI_TEST_LOG"
mkdir -p "$3/app"
touch "$3/app/layout.tsx" "$3/app/page.tsx"
printf 'import type { NextConfig } from "next";\n\nconst nextConfig: NextConfig = {\n  cacheComponents: true,\n};\n\nexport default nextConfig;\n' > "$3/next.config.ts"
"#;

struct Sandbox {
    root: PathBuf,
    bin: PathBuf,
    home: PathBuf,
    log: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = fs::remove_dir_all(&root);
        let bin = root.join("bin");
        let home = root.join("home");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(&home).unwrap();
        for manager in ["npm", "pnpm", "yarn", "bun"] {
            let script = bin.join(manager);
            fs::write(&script, FAKE_NPM).unwrap();
            make_executable(&script);
        }
        let log = root.join("npm.log");
        Self {
            root,
            bin,
            home,
            log,
        }
    }

    fn run(&self, cwd: &Path, args: &[&str]) -> Output {
        let path = format!("{}:{}", self.bin.display(), std::env::var("PATH").unwrap());
        Command::new(env!("CARGO_BIN_EXE_aarshi"))
            .args(args)
            .current_dir(cwd)
            .env("PATH", path)
            .env("HOME", &self.home)
            .env("AARSHI_TEST_LOG", &self.log)
            .output()
            .unwrap()
    }

    fn setup(&self, name: &str, extra: &[&str]) -> PathBuf {
        let mut args = vec!["setup", name, "--site-url", "https://example.com/", "--yes"];
        args.extend_from_slice(extra);
        let output = self.run(&self.root, &args);
        assert!(output.status.success(), "{}", stderr(&output));
        self.root.join(name)
    }

    fn add(&self, project: &Path, args: &[&str]) -> Output {
        self.run(project, &[&["add"], args].concat())
    }
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn setup_runs_create_next_app_with_explicit_flags_and_telemetry_off() {
    let sandbox = Sandbox::new("setup-flags");
    let project = sandbox.setup("site", &["--pnpm", "--agent-feedback"]);

    let log = read(&sandbox.log);
    let mut lines = log.lines();
    let args = lines.next().unwrap();
    assert!(args.starts_with("create next-app@latest site"), "{args}");
    assert!(args.contains("--ts --tailwind --eslint --app"), "{args}");
    assert!(args.contains("--agent-feedback"), "{args}");
    assert!(args.contains("--use-pnpm"), "{args}");
    assert!(!args.contains("--yes"), "{args}");
    assert_eq!(lines.next(), Some("1"));

    assert!(read(&project.join(".env")).contains("NEXT_TELEMETRY_DISABLED=1"));
    assert!(!project.join("app/layout.tsx").exists());
    assert!(!project.join("app/page.tsx").exists());
}

#[test]
fn cache_components_are_off_unless_requested() {
    let sandbox = Sandbox::new("cache-components");
    let off = sandbox.setup("off", &[]);
    assert!(read(&off.join("next.config.ts")).contains("cacheComponents: false"));

    let on = sandbox.setup("on", &["--cache-components"]);
    assert!(read(&on.join("next.config.ts")).contains("cacheComponents: true"));
}

#[test]
fn setup_defaults_to_npm_and_writes_project_config() {
    let sandbox = Sandbox::new("setup-defaults");
    let project = sandbox.setup("site", &[]);

    assert!(read(&sandbox.log).contains("--use-npm"));
    let config = read(&project.join("aarshi.toml"));
    assert!(
        config.contains("languages = [\"fr\", \"en\", \"es\", \"ko\"]"),
        "{config}"
    );
    assert!(config.contains("package_manager = \"npm\""), "{config}");
}

#[test]
fn setup_puts_the_default_language_at_root_and_others_under_their_prefix() {
    let sandbox = Sandbox::new("setup-routes");
    let project = sandbox.setup("site", &[]);

    assert!(project.join("app/[locale]/layout.tsx").exists());
    assert!(project.join("app/[locale]/page.tsx").exists());
    assert!(!project.join("app/(fr)").exists());
    assert!(read(&project.join("lib/i18n/config.ts")).contains("defaultLocale: Locale = \"fr\""));
    let config = read(&project.join("next.config.ts"));
    assert!(config.contains("destination: \"/fr\""), "{config}");
}

#[test]
fn setup_rejects_an_existing_folder() {
    let sandbox = Sandbox::new("setup-existing");
    fs::create_dir_all(sandbox.root.join("site")).unwrap();
    let output = sandbox.run(
        &sandbox.root,
        &["setup", "site", "--site-url", "https://example.com"],
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn user_config_sets_languages_and_direction() {
    let sandbox = Sandbox::new("user-config");
    fs::create_dir_all(sandbox.home.join(".config/aarshi")).unwrap();
    fs::write(
        sandbox.home.join(".config/aarshi/config.toml"),
        "languages = [\"ar\", \"en\"]\npackage_manager = \"bun\"\n",
    )
    .unwrap();
    let project = sandbox.setup("site", &[]);

    let config = read(&project.join("aarshi.toml"));
    assert!(config.contains("languages = [\"ar\", \"en\"]"), "{config}");
    assert!(config.contains("package_manager = \"bun\""), "{config}");
    assert!(read(&sandbox.log).contains("--use-bun"));
    assert!(read(&project.join("lib/i18n/config.ts")).contains("ar: \"rtl\""));
    assert!(read(&project.join("app/[locale]/layout.tsx")).contains("dir={direction[locale]}"));
}

#[test]
fn add_page_creates_view_routes_keys_and_sitemap_entry() {
    let sandbox = Sandbox::new("add-page");
    let project = sandbox.setup("site", &[]);

    let output = sandbox.add(&project, &["-p", "about"]);
    assert!(output.status.success(), "{}", stderr(&output));

    assert!(project.join("views/about/about.view.tsx").exists());
    assert!(project.join("app/[locale]/about/page.tsx").exists());
    assert!(read(&project.join("app/sitemap.ts")).contains("\"/about\","));
    assert!(read(&project.join("lib/i18n/dictionaries/fr.ts")).contains("\"about.title\": \"\","));
    assert!(read(&project.join("lib/i18n/dictionaries/en.ts")).contains("\"about.title\": \"\","));
}

#[test]
fn add_section_creates_component_and_keys_in_every_language() {
    let sandbox = Sandbox::new("add-section");
    let project = sandbox.setup("site", &[]);
    sandbox.add(&project, &["-p", "about"]);

    let output = sandbox.add(&project, &["-s", "hero", "about"]);
    assert!(output.status.success(), "{}", stderr(&output));

    assert!(
        read(&project.join("views/about/sections/hero.tsx"))
            .contains("export const HeroSection = ({ locale }")
    );
    let view = read(&project.join("views/about/about.view.tsx"));
    assert!(
        view.contains("import { HeroSection } from \"./sections/hero\";"),
        "{view}"
    );
    assert!(view.contains("<HeroSection locale={locale} />"), "{view}");
    assert!(read(&project.join("lib/i18n/dictionaries/fr.ts")).contains("\"about.hero.title\""));
    assert!(read(&project.join("lib/i18n/dictionaries/en.ts")).contains("\"about.hero.title\""));
}

#[test]
fn add_section_on_a_missing_page_fails_without_a_terminal() {
    let sandbox = Sandbox::new("add-missing-page");
    let project = sandbox.setup("site", &[]);

    let output = sandbox.add(&project, &["-s", "hero", "contact"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("page `contact` does not exist"));
    assert!(!project.join("views/contact").exists());
}

#[test]
fn add_refuses_to_overwrite_an_existing_page() {
    let sandbox = Sandbox::new("add-twice");
    let project = sandbox.setup("site", &[]);
    assert!(sandbox.add(&project, &["-p", "about"]).status.success());

    let output = sandbox.add(&project, &["-p", "about"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn add_lang_creates_dictionary_and_updates_config_without_new_routes() {
    let sandbox = Sandbox::new("add-lang");
    let project = sandbox.setup("site", &[]);
    assert!(sandbox.add(&project, &["-p", "about"]).status.success());

    let output = sandbox.run(&project, &["add-lang", "de"]);
    assert!(output.status.success(), "{}", stderr(&output));

    let dictionary = read(&project.join("lib/i18n/dictionaries/de.ts"));
    assert!(dictionary.contains("import type { messages as base } from \"./fr\";"));
    assert!(
        dictionary.contains("\"about.title\": \"\","),
        "{dictionary}"
    );
    assert!(dictionary.contains("\"home.title\": \"\","), "{dictionary}");
    assert!(!project.join("app/(de)").exists());
    assert!(project.join("app/[locale]/about/page.tsx").exists());
    assert!(
        read(&project.join("lib/i18n/config.ts"))
            .contains("\"fr\", \"en\", \"es\", \"ko\", \"de\"")
    );
    assert!(read(&project.join("aarshi.toml")).contains("\"de\""));
    assert!(read(&project.join("aarshi.toml")).contains("package_manager = \"npm\""));
}

#[test]
fn add_lang_rejects_a_language_already_in_the_project() {
    let sandbox = Sandbox::new("add-lang-twice");
    let project = sandbox.setup("site", &[]);

    let output = sandbox.run(&project, &["add-lang", "en"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn check_reports_empty_translations_until_they_are_filled() {
    let sandbox = Sandbox::new("check");
    let project = sandbox.setup("site", &[]);
    sandbox.run(&project, &["add-lang", "de"]);

    let output = sandbox.run(&project, &["check"]);
    assert!(!output.status.success());
    let report = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        report.contains("de: empty translation for `home.title`"),
        "{report}"
    );

    for locale in ["fr", "en", "es", "ko", "de"] {
        let path = project.join(format!("lib/i18n/dictionaries/{locale}.ts"));
        let filled = read(&path).replace("\"home.title\": \"\"", "\"home.title\": \"Home\"");
        fs::write(&path, filled).unwrap();
    }
    let output = sandbox.run(&project, &["check"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn setup_with_yes_needs_a_folder_name() {
    let sandbox = Sandbox::new("setup-yes-no-dir");
    let output = sandbox.run(&sandbox.root, &["setup", "--yes"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("folder is required"));
}

#[test]
fn add_reads_the_languages_written_at_setup() {
    let sandbox = Sandbox::new("add-languages");
    fs::create_dir_all(sandbox.home.join(".config/aarshi")).unwrap();
    fs::write(
        sandbox.home.join(".config/aarshi/config.toml"),
        "languages = [\"ar\", \"en\"]\n",
    )
    .unwrap();
    let project = sandbox.setup("site", &[]);

    assert!(sandbox.add(&project, &["-p", "about"]).status.success());
    assert!(project.join("app/[locale]/about/page.tsx").exists());
    assert!(read(&project.join("lib/i18n/config.ts")).contains("\"ar\", \"en\""));
}
