//! The imports of a project file, read textually, and what each one points
//! at. The module reads and resolves; the dimensions decide which findings an
//! import raises.
// Unwired until the dimensions migrate to it; the migration removes this.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Asset extensions an import may point into; anything else without the
/// `.ts`/`.tsx` ending is resolved as a TypeScript module.
const ASSET_EXTENSIONS: &[&str] = &[
    ".css", ".scss", ".sass", ".svg", ".json", ".png", ".jpg", ".jpeg", ".webp", ".ico", ".md",
];

/// One import of a file, with where its module points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    /// The text between the quotes, without any `with` clause.
    pub module: String,
    /// The default name and the `{ A, B }` members, `type ` markers dropped.
    pub names: Vec<String>,
    pub target: Target,
}

/// Where an import points. Paths are relative to the project root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A TypeScript module: `found` is the first candidate that exists.
    Local {
        candidates: Vec<PathBuf>,
        found: Option<PathBuf>,
    },
    /// An asset imported for its content; `found` is set when it exists.
    Asset { found: Option<PathBuf> },
    /// An npm package, the business of the `dependencies` dimension.
    Bare,
    /// A relative path that climbs above the project root.
    Escapes,
}

/// The imports of `content`, the text of the project file `file`, in order.
/// Textual only: multi-line imports are not understood.
pub fn imports_of(root: &Path, file: &str, content: &str) -> Vec<ResolvedImport> {
    content
        .lines()
        .filter_map(parse_import)
        .map(|(module, names)| {
            let target = target_of(root, file, &module);
            ResolvedImport {
                module,
                names,
                target,
            }
        })
        .collect()
}

/// The module and names of one `import` or `export ... from` line. Both quote
/// styles and a `with` clause are understood; `import "x"` has no names.
fn parse_import(line: &str) -> Option<(String, Vec<String>)> {
    let line = line.trim();
    let rest = line
        .strip_prefix("import ")
        .or_else(|| line.strip_prefix("export "))?
        .trim_start();
    let (specifiers, source) = match rest.split_once(" from ") {
        Some((specifiers, source)) => (Some(specifiers.trim()), source),
        None => (None, rest),
    };
    // Import assertions (`import "x" with { ... }`) are not part of the path.
    let source = source.split_once(" with ").map_or(source, |(path, _)| path);
    let source = source.trim().trim_end_matches(';').trim();
    let quote = source.chars().next().filter(|c| matches!(c, '"' | '\''))?;
    let module = source.strip_prefix(quote)?.strip_suffix(quote)?;
    if module.is_empty() {
        return None;
    }
    let names = specifiers.map(names_of).unwrap_or_default();
    Some((module.to_string(), names))
}

fn names_of(specifiers: &str) -> Vec<String> {
    let mut names = Vec::new();
    match specifiers.find('{') {
        Some(brace) => {
            // An optional default import stands before the braces.
            let default = specifiers[..brace].trim().trim_matches(',').trim();
            if let Some(name) = default.split_whitespace().next()
                && name != "type"
                && name != "*"
            {
                names.push(name.to_string());
            }
            if let Some(close) = specifiers[brace..].find('}') {
                for member in specifiers[brace + 1..brace + close].split(',') {
                    let member = member.trim();
                    let member = member.strip_prefix("type ").unwrap_or(member).trim();
                    let member = member.split_whitespace().next().unwrap_or(member);
                    if !member.is_empty() {
                        names.push(member.to_string());
                    }
                }
            }
        }
        None => {
            if let Some(name) = specifiers.split_whitespace().next()
                && name != "type"
                && name != "*"
            {
                names.push(name.to_string());
            }
        }
    }
    names
}

fn target_of(root: &Path, file: &str, module: &str) -> Target {
    if !is_local(module) {
        return Target::Bare;
    }
    let Some(base) = project_path(file, module) else {
        return Target::Escapes;
    };
    if ASSET_EXTENSIONS.iter().any(|ext| module.ends_with(ext)) {
        let path = PathBuf::from(&base);
        let found = root.join(&path).is_file().then_some(path);
        return Target::Asset { found };
    }
    let candidates: Vec<PathBuf> = if module.ends_with(".ts") || module.ends_with(".tsx") {
        vec![PathBuf::from(&base)]
    } else {
        ["", ".ts", ".tsx", "/index.ts", "/index.tsx"]
            .into_iter()
            .map(|suffix| PathBuf::from(format!("{base}{suffix}")))
            .collect()
    };
    let found = candidates
        .iter()
        .find(|candidate| root.join(candidate).is_file())
        .cloned();
    Target::Local { candidates, found }
}

