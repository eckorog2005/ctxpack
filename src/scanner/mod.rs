pub mod exclusion;
mod sniff;

use std::path::{Path, PathBuf};

use anyhow::Context;
use ignore::WalkBuilder;
use ignore::overrides::{Override, OverrideBuilder};
use ignore::types::{Types, TypesBuilder};

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
    TooLarge { len: u64 },
    Binary,
    Unreadable(std::io::ErrorKind),
}

#[derive(Debug)]
pub struct ScanResult {
    /// Sorted by `display`, without duplicates.
    pub candidates: Vec<Candidate>,
    pub skipped: Vec<(String, Skip)>,
}

pub fn scan(opts: &Options) -> anyhow::Result<ScanResult> {
    let mut found = Vec::new();
    let mut skipped = Vec::new();
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
            walk(path, display, opts, &mut found, &mut skipped)?;
        }
    }
    found.sort_by(|a, b| (&a.display, a.origin).cmp(&(&b.display, b.origin)));
    found.dedup_by(|a, b| a.display == b.display);

    let mut candidates = Vec::with_capacity(found.len());
    for candidate in found {
        match admit(&candidate, opts) {
            Ok(()) => candidates.push(candidate),
            Err(skip) => skipped.push((candidate.display, skip)),
        }
    }
    skipped.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ScanResult {
        candidates,
        skipped,
    })
}

/// Exclusion comes first so that a secret file is never opened.
fn admit(candidate: &Candidate, opts: &Options) -> Result<(), Skip> {
    let secret_allowed = candidate.origin == Origin::Explicit && opts.allow_secret_file;
    // Explicit arguments are always followed, so their link targets are checked too.
    let followed = candidate.origin == Origin::Explicit || opts.filter.follow_symlinks;
    let excluded = exclusion::is_excluded(&candidate.path)
        || followed
            && std::fs::canonicalize(&candidate.path)
                .is_ok_and(|target| exclusion::is_excluded(&target));
    if excluded && !secret_allowed {
        return Err(Skip::Excluded);
    }
    let unreadable = |err: std::io::Error| Skip::Unreadable(err.kind());
    let len = std::fs::metadata(&candidate.path)
        .map_err(unreadable)?
        .len();
    if len > opts.filter.max_file_size {
        return Err(Skip::TooLarge { len });
    }
    if sniff::is_binary(&candidate.path).map_err(unreadable)? {
        return Err(Skip::Binary);
    }
    Ok(())
}

fn walk(
    root: &Path,
    root_display: &str,
    opts: &Options,
    out: &mut Vec<Candidate>,
    skipped: &mut Vec<(String, Skip)>,
) -> anyhow::Result<()> {
    let walker = WalkBuilder::new(root)
        .types(include_types(&opts.filter.include)?)
        .overrides(exclude_overrides(root, &opts.filter.exclude)?)
        .follow_links(opts.filter.follow_symlinks)
        .hidden(false)
        .require_git(false)
        .add_custom_ignore_filename(".pithignore")
        .filter_entry(|e| e.file_name() != ".git")
        .build();
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => match (err.io_error(), error_path(&err)) {
                (Some(io), Some(path)) => {
                    let skip = Skip::Unreadable(io.kind());
                    skipped.push((join_display(root_display, root, path)?, skip));
                    continue;
                }
                // A link back to an ancestor adds nothing the walk has not already seen.
                _ if is_loop(&err) => continue,
                _ => return Err(err.into()),
            },
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        out.push(Candidate {
            display: join_display(root_display, root, entry.path())?,
            path: entry.into_path(),
            origin: Origin::Walked,
        });
    }
    Ok(())
}

fn include_types(extensions: &[String]) -> anyhow::Result<Types> {
    let mut types = TypesBuilder::new();
    if !extensions.is_empty() {
        for ext in extensions {
            types.add("included", &format!("*.{}", globset::escape(ext)))?;
        }
        types.select("included");
    }
    Ok(types.build()?)
}

fn exclude_overrides(root: &Path, globs: &[String]) -> anyhow::Result<Override> {
    let mut overrides = OverrideBuilder::new(root);
    for glob in globs {
        overrides.add(&format!("!{glob}"))?;
    }
    Ok(overrides.build()?)
}

fn join_display(root_display: &str, root: &Path, path: &Path) -> anyhow::Result<String> {
    let relative = display_path(path.strip_prefix(root)?);
    Ok(if root_display.is_empty() || root_display.ends_with('/') {
        format!("{root_display}{relative}")
    } else {
        format!("{root_display}/{relative}")
    })
}

fn is_loop(err: &ignore::Error) -> bool {
    match err {
        ignore::Error::Loop { .. } => true,
        ignore::Error::WithPath { err, .. }
        | ignore::Error::WithDepth { err, .. }
        | ignore::Error::WithLineNumber { err, .. } => is_loop(err),
        _ => false,
    }
}

fn error_path(err: &ignore::Error) -> Option<&Path> {
    match err {
        ignore::Error::WithPath { path, .. } => Some(path),
        ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => {
            error_path(err)
        }
        _ => None,
    }
}
