use std::path::PathBuf;

use clap::Parser;

use crate::encoder::TokenizerKind;
use crate::formatter::Format;
use crate::parser::Language;
use crate::report::ReportFormat;

#[derive(Parser, Debug)]
#[command(name = "pithpack", version, about)]
pub struct CliArgs {
    /// Files or directories to bundle; `-` reads stdin, as does giving none with stdin piped
    pub paths: Vec<PathBuf>,

    /// Keep only files with these extensions, e.g. `rs,toml`
    #[arg(short, long, value_delimiter = ',', value_name = "EXT,...")]
    pub include: Vec<String>,

    /// Skip files matching this gitignore-style glob, relative to each walked directory
    #[arg(short, long, value_name = "GLOB", value_parser = parse_exclude)]
    pub exclude: Vec<String>,

    /// Skip files larger than this many bytes; accepts `KiB` and `MiB` suffixes [default: 1MiB]
    #[arg(long, value_name = "SIZE", value_parser = parse_size)]
    pub max_file_size: Option<u64>,

    /// Follow symbolic links; Exclusion still applies to the link target
    #[arg(long)]
    pub follow_symlinks: bool,

    /// Admit a secret-looking file, but only one whose exact path is given as an argument
    #[arg(long)]
    pub allow_secret_file: bool,

    /// Disable Redaction of secret-looking values; always warns on stderr
    #[arg(long)]
    pub no_redact: bool,

    /// Language of stdin, for labelling; without it stdin is Plain Text
    #[arg(long, value_name = "LANG", value_parser = parse_lang)]
    pub lang: Option<Language>,

    /// Bundle format [default: xml]
    #[arg(short, long, value_enum)]
    pub format: Option<Format>,

    /// Omit the File Tree from the Bundle
    #[arg(long)]
    pub no_tree: bool,

    /// Tokenizer used for token counts [default: o200k]
    #[arg(long, value_enum)]
    pub tokenizer: Option<TokenizerKind>,

    /// Report format on stderr [default: table]
    #[arg(long, value_enum)]
    pub report: Option<ReportFormat>,

    /// Same as `--report none`
    #[arg(short, long)]
    pub quiet: bool,
}

fn parse_exclude(glob: &str) -> Result<String, String> {
    ignore::overrides::OverrideBuilder::new("")
        .add(&format!("!{glob}"))
        .map_err(|err| err.to_string())?;
    Ok(glob.to_string())
}

fn parse_lang(name: &str) -> Result<Language, String> {
    Language::from_name(name).ok_or_else(|| {
        let names: Vec<&str> = Language::ALL.iter().map(|lang| lang.name()).collect();
        format!("expected one of {}", names.join(", "))
    })
}

fn parse_size(size: &str) -> Result<u64, String> {
    let (digits, unit) = if let Some(n) = size.strip_suffix("MiB") {
        (n, 1024 * 1024)
    } else if let Some(n) = size.strip_suffix("KiB") {
        (n, 1024)
    } else {
        (size, 1)
    };
    digits
        .parse::<u64>()
        .ok()
        .and_then(|n| n.checked_mul(unit))
        .ok_or_else(|| {
            "expected a whole number of bytes, optionally with a KiB or MiB suffix".into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_parse_as_bytes_or_binary_suffixes() {
        assert_eq!(parse_size("0"), Ok(0));
        assert_eq!(parse_size("1500"), Ok(1500));
        assert_eq!(parse_size("512KiB"), Ok(524_288));
        assert_eq!(parse_size("1MiB"), Ok(1_048_576));
    }

    #[test]
    fn malformed_or_overflowing_sizes_are_rejected() {
        for size in [
            "",
            "MiB",
            "1 MiB",
            "1mib",
            "1GiB",
            "-1",
            "1.5KiB",
            "99999999999999999MiB",
        ] {
            assert!(parse_size(size).is_err(), "{size:?} should be rejected");
        }
    }
}
