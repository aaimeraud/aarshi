use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Fake package manager: records its arguments and environment, creates a bare Next.js
/// project on `create` (the files create-next-app writes, ready for the
/// `status` checks), and does nothing on `add` (installations).
const FAKE_NPM: &str = r#"#!/bin/sh
echo "$@" >> "$AARSHI_TEST_LOG"
echo "$NEXT_TELEMETRY_DISABLED" >> "$AARSHI_TEST_LOG"
if [ "$1" = "create" ]; then
  mkdir -p "$3/app"
  touch "$3/app/layout.tsx" "$3/app/page.tsx"
  printf 'import type { NextConfig } from "next";\n\nconst nextConfig: NextConfig = {\n  cacheComponents: true,\n};\n\nexport default nextConfig;\n' > "$3/next.config.ts"
  printf '{\n  "dependencies": {\n    "@base-ui/react": "^1.0.0",\n    "@sglara/cn": "^1.0.0",\n    "lucide-react": "^0.500.0",\n    "next": "^15.0.0",\n    "react": "^19.0.0"\n  }\n}\n' > "$3/package.json"
  printf '{}\n' > "$3/tsconfig.json"
  printf 'export default [];\n' > "$3/eslint.config.mjs"
  printf 'export default {};\n' > "$3/postcss.config.mjs"
fi
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
            .env("AARSHI_NO_HEADER", "1")
            .output()
            .unwrap()
    }

    /// `base-ui` keeps the tests offline: `shadcn` would need npx and the network.
    fn setup(&self, name: &str, extra: &[&str]) -> PathBuf {
        let has_ui = extra.contains(&"--ui");
        let has_icons = extra.contains(&"--icons");
        let mut args = vec!["setup", name, "--site-url", "https://example.com/", "--yes"];
        if !has_ui {
            args.extend(["--ui", "base-ui"]);
        }
        if !has_icons {
            args.extend(["--icons", "lucide"]);
        }
        args.extend_from_slice(extra);
        let output = self.run(&self.root, &args);
        assert!(output.status.success(), "{}", stderr(&output));
        self.root.join(name)
    }

    fn add(&self, project: &Path, args: &[&str]) -> Output {
        self.run(project, &[&["add"], args].concat())
    }

    /// Runs without `AARSHI_NO_HEADER`, so `status` shows its banner.
    fn run_with_header(&self, cwd: &Path, args: &[&str]) -> Output {
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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

// ==== `status` (tracer slice) ============================================

/// Writes one `<locale>.ts` dictionary with `"key": "value",` lines.
fn write_dictionary(root: &Path, locale: &str, entries: &[(&str, &str)]) {
    let keys = entries
        .iter()
        .map(|(key, value)| format!("  \"{key}\": \"{value}\",\n"))
        .collect::<String>();
    fs::write(
        root.join(format!("lib/i18n/dictionaries/{locale}.ts")),
        format!("export const messages = {{\n{keys}}};\n"),
    )
    .unwrap();
}

/// A faithful `lib/i18n/config.ts` for the default fixture languages, as the
/// setup template writes it.
const FIXTURE_CONFIG: &str = r#"export const languages = ["fr", "en"] as const;

export type Locale = (typeof languages)[number];

export const defaultLocale: Locale = "fr";

export const direction: Record<Locale, "ltr" | "rtl"> = {
  fr: "ltr",
  en: "ltr"
};

export const isLocale = (value: string): value is Locale => {
  return (languages as readonly string[]).includes(value);
};

/** Public URL of a page within a locale, e.g. pagePath("fr", "/about") === "/fr/about". */
export const pagePath = (locale: Locale, path: string): string => {
  return path === "/" ? `/${locale}` : `/${locale}${path}`;
};
"#;

/// A faithful `lib/i18n/index.ts` for the default fixture languages.
const FIXTURE_INDEX: &str = r#"import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { SITE_URL } from "@/lib/site";

import { isLocale, languages, pagePath, type Locale } from "./config";
import { messages as fr } from "./dictionaries/fr";
import { messages as en } from "./dictionaries/en";

export * from "./config";

export type MessageKey = keyof typeof fr;

const messages = { fr, en } satisfies Record<Locale, Record<MessageKey, string>>;
"#;

/// Writes the base files and folders of a generated project (inventory 0008)
/// with a marker, a redirect and options matching the full `aarshi.toml`, so
/// the `config` checks of `status` stay clean on the fixture. `config.ts` and
/// `index.ts` are left to the faithful i18n fixture files.
fn write_base_files(root: &Path) {
    for (file, content) in [
        (
            "app/[locale]/layout.tsx",
            "import LanguageSwitcher from \"@/components/language-switcher\";\n",
        ),
        (
            "app/sitemap.ts",
            "const pages = [\"/\",\n  // aarshi:pages:end\n];\n",
        ),
        ("app/robots.ts", "export const robots = [];\n"),
        ("lib/site.ts", "export const SITE_URL = \"https://example.com\";\n"),
        ("components/language-switcher.tsx", "import Link from \"next/link\";\n"),
        ("tsconfig.json", "{}\n"),
        ("eslint.config.mjs", "export default [];\n"),
        ("postcss.config.mjs", "export default {};\n"),
        (
            "next.config.ts",
            "const nextConfig = {\n  async redirects() {\n    return [{ source: \"/\", destination: \"/fr\", permanent: false }];\n  },\n  cacheComponents: false,\n};\n\nexport default nextConfig;\n",
        ),
        ("components/ui/button.tsx", "export const Button = () => null;\n"),
        (
            "components/ui/button-group.tsx",
            "export const ButtonGroup = () => null;\n",
        ),
        (
            "package.json",
            "{\n  \"dependencies\": {\n    \"@base-ui/react\": \"^1.0.0\",\n    \"@sglara/cn\": \"^0.4.0\",\n    \"lucide-react\": \"^0.500.0\"\n  }\n}\n",
        ),
    ] {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    fs::create_dir_all(root.join("components/custom")).unwrap();
    fs::create_dir_all(root.join("components/icons")).unwrap();
}

/// A minimal generated project covering the `i18n` checks: the whole base
/// inventory with consistent config and complete dictionaries. The project
/// lives in a `project/` subfolder to stay clear of the sandbox files.
fn i18n_project(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name).join("project");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("lib/i18n/dictionaries")).unwrap();
    fs::write(
        root.join("aarshi.toml"),
        "languages = [\"fr\", \"en\"]\npackage_manager = \"npm\"\nui = \"base-ui\"\nicons = \"lucide\"\ncache_components = false\nsite_url = \"https://example.com\"\n",
    )
    .unwrap();
    fs::write(root.join("lib/i18n/config.ts"), FIXTURE_CONFIG).unwrap();
    fs::write(root.join("lib/i18n/index.ts"), FIXTURE_INDEX).unwrap();
    write_dictionary(&root, "fr", &[("home.title", "Accueil")]);
    write_dictionary(&root, "en", &[("home.title", "Home")]);
    write_base_files(&root);
    root
}

/// The clean project plus one error (dictionary gone: the i18n rule fires
/// and the import in the faithful index breaks for `config`) and one to do
/// (empty-after-trim value), the two severities the tracer covers.
fn broken_project(name: &str) -> PathBuf {
    let root = i18n_project(name);
    fs::remove_file(root.join("lib/i18n/dictionaries/en.ts")).unwrap();
    write_dictionary(&root, "fr", &[("home.title", "Accueil"), ("home.intro", " ")]);
    root
}

/// The clean project plus one `todo` only: the same empty key in both
/// dictionaries, so no key is missing.
fn todo_project(name: &str) -> PathBuf {
    let root = i18n_project(name);
    write_dictionary(&root, "fr", &[("home.title", "Accueil"), ("home.intro", "")]);
    write_dictionary(&root, "en", &[("home.title", "Home"), ("home.intro", "")]);
    root
}

#[test]
fn status_reproduces_the_contract_render_on_a_fixture() {
    let sandbox = Sandbox::new("status-render");
    let project = broken_project("status-render");

    let output = sandbox.run(&project, &["status"]);
    assert_eq!(
        stdout(&output),
        "config\n\
         \x20 error: lib/i18n/index.ts imports `./dictionaries/en`, but the file does not exist\n\
         i18n\n\
         \x20 error: no dictionary for `en`\n\
         \x20 todo: fr has an empty translation for `home.intro`\n\
         pages ✓\n\
         sections ✓\n\
         components ✓\n\
         dependencies ✓\n\
         \n\
         error: 2 errors, 1 to do\n"
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).is_empty());
}

