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
