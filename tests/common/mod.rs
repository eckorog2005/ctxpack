#![allow(dead_code)]

use std::fs;
use std::path::Path;

use clap::Parser;
use pithpack::cli::CliArgs;
use pithpack::config::{self, Env};
use pithpack::{Io, Outcome};
use tempfile::TempDir;

pub fn fixture(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, content) in files {
        write(dir.path(), path, content);
    }
    dir
}

pub fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// The `path` attribute of every `<file>` block, in Bundle order.
pub fn file_paths(bundle: &str) -> Vec<&str> {
    bundle
        .lines()
        .filter_map(|l| l.strip_prefix("<file path=\""))
        .map(|l| l.split('"').next().unwrap())
        .collect()
}

pub struct Run {
    pub outcome: anyhow::Result<Outcome>,
    pub stdout: String,
    pub stderr: String,
}

/// Runs pithpack in-process with `cwd` as the working directory.
pub fn run(cwd: &Path, args: &[&str]) -> Run {
    run_with_tty(cwd, args, false)
}

pub fn run_with_tty(cwd: &Path, args: &[&str], stderr_is_tty: bool) -> Run {
    let cli =
        CliArgs::try_parse_from(std::iter::once("pithpack").chain(args.iter().copied())).unwrap();
    let env = Env {
        cwd: cwd.to_path_buf(),
    };
    let opts = config::resolve(cli, &env).unwrap();
    let mut stdin = std::io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let outcome = pithpack::run(
        opts,
        Io {
            stdin: &mut stdin,
            stdout: &mut stdout,
            stderr: &mut stderr,
            stderr_is_tty,
        },
    );
    Run {
        outcome,
        stdout: String::from_utf8(stdout).unwrap(),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}
