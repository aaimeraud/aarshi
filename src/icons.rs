use std::{env, fs, path::Path};

use anyhow::{Context, Result, bail};
use inquire::MultiSelect;
use roxmltree::{Document, Node};

use crate::{config, output, project};

const ICONS_DIR: &str = "components/icons";

/// Converts the `.svg` files of `components/icons` into React components.
/// Without `--all`, the files are picked from a checklist.
pub fn run(all: bool) -> Result<()> {
    let root = env::current_dir()?;
    if !config::is_aarshi_project(&root) {
        bail!("not an aarshi project: no aarshi.toml in this folder");
    }
    let dir = root.join(ICONS_DIR);
    let svgs = svg_files(&dir)?;
    if svgs.is_empty() {
        bail!("no SVG file in {ICONS_DIR}; add one and run `aarshi icons` again");
    }

    let selected = if all {
        svgs
    } else {
        MultiSelect::new("Icons to convert", svgs).prompt()?
    };
    if selected.is_empty() {
        println!("{}", output::info("Nothing selected."));
        return Ok(());
    }

    for name in &selected {
        let source = project::read_file(&dir.join(format!("{name}.svg")))?;
        let target = dir.join(format!("{name}.tsx"));
        project::create_file(&target, &convert(&source, name)?)?;
        println!(
            "{}",
            output::success(&format!(
                "wrote {}",
                target.strip_prefix(&root).unwrap_or(&target).display()
            ))
        );
    }
    Ok(())
}

