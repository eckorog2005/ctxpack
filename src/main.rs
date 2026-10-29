use std::io::IsTerminal;
use std::process::ExitCode;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use pithpack::cli::CliArgs;
use pithpack::config::{self, ConfigError, Env};
use pithpack::{Io, Outcome};

fn main() -> ExitCode {
    let cli = CliArgs::parse();
    match try_main(cli) {
        Ok(Outcome::Ok) => ExitCode::SUCCESS,
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
