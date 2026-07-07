mod complexity;
mod no_prod_tests;

use std::{env, path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args = args();
    let command = args.first().context("missing quality command")?;
    dispatch(command, &args[1..])
}

fn args() -> Vec<String> {
    env::args().skip(1).collect()
}

fn dispatch(command: &str, args: &[String]) -> Result<()> {
    match command {
        "check-no-prod-tests" => no_prod_tests::run(root_arg(args)?),
        "check-complexity" => complexity::run(root_arg(args)?, max_arg(args)?),
        _ => anyhow::bail!("unknown quality command {command}"),
    }
}

fn root_arg(args: &[String]) -> Result<PathBuf> {
    option_value(args, "--root")
        .map(PathBuf::from)
        .context("missing --root")
}

fn max_arg(args: &[String]) -> Result<usize> {
    option_value(args, "--max-cyclomatic")
        .context("missing --max-cyclomatic")?
        .parse()
        .context("invalid --max-cyclomatic")
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find_map(|pair| match pair[0] == name {
        true => Some(pair[1].clone()),
        false => None,
    })
}
