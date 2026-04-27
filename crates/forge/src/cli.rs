//! Command-line surface for `forge`.
//!
//! Two top-level domains:
//! * Host provisioning — `setup`, `switch`, `teardown`, `status`, `diff`.
//!   These read `roles.toml` (embedded) and reconcile the local machine
//!   with a role. Always operates on the local host: a previous
//!   `--target=container:` / `--target=vm:` flag was retired with the
//!   `forge vm` subcommand. To provision a remote box, you SSH in and
//!   run `forge setup <role>` there; that's the workflow.
//! * Dev pods — `theme`, `dev`. Container-side conveniences for daily
//!   work.

use clap::{Parser, Subcommand};
use forge_common::Role;

#[derive(Parser, Debug)]
#[command(name = "forge", about = "Workstation provisioning CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install a role on the local machine.
    Setup {
        role: Role,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        diff: bool,
        /// Re-download binaries even if they already exist on disk.
        #[arg(long)]
        refresh_downloads: bool,
    },
    /// Teardown, then setup a new role.
    Switch {
        role: Role,
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove what the current role installed.
    Teardown {
        #[arg(long)]
        dry_run: bool,
    },
    /// Show current role, theme, and applied timestamp.
    Status,
    /// Preview what `setup` or `sync` would change.
    Diff { role: Role },
    /// Color scheme switcher.
    Theme {
        #[command(subcommand)]
        sub: ThemeCmd,
    },
    /// Dev container (podman rootless) management.
    Dev {
        #[command(subcommand)]
        sub: DevCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum ThemeCmd {
    List,
    Current,
    Apply { name: String },
    Preview { name: String },
}

#[derive(Subcommand, Debug)]
pub enum DevCmd {
    /// Build the dev container image. There is exactly one image (defined
    /// in `config/dev-container.toml`); no name argument is needed.
    Build,
    /// Create and attach to a dev container.
    Up { name: String },
    /// List dev containers.
    Ls,
    /// Attach to an existing container.
    Attach { name: String },
    /// Run a one-off command in a container.
    Exec {
        name: String,
        #[arg(last = true)]
        cmd: Vec<String>,
    },
    /// Stop a container.
    Down { name: String },
    /// Remove a container.
    Rm { name: String },
}
