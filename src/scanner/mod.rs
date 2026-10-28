pub mod exclusion;

use std::path::{Path, PathBuf};

use anyhow::Context;
use ignore::WalkBuilder;

use crate::config::{Input, Options, display_path};

#[derive(Debug)]
pub struct Candidate {
    pub path: PathBuf,
    /// Relative to the current directory as the user typed it, with `/` separators.
    pub display: String,
    pub origin: Origin,
}

/// Ordered so that sorting puts an `Explicit` Candidate before a `Walked` one with the same path.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Origin {
    Explicit,
    Walked,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Skip {
    Excluded,
}

#[derive(Debug)]
pub struct ScanResult {
    /// Sorted by `display`, without duplicates.
    pub candidates: Vec<Candidate>,
    pub skipped: Vec<(String, Skip)>,
}

pub fn scan(opts: &Options) -> anyhow::Result<ScanResult> {
    let mut found = Vec::new();
    for input in &opts.inputs {
        let Input::Path { path, display } = input;
        let meta =
            std::fs::metadata(path).with_context(|| format!("cannot read path `{display}`"))?;
        if meta.is_file() {
            found.push(Candidate {
                path: path.clone(),
                display: display.clone(),
                origin: Origin::Explicit,
            });
        } else {
            walk(path, display, &mut found)?;
        }
    }
    found.sort_by(|a, b| (&a.display, a.origin).cmp(&(&b.display, b.origin)));
    found.dedup_by(|a, b| a.display == b.display);

    let mut candidates = Vec::with_capacity(found.len());
    let mut skipped = Vec::new();
    for candidate in found {
        let admitted = candidate.origin == Origin::Explicit && opts.allow_secret_file;
        if exclusion::is_excluded(&candidate.path) && !admitted {
            skipped.push((candidate.display, Skip::Excluded));
        } else {
            candidates.push(candidate);
        }
    }
    Ok(ScanResult {
        candidates,
        skipped,
    })
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
            origin: Origin::Walked,
        });
    }
    Ok(())
}
