use anyhow::Result;

use crate::cli::{Cli, Command, Target};

pub mod dev;
pub mod setup;
pub mod status;
pub mod target;
pub mod theme;
pub mod vm;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Setup {
            role,
            dry_run,
            diff,
            target,
            refresh_downloads,
        } => match target {
            Target::Local => setup::run(role, dry_run, diff, refresh_downloads),
            Target::Container(name) => target::setup_in_container(&name, role),
            Target::Vm(name) => target::setup_in_vm(&name, role),
        },
        Command::Switch { role, dry_run } => setup::switch(role, dry_run),
        Command::Teardown { dry_run } => setup::teardown(dry_run),
        Command::Status => status::show(),
        Command::Diff { role } => setup::diff(role),
        Command::Theme { sub } => theme::dispatch(sub),
        Command::Dev { sub } => dev::dispatch(sub),
        Command::Vm { sub } => vm::dispatch(sub),
    }
}