/// Names of the `.svg` files, without extension.
fn svg_files(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("cannot read {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "svg")
            && let Some(stem) = path.file_stem()
        {
            names.push(stem.to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// Turns one SVG file into a component: React attributes and, when it is simple,
/// Tailwind classes for the size and the current color.
fn convert(source: &str, name: &str) -> Result<String> {
    let document = Document::parse(source).with_context(|| format!("cannot parse {name}.svg"))?;
    let svg = document.root_element();
    if svg.tag_name().name() != "svg" {
        bail!("{name}.svg has no <svg> root element");
    }
    let component = component_name(name);
    let body = render(svg, 4, true)?;
    Ok(format!(
        "import type {{ SVGProps }} from \"react\";\n\nimport {{ cn }} from \"@sglara/cn\";\n\nconst {component} = ({{ className, ...props }}: SVGProps<SVGSVGElement>) => {{\n  return (\n{body}\n  );\n}};\n\nexport {{ {component} }};\n"
    ))
}

/// `github` becomes `GithubIcon`, `logo-mark` becomes `LogoMarkIcon`.
fn component_name(name: &str) -> String {
    let pascal = project::to_pascal_case(name);
    if pascal.ends_with("Icon") {
        pascal
    } else {
        format!("{pascal}Icon")
    }
}

fn render(node: Node, indent: usize, root: bool) -> Result<String> {
    let pad = " ".repeat(indent);
    let tag = node.tag_name().name();
    let mut lines: Vec<String> = Vec::new();
    let mut classes: Vec<String> = Vec::new();

    // Only the root can hold the `className` that `cn` builds.
    let size = match (node.attribute("width"), node.attribute("height")) {
        (Some(width), Some(height)) if root => size_class(width, height),
        _ => None,
    };

    for attribute in node.attributes() {
        let name = attribute.name();
        let value = attribute.value();
        if root {
            match name {
                "width" | "height" if size.is_some() => continue,
                "fill" if value == "none" => {
                    classes.push("fill-none".to_string());
                    continue;
                }
                "fill" if value == "currentColor" => {
                    classes.push("fill-current".to_string());
                    continue;
                }
                "stroke" if value == "currentColor" => {
                    classes.push("stroke-current".to_string());
                    continue;
                }
                "class" => {
                    classes.push(value.trim().to_string());
                    continue;
                }
                _ => {}
            }
        }
        if name == "style"
            && let Some(expression) = style_expression(value)
        {
            lines.push(format!("style={expression}"));
            continue;
        }
        lines.push(render_attribute(name, value));
    }

    if let Some(size) = size {
        classes.insert(0, size);
    }
    if root {
        let classes = classes.join(" ");
        let expression = if classes.is_empty() {
            "{cn(className)}".to_string()
        } else {
            format!("{{cn({}, className)}}", json_string(&classes))
        };
        lines.push(format!("className={expression}"));
        lines.push("{...props}".to_string());
    }

    let children: Vec<Node> = node
        .children()
        .filter(|child| {
            child.is_element() || (child.is_text() && !child.text().unwrap_or("").trim().is_empty())
        })
        .collect();

    if children.is_empty() {
        if lines.is_empty() {
            return Ok(format!("{pad}<{tag} />"));
        }
        if lines.len() == 1 {
            return Ok(format!("{pad}<{tag} {} />", lines[0]));
        }
        let mut out = format!("{pad}<{tag}");
        for line in &lines {
            out.push('\n');
            out.push_str(&pad);
            out.push_str("  ");
            out.push_str(line);
        }
        out.push('\n');
        out.push_str(&pad);
        out.push_str("/>");
        return Ok(out);
    }

    let mut out = if lines.is_empty() {
        format!("{pad}<{tag}>")
    } else {
        let mut out = format!("{pad}<{tag}");
        for line in &lines {
            out.push('\n');
            out.push_str(&pad);
            out.push_str("  ");
            out.push_str(line);
        }
        out.push('\n');
        out.push_str(&pad);
        out.push('>');
        out
    };
    for child in &children {
        out.push('\n');
        if child.is_text() {
            out.push_str(&pad);
            out.push_str("  ");
            out.push_str(&text_jsx(child.text().unwrap_or("").trim()));
        } else {
            out.push_str(&render(*child, indent + 2, false)?);
        }
    }
    out.push('\n');
    out.push_str(&pad);
    out.push_str(&format!("</{tag}>"));
    Ok(out)
}

fn render_attribute(name: &str, value: &str) -> String {
    let name = jsx_name(name);
    if !value.contains('"') && !value.contains('\n') {
        return format!("{name}=\"{value}\"");
    }
    format!("{name}={}", json_string(value))
}

/// `class` becomes `className`, kebab-case becomes camelCase, and `data-*` is left alone.
fn jsx_name(name: &str) -> String {
    if name.starts_with("data-") || name.starts_with("aria-") {
        return name.to_string();
    }
    if name == "class" {
        return "className".to_string();
    }
    let mut parts = name.split(['-', ':']);
    let mut out = parts.next().unwrap_or_default().to_string();
    for part in parts {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// `fill: none; stroke: red` becomes `{{ fill: "none", stroke: "red" }}`.
fn style_expression(value: &str) -> Option<String> {
    let mut entries = Vec::new();
    for declaration in value.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        if let Some((property, raw)) = declaration.split_once(':') {
            entries.push(format!(
                "{}: {}",
                css_name(property.trim()),
                json_string(raw.trim())
            ));
        }
    }
    if entries.is_empty() {
        None
    } else {
        Some(format!("{{{}}}", entries.join(", ")))
    }
}

/// CSS property to React style key, `-webkit-mask` included.
fn css_name(property: &str) -> String {
    let camel = jsx_name(property);
    match camel.strip_prefix('-') {
        Some(rest) => {
            let mut chars = rest.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
                None => camel,
            }
        }
        None => camel,
    }
}

fn text_jsx(text: &str) -> String {
    if text.contains(['{', '}', '<', '>']) {
        json_string(text)
    } else {
        text.to_string()
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(character),
        }
    }
    out.push('"');
    out
}

/// `width="24" height="24"` becomes `size-6`, the Tailwind step closest to the pixel value.
fn size_class(width: &str, height: &str) -> Option<String> {
    let width = pixels(width)?;
    let height = pixels(height)?;
    if width != height || width % 4 != 0 {
        return None;
    }
    let step = width / 4;
    (1..=96).contains(&step).then(|| format!("size-{step}"))
}

fn pixels(value: &str) -> Option<u32> {
    value.trim().trim_end_matches("px").trim().parse().ok()
}
