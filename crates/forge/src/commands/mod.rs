use anyhow::Result;

use crate::cli::{Cli, Command};

pub mod dev;
pub mod scratch;
pub mod setup;
pub mod status;
pub mod theme;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Setup {
            role,
            dry_run,
            diff,
            refresh_downloads,
        } => setup::run(role, dry_run, diff, refresh_downloads),
        Command::Switch { role, dry_run } => setup::switch(role, dry_run),
        Command::Teardown { dry_run } => setup::teardown(dry_run),
        Command::Status => status::show(),
        Command::Diff { role } => setup::diff(role),
        Command::Theme { sub } => theme::dispatch(sub),
        Command::Dev { sub } => dev::dispatch(sub),
        Command::Lxc { name, no_shell, ip } => scratch::lxc(name, no_shell, ip),
        Command::Vm { name, no_shell, ip } => scratch::vm(name, no_shell, ip),
    }
}
