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

/// Does an image with this tag exist locally?
///
/// Used by `forge dev up` to short-circuit with a useful error before
/// `podman run` would otherwise fail with the cryptic
/// `short-name "<tag>" did not resolve to an alias and no
/// unqualified-search registries are defined` — that message comes from
/// podman's short-name handling falling through to a registry lookup
/// when the local image is missing, and it does not mention `forge dev
/// build` (which is what the user actually needs to run).
pub fn image_exists(tag: &str) -> bool {
    Command::new("podman")
        .args(["image", "exists", tag])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Is the `podman` binary on PATH at all?
///
/// Checked at the top of every `forge dev` subcommand so the user gets a
/// clear "install podman" hint instead of the bare `os error 2` that
/// `Command::new("podman").status()?` produces when the binary is missing.
/// We probe with `podman --version` rather than `which podman` so we don't
/// depend on `which` being installed and so a podman that's on PATH but
/// broken (linked against a missing libc, for example) also surfaces
/// here, where the error message can be helpful.
pub fn installed() -> bool {
    Command::new("podman")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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

/// Render a [`Volume`] into the `host:container[:ro]` string podman
/// accepts after `-v`.
///
/// Extracted from [`create`] so the formatting can be unit-tested
/// without spawning podman. Named volumes never get a mode suffix
/// (the data is podman-managed; ro/rw doesn't apply at the syntax
/// level here). Bind mounts append `:ro` when `readonly` is set.
fn format_volume(v: &Volume) -> String {
    match v {
        Volume::Named { volume, container } => format!("{volume}:{container}"),
        Volume::Bind {
            host,
            container,
            readonly: false,
        } => format!("{host}:{container}"),
        Volume::Bind {
            host,
            container,
            readonly: true,
        } => format!("{host}:{container}:ro"),
    }
}

/// Volume argument for `create`: either a named volume or a host bind mount.
pub enum Volume {
    /// Named podman volume: `volume_name:container_path`.
    Named { volume: String, container: String },
    /// Host path bind mount: `host_path:container_path[:ro]`.
    ///
    /// When `readonly` is `true`, the mount is rendered with
    /// podman's `:ro` suffix so the container cannot mutate the
    /// host path. The Linux kernel mount-flag honours this even
    /// under rootless podman + `--userns keep-id`, where the
    /// container UID maps onto the host invoking-user UID; without
    /// `:ro` a misbehaving container could rewrite host system
    /// files like `/etc/localtime`.
    Bind {
        host: String,
        container: String,
        readonly: bool,
    },
}

/// Build the `podman run` argv for a dev container.
///
/// Extracted from [`create`] so the network / volume / agent flags
/// can be unit-tested without spawning podman.
///
/// The container always uses `--network=host`: rootless podman can't
/// give a container its own LAN identity (CAP_NET_ADMIN lives on the
/// host namespace), and the only practical "reach the container from
/// elsewhere" story for `forge dev` is for whatever the container
/// binds to also be bound on the host. Host networking gives that for
/// free — no per-project port lists to maintain, no `-p` enumeration —
/// at the cost of `localhost` inside the container meaning the host
/// (so any port collision with a host service is a real collision).
/// True LAN-citizen workloads belong on `forge lxc` / `forge vm`.
fn build_run_argv(
    name: &str,
    image: &str,
    volumes: &[Volume],
    ssh_agent: bool,
    ssh_auth_sock: Option<&str>,
) -> Vec<String> {
    let q = qualified(name);
    let mut argv: Vec<String> = vec![
        "podman".into(),
        "run".into(),
        "-d".into(),
        "--name".into(),
        q,
        "--userns".into(),
        "keep-id".into(),
        "--network".into(),
        "host".into(),
    ];
    for v in volumes {
        argv.push("-v".into());
        argv.push(format_volume(v));
    }
    if ssh_agent {
        if let Some(sock) = ssh_auth_sock {
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
    argv
}

/// Create a new container from an image. Stays detached and running so
/// subsequent `attach`/`exec` calls can enter it. `sleep infinity` keeps
/// PID 1 alive; real shells are opened via `podman exec`.
///
/// When `ssh_agent` is true, the host's `$SSH_AUTH_SOCK` is mounted into
/// the container and the env var is set so git/ssh can request signatures
/// from the host agent. No key files are copied — the agent holds the
/// keys and the container only talks to it via the socket.
pub fn create(name: &str, image: &str, volumes: &[Volume], ssh_agent: bool) -> Result<()> {
    let sock = std::env::var("SSH_AUTH_SOCK").ok();
    let argv = build_run_argv(name, image, volumes, ssh_agent, sock.as_deref());
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

    /// Named volumes render as `volume:container` with no mode
    /// suffix — podman manages the data so ro/rw doesn't enter the
    /// syntax.
    #[test]
    fn format_volume_named() {
        let v = Volume::Named {
            volume: "myapp-home".into(),
            container: "/home/dev".into(),
        };
        assert_eq!(format_volume(&v), "myapp-home:/home/dev");
    }

    /// Bind mounts default to read-write — no `:ro` suffix unless
    /// the caller explicitly asks for it.
    #[test]
    fn format_volume_bind_rw() {
        let v = Volume::Bind {
            host: "/home/august/repo".into(),
            container: "/home/dev/repo".into(),
            readonly: false,
        };
        assert_eq!(format_volume(&v), "/home/august/repo:/home/dev/repo");
    }

    /// Read-only bind mounts gain podman's `:ro` mode suffix. This
    /// is what makes the dev-container.toml `/etc/localtime` and
    /// `/etc/timezone` mounts safe under rootless podman + `--userns
    /// keep-id`, where the container UID maps onto the host invoking
    /// user — without `:ro` a misbehaving container could rewrite
    /// host system files.
    #[test]
    fn format_volume_bind_ro() {
        let v = Volume::Bind {
            host: "/etc/localtime".into(),
            container: "/etc/localtime".into(),
            readonly: true,
        };
        assert_eq!(format_volume(&v), "/etc/localtime:/etc/localtime:ro");
    }

    /// `podman run` argv must always include `--network host` and must
    /// not include any `-p` port mappings. Rootless podman can't put a
    /// container on the LAN as its own host, so the dev container's
    /// only "reachable from elsewhere" story is host networking;
    /// pinning the flag here keeps a future refactor from silently
    /// reverting to port-forward semantics (which would leave a
    /// container's services unreachable until the user enumerated
    /// every port by hand).
    #[test]
    fn run_argv_uses_host_network() {
        let argv = build_run_argv("myapp", "forge-dev", &[], false, None);
        assert!(
            argv.windows(2)
                .any(|w| w[0] == "--network" && w[1] == "host"),
            "argv must contain `--network host`; got {argv:?}"
        );
        assert!(
            !argv.iter().any(|a| a == "-p"),
            "argv must not contain any `-p` port mappings under host networking; got {argv:?}"
        );
    }

    /// `--userns keep-id` survives the host-networking switch: user
    /// namespace and network namespace are independent in podman, and
    /// the container UID-mapping is the reason files written into bind
    /// mounts come back owned by the host user. Pinning here so a
    /// future "simplify the flags" refactor doesn't drop it together
    /// with the port loop.
    #[test]
    fn run_argv_keeps_userns_keep_id() {
        let argv = build_run_argv("myapp", "forge-dev", &[], false, None);
        assert!(
            argv.windows(2)
                .any(|w| w[0] == "--userns" && w[1] == "keep-id"),
            "argv must keep `--userns keep-id`; got {argv:?}"
        );
    }

    /// ssh-agent forwarding rides on `-v` + `-e SSH_AUTH_SOCK=…`, both
    /// of which are independent of network mode. Verify the agent
    /// plumbing is present when a socket is supplied and absent
    /// otherwise.
    #[test]
    fn run_argv_ssh_agent_forwarding() {
        let with = build_run_argv("a", "img", &[], true, Some("/run/user/1000/keyring/ssh"));
        assert!(
            with.iter()
                .any(|a| a.contains("SSH_AUTH_SOCK=/tmp/ssh-agent.sock")),
            "argv must set SSH_AUTH_SOCK env when agent forwarding is on; got {with:?}"
        );
        assert!(
            with.iter()
                .any(|a| a == "/run/user/1000/keyring/ssh:/tmp/ssh-agent.sock"),
            "argv must bind the host's auth socket to /tmp/ssh-agent.sock; got {with:?}"
        );

        let without = build_run_argv("a", "img", &[], false, Some("/whatever"));
        assert!(
            !without.iter().any(|a| a.contains("SSH_AUTH_SOCK")),
            "ssh_agent=false must suppress the SSH_AUTH_SOCK env even when a socket is set; \
             got {without:?}"
        );
    }
}
