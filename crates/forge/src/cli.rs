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
    /// Spin up a clean Debian trixie LXC pre-bootstrapped for testing
    /// `forge setup` from zero.
    ///
    /// Wraps the otherwise-by-hand sequence we discovered while
    /// validating forge against fresh containers: launch the upstream
    /// `images:debian/13` image, register a static IP with incusbr0's
    /// dnsmasq, replace the in-guest networkd config with a static
    /// one (DHCPv4 doesn't fire in the upstream image), swap
    /// `/etc/resolv.conf` off the resolved stub for public DNS,
    /// install sudo + ca-certificates, useradd `august` (UID 1000)
    /// with NOPASSWD sudoers + `loginctl enable-linger`, push the
    /// local `target/release/forge` binary in, and (by default) drop
    /// you at a shell as `august` with `./forge` ready to run. After
    /// the test, `incus delete --force <name>`.
    Lxc {
        /// Container name (default: `forge-scratch`). Must not collide
        /// with an existing incus instance.
        name: Option<String>,
        /// Bootstrap and exit instead of dropping into a shell.
        /// Useful when chaining into a script.
        #[arg(long)]
        no_shell: bool,
        /// Static IPv4 to assign on incusbr0. Default `10.160.110.50`
        /// — chosen to avoid the typical `forge-trixie` (.10),
        /// `forge-vm` (.30), and gateway (.1) assignments.
        #[arg(long)]
        ip: Option<String>,
    },
    /// Spin up a clean Debian trixie KVM VM pre-bootstrapped for
    /// testing `forge setup` from zero. Same shape as `forge lxc`
    /// but spawns a real virtual machine via `incus launch --vm`,
    /// which lets you exercise the systemd-user-service / graphical
    /// session / virtio-gpu paths the LXC can't reach.
    ///
    /// VMs take longer to boot than LXCs (kernel + cloud-init), so
    /// expect ~30-60s before the in-guest setup begins.
    Vm {
        /// VM name (default: `forge-scratch-vm`).
        name: Option<String>,
        /// Bootstrap and exit instead of dropping into a shell.
        #[arg(long)]
        no_shell: bool,
        /// Static IPv4 on incusbr0. Default `10.160.110.51`.
        #[arg(long)]
        ip: Option<String>,
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