#[test]
fn status_lists_every_dimension_and_verifies_them_all_clean() {
    let sandbox = Sandbox::new("status-clean");
    let project = i18n_project("status-clean");

    let output = sandbox.run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "config ✓\n\
         i18n ✓\n\
         pages ✓\n\
         sections ✓\n\
         components ✓\n\
         dependencies ✓\n\
         \n\
         ✓ project is conformant\n"
    );
}

#[test]
fn status_todos_alone_still_exit_zero() {
    let sandbox = Sandbox::new("status-todos");
    let project = todo_project("status-todos");

    let output = sandbox.run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "config ✓\n\
         i18n\n\
         \x20 todo: en has an empty translation for `home.intro`\n\
         \x20 todo: fr has an empty translation for `home.intro`\n\
         pages ✓\n\
         sections ✓\n\
         components ✓\n\
         dependencies ✓\n\
         \n\
         ✓ project is conformant (2 to do)\n"
    );
}

#[test]
fn status_counts_whitespace_values_as_empty() {
    let sandbox = Sandbox::new("status-blank");
    let project = i18n_project("status-blank");
    // `intro.text` must exist in the reference dictionary too, so only the
    // empty-value rule is at play.
    write_dictionary(
        &project,
        "fr",
        &[("home.title", "Accueil"), ("intro.text", "Intro")],
    );
    write_dictionary(&project, "en", &[("home.title", " "), ("intro.text", "\n")]);

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert!(
        report.contains("todo: en has an empty translation for `home.title`"),
        "{report}"
    );
    assert!(
        report.contains("todo: en has an empty translation for `intro.text`"),
        "{report}"
    );
    assert!(report.contains("✓ project is conformant (2 to do)"), "{report}");
}

#[test]
fn status_reports_a_key_missing_from_a_non_default_dictionary() {
    let sandbox = Sandbox::new("status-key-missing");
    let project = i18n_project("status-key-missing");
    write_dictionary(&project, "en", &[]);

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: en is missing a translation for `home.title`"),
        "{report}"
    );
    assert!(report.contains("error: 1 error"), "{report}");
}

#[test]
fn status_reports_a_languages_mismatch() {
    let sandbox = Sandbox::new("status-mismatch");
    let project = i18n_project("status-mismatch");
    fs::write(
        project.join("lib/i18n/config.ts"),
        "export const languages = [\"fr\"] as const;\n",
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: lib/i18n/config.ts lists [\"fr\"], but aarshi.toml lists [\"fr\", \"en\"]"),
        "{report}"
    );
}

#[test]
fn check_gives_the_same_report_as_status() {
    let sandbox = Sandbox::new("status-alias");
    let project = broken_project("status-alias");

    let with_status = sandbox.run(&project, &["status"]);
    let with_check = sandbox.run(&project, &["check"]);
    assert_eq!(stdout(&with_status), stdout(&with_check));
    assert_eq!(stderr(&with_status), stderr(&with_check));
    assert_eq!(with_status.status.code(), with_check.status.code());
    assert_eq!(with_status.status.code(), Some(1));
}