/// `@/x` is the project root, `./x` and `../x` sit next to the importer. Any
/// other module names an npm package.
fn is_local(module: &str) -> bool {
    module.starts_with("@/")
        || module == "."
        || module.starts_with("./")
        || module.starts_with("../")
}

/// The project-relative path an import names, from the directory of `file`;
/// `@/` starts at the project root. `None` when the path climbs above it.
fn project_path(file: &str, module: &str) -> Option<String> {
    let (mut cleaned, rest): (Vec<&str>, &str) = match module.strip_prefix("@/") {
        Some(rest) => (Vec::new(), rest),
        None => {
            let dir = file.rsplit_once('/').map_or("", |(dir, _)| dir);
            (
                dir.split('/').filter(|part| !part.is_empty()).collect(),
                module,
            )
        }
    };
    for segment in rest.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                cleaned.pop()?;
            }
            other => cleaned.push(other),
        }
    }
    Some(cleaned.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_both_quote_styles_and_a_with_clause() {
        assert_eq!(
            parse_import(r#"import { A } from "@/x";"#),
            Some(("@/x".to_string(), vec!["A".to_string()]))
        );
        assert_eq!(
            parse_import("import B from './y'"),
            Some(("./y".to_string(), vec!["B".to_string()]))
        );
        assert_eq!(
            parse_import(r#"import data from "./d.json" with { type: "json" };"#),
            Some(("./d.json".to_string(), vec!["data".to_string()]))
        );
    }

    #[test]
    fn parses_bare_side_effect_and_re_exports() {
        assert_eq!(
            parse_import(r#"import "./globals.css";"#),
            Some(("./globals.css".to_string(), vec![]))
        );
        assert_eq!(
            parse_import(r#"export { X } from "./x";"#),
            Some(("./x".to_string(), vec!["X".to_string()]))
        );
    }

    #[test]
    fn ignores_lines_that_are_not_imports() {
        assert_eq!(parse_import(r#"export const x = "y";"#), None);
        assert_eq!(parse_import("const z = 1;"), None);
        assert_eq!(parse_import(r#"import "";"#), None);
    }

    #[test]
    fn names_drop_type_markers_and_namespaces() {
        assert_eq!(
            names_of("type { A, type B as C }"),
            vec!["A".to_string(), "B".to_string()]
        );
        assert_eq!(names_of("* as ns"), Vec::<String>::new());
        assert_eq!(
            names_of("Def, { M }"),
            vec!["Def".to_string(), "M".to_string()]
        );
    }

    #[test]
    fn project_paths_follow_the_importer_and_the_alias() {
        assert_eq!(
            project_path("views/home/home.view.tsx", "./sections/hero").as_deref(),
            Some("views/home/sections/hero")
        );
        assert_eq!(
            project_path("views/home/home.view.tsx", "../shared/x").as_deref(),
            Some("views/shared/x")
        );
        assert_eq!(
            project_path("views/home/home.view.tsx", "@/lib/y").as_deref(),
            Some("lib/y")
        );
    }

    #[test]
    fn a_path_above_the_root_escapes() {
        assert_eq!(project_path("index.ts", "../x"), None);
        assert_eq!(project_path("a/b.ts", "../../../x"), None);
        assert_eq!(project_path("a.ts", "@/../x"), None);
    }

    #[test]
    fn packages_are_bare_and_assets_are_classified() {
        let root = scratch("classify");
        let imports = imports_of(
            &root,
            "a.ts",
            "import x from \"next\";\nimport \"./s.css\";",
        );
        assert_eq!(imports[0].target, Target::Bare);
        assert_eq!(imports[1].target, Target::Asset { found: None });
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_local_import_finds_its_first_existing_candidate() {
        let root = scratch("local");
        fs::create_dir_all(root.join("lib/dict")).unwrap();
        fs::write(root.join("lib/dict/index.ts"), "").unwrap();
        let imports = imports_of(&root, "app.ts", "import d from \"@/lib/dict\";");
        assert_eq!(
            imports[0].target,
            Target::Local {
                candidates: vec![
                    PathBuf::from("lib/dict"),
                    PathBuf::from("lib/dict.ts"),
                    PathBuf::from("lib/dict.tsx"),
                    PathBuf::from("lib/dict/index.ts"),
                    PathBuf::from("lib/dict/index.tsx"),
                ],
                found: Some(PathBuf::from("lib/dict/index.ts")),
            }
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn an_escaping_import_is_reported_as_escaping() {
        let root = scratch("escape");
        let imports = imports_of(&root, "a.ts", "import x from \"../../x\";");
        assert_eq!(imports[0].target, Target::Escapes);
        fs::remove_dir_all(&root).ok();
    }

    fn scratch(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("aarshi-imports-{}-{name}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
