use std::io::IsTerminal;
use std::process::ExitCode;

use anyhow::Context;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use pithpack::cli::CliArgs;
use pithpack::config::{self, ConfigError, Env};
use pithpack::{Io, Outcome};

fn main() -> ExitCode {
    pithpack::sink::run_as_clipboard_holder();
    let cli = CliArgs::parse();
    match try_main(cli) {
        Ok(Outcome::Ok) => ExitCode::SUCCESS,
        Ok(Outcome::OverBudget) => ExitCode::from(3),
        Err(err) if matches!(err.downcast_ref(), Some(ConfigError::NoInput)) => CliArgs::command()
            .error(ErrorKind::MissingRequiredArgument, err)
            .exit(),
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn try_main(cli: CliArgs) -> anyhow::Result<Outcome> {
    if cli.command.is_some() {
        let path = config::init(&std::env::current_dir()?).context("cannot write pithpack.toml")?;
        eprintln!("created {}", path.display());
        return Ok(Outcome::Ok);
    }
    let env = Env::from_process()?;
    let opts = config::resolve(cli, &env)?;
    let stderr = std::io::stderr();
    let stderr_is_tty = stderr.is_terminal();
    pithpack::run(
        opts,
        Io {
            stdin: &mut std::io::stdin().lock(),
            stdout: &mut std::io::stdout().lock(),
            stderr: &mut stderr.lock(),
            stderr_is_tty,
        },
    )
}
