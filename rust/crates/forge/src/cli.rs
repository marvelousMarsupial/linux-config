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
    /// Install a role on the current system (or a target).
    Setup {
        role: Role,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        diff: bool,
    },
    /// Teardown, then setup a new role.
    Switch {
        role: Role,
    },
    /// Remove what the current role installed.
    Teardown {
        #[arg(long)]
        full: bool,
    },
    /// Show current role, theme, and applied timestamp.
    Status,
    /// Preview what `setup` or `sync` would change.
    Diff {
        role: Role,
    },
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
    /// Incus VM / container management.
    Vm {
        #[command(subcommand)]
        sub: VmCmd,
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
    /// Build a dev container image.
    Build { image: String },
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

#[derive(Subcommand, Debug)]
pub enum VmCmd {
    Create { profile: String },
    Ls,
    Start { name: String },
    Stop { name: String },
    Shell { name: String },
}
