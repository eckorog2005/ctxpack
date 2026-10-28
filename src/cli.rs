use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pithpack", version, about)]
pub struct CliArgs {
    /// Files or directories to bundle
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Omit the File Tree from the Bundle
    #[arg(long)]
    pub no_tree: bool,
}
