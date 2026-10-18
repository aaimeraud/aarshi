use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

/// A dictionary entry: its key and its raw value as written in the file, quotes included.
pub struct Entry {
    pub key: String,
    pub value: String,
}

pub fn file(root: &Path, locale: &str) -> PathBuf {
    root.join("lib/i18n/dictionaries")
        .join(format!("{locale}.ts"))
}

/// Reads the `"key": "value",` lines written by aarshi.
pub fn entries(content: &str) -> Vec<Entry> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix('"')?;
            let (key, rest) = rest.split_once("\": ")?;
            let value = rest.trim_end_matches(',').trim();
            Some(Entry {
                key: key.to_string(),
                value: value.to_string(),
            })
        })
        .collect()
}

pub fn is_empty(entry: &Entry) -> bool {
    entry.value.is_empty() || entry.value == "\"\""
}

/// Page names are the folders of `views/` that hold a `<name>.view.tsx` file.
pub fn pages(root: &Path) -> Result<Vec<String>> {
    let views = root.join("views");
    let mut pages = Vec::new();
    for entry in fs::read_dir(&views).with_context(|| format!("cannot read {}", views.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().join(format!("{name}.view.tsx")).exists() {
            pages.push(name);
        }
    }
    pages.sort();
    Ok(pages)
}
