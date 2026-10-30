use std::io::IsTerminal;
use std::path::{Component, Path, PathBuf};

use crate::cli::CliArgs;
use crate::encoder::TokenizerKind;
use crate::parser::Language;
use crate::report::ReportFormat;
use crate::sink::SinkKind;

/// The process environment `resolve` depends on, injected so it can be faked in tests.
pub struct Env {
    pub cwd: PathBuf,
    pub stdin_is_tty: bool,
}

impl Env {
    pub fn from_process() -> std::io::Result<Env> {
        Ok(Env {
            cwd: std::env::current_dir()?,
            stdin_is_tty: std::io::stdin().is_terminal(),
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
    Stdin,
}

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct FilterOptions {
    /// Extensions without the leading dot; empty keeps every extension.
    pub include: Vec<String>,
    /// Gitignore-style globs, matched relative to each walked directory.
    pub exclude: Vec<String>,
    pub max_file_size: u64,
    pub follow_symlinks: bool,
}

#[derive(Debug)]
pub struct Options {
    pub inputs: Vec<Input>,
    pub filter: FilterOptions,
    pub allow_secret_file: bool,
    pub stdin_lang: Option<Language>,
    /// Only the CLI can turn this off; no config key may.
    pub redact: bool,
    pub tree: bool,
    pub tokenizer: TokenizerKind,
    pub report: ReportFormat,
    pub sink: SinkKind,
}

#[derive(Debug)]
pub enum ConfigError {
    /// A usage error: there are no paths, and reading a terminal would wait forever.
    NoInput,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoInput => f.write_str(
                "no paths given and stdin is a terminal; pass paths, or pipe input into pithpack",
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn resolve(cli: CliArgs, env: &Env) -> Result<Options, ConfigError> {
    if cli.paths.is_empty() && env.stdin_is_tty {
        return Err(ConfigError::NoInput);
    }
    let (dashes, paths): (Vec<_>, Vec<_>) =
        cli.paths.iter().partition(|arg| arg.as_os_str() == "-");
    let mut inputs: Vec<Input> = paths
        .into_iter()
        .map(|arg| Input::Path {
            path: env.cwd.join(arg),
            display: display_path(arg),
        })
        .collect();
    if cli.paths.is_empty() || !dashes.is_empty() {
        inputs.push(Input::Stdin);
    }
    Ok(Options {
        inputs,
        filter: FilterOptions {
            include: cli
                .include
                .iter()
                .map(|ext| ext.trim_start_matches('.').to_string())
                .filter(|ext| !ext.is_empty())
                .collect(),
            exclude: cli.exclude,
            max_file_size: cli.max_file_size.unwrap_or(DEFAULT_MAX_FILE_SIZE),
            follow_symlinks: cli.follow_symlinks,
        },
        allow_secret_file: cli.allow_secret_file,
        stdin_lang: cli.lang,
        redact: !cli.no_redact,
        tree: !cli.no_tree,
        tokenizer: cli.tokenizer.unwrap_or(TokenizerKind::O200k),
        sink: match (cli.output, cli.clipboard) {
            (Some(path), _) => SinkKind::File(env.cwd.join(path)),
            (None, true) => SinkKind::Clipboard,
            (None, false) => SinkKind::Stdout,
        },
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
