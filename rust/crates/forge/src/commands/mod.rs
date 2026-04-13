use anyhow::Result;

use crate::cli::{Cli, Command};

pub mod dev;
pub mod setup;
pub mod status;
pub mod theme;
pub mod vm;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Setup { role, dry_run, diff } => {
            setup::run(role, dry_run, diff)
        }
        Command::Switch { role } => setup::switch(role),
        Command::Teardown { full } => setup::teardown(full),
        Command::Status => status::show(),
        Command::Diff { role } => setup::diff(role),
        Command::Theme { sub } => theme::dispatch(sub),
        Command::Dev { sub } => dev::dispatch(sub),
        Command::Vm { sub } => vm::dispatch(sub),
    }
}
