use std::path::PathBuf;

use clap::Parser;

use crate::encoder::TokenizerKind;
use crate::report::ReportFormat;

#[derive(Parser, Debug)]
#[command(name = "pithpack", version, about)]
pub struct CliArgs {
    /// Files or directories to bundle
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Admit a secret-looking file, but only one whose exact path is given as an argument
    #[arg(long)]
    pub allow_secret_file: bool,

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
