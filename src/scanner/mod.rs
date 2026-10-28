use std::path::{Path, PathBuf};

use anyhow::Context;
use ignore::WalkBuilder;

use crate::config::{Input, Options, display_path};

#[derive(Debug)]
pub struct Candidate {
    pub path: PathBuf,
    /// Relative to the current directory as the user typed it, with `/` separators.
    pub display: String,
}

#[derive(Debug)]
pub struct ScanResult {
    /// Sorted by `display`, without duplicates.
    pub candidates: Vec<Candidate>,
}

pub fn scan(opts: &Options) -> anyhow::Result<ScanResult> {
    let mut candidates = Vec::new();
    for input in &opts.inputs {
        let Input::Path { path, display } = input;
        let meta =
            std::fs::metadata(path).with_context(|| format!("cannot read path `{display}`"))?;
        if meta.is_file() {
            candidates.push(Candidate {
                path: path.clone(),
                display: display.clone(),
            });
        } else {
            walk(path, display, &mut candidates)?;
        }
    }
    candidates.sort_by(|a, b| a.display.cmp(&b.display));
    candidates.dedup_by(|a, b| a.display == b.display);
    Ok(ScanResult { candidates })
}

fn walk(root: &Path, root_display: &str, out: &mut Vec<Candidate>) -> anyhow::Result<()> {
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(|e| e.file_name() != ".git")
        .build();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let relative = display_path(entry.path().strip_prefix(root)?);
        let display = if root_display.is_empty() || root_display.ends_with('/') {
            format!("{root_display}{relative}")
        } else {
            format!("{root_display}/{relative}")
        };
        out.push(Candidate {
            path: entry.into_path(),
            display,
        });
    }
    Ok(())
}
