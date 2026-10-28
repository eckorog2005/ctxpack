use std::path::{Component, Path, PathBuf};

use crate::cli::CliArgs;
use crate::encoder::TokenizerKind;
use crate::report::ReportFormat;

/// The process environment `resolve` depends on, injected so it can be faked in tests.
pub struct Env {
    pub cwd: PathBuf,
}

impl Env {
    pub fn from_process() -> std::io::Result<Env> {
        Ok(Env {
            cwd: std::env::current_dir()?,
        })
    }
}

#[derive(Debug)]
pub enum Input {
    Path {
        path: PathBuf,
        /// The argument as typed, normalised to `/` separators; prefixes every display path.
        display: String,
    },
}

#[derive(Debug)]
pub struct Options {
    pub inputs: Vec<Input>,
    pub tree: bool,
    pub tokenizer: TokenizerKind,
    pub report: ReportFormat,
}

#[derive(Debug)]
pub enum ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for ConfigError {}

pub fn resolve(cli: CliArgs, env: &Env) -> Result<Options, ConfigError> {
    let inputs = cli
        .paths
        .iter()
        .map(|arg| Input::Path {
            path: env.cwd.join(arg),
            display: display_path(arg),
        })
        .collect();
    Ok(Options {
        inputs,
        tree: !cli.no_tree,
        tokenizer: cli.tokenizer.unwrap_or(TokenizerKind::O200k),
        report: if cli.quiet {
            ReportFormat::None
        } else {
            cli.report.unwrap_or(ReportFormat::Table)
        },
    })
}

/// Joins `path`'s components with `/` on every OS, dropping `.` components.
pub(crate) fn display_path(path: &Path) -> String {
    let mut out = String::new();
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push_str(&prefix.as_os_str().to_string_lossy()),
            Component::RootDir => out.push('/'),
            Component::CurDir => {}
            other => parts.push(other.as_os_str().to_string_lossy()),
        }
    }
    out.push_str(&parts.join("/"));
    out
}
