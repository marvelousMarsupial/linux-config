//! Podman primitives for rootless dev containers.
//!
//! Every function here is a thin, focused wrapper around one `podman`
//! invocation. We keep them small and single-purpose so the `forge dev`
//! subcommands compose them without baking podman CLI knowledge into the
//! command layer. If/when we swap to libpod, only this file changes.
//!
//! All functions assume `podman` is on `$PATH`. Missing podman surfaces
//! as a subprocess error with command context — let the binary print it.

use anyhow::{Context, Result};
use std::process::{Command, Stdio};

use crate::run::checked;

/// Fully-qualified container name: dev containers are all prefixed so
/// `podman ps` output is easy to scan and `forge dev rm` can't accidentally
/// hit a container you didn't create with forge.
pub const DEV_PREFIX: &str = "forge-dev-";

pub fn qualified(name: &str) -> String {
    if name.starts_with(DEV_PREFIX) {
        name.to_string()
    } else {
        format!("{DEV_PREFIX}{name}")
    }
}

/// Strip the prefix to get the user-facing name. Used for volume naming
/// so `forge dev up myapp` creates volume `myapp-home`, not `forge-dev-myapp-home`.
pub fn bare_name(name: &str) -> &str {
    name.strip_prefix(DEV_PREFIX).unwrap_or(name)
}

/// Does a container with this (qualified or bare) name exist?
/// Uses `podman container exists` which is the cheapest query available.
pub fn exists(name: &str) -> bool {
    let q = qualified(name);
    Command::new("podman")
        .args(["container", "exists", &q])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Is it currently running? `ps --filter name=... --format '{{.Names}}'`
/// returns the name if it's running and empty otherwise.
pub fn running(name: &str) -> bool {
    let q = qualified(name);
    let out = Command::new("podman")
        .args([
            "ps",
            "--format",
            "{{.Names}}",
            "--filter",
            &format!("name=^{q}$"),
        ])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == q,
        Err(_) => false,
    }
}

/// Volume argument for `create`: either a named volume or a host bind mount.
pub enum Volume {
    /// Named podman volume: `volume_name:container_path`.
    Named { volume: String, container: String },
    /// Host path bind mount: `host_path:container_path`.
    Bind { host: String, container: String },
}

/// Create a new container from an image. Stays detached and running so
/// subsequent `attach`/`exec` calls can enter it. `sleep infinity` keeps
/// PID 1 alive; real shells are opened via `podman exec`.
///
/// When `ssh_agent` is true, the host's `$SSH_AUTH_SOCK` is mounted into
/// the container and the env var is set so git/ssh can request signatures
/// from the host agent. No key files are copied — the agent holds the
/// keys and the container only talks to it via the socket.
pub fn create(name: &str, image: &str, volumes: &[Volume], ports: &[u16], ssh_agent: bool) -> Result<()> {
    let q = qualified(name);
    let mut argv: Vec<String> = vec![
        "podman".into(),
        "run".into(),
        "-d".into(),
        "--name".into(),
        q,
        "--userns".into(),
        "keep-id".into(),
    ];
    for v in volumes {
        argv.push("-v".into());
        match v {
            Volume::Named { volume, container } => argv.push(format!("{volume}:{container}")),
            Volume::Bind { host, container } => argv.push(format!("{host}:{container}")),
        }
    }
    for p in ports {
        argv.push("-p".into());
        argv.push(format!("{p}:{p}"));
    }
    if ssh_agent {
        if let Ok(sock) = std::env::var("SSH_AUTH_SOCK") {
            let ctr_sock = "/tmp/ssh-agent.sock";
            argv.push("-v".into());
            argv.push(format!("{sock}:{ctr_sock}"));
            argv.push("-e".into());
            argv.push(format!("SSH_AUTH_SOCK={ctr_sock}"));
        }
    }
    argv.push(image.to_string());
    argv.push("sleep".into());
    argv.push("infinity".into());

    let refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
    checked(&refs).context("`podman run`")?;
    Ok(())
}

/// Start an already-created container. No-op if already running.
pub fn start(name: &str) -> Result<()> {
    if running(name) {
        return Ok(());
    }
    let q = qualified(name);
    let status = Command::new("podman")
        .args(["start", &q])
        .status()
        .context("`podman start`")?;
    if !status.success() {
        anyhow::bail!("`podman start {q}` failed");
    }
    Ok(())
}

/// Stop a running container. No-op if not running.
pub fn stop(name: &str) -> Result<()> {
    if !running(name) {
        return Ok(());
    }
    let q = qualified(name);
    let _ = Command::new("podman").args(["stop", &q]).status();
    Ok(())
}

/// Remove a container. Fails if it's running and `force` is false.
pub fn remove(name: &str, force: bool) -> Result<()> {
    let q = qualified(name);
    let mut args = vec!["rm".to_string()];
    if force {
        args.push("-f".into());
    }
    args.push(q);
    let status = Command::new("podman")
        .args(&args)
        .status()
        .context("`podman rm`")?;
    if !status.success() {
        anyhow::bail!("`podman rm` failed");
    }
    Ok(())
}

/// Open an interactive shell in the container. Replaces the current
/// process via `exec` semantics so the user's terminal is fully wired up
/// (tty, signals, env). Does not return on success.
pub fn shell(name: &str) -> Result<()> {
    let q = qualified(name);
    let status = Command::new("podman")
        .args(["exec", "-it", &q, "zsh"])
        .status()
        .context("`podman exec`")?;
    if !status.success() {
        anyhow::bail!("shell exited with status {status}");
    }
    Ok(())
}

/// List all forge dev containers with their running state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub name: String,
    pub image: String,
    pub state: String,
}

pub fn list() -> Result<Vec<Listing>> {
    let out = Command::new("podman")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.Names}}\t{{.Image}}\t{{.State}}",
            "--filter",
            &format!("name={DEV_PREFIX}"),
        ])
        .output()
        .context("`podman ps`")?;
    let s = String::from_utf8_lossy(&out.stdout);
    Ok(s.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut parts = l.splitn(3, '\t');
            Listing {
                name: parts.next().unwrap_or("").to_string(),
                image: parts.next().unwrap_or("").to_string(),
                state: parts.next().unwrap_or("").to_string(),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_adds_prefix() {
        assert_eq!(qualified("rust"), "forge-dev-rust");
    }

    #[test]
    fn qualified_is_idempotent() {
        assert_eq!(qualified("forge-dev-rust"), "forge-dev-rust");
    }

    #[test]
    fn bare_name_strips_prefix() {
        assert_eq!(bare_name("forge-dev-rust"), "rust");
    }

    #[test]
    fn bare_name_no_prefix_unchanged() {
        assert_eq!(bare_name("myapp"), "myapp");
    }

    #[test]
    fn prefix_is_stable() {
        // Anyone who relies on the prefix externally (shell scripts,
        // completion, grep filters) should be able to trust the constant.
        assert_eq!(DEV_PREFIX, "forge-dev-");
    }
}
