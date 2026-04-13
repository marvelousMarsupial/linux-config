use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod embed;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    commands::dispatch(args)
}