#[test]
fn the_help_lists_status_last_with_check_as_alias() {
    let sandbox = Sandbox::new("status-help");
    let output = sandbox.run(&sandbox.root, &["--help"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let help = stdout(&output);

    let position = |needle: &str| help.find(needle).expect(needle);
    assert!(
        position("setup") < position("add") && position("add") < position("add-lang"),
        "{help}"
    );
    assert!(position("add-lang") < position("icons"), "{help}");
    assert!(position("icons") < position("status"), "{help}");
    assert!(status_last_with_alias_shown_in_help(&help), "{help}");
    assert!(
        !help.contains("Report empty translations and missing routes"),
        "`--help` should no longer promote `check`: {help}"
    );

    let about = sandbox.run(&sandbox.root, &["status", "--help"]);
    let status_help = stdout(&about);
    assert!(
        status_help.contains("Report what is missing or wrong in the Next.js project"),
        "{status_help}"
    );
    assert!(status_help.contains("--dimension"), "{status_help}");
    assert!(status_help.contains("--errors-only"), "{status_help}");
    assert!(status_help.contains("--json"), "{status_help}");
}

/// Stands for the visible `check` alias in the `status` help entry.
fn status_last_with_alias_shown_in_help(help: &str) -> bool {
    help.split('\n')
        .find(|line| line.contains("status"))
        .map(|entry| entry.contains("check"))
        .unwrap_or(false)
}

#[test]
fn an_unknown_dimension_is_a_usage_error() {
    let sandbox = Sandbox::new("status-dimension-unknown");
    let project = i18n_project("status-dimension-unknown");

    let output = sandbox.run(&project, &["status", "--dimension", "nope"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("invalid value 'nope'"), "{}", stderr(&output));
    assert!(stdout(&output).is_empty());
}

#[test]
fn status_exits_two_outside_a_project_and_prints_nothing() {
    let sandbox = Sandbox::new("status-outside");

    let output = sandbox.run(&sandbox.root, &["status"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("no aarshi.toml in this folder; nothing to check"),
        "{}",
        stderr(&output)
    );

    // The message replaces the old `check` wording, also under the alias.
    let output = sandbox.run(&sandbox.root, &["check"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("no aarshi.toml in this folder; nothing to check"));
}

#[test]
fn status_json_matches_the_v1_schema_field_per_field() {
    let sandbox = Sandbox::new("status-json");
    let project = broken_project("status-json");

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        r#"{
  "format_version": 1,
  "summary": {
    "errors": 2,
    "todos": 1,
    "conformant": false
  },
  "dimensions": [
    {
      "id": "config",
      "errors": 1,
      "todos": 0,
      "findings": [
        {
          "rule": "config.import-broken",
          "severity": "error",
          "summary": "lib/i18n/index.ts imports `./dictionaries/en`, but the file does not exist",
          "location": {
            "file": "lib/i18n/index.ts"
          }
        }
      ]
    },
    {
      "id": "i18n",
      "errors": 1,
      "todos": 1,
      "findings": [
        {
          "rule": "i18n.dictionary-missing",
          "severity": "error",
          "summary": "no dictionary for `en`",
          "detail": "expected lib/i18n/dictionaries/en.ts",
          "location": {
            "file": "lib/i18n/dictionaries/en.ts",
            "locale": "en"
          }
        },
        {
          "rule": "i18n.key-empty",
          "severity": "todo",
          "summary": "fr has an empty translation for `home.intro`",
          "location": {
            "file": "lib/i18n/dictionaries/fr.ts",
            "locale": "fr",
            "key": "home.intro"
          }
        }
      ]
    },
    {
      "id": "pages",
      "errors": 0,
      "todos": 0,
      "findings": []
    },
    {
      "id": "sections",
      "errors": 0,
      "todos": 0,
      "findings": []
    },
    {
      "id": "components",
      "errors": 0,
      "todos": 0,
      "findings": []
    },
    {
      "id": "dependencies",
      "errors": 0,
      "todos": 0,
      "findings": []
    }
  ]
}
"#
    );
}

#[test]
fn status_json_stays_strict_without_header_env_and_outside_a_project() {
    let sandbox = Sandbox::new("status-json-strict");
    let project = i18n_project("status-json-strict");

    // AARSHI_NO_HEADER unset: no banner, no color, valid JSON only.
    let output = sandbox.run_with_header(&project, &["status", "--json"]);
    let document = stdout(&output);
    assert!(document.starts_with('{'), "{document}");
    assert!(!document.contains('\u{1b}'), "{document}");
    assert!(document.contains("format_version"), "{document}");
    serde_json::from_str::<serde_json::Value>(&document)
        .unwrap_or_else(|e| panic!("invalid JSON: {e}\n{document}"));

    // Stopped run: nothing on stdout even in JSON mode, still code 2.
    let output = sandbox.run(&sandbox.root, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(serde_json::from_str::<serde_json::Value>(&stdout(&output)).is_err());
}

#[test]
fn status_json_keeps_the_filter_to_selected_dimensions() {
    let sandbox = Sandbox::new("status-json-filter");
    let project = broken_project("status-json-filter");

    let output = sandbox.run(
        &project,
        &["status", "--json", "--dimension", "i18n,dependencies"],
    );
    assert_eq!(output.status.code(), Some(1));
    let document = serde_json::from_str::<serde_json::Value>(&stdout(&output)).unwrap();
    let ids: Vec<&str> = document["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["i18n", "dependencies"], "{}", stdout(&output));
    // The summary never narrows to the filter: the hidden error still bites.
    assert_eq!(document["summary"]["errors"], 2);
}

#[test]
fn status_json_errors_only_drops_todos_but_keeps_the_counters() {
    let sandbox = Sandbox::new("status-json-errors-only");
    let project = todo_project("status-json-errors-only");

    let output = sandbox.run(&project, &["status", "--json", "--errors-only"]);
    assert_eq!(output.status.code(), Some(0));
    let document = serde_json::from_str::<serde_json::Value>(&stdout(&output)).unwrap();
    let i18n = &document["dimensions"][1];
    assert_eq!(i18n["id"], "i18n");
    assert_eq!(i18n["todos"], 2, "{}", stdout(&output));
    assert!(
        i18n["findings"].as_array().unwrap().is_empty(),
        "{}",
        stdout(&output)
    );
    // Summary still counts the to dos.
    assert_eq!(document["summary"]["todos"], 2);
}

#[test]
fn status_human_output_reports_hidden_findings_under_a_filter() {
    let sandbox = Sandbox::new("status-hidden");
    let project = broken_project("status-hidden");

    let output = sandbox.run(&project, &["status", "--dimension", "pages"]);
    assert_eq!(
        stdout(&output),
        "pages ✓\n\
         \n\
         error: 2 errors, 1 to do\n\
         info: 3 findings hidden by --dimension\n"
    );

    // Repeats and comma lists pick the same dimensions.
    let repeated = sandbox.run(
        &project,
        &["status", "--dimension", "pages", "--dimension", "pages"],
    );
    assert_eq!(stdout(&repeated), stdout(&output));
    let listed = sandbox.run(&project, &["status", "--dimension", "pages,components"]);
    assert!(stdout(&listed).starts_with("pages ✓\ncomponents ✓\n"), "{}", stdout(&listed));
    // The exit code still comes from the hidden errors of the whole project.
    assert_eq!(listed.status.code(), Some(1));
}

#[test]
fn status_shows_the_header_in_human_mode_but_not_in_json() {
    let sandbox = Sandbox::new("status-header");
    let project = i18n_project("status-header");

    let output = sandbox.run_with_header(&project, &["status"]);
    assert!(stdout(&output).contains("version"), "{}", stdout(&output));

    let header = sandbox.run_with_header(&project, &["status", "--json"]);
    let document = stdout(&header);
    assert!(document.starts_with('{'), "{document}");
    assert!(!document.contains("github"), "{document}");
}

#[test]
fn status_reports_empty_translations_until_they_are_filled() {
    let sandbox = Sandbox::new("status-filled");
    let project = sandbox.setup("site", &[]);
    sandbox.run(&project, &["add-lang", "de"]);

    let output = sandbox.run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(0));
    let report = stdout(&output);
    assert!(
        report.contains("todo: de has an empty translation for `home.title`"),
        "{report}"
    );
    assert!(report.contains("✓ project is conformant (3 to do)"), "{report}");

    for locale in ["fr", "en", "de"] {
        let path = project.join(format!("lib/i18n/dictionaries/{locale}.ts"));
        let filled = read(&path).replace("\"home.title\": \"\"", "\"home.title\": \"Home\"");
        fs::write(&path, filled).unwrap();
    }
    let output = sandbox.run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(0));
    let report = stdout(&output);
    assert!(report.contains("✓ project is conformant\n"), "{report}");
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
fn setup_omits_the_version_tag_for_yarn() {
    let sandbox = Sandbox::new("setup-yarn");
    sandbox.setup("site", &["--yarn"]);

    let log = read(&sandbox.log);
    let args = log.lines().next().unwrap();
    assert!(args.starts_with("create next-app site"), "{args}");
    assert!(!args.contains("next-app@latest"), "{args}");
}

#[test]
fn setup_installs_the_class_helper_and_the_icon_library() {
    let sandbox = Sandbox::new("setup-packages");
    sandbox.setup("site", &[]);

    let log = read(&sandbox.log);
    assert!(log.contains("add @sglara/cn"), "{log}");
    assert!(log.contains("lucide-react"), "{log}");
    assert!(log.contains("@base-ui/react"), "{log}");
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
    assert!(config.contains("languages = [\"fr\", \"en\"]"), "{config}");
    assert!(config.contains("package_manager = \"npm\""), "{config}");
}

#[test]
fn setup_lowercases_language_ui_and_icon_flags() {
    let sandbox = Sandbox::new("setup-lowercase");
    let output = sandbox.run(
        &sandbox.root,
        &[
            "setup",
            "site",
            "--site-url",
            "https://example.com/",
            "--yes",
            "--languages",
            "FR,EN",
            "--default-language",
            "EN",
            "--ui",
            "BASE-UI",
            "--icons",
            "LUCIDE",
        ],
    );
    assert!(output.status.success(), "{}", stderr(&output));

    let project = sandbox.root.join("site");
    let config = read(&project.join("aarshi.toml"));
    assert!(config.contains("languages = [\"en\", \"fr\"]"), "{config}");
    let log = read(&sandbox.log);
    assert!(log.contains("@base-ui/react"), "{log}");
    assert!(log.contains("lucide-react"), "{log}");
}

#[test]
fn setup_writes_only_the_chosen_languages_with_the_default_first() {
    let sandbox = Sandbox::new("setup-languages");
    let project = sandbox.setup(
        "site",
        &["--languages", "fr,ja", "--default-language", "ja"],
    );

    let config = read(&project.join("aarshi.toml"));
    assert!(config.contains("languages = [\"ja\", \"fr\"]"), "{config}");
    assert!(project.join("lib/i18n/dictionaries/ja.ts").exists());
    assert!(project.join("lib/i18n/dictionaries/fr.ts").exists());
    assert!(!project.join("lib/i18n/dictionaries/en.ts").exists());
    assert!(read(&project.join("lib/i18n/config.ts")).contains("defaultLocale: Locale = \"ja\""));
    assert!(read(&project.join("lib/i18n/config.ts")).contains("ja: \"ltr\""));
    assert!(read(&project.join("next.config.ts")).contains("destination: \"/ja\""));
}

#[test]
fn setup_rejects_a_default_language_that_was_not_chosen() {
    let sandbox = Sandbox::new("setup-default-unknown");
    let output = sandbox.run(
        &sandbox.root,
        &[
            "setup",
            "site",
            "--site-url",
            "https://example.com",
            "--languages",
            "fr,en",
            "--default-language",
            "de",
            "--ui",
            "base-ui",
            "--yes",
        ],
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("not in the selected languages"));
}

#[test]
fn setup_writes_the_base_ui_button_and_the_class_helper() {
    let sandbox = Sandbox::new("setup-ui");
    let project = sandbox.setup("site", &[]);

    let button = read(&project.join("components/ui/button.tsx"));
    assert!(button.contains("@base-ui/react/button"), "{button}");
    assert!(button.contains("from \"@sglara/cn\""), "{button}");
    assert!(read(&project.join("components/ui/button-group.tsx")).contains("ButtonGroup"));
    assert!(!project.join("lib/utils.ts").exists());
    assert!(project.join("components/custom/.gitkeep").exists());
    assert!(project.join("components/icons/.gitkeep").exists());
}

#[test]
fn setup_rejects_an_unknown_ui() {
    let sandbox = Sandbox::new("setup-ui-unknown");
    let output = sandbox.run(
        &sandbox.root,
        &[
            "setup",
            "site",
            "--site-url",
            "https://example.com",
            "--ui",
            "chakra",
            "--yes",
        ],
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("unknown UI"));
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
    assert!(
        read(&project.join("components/language-switcher.tsx")).contains("ButtonGroup"),
        "the switcher uses the UI components"
    );
    assert!(
        read(&project.join("components/language-switcher.tsx")).contains("buttonVariants"),
        "links take the button styles"
    );
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
fn add_component_creates_a_starter_in_components_custom() {
    let sandbox = Sandbox::new("add-component");
    let project = sandbox.setup("site", &[]);

    let output = sandbox.add(&project, &["-c", "tag"]);
    assert!(output.status.success(), "{}", stderr(&output));

    let component = read(&project.join("components/custom/tag.tsx"));
    assert!(component.contains("export const Tag ="), "{component}");
    assert!(component.contains("from \"@sglara/cn\""), "{component}");
}

#[test]
fn add_component_refuses_to_overwrite() {
    let sandbox = Sandbox::new("add-component-twice");
    let project = sandbox.setup("site", &[]);
    assert!(sandbox.add(&project, &["-c", "tag"]).status.success());

    let output = sandbox.add(&project, &["-c", "tag"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn add_lowercases_page_section_and_component_names_from_flags() {
    let sandbox = Sandbox::new("add-lowercase");
    let project = sandbox.setup("site", &[]);

    assert!(sandbox.add(&project, &["-p", "About"]).status.success());
    assert!(project.join("views/about/about.view.tsx").exists());

    assert!(
        sandbox
            .add(&project, &["-s", "Hero", "about"])
            .status
            .success()
    );
    assert!(project.join("views/about/sections/hero.tsx").exists());

    assert!(sandbox.add(&project, &["-c", "TagButton"]).status.success());
    assert!(project.join("components/custom/tagbutton.tsx").exists());
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
fn icons_converts_every_svg_when_asked() {
    let sandbox = Sandbox::new("icons");
    let project = sandbox.setup("site", &[]);
    fs::write(
        project.join("components/icons/logo.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <path d="M4 4h16v16H4z" />
  <!-- a comment is dropped -->
</svg>
"#,
    )
    .unwrap();

    let output = sandbox.run(&project, &["icons", "--all"]);
    assert!(output.status.success(), "{}", stderr(&output));

    let component = read(&project.join("components/icons/logo.tsx"));
    assert!(
        component.contains("const LogoIcon = ({ className, ...props }: SVGProps<SVGSVGElement>)"),
        "{component}"
    );
    assert!(component.contains("size-6"), "{component}");
    assert!(component.contains("fill-none"), "{component}");
    assert!(component.contains("stroke-current"), "{component}");
    assert!(component.contains("strokeWidth=\"2\""), "{component}");
    assert!(component.contains("strokeLinecap=\"round\""), "{component}");
    assert!(component.contains("from \"@sglara/cn\""), "{component}");
    assert!(component.contains("export { LogoIcon };"), "{component}");
    assert!(!component.contains("<!--"), "{component}");
}

#[test]
fn icons_refuses_to_overwrite_an_existing_component() {
    let sandbox = Sandbox::new("icons-twice");
    let project = sandbox.setup("site", &[]);
    fs::write(
        project.join("components/icons/logo.svg"),
        r#"<svg width="16" height="16"><path d="M0 0h16v16H0z" /></svg>"#,
    )
    .unwrap();
    assert!(sandbox.run(&project, &["icons", "--all"]).status.success());

    let output = sandbox.run(&project, &["icons", "--all"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
}

#[test]
fn icons_reports_an_empty_folder_without_a_terminal() {
    let sandbox = Sandbox::new("icons-empty");
    let project = sandbox.setup("site", &[]);

    let output = sandbox.run(&project, &["icons"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("no SVG file"));
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
    assert!(read(&project.join("lib/i18n/config.ts")).contains("\"fr\", \"en\", \"de\""));
    assert!(read(&project.join("aarshi.toml")).contains("\"de\""));
    assert!(read(&project.join("aarshi.toml")).contains("package_manager = \"npm\""));
}

#[test]
fn setup_writes_the_extended_config_keys() {
    let sandbox = Sandbox::new("setup-config-keys");
    let project = sandbox.setup("site", &["--cache-components"]);

    let config = read(&project.join("aarshi.toml"));
    assert!(config.contains("languages = [\"fr\", \"en\"]"), "{config}");
    assert!(config.contains("package_manager = \"npm\""), "{config}");
    assert!(config.contains("ui = \"base-ui\""), "{config}");
    assert!(config.contains("icons = \"lucide\""), "{config}");
    assert!(config.contains("cache_components = true"), "{config}");
    assert!(
        config.contains("site_url = \"https://example.com\""),
        "the trailing slash is trimmed: {config}"
    );

    let off = sandbox.setup("plain", &[]);
    let config = read(&off.join("aarshi.toml"));
    assert!(config.contains("cache_components = false"), "{config}");
}

#[test]
fn add_lang_preserves_the_extended_config_keys() {
    let sandbox = Sandbox::new("add-lang-preserve");
    let project = sandbox.setup("site", &["--cache-components"]);

    let output = sandbox.run(&project, &["add-lang", "de"]);
    assert!(output.status.success(), "{}", stderr(&output));

    let config = read(&project.join("aarshi.toml"));
    assert!(config.contains("languages = [\"fr\", \"en\", \"de\"]"), "{config}");
    assert!(config.contains("package_manager = \"npm\""), "{config}");
    assert!(config.contains("ui = \"base-ui\""), "{config}");
    assert!(config.contains("icons = \"lucide\""), "{config}");
    assert!(config.contains("cache_components = true"), "{config}");
    assert!(config.contains("site_url = \"https://example.com\""), "{config}");
}

#[test]
fn extended_config_parses_an_old_format_file() {
    let sandbox = Sandbox::new("config-old-format");
    let project = sandbox.setup("site", &[]);
    // A project generated before the extension only carries `languages`.
    fs::write(project.join("aarshi.toml"), "languages = [\"fr\", \"en\"]\n").unwrap();

    let output = sandbox.run(&project, &["add-lang", "de"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        read(&project.join("aarshi.toml")).contains("\"de\""),
        "the file was rewritten"
    );
}

#[test]
fn extended_config_still_rejects_unknown_keys() {
    let sandbox = Sandbox::new("config-unknown-keys");
    let project = sandbox.setup("site", &[]);
    fs::write(
        project.join("aarshi.toml"),
        "languages = [\"fr\", \"en\"]\ntemplates = \"custom\"\n",
    )
    .unwrap();

    let output = sandbox.run(&project, &["add-lang", "de"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid config"),);
}

#[test]
fn extended_config_validates_the_new_keys() {
    let sandbox = Sandbox::new("config-invalid-keys");
    let project = sandbox.setup("site", &[]);
    fs::write(
        project.join("aarshi.toml"),
        "languages = [\"fr\", \"en\"]\nui = \"chakra\"\n",
    )
    .unwrap();

    let output = sandbox.run(&project, &["add-lang", "de"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("unknown UI"),);
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

// ==== `status` (i18n dimension) ==========================================

#[test]
fn status_reports_a_default_locale_mismatch() {
    let sandbox = Sandbox::new("status-default-locale");
    let project = i18n_project("status-default-locale");
    fs::write(
        project.join("lib/i18n/config.ts"),
        read(&project.join("lib/i18n/config.ts")).replace(
            "defaultLocale: Locale = \"fr\"",
            "defaultLocale: Locale = \"en\"",
        ),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: lib/i18n/config.ts defaults to `en`, but aarshi.toml defaults to `fr`"),
        "{report}"
    );
    // The direction table is still consistent: no extra finding.
    assert!(!report.contains("direction"), "{report}");
}

#[test]
fn status_reports_a_missing_default_locale() {
    let sandbox = Sandbox::new("status-no-default-locale");
    let project = i18n_project("status-no-default-locale");
    fs::write(
        project.join("lib/i18n/config.ts"),
        read(&project.join("lib/i18n/config.ts"))
            .replace("export const defaultLocale: Locale = \"fr\";\n\n", ""),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert!(report.contains("error: lib/i18n/config.ts has no defaultLocale"), "{report}");
}

#[test]
fn status_reports_an_rtl_language_declared_ltr() {
    let sandbox = Sandbox::new("status-direction-rtl");
    fs::create_dir_all(sandbox.home.join(".config/aarshi")).unwrap();
    fs::write(
        sandbox.home.join(".config/aarshi/config.toml"),
        "languages = [\"ar\", \"en\"]\n",
    )
    .unwrap();
    let project = sandbox.setup("site", &[]);
    // The `ar` entry loses its `rtl`: `ltr` is wrong for a right-to-left language.
    fs::write(
        project.join("lib/i18n/config.ts"),
        read(&project.join("lib/i18n/config.ts")).replace("ar: \"rtl\"", "ar: \"ltr\""),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: ar has direction `ltr` instead of `rtl`"),
        "{report}"
    );
}

#[test]
fn status_reports_a_direction_entry_missing_for_a_language() {
    let sandbox = Sandbox::new("status-direction-missing");
    let project = i18n_project("status-direction-missing");
    fs::write(
        project.join("lib/i18n/config.ts"),
        read(&project.join("lib/i18n/config.ts"))
            .replace("\n  en: \"ltr\"\n};", "\n};"),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: en is missing a direction entry"),
        "{report}"
    );
}

#[test]
fn status_accepts_a_conformant_rtl_default_language() {
    let sandbox = Sandbox::new("status-rtl-clean");
    fs::create_dir_all(sandbox.home.join(".config/aarshi")).unwrap();
    fs::write(
        sandbox.home.join(".config/aarshi/config.toml"),
        "languages = [\"ar\", \"en\"]\n",
    )
    .unwrap();
    let project = sandbox.setup("site", &[]);

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(0));
    assert!(report.contains("✓ project is conformant"), "{report}");
}

#[test]
fn status_checks_the_imports_of_index_ts_per_language() {
    let sandbox = Sandbox::new("status-index-import");
    let project = i18n_project("status-index-import");
    fs::write(
        project.join("lib/i18n/index.ts"),
        read(&project.join("lib/i18n/index.ts"))
            .replace("import { messages as en } from \"./dictionaries/en\";\n", "")
            .replace("{ fr, en }", "{ fr, pod }"),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: lib/i18n/index.ts has no import for `en`"),
        "{report}"
    );
    assert!(
        report.contains("error: lib/i18n/index.ts has no map entry for `en`"),
        "{report}"
    );
    assert!(
        report.contains("error: lib/i18n/index.ts has an extra map entry `pod`"),
        "{report}"
    );
    assert!(report.contains("error: 3 errors"), "{report}");
}

#[test]
fn status_reports_an_index_import_of_an_undeclared_language() {
    let sandbox = Sandbox::new("status-index-extra-import");
    let project = i18n_project("status-index-extra-import");
    fs::write(
        project.join("lib/i18n/index.ts"),
        read(&project.join("lib/i18n/index.ts")).replace(
            "import { messages as fr } from \"./dictionaries/fr\";",
            "import { messages as zz } from \"./dictionaries/zz\";\nimport { messages as fr } from \"./dictionaries/fr\";",
        ),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert!(
        report.contains("error: lib/i18n/index.ts has an extra import `zz`"),
        "{report}"
    );
}

#[test]
fn status_reports_a_message_key_taken_from_the_wrong_language() {
    let sandbox = Sandbox::new("status-index-message-key");
    let project = i18n_project("status-index-message-key");
    fs::write(
        project.join("lib/i18n/index.ts"),
        read(&project.join("lib/i18n/index.ts")).replace("keyof typeof fr", "keyof typeof en"),
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains(
            "error: lib/i18n/index.ts derives MessageKey from `en`, but the default language is `fr`"
        ),
        "{report}"
    );
}

#[test]
fn status_reports_a_missing_index_file_without_stopping_the_report() {
    let sandbox = Sandbox::new("status-index-missing");
    let project = i18n_project("status-index-missing");
    fs::remove_file(project.join("lib/i18n/index.ts")).unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: lib/i18n/index.ts is missing or unreadable"),
        "{report}"
    );
    // No early stop: the report still renders every dimension.
    assert!(report.contains("pages ✓\nsections ✓\ncomponents ✓\ndependencies ✓"), "{report}");
    // Both dimensions see the gone file: `i18n` misses it and `config`
    // counts the missing base file.
    assert!(report.contains("error: 2 errors"), "{report}");
}

#[test]
fn status_reports_duplicate_keys_in_each_dictionary() {
    let sandbox = Sandbox::new("status-key-duplicate");
    let project = i18n_project("status-key-duplicate");
    // In a non-default dictionary…
    write_dictionary(
        &project,
        "en",
        &[("home.title", "Home"), ("home.title", "Home again")],
    );
    // …and in the default one.
    write_dictionary(
        &project,
        "fr",
        &[("home.title", "Accueil"), ("home.title", "Salut")],
    );

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: en has a duplicate entry for `home.title`"),
        "{report}"
    );
    assert!(
        report.contains("error: fr has a duplicate entry for `home.title`"),
        "{report}"
    );
    // The key exists: no missing-translation finding on top of it.
    assert!(!report.contains("is missing a translation"), "{report}");
}

#[test]
fn status_reports_a_key_extra_in_a_non_default_dictionary() {
    let sandbox = Sandbox::new("status-key-extra");
    let project = i18n_project("status-key-extra");
    write_dictionary(&project, "en", &[("home.title", "Home"), ("extra.text", "Extras")]);

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    // A `todo` alone keeps the run green.
    assert_eq!(output.status.code(), Some(0));
    assert!(
        report.contains("todo: en has an extra translation for `extra.text`"),
        "{report}"
    );
    assert!(report.contains("✓ project is conformant (1 to do)"), "{report}");

    // The JSON carries the detail: the default dictionary is the reference.
    let output = sandbox.run(&project, &["status", "--json"]);
    let document = serde_json::from_str::<serde_json::Value>(&stdout(&output)).unwrap();
    let finding = &document["dimensions"][1]["findings"][0];
    assert_eq!(finding["rule"], "i18n.key-extra", "{}", stdout(&output));
    assert_eq!(finding["detail"], "reference dictionary `fr`");
    assert_eq!(finding["severity"], "todo");
    assert_eq!(finding["location"]["key"], "extra.text");
}

#[test]
fn status_reports_a_dictionary_orphan() {
    let sandbox = Sandbox::new("status-dictionary-orphan");
    let project = i18n_project("status-dictionary-orphan");
    write_dictionary(&project, "zz", &[("home.title", "Salut tout le monde")]);

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        report.contains("todo: no language for the `zz` dictionary"),
        "{report}"
    );
}

#[test]
fn status_reports_a_missing_config_file_without_stopping_the_report() {
    let sandbox = Sandbox::new("status-config-missing");
    let project = i18n_project("status-config-missing");
    fs::remove_file(project.join("lib/i18n/config.ts")).unwrap();

    let output = sandbox.run(&project, &["status"]);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        report.contains("error: lib/i18n/config.ts is missing or unreadable"),
        "{report}"
    );
    // No early stop: the dictionaries are still walked on the same run.
    assert!(report.contains("pages ✓\nsections ✓\ncomponents ✓\ndependencies ✓"), "{report}");
}

/// A dictionary mixing a duplicate (`error`), an empty value and an extra
/// key (`todo`), plus an orphan dictionary file: the findings come out
/// sorted by file then rule, and the verdict counts each severity.
#[test]
fn status_sorts_a_mixed_i18n_dictionary_file_by_file_then_rule() {
    let sandbox = Sandbox::new("status-i18n-mixed");
    let project = i18n_project("status-i18n-mixed");
    write_dictionary(
        &project,
        "en",
        &[
            ("home.title", "Home"),
            ("home.title", ""),
            ("extra.text", "Extras"),
        ],
    );
    write_dictionary(&project, "zz", &[("home.title", "ZZ")]);

    let output = sandbox.run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "config ✓\n\
         i18n\n\
         \x20 error: en has a duplicate entry for `home.title`\n\
         \x20 todo: en has an empty translation for `home.title`\n\
         \x20 todo: en has an extra translation for `extra.text`\n\
         \x20 todo: no language for the `zz` dictionary\n\
         pages ✓\n\
         sections ✓\n\
         components ✓\n\
         dependencies ✓\n\
         \n\
         error: 1 error, 3 to do\n"
    );
}

// ==== `status` (pages dimension) ==========================================

/// The PascalCase name of a page, the same rule the generator applies.
fn pascal(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Writes `views/<page>/<page>.view.tsx`.
fn write_view(root: &Path, page: &str, content: &str) {
    let file = root
        .join("views")
        .join(page)
        .join(format!("{page}.view.tsx"));
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, content).unwrap();
}

/// Writes `app/[locale]<path>/page.tsx`; `home` covers the root page.
fn write_route(root: &Path, page: &str, content: &str) {
    let dir = if page == "home" {
        root.join("app/[locale]")
    } else {
        root.join("app/[locale]").join(page)
    };
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("page.tsx"), content).unwrap();
}

/// The minimal view and route aarshi writes for a page: the route imports
/// the view through the alias, the view imports nothing.
fn write_generated_page(root: &Path, page: &str) {
    let component = pascal(page);
    write_view(
        root,
        page,
        &format!("const {component}View = () => null;\n\nexport default {component}View;\n"),
    );
    write_route(
        root,
        page,
        &format!(
            "import {component}View from \"@/views/{page}/{page}.view\";\n\nexport default {component}View;\n"
        ),
    );
}

/// Replaces the registered entries of `app/sitemap.ts`; the hardcoded `/` is
/// part of the list, as written by the template.
fn write_sitemap(root: &Path, entries: &[&str]) {
    let listed = entries
        .iter()
        .map(|entry| format!("  \"{entry}\","))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        root.join("app/sitemap.ts"),
        format!("const pages = [\n{listed}\n  // aarshi:pages:end\n];\n"),
    )
    .unwrap();
}

/// The findings of the `pages` dimension of a `status --json` run.
fn pages_findings(output: &Output) -> Vec<serde_json::Value> {
    let document: serde_json::Value = serde_json::from_str(&stdout(output)).unwrap();
    document["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|dimension| dimension["id"].as_str().unwrap() == "pages")
        .map(|dimension| dimension["findings"].as_array().unwrap().clone())
        .unwrap()
}

/// A project shaped like the generated one, kept minimal: home view, its
/// route, the sitemap and dictionaries carrying nothing but `home.title`.
fn pages_project(name: &str) -> PathBuf {
    let root = i18n_project(name);
    fs::create_dir_all(root.join("app/[locale]")).unwrap();
    write_sitemap(&root, &["/"]);
    write_generated_page(&root, "home");
    root
}

fn complete_about_page(project: &Path, page: &str) {
    write_generated_page(project, page);
    write_sitemap(project, &["/", "/about"]);
    write_dictionary(
        project,
        "fr",
        &[("home.title", "Accueil"), ("about.title", "À propos")],
    );
    write_dictionary(
        project,
        "en",
        &[("home.title", "Home"), ("about.title", "About")],
    );
}

#[test]
fn status_reports_a_view_without_its_route() {
    let sandbox = Sandbox::new("pages-route-missing");
    let project = pages_project("pages-route-missing");
    write_view(
        &project,
        "about",
        "const AboutView = () => null;\n\nexport default AboutView;\n",
    );
    write_dictionary(
        &project,
        "fr",
        &[("home.title", "Accueil"), ("about.title", "À propos")],
    );
    write_dictionary(
        &project,
        "en",
        &[("home.title", "Home"), ("about.title", "About")],
    );
    write_sitemap(&project, &["/", "/about"]);

    let human = sandbox.run(&project, &["status"]);
    assert_eq!(human.status.code(), Some(1));
    let report = stdout(&human);
    assert!(
        report.contains("pages\n  error: no route for the page `about`\n"),
        "{report}"
    );
    // An existing view never books an orphan: only the missing route fires.
    assert!(!report.contains("no view for"), "{report}");

    let json = sandbox.run(&project, &["status", "--json"]);
    let findings = pages_findings(&json);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding["rule"].as_str().unwrap(), "pages.route-missing");
    assert_eq!(finding["severity"].as_str().unwrap(), "error");
    assert_eq!(
        finding["detail"].as_str().unwrap(),
        "expected app/[locale]/about/page.tsx"
    );
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "app/[locale]/about/page.tsx"
    );
}

#[test]
fn status_reports_a_hand_added_route_without_a_view() {
    let sandbox = Sandbox::new("pages-route-orphan");
    let project = pages_project("pages-route-orphan");
    write_route(
        &project,
        "contact",
        "import ContactView from \"@/views/contact/contact.view\";\n\nexport default ContactView;\n",
    );

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let findings = pages_findings(&output);
    // One finding only: the view import a hand-added route waits for is
    // covered by `pages.route-orphan`, never repeated as a broken import.
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding["rule"].as_str().unwrap(), "pages.route-orphan");
    assert_eq!(finding["severity"].as_str().unwrap(), "error");
    assert_eq!(
        finding["summary"].as_str().unwrap(),
        "no view for the route `/contact`"
    );
    assert_eq!(
        finding["detail"].as_str().unwrap(),
        "expected views/contact/contact.view.tsx"
    );
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "app/[locale]/contact/page.tsx"
    );
}

#[test]
fn status_reports_a_title_key_missing_from_a_dictionary() {
    let sandbox = Sandbox::new("pages-title-key");
    let project = pages_project("pages-title-key");
    // The full page is in place; only its `<page>.title` key stays unknown.
    write_generated_page(&project, "about");
    write_sitemap(&project, &["/", "/about"]);

    let human = sandbox.run(&project, &["status"]);
    assert_eq!(human.status.code(), Some(1));
    let report = stdout(&human);
    assert!(
        report.contains("error: `about.title` is missing from the dictionary of `fr`\n"),
        "{report}"
    );
    assert!(
        report.contains("error: `about.title` is missing from the dictionary of `en`\n"),
        "{report}"
    );

    let json = sandbox.run(&project, &["status", "--json"]);
    let findings = pages_findings(&json);
    assert_eq!(findings.len(), 2);
    // Sorted by dictionary file: `en` before `fr`.
    for (finding, locale, title) in [
        (&findings[0], "en", "Home"),
        (&findings[1], "fr", "À propos"),
    ] {
        assert_eq!(finding["rule"].as_str().unwrap(), "pages.title-key-missing");
        assert_eq!(finding["severity"].as_str().unwrap(), "error");
        assert!(
            finding["summary"]
                .as_str()
                .unwrap()
                .starts_with("`about.title` is missing from the dictionary of `"),
            "{finding:?}"
        );
        assert_eq!(
            finding["location"]["file"].as_str().unwrap(),
            format!("lib/i18n/dictionaries/{locale}.ts")
        );
        assert_eq!(finding["location"]["locale"].as_str().unwrap(), locale);
        assert_eq!(finding["location"]["key"].as_str().unwrap(), "about.title");
        let _ = title;
    }
}

#[test]
fn status_reports_a_missing_sitemap_entry() {
    let sandbox = Sandbox::new("pages-sitemap-missing");
    let project = pages_project("pages-sitemap-missing");
    complete_about_page(&project, "about");
    // `/about` was created but never registered in the sitemap.
    write_sitemap(&project, &["/"]);

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let findings = pages_findings(&output);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(
        finding["rule"].as_str().unwrap(),
        "pages.sitemap-entry-missing"
    );
    assert_eq!(finding["severity"].as_str().unwrap(), "error");
    assert_eq!(
        finding["summary"].as_str().unwrap(),
        "no sitemap entry for `/about`"
    );
    assert_eq!(finding["detail"].as_str().unwrap(), "expected \"/about\",");
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "app/sitemap.ts"
    );
    assert_eq!(finding["location"]["key"].as_str().unwrap(), "/about");
}

#[test]
fn status_reports_an_orphan_sitemap_entry() {
    let sandbox = Sandbox::new("pages-sitemap-orphan");
    let project = pages_project("pages-sitemap-orphan");
    write_sitemap(&project, &["/", "/contact"]);

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let findings = pages_findings(&output);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(
        finding["rule"].as_str().unwrap(),
        "pages.sitemap-entry-orphan"
    );
    assert_eq!(finding["severity"].as_str().unwrap(), "error");
    assert_eq!(
        finding["summary"].as_str().unwrap(),
        "sitemap entry `/contact` has no page"
    );
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "app/sitemap.ts"
    );
    assert_eq!(finding["location"]["key"].as_str().unwrap(), "/contact");
}

#[test]
fn status_reports_a_missing_view_import() {
    let sandbox = Sandbox::new("pages-view-import");
    let project = pages_project("pages-view-import");
    complete_about_page(&project, "about");
    // The route stopped importing its view.
    write_route(&project, "about", "export default null;\n");

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let findings = pages_findings(&output);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(
        finding["rule"].as_str().unwrap(),
        "pages.view-import-missing"
    );
    assert_eq!(finding["severity"].as_str().unwrap(), "error");
    assert_eq!(
        finding["summary"].as_str().unwrap(),
        "`/about` does not import its view"
    );
    assert_eq!(
        finding["detail"].as_str().unwrap(),
        "expected @/views/about/about.view"
    );
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "app/[locale]/about/page.tsx"
    );
}

#[test]
fn status_reports_a_broken_import_in_a_view() {
    let sandbox = Sandbox::new("pages-import-broken");
    let project = pages_project("pages-import-broken");
    // One unresolvable component import and one dangling section import next
    // to a well-placed canonical route import.
    write_view(
        &project,
        "about",
        "import { Tag } from \"@/components/custom/tag\";\n\nimport { Hero } from \"./sections/hero\";\n\nconst AboutView = () => null;\n\nexport default AboutView;\n",
    );
    write_route(
        &project,
        "about",
        "import AboutView from \"@/views/about/about.view\";\n\nexport default AboutView;\n",
    );
    write_dictionary(
        &project,
        "fr",
        &[("home.title", "Accueil"), ("about.title", "À propos")],
    );
    write_dictionary(
        &project,
        "en",
        &[("home.title", "Home"), ("about.title", "About")],
    );
    write_sitemap(&project, &["/", "/about"]);

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let findings = pages_findings(&output);
    // The dangling `./sections/hero` import belongs to the `sections`
    // dimension (ticket 0027) and stays silent here.
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding["rule"].as_str().unwrap(), "pages.import-broken");
    assert_eq!(finding["severity"].as_str().unwrap(), "error");
    assert_eq!(
        finding["summary"].as_str().unwrap(),
        "broken import `@/components/custom/tag` in `views/about/about.view.tsx`"
    );
    assert_eq!(
        finding["detail"].as_str().unwrap(),
        "no file matches `@/components/custom/tag`"
    );
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "views/about/about.view.tsx"
    );
}

#[test]
fn status_scans_routes_recursively_under_the_locale_folder() {
    let sandbox = Sandbox::new("pages-recursive");
    let project = pages_project("pages-recursive");
    // A hand-added nested route, several segments deep.
    write_route(&project, "docs/guide", "export default null;\n");

    let human = sandbox.run(&project, &["status"]);
    assert_eq!(human.status.code(), Some(1));
    let report = stdout(&human);
    assert!(
        report.contains("error: no view for the route `/docs/guide`\n"),
        "{report}"
    );
    assert!(report.contains("error: 1 error"), "{report}");

    let json = sandbox.run(&project, &["status", "--json"]);
    let findings = pages_findings(&json);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(
        finding["location"]["file"].as_str().unwrap(),
        "app/[locale]/docs/guide/page.tsx"
    );
    // A nested route has no conventional view to offer as a fix.
    assert!(finding.get("detail").is_none(), "{finding:?}");
}

#[test]
fn status_reports_no_pages_finding_without_views_or_routes() {
    // Transverse rule: a project without any `views/` folder (any route
    // either) yields findings without ever crashing.
    let sandbox = Sandbox::new("pages-empty");
    let project = i18n_project("pages-empty");

    let human = sandbox.run(&project, &["status"]);
    assert_eq!(human.status.code(), Some(0));
    let report = stdout(&human);
    assert!(report.contains("pages ✓"), "{report}");

    let json = sandbox.run(&project, &["status", "--json"]);
    let document: serde_json::Value = serde_json::from_str(&stdout(&json)).unwrap();
    let pages = document["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|dimension| dimension["id"].as_str().unwrap() == "pages")
        .unwrap();
    assert_eq!(pages["errors"], 0, "{}", stdout(&json));
    assert_eq!(pages["todos"], 0);
    assert!(pages["findings"].as_array().unwrap().is_empty());
}

// ==== `status` (config dimension) =========================================

/// Writes one file of a fixture; parent folders already exist there but are
/// created when needed too.
fn write_project_file(root: &Path, file: &str, content: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// Runs `status` restricted to the `config` dimension.
fn status_config(sandbox: &Sandbox, project: &Path) -> Output {
    sandbox.run(project, &["status", "--dimension", "config"])
}

#[test]
fn config_reports_each_missing_base_file() {
    let sandbox = Sandbox::new("config-file-missing");
    let project = i18n_project("config-file-missing");
    fs::remove_file(project.join("app/[locale]/layout.tsx")).unwrap();
    fs::remove_file(project.join("tsconfig.json")).unwrap();
    fs::remove_dir_all(project.join("components/custom")).unwrap();

    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    let report = stdout(&output);
    assert!(
        report.contains("error: missing `app/[locale]/layout.tsx`"),
        "{report}"
    );
    assert!(report.contains("error: missing `tsconfig.json`"), "{report}");
    // Dirs are objects too: a missing folder is a finding, not a stop.
    assert!(report.contains("error: missing `components/custom/`"), "{report}");
}

#[test]
fn config_reports_a_project_without_base_files_and_still_exits_one() {
    let sandbox = Sandbox::new("config-stripped");
    let project = i18n_project("config-stripped");
    for file in [
        "app/[locale]/layout.tsx",
        "app/sitemap.ts",
        "app/robots.ts",
        "lib/site.ts",
        "lib/i18n/config.ts",
        "lib/i18n/index.ts",
        "components/language-switcher.tsx",
        "tsconfig.json",
        "eslint.config.mjs",
        "postcss.config.mjs",
        "next.config.ts",
    ] {
        fs::remove_file(project.join(file)).unwrap();
    }
    fs::remove_dir_all(project.join("components/custom")).unwrap();
    fs::remove_dir_all(project.join("components/icons")).unwrap();

    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    // One finding per missing entry of the inventory, then a clean run.
    assert_eq!(
        stdout(&output).matches("error: missing `").count(),
        13,
        "{}",
        stdout(&output)
    );
    // The checks of the gone files are absorbed, never duplicated.
    let report = stdout(&output);
    assert!(!report.contains("marker"), "{report}");
    assert!(!report.contains("redirect"), "{report}");
    assert!(!report.contains("cacheComponents"), "{report}");
    assert!(!report.contains("SITE_URL"), "{report}");
}

#[test]
fn config_reports_a_missing_pages_marker_in_the_sitemap() {
    let sandbox = Sandbox::new("config-marker");
    let project = i18n_project("config-marker");
    write_project_file(
        &project,
        "app/sitemap.ts",
        "const pages = [\"/\", \"/about\"];\n",
    );

    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output)
            .contains("error: app/sitemap.ts has no `// aarshi:pages:end` marker"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn config_checks_the_slash_redirect_against_the_default_language() {
    let sandbox = Sandbox::new("config-redirect");
    let project = i18n_project("config-redirect");

    // A redirect that points elsewhere than `languages[0]` (`fr`).
    write_project_file(
        &project,
        "next.config.ts",
        "const nextConfig = {\n  async redirects() {\n    return [{ source: \"/\", destination: \"/de\", permanent: false }];\n  },\n  cacheComponents: false,\n};\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output).contains(
            "error: next.config.ts redirects \"/\" to \"/de\", expected \"/fr\""
        ),
        "{}",
        stdout(&output)
    );

    // A project without any redirect for the root either fails.
    write_project_file(
        &project,
        "next.config.ts",
        "const nextConfig = {\n  cacheComponents: false,\n};\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output).contains("error: next.config.ts has no redirect for \"/\""),
        "{}",
        stdout(&output)
    );
}

/// An old `aarshi.toml` parses thanks to the extended format (0022): the
/// user defaults fill the gaps, and the report stays one `todo` per missing
/// key, with the exit code at `0`.
#[test]
fn config_reports_each_missing_key_of_an_old_format_project_file() {
    let sandbox = Sandbox::new("config-old-format-status");
    let project = i18n_project("config-old-format-status");
    fs::write(
        project.join("aarshi.toml"),
        "languages = [\"fr\", \"en\"]\npackage_manager = \"npm\"\n",
    )
    .unwrap();
    // `site_url` falls back to the user default, so `lib/site.ts` follows it.
    fs::write(
        project.join("lib/site.ts"),
        "export const SITE_URL = \"http://localhost:3000\";\n",
    )
    .unwrap();

    let output = sandbox.run(&project, &["status"]);
    assert_eq!(output.status.code(), Some(0));
    let report = stdout(&output);
    assert!(report.contains("✓ project is conformant (4 to do)"), "{report}");
    assert!(
        report.contains("todo: aarshi.toml is missing the `ui` key"),
        "{report}"
    );

    let output = sandbox.run(&project, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let document: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let config = &document["dimensions"][0];
    assert_eq!(config["id"], "config");
    let findings = config["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{}", stdout(&output));
    assert!(
        findings
            .iter()
            .all(|finding| finding["rule"] == "config.project-file-incomplete"
                && finding["severity"] == "todo"),
        "{}",
        stdout(&output)
    );
    let keys: Vec<&str> = findings
        .iter()
        .map(|finding| finding["location"]["key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["ui", "icons", "cache_components", "site_url"]);
    assert_eq!(document["summary"]["todos"], 4);
}

#[test]
fn config_checks_cache_components_against_next_config() {
    let sandbox = Sandbox::new("config-cache-components");
    let project = i18n_project("config-cache-components");

    // `aarshi.toml` asks for cache components, `next.config.ts` skips them.
    let toml = read(&project.join("aarshi.toml"))
        .replace("cache_components = false", "cache_components = true");
    fs::write(project.join("aarshi.toml"), toml).unwrap();
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output)
            .contains("error: next.config.ts does not enable `cacheComponents`"),
        "{}",
        stdout(&output)
    );

    // The reverse drift: enabled for a project that never asked for it.
    fs::write(
        project.join("aarshi.toml"),
        "languages = [\"fr\", \"en\"]\npackage_manager = \"npm\"\nui = \"base-ui\"\nicons = \"lucide\"\ncache_components = false\nsite_url = \"https://example.com\"\n",
    )
    .unwrap();
    write_project_file(
        &project,
        "next.config.ts",
        "const nextConfig = {\n  cacheComponents: true,\n};\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output)
            .contains("error: next.config.ts enables `cacheComponents`"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn config_checks_the_site_url_against_lib_site() {
    let sandbox = Sandbox::new("config-site-url");
    let project = i18n_project("config-site-url");
    write_project_file(
        &project,
        "lib/site.ts",
        "export const SITE_URL = \"https://other.example.com\";\n",
    );

    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output).contains(
            "error: lib/site.ts sets SITE_URL to \"https://other.example.com\", expected \"https://example.com\""
        ),
        "{}",
        stdout(&output)
    );
}

#[test]
fn config_checks_the_ui_option() {
    let sandbox = Sandbox::new("config-ui");
    let project = i18n_project("config-ui");

    // Nominal `base-ui`: the dependency is declared and the components exist.
    let output = status_config(&sandbox, &project);
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("config ✓"), "{}", stdout(&output));

    // A `base-ui` project without the `@base-ui/react` dependency fails.
    write_project_file(
        &project,
        "package.json",
        "{\n  \"dependencies\": {\"@sglara/cn\": \"^0.4.0\", \"lucide-react\": \"^0.500.0\"}\n}\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output)
            .contains("error: package.json does not declare `@base-ui/react`"),
        "{}",
        stdout(&output)
    );

    // A `base-ui` project without its components fails, one per component.
    write_project_file(
        &project,
        "package.json",
        "{\n  \"dependencies\": {\"@base-ui/react\": \"^1.0.0\"}\n}\n",
    );
    fs::remove_file(project.join("components/ui/button.tsx")).unwrap();
    fs::remove_file(project.join("components/ui/button-group.tsx")).unwrap();
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    let report = stdout(&output);
    assert!(
        report.contains("error: missing `components/ui/button.tsx`"),
        "{report}"
    );
    assert!(
        report.contains("error: missing `components/ui/button-group.tsx`"),
        "{report}"
    );
}

#[test]
fn config_checks_a_shadcn_ui_project() {
    let sandbox = Sandbox::new("config-ui-shadcn");
    let project = i18n_project("config-ui-shadcn");
    let toml = read(&project.join("aarshi.toml")).replace("ui = \"base-ui\"", "ui = \"shadcn\"");
    fs::write(project.join("aarshi.toml"), toml).unwrap();

    // Nominal `shadcn`: the components are enough, no dependency required.
    let output = status_config(&sandbox, &project);
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("config ✓"), "{}", stdout(&output));

    // A `shadcn` project without its components fails, one per component.
    fs::remove_file(project.join("components/ui/button.tsx")).unwrap();
    fs::remove_file(project.join("components/ui/button-group.tsx")).unwrap();
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    let report = stdout(&output);
    assert!(
        report.contains("error: missing `components/ui/button.tsx`"),
        "{report}"
    );
    assert!(
        report.contains("error: missing `components/ui/button-group.tsx`"),
        "{report}"
    );
}

#[test]
fn config_checks_the_icons_option() {
    let sandbox = Sandbox::new("config-icons");
    let project = i18n_project("config-icons");

    // Nominal `lucide`: the library is declared in `package.json`.
    let output = status_config(&sandbox, &project);
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("config ✓"), "{}", stdout(&output));

    // A `lucide` project without `lucide-react` fails.
    write_project_file(
        &project,
        "package.json",
        "{\n  \"dependencies\": {\"@base-ui/react\": \"^1.0.0\", \"@sglara/cn\": \"^0.4.0\"}\n}\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output)
            .contains("error: package.json does not declare `lucide-react`"),
        "{}",
        stdout(&output)
    );

    // `none` asks for nothing, whatever `package.json` declares.
    let toml = read(&project.join("aarshi.toml")).replace("icons = \"lucide\"", "icons = \"none\"");
    fs::write(project.join("aarshi.toml"), toml).unwrap();
    let output = status_config(&sandbox, &project);
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(stdout(&output).contains("config ✓"), "{}", stdout(&output));
}

#[test]
fn config_reports_a_broken_import_in_a_base_file() {
    let sandbox = Sandbox::new("config-import-broken");
    let project = i18n_project("config-import-broken");

    // An alias import no file in the project provides.
    write_project_file(
        &project,
        "app/[locale]/layout.tsx",
        "import \"@/app/globals.css\";\nimport { helper } from \"@/lib/nope\";\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    let report = stdout(&output);
    assert!(
        report
            .contains("error: app/[locale]/layout.tsx imports `@/lib/nope`, but the file does not exist"),
        "{report}"
    );
    // Asset imports (`.css`, …) stay out of the textual reach of v1.
    assert!(!report.contains("globals.css"), "{report}");

    // A relative import reaches the file next to the importer.
    write_project_file(
        &project,
        "app/[locale]/layout.tsx",
        "import { helper } from \"./nowhere/nothing\";\n",
    );
    let output = status_config(&sandbox, &project);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output).contains(
            "error: app/[locale]/layout.tsx imports `./nowhere/nothing`, but the file does not exist"
        ),
        "{}",
        stdout(&output)
    );

    // `components/` imports are resolved by the `components` masters.
    write_project_file(
        &project,
        "app/[locale]/layout.tsx",
        "import { Thing } from \"@/components/nope/thing\";\n",
    );
    let output = status_config(&sandbox, &project);
    assert!(output.status.success(), "{}", stdout(&output));
    assert!(!stdout(&output).contains("nope/thing"), "{}", stdout(&output));
}

#[test]
fn config_leaves_missing_base_files_to_the_file_missing_rule() {
    let sandbox = Sandbox::new("config-import-file-missing");
    let project = i18n_project("config-import-file-missing");
    write_project_file(
        &project,
        "app/[locale]/layout.tsx",
        "import { resolve } from \"@/lib/i18n\";\n",
    );
    fs::remove_file(project.join("lib/i18n/index.ts")).unwrap();

    // The import of a missing base file is already reported once, by
    // `config.file-missing`: no duplicate `config.import-broken` finding.
    let output = status_config(&sandbox, &project);
    let report = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(report.contains("error: missing `lib/i18n/index.ts`"), "{report}");
    assert!(!report.contains("imports `@/lib/i18n`"), "{report}");
}
