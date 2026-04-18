//! Incus primitives for VMs and system containers.
//!
//! Mirrors `podman.rs` in shape: every function is one focused wrapper
//! around a single `incus` invocation. The `forge vm` subcommands
//! compose these into lifecycles. Incus handles both containers and
//! VMs with the same CLI (`--vm` at create time is the only divergence).

use anyhow::{Context, Result};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::run::checked;

pub const VM_PREFIX: &str = "forge-";

pub fn qualified(name: &str) -> String {
    if name.starts_with(VM_PREFIX) {
        name.to_string()
    } else {
        format!("{VM_PREFIX}{name}")
    }
}

/// Does the instance (container or VM) exist by name?
pub fn exists(name: &str) -> bool {
    let q = qualified(name);
    let out = Command::new("incus")
        .args(["list", "--format=csv", "-c", "n", &q])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .any(|l| l.trim() == q),
        Err(_) => false,
    }
}

/// Launch an instance from an image. `kind` decides container vs VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceKind {
    Container,
    Vm,
}

pub fn launch(name: &str, image: &str, kind: InstanceKind) -> Result<()> {
    let q = qualified(name);
    let mut argv: Vec<String> =
        vec!["incus".into(), "launch".into(), image.into(), q];
    if kind == InstanceKind::Vm {
        argv.push("--vm".into());
    }
    let refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
    checked(&refs).context("`incus launch`")?;
    Ok(())
}

/// Wait up to `timeout` for the instance to report an IPv4 address,
/// which is our proxy for "boot finished, network up". Returns false
/// on timeout so the caller can decide whether to bail or retry.
pub fn wait_ready(name: &str, timeout: Duration) -> bool {
    let q = qualified(name);
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let out = Command::new("incus")
            .args(["list", "--format=csv", "-c", "4", &q])
            .output();
        if let Ok(o) = out {
            let s = String::from_utf8_lossy(&o.stdout);
            if s.lines().any(|l| l.contains('.')) {
                return true;
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    false
}

pub fn start(name: &str) -> Result<()> {
    let q = qualified(name);
    let _ = Command::new("incus").args(["start", &q]).status();
    Ok(())
}

pub fn stop(name: &str) -> Result<()> {
    let q = qualified(name);
    let _ = Command::new("incus").args(["stop", &q]).status();
    Ok(())
}

pub fn delete(name: &str, force: bool) -> Result<()> {
    let q = qualified(name);
    let mut args = vec!["delete".to_string(), q];
    if force {
        args.push("--force".into());
    }
    let status = Command::new("incus")
        .args(&args)
        .status()
        .context("`incus delete`")?;
    if !status.success() {
        anyhow::bail!("`incus delete` failed");
    }
    Ok(())
}

/// Open an interactive shell inside the instance.
pub fn shell(name: &str) -> Result<()> {
    let q = qualified(name);
    let status = Command::new("incus")
        .args(["exec", &q, "--", "zsh"])
        .status()
        .context("`incus exec`")?;
    if !status.success() {
        anyhow::bail!("shell exited with status {status}");
    }
    Ok(())
}

/// Copy a local file into an instance. Used by `forge vm provision` to
/// push the forge binary onto a freshly-launched VM so it can run itself.
pub fn push_file(name: &str, src: &std::path::Path, dest: &str) -> Result<()> {
    let q = qualified(name);
    let src_str = src
        .to_str()
        .with_context(|| format!("non-UTF8 path: {}", src.display()))?;
    let status = Command::new("incus")
        .args(["file", "push", src_str, &format!("{q}{dest}")])
        .status()
        .context("`incus file push`")?;
    if !status.success() {
        anyhow::bail!("`incus file push` failed");
    }
    Ok(())
}

/// Listing row for `forge vm ls`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub name: String,
    pub state: String,
    pub ipv4: String,
    pub kind: String,
}

pub fn list() -> Result<Vec<Listing>> {
    let out = Command::new("incus")
        .args([
            "list",
            "--format=csv",
            "-c",
            "n,s,4,t",
            &format!("^{VM_PREFIX}"),
        ])
        .output()
        .context("`incus list`")?;
    let s = String::from_utf8_lossy(&out.stdout);
    Ok(s.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut parts = l.splitn(4, ',');
            Listing {
                name: parts.next().unwrap_or("").to_string(),
                state: parts.next().unwrap_or("").to_string(),
                ipv4: parts.next().unwrap_or("").to_string(),
                kind: parts.next().unwrap_or("").to_string(),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_adds_prefix() {
        assert_eq!(qualified("net"), "forge-net");
    }

    #[test]
    fn qualified_is_idempotent() {
        assert_eq!(qualified("forge-net"), "forge-net");
    }

    #[test]
    fn vm_prefix_is_stable() {
        assert_eq!(VM_PREFIX, "forge-");
    }
}
