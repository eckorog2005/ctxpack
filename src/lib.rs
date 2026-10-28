pub mod cli;
pub mod config;
pub mod formatter;
pub mod parser;
pub mod scanner;

use std::io::{Read, Write};

use anyhow::Context;

use config::Options;
use formatter::{BundleMeta, FileBlock};

/// Content passes through uncompressed until the compressor lands, so no Mode is claimed.
const UNCOMPRESSED: &str = "none";

pub struct Io<'a> {
    pub stdin: &'a mut dyn Read,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write,
    pub stderr_is_tty: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
}

pub fn run(opts: Options, io: Io) -> anyhow::Result<Outcome> {
    let scan = scanner::scan(&opts)?;
    let mut texts = Vec::with_capacity(scan.candidates.len());
    for candidate in &scan.candidates {
        let bytes = std::fs::read(&candidate.path)
            .with_context(|| format!("cannot read `{}`", candidate.display))?;
        texts.push(String::from_utf8_lossy(&bytes).into_owned());
    }
    let blocks: Vec<FileBlock> = scan
        .candidates
        .iter()
        .zip(&texts)
        .map(|(c, text)| FileBlock {
            display: &c.display,
            lang: parser::detect(&c.display),
            mode: UNCOMPRESSED,
            text,
        })
        .collect();
    let meta = BundleMeta {
        mode: UNCOMPRESSED,
        tree: opts.tree,
    };
    io.stdout
        .write_all(formatter::render(&meta, &blocks).as_bytes())
        .context("cannot write the Bundle to stdout")?;
    io.stdout.flush()?;
    Ok(Outcome::Ok)
}
