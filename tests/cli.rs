use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Fake package manager: records its arguments and environment, creates a bare Next.js
/// project on `create`, and does nothing on `add` (installations).
const FAKE_NPM: &str = r#"#!/bin/sh
echo "$@" >> "$AARSHI_TEST_LOG"
echo "$NEXT_TELEMETRY_DISABLED" >> "$AARSHI_TEST_LOG"
if [ "$1" = "create" ]; then
  mkdir -p "$3/app"
  touch "$3/app/layout.tsx" "$3/app/page.tsx"
  printf 'import type { NextConfig } from "next";\n\nconst nextConfig: NextConfig = {\n  cacheComponents: true,\n};\n\nexport default nextConfig;\n' > "$3/next.config.ts"
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

/// A minimal generated project covering the tracer `i18n` checks: two
/// languages with consistent config and complete dictionaries. The project
/// lives in a `project/` subfolder to stay clear of the sandbox files.
fn i18n_project(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name).join("project");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("lib/i18n/dictionaries")).unwrap();
    fs::write(
        root.join("aarshi.toml"),
        "languages = [\"fr\", \"en\"]\npackage_manager = \"npm\"\n",
    )
    .unwrap();
    fs::write(
        root.join("lib/i18n/config.ts"),
        "export const languages = [\"fr\", \"en\"] as const;\n",
    )
    .unwrap();
    write_dictionary(&root, "fr", &[("home.title", "Accueil")]);
    write_dictionary(&root, "en", &[("home.title", "Home")]);
    root
}

/// The clean project plus one error (dictionary gone) and one to do
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
        "config ✓\n\
         i18n\n\
         \x20 error: no dictionary for `en`\n\
         \x20 todo: fr has an empty translation for `home.intro`\n\
         pages ✓\n\
         sections ✓\n\
         components ✓\n\
         dependencies ✓\n\
         \n\
         error: 1 error, 1 to do\n"
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
    "errors": 1,
    "todos": 1,
    "conformant": false
  },
  "dimensions": [
    {
      "id": "config",
      "errors": 0,
      "todos": 0,
      "findings": []
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
    assert_eq!(document["summary"]["errors"], 1);
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
         error: 1 error, 1 to do\n\
         info: 2 findings hidden by --dimension\n"
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
