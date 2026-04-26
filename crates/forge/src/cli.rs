use clap::{Parser, Subcommand};
use forge_common::Role;
use std::str::FromStr;

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
        /// Where to provision: `local` (default), `container:<name>`, or `vm:<name>`.
        #[arg(long, default_value = "local")]
        target: Target,
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

/// Where `forge setup` should run: locally, inside a podman container, or
/// inside an incus VM/container. Parsed from strings like `local`,
/// `container:myapp`, `vm:attack-box`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Local,
    Container(String),
    Vm(String),
}

impl FromStr for Target {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "local" => Ok(Target::Local),
            _ if s.starts_with("container:") => Ok(Target::Container(s[10..].to_string())),
            _ if s.starts_with("vm:") => Ok(Target::Vm(s[3..].to_string())),
            _ => Err(anyhow::anyhow!(
                "invalid target `{s}` — expected local, container:<name>, or vm:<name>"
            )),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Local => write!(f, "local"),
            Target::Container(n) => write!(f, "container:{n}"),
            Target::Vm(n) => write!(f, "vm:{n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_parse_local() {
        assert_eq!(Target::from_str("local").unwrap(), Target::Local);
    }

    #[test]
    fn target_parse_container() {
        assert_eq!(
            Target::from_str("container:myapp").unwrap(),
            Target::Container("myapp".into())
        );
    }

    #[test]
    fn target_parse_vm() {
        assert_eq!(
            Target::from_str("vm:attack-box").unwrap(),
            Target::Vm("attack-box".into())
        );
    }

    #[test]
    fn target_parse_invalid() {
        assert!(Target::from_str("docker:foo").is_err());
    }

    #[test]
    fn target_roundtrip() {
        for t in [
            Target::Local,
            Target::Container("dev".into()),
            Target::Vm("net-box".into()),
        ] {
            assert_eq!(Target::from_str(&t.to_string()).unwrap(), t);
        }
    }
}
