pub mod cli;
pub mod config;
pub mod encoder;
pub mod formatter;
pub mod parser;
pub mod redact;
pub mod report;
pub mod scanner;

use std::io::{Read, Write};
use std::time::Instant;

use anyhow::Context;

use config::Options;
use encoder::Encoder;
use formatter::{BundleMeta, FileBlock};
use report::{FileStat, Report, ReportFormat};
use scanner::{Origin, Skip};

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
    OverBudget,
}

fn group_thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub fn run(opts: Options, io: Io) -> anyhow::Result<Outcome> {
    let started = Instant::now();
    let tokenizer = opts.tokenizer;
    let warm = std::thread::spawn(move || Encoder::get(tokenizer));
    if !opts.redact {
        writeln!(
            io.stderr,
            "warning: Redaction is disabled by --no-redact; secrets in Source Files reach the Bundle"
        )
        .context("cannot write a warning to stderr")?;
    }
    let scan = scanner::scan(&opts)?;
    for (display, skip) in &scan.skipped {
        if let Skip::Unreadable(kind) = skip {
            writeln!(io.stderr, "warning: skipped unreadable `{display}`: {kind}")
                .context("cannot write a warning to stderr")?;
        }
    }
    let mut texts = Vec::with_capacity(scan.candidates.len());
    let mut read_bytes = Vec::with_capacity(scan.candidates.len());
    let mut redactions = Vec::with_capacity(scan.candidates.len());
    for candidate in &scan.candidates {
        let bytes = if candidate.origin == Origin::Stdin {
            let mut bytes = Vec::new();
            io.stdin
                .read_to_end(&mut bytes)
                .context("cannot read stdin")?;
            bytes
        } else {
            std::fs::read(&candidate.path)
                .with_context(|| format!("cannot read `{}`", candidate.display))?
        };
        read_bytes.push(bytes.len() as u64);
        let raw = String::from_utf8_lossy(&bytes);
        if opts.redact {
            let redacted = redact::redact(&raw, redact::Rules::builtin());
            redactions.push(redacted.hits.len());
            texts.push(redacted.text.into_owned());
        } else {
            redactions.push(0);
            texts.push(raw.into_owned());
        }
    }
    let encoder = warm.join().expect("encoder initialisation panicked");
    let blocks: Vec<FileBlock> = scan
        .candidates
        .iter()
        .zip(&texts)
        .map(|(c, text)| FileBlock {
            display: &c.display,
            lang: match c.origin {
                Origin::Stdin => opts.stdin_lang,
                Origin::Explicit | Origin::Walked => parser::detect(&c.display),
            },
            mode: UNCOMPRESSED,
            tokens: encoder.count(text),
            text,
        })
        .collect();
    let mut meta = BundleMeta {
        format: opts.format,
        mode: UNCOMPRESSED,
        tokenizer: encoder.name(),
        tokens: 0,
        tree: opts.tree,
    };
    // The total sits inside the text it counts; the re-render's digits may shift it by ±2.
    meta.tokens = encoder.count(&formatter::render(&meta, &blocks));
    let bundle = formatter::render(&meta, &blocks);
    io.stdout
        .write_all(bundle.as_bytes())
        .context("cannot write the Bundle to stdout")?;
    io.stdout.flush()?;

    if opts.report == ReportFormat::Table {
        let report = Report {
            files: blocks
                .iter()
                .zip(&read_bytes)
                .zip(&redactions)
                .map(|((b, &original_bytes), &redactions)| FileStat {
                    display: b.display.to_string(),
                    lang: b.lang,
                    mode: b.mode,
                    original_bytes,
                    original_tokens: b.tokens,
                    compressed_bytes: b.text.len() as u64,
                    compressed_tokens: b.tokens,
                    redactions,
                })
                .collect(),
            skipped: scan.skipped,
            bundle_tokens: meta.tokens,
            tokenizer: encoder.name(),
            elapsed: started.elapsed(),
        };
        io.stderr
            .write_all(report::render_table(&report, io.stderr_is_tty).as_bytes())
            .context("cannot write the Report to stderr")?;
    }
    match opts.budget {
        Some(budget) if meta.tokens > budget => {
            writeln!(
                io.stderr,
                "warning: bundle is {} tokens, budget {}",
                group_thousands(meta.tokens),
                group_thousands(budget)
            )
            .context("cannot write a warning to stderr")?;
            Ok(Outcome::OverBudget)
        }
        _ => Ok(Outcome::Ok),
    }
}
