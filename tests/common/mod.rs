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

pub struct Run {
    pub outcome: anyhow::Result<Outcome>,
    pub stdout: String,
    pub stderr: String,
}

/// Runs pithpack in-process with `cwd` as the working directory.
pub fn run(cwd: &Path, args: &[&str]) -> Run {
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
            stderr_is_tty: false,
        },
    );
    Run {
        outcome,
        stdout: String::from_utf8(stdout).unwrap(),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}
