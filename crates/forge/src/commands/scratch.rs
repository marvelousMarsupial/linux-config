//! `forge lxc` / `forge vm` — bootstrap a clean Debian trixie incus
//! instance pre-configured for testing `forge setup` from zero.
//!
//! These two commands replace what used to be a 5+ minute manual
//! song-and-dance every time we wanted to spin up a fresh container
//! or VM for testing or general use:
//!
//!   1. `incus launch images:debian/13 NAME` (`--vm` for the VM
//!      variant) — pull the upstream image, start the guest. We
//!      deliberately skip the `incus config device override eth0
//!      ipv4.address=…` + `incus restart` pair an earlier draft
//!      had: the override was meant to register a dnsmasq lease so
//!      the gateway's DNS would accept queries from our static IP,
//!      but we *also* swap the guest's /etc/resolv.conf to public
//!      DNS below — which bypasses the gateway dnsmasq entirely —
//!      so the override gave us nothing the in-guest DNS swap
//!      doesn't already cover. Crucially, dropping the override
//!      let us drop the `incus restart` too: VM restart on this
//!      host hangs for 6+ minutes (ACPI shutdown timing in the
//!      upstream image), which made `forge vm` effectively unusable
//!      in the previous shape.
//!   2. Wait for the guest agent socket AND systemd's dbus to be
//!      ready. The two come up at different times — the agent
//!      socket is online almost immediately on LXCs and shortly
//!      after kernel boot on VMs, but PID 1's dbus is what
//!      `systemctl restart` needs and that lags by another few
//!      seconds for LXCs / cloud-init's lifetime for VMs.
//!   3. Force a static IPv4 inside the guest. The upstream
//!      `images:debian/13` image's `systemd-networkd` config
//!      doesn't actually fire DHCPDISCOVER — the link comes up via
//!      IPv6 SLAAC, networkd marks it `routable (configured)`, and
//!      walks away. Working around the upstream quirk inside the
//!      guest is cheaper than chasing it. The interface name
//!      differs between LXC (`eth0`) and VM (`enp5s0` — kernel PCI
//!      naming for the virtio-net device) so we probe at runtime
//!      rather than hardcoding.
//!   4. Replace `/etc/resolv.conf`'s symlink-to-resolved-stub with
//!      a real file pointing at public DNS (1.1.1.1, 9.9.9.9).
//!      Chosen over the gateway dnsmasq because incus's bridge
//!      nftables policy chain drops UDP/53 from unleased source
//!      IPs (and we're not registering a lease — see step 1).
//!   5. apt-get install sudo + ca-certificates (the bare
//!      `trixie-slim` image lacks both); useradd `august` (UID
//!      1000, matches forge's `/home/august` baked-in path
//!      expectations); NOPASSWD sudoers drop-in (so forge's
//!      `sudo chsh` / `sudo flatpak …` post-cmds run
//!      non-interactively the same way they will on the user's real
//!      host); `loginctl enable-linger august` so
//!      `systemctl --user` works without a graphical login session.
//!   6. `incus file push <forge-binary> → /home/august/forge` with
//!      the +x bit set, and chown to august.
//!   7. By default, drop the user into a shell as august with
//!      `./forge` ready to run. `--no-shell` skips the handoff.
//!
//! The forge binary pushed in is the **currently-running** one
//! (resolved via `current_exe()`), with a preference for
//! `~/linux-config/target/release/forge` if it exists — that way a
//! `cargo build --release -p forge` followed by `forge lxc` always
//! tests the just-built version, even if `which forge` resolves to
//! a stale `~/.cargo/bin/forge`. (See the project's CLAUDE.md notes
//! on the binary-sync gotcha.)
//!
//! After the test, `incus delete --force <name>`.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Whether we're bootstrapping a system container (LXC) or a full
/// virtual machine (KVM). Drives a small handful of differences:
/// `--vm` flag on `incus launch`, a longer agent-readiness timeout
/// for VMs (kernel boot + cloud-init), and the default IP / name.
#[derive(Clone, Copy, Debug)]
enum Kind {
    Lxc,
    Vm,
}

impl Kind {
    fn default_name(self) -> &'static str {
        match self {
            Kind::Lxc => "forge-scratch",
            Kind::Vm => "forge-scratch-vm",
        }
    }

    fn default_ip(self) -> &'static str {
        match self {
            // `.50` and `.51` chosen to avoid the typical
            // hand-launched test boxes (.10 forge-trixie LXC,
            // .30 forge-vm) and the gateway (.1) / DHCP pool start
            // (.2). If you keep more than one scratch instance
            // alive at a time, pass --ip to disambiguate.
            Kind::Lxc => "10.160.110.50",
            Kind::Vm => "10.160.110.51",
        }
    }

    /// How long to wait for `incus exec` to start working after
    /// `incus launch`. LXCs come up almost immediately (the host
    /// kernel is already running); VMs need to boot a kernel, run
    /// cloud-init, and start the lxd-agent — so we give them more
    /// budget. The check itself is cheap (`incus exec NAME -- echo`
    /// over the local guest-agent socket), so polling is fine.
    fn agent_timeout(self) -> Duration {
        match self {
            Kind::Lxc => Duration::from_secs(30),
            Kind::Vm => Duration::from_secs(120),
        }
    }

    /// Extra arguments appended to `incus launch …` for this kind.
    ///
    /// LXCs use the upstream defaults (no extra args). VMs get
    /// `--vm` plus resource overrides:
    ///   * `limits.cpu=2` — incus defaults to 1 vCPU, which is
    ///     too lean for the apt-resolve / package-install workload
    ///     forge runs inside the guest.
    ///   * `limits.memory=4GiB` — 1 GiB default OOMs during the
    ///     flatpak runtime pull (~1 GiB resident at peak for the
    ///     concurrent ostree fetches).
    ///   * `root,size=20GiB` — overrides the default ~10 GiB root
    ///     disk so binary downloads (nvim, starship, yazi, whisper
    ///     model ~150 MiB) + apt cache + flatpak app installs fit
    ///     comfortably under the 90%-full mark where ext4 perf
    ///     collapses.
    ///
    /// Pinned together by `vm_launch_args_set_resource_defaults`
    /// so a future "let me trim these" pass that drops one is
    /// caught at `cargo test`, not by the user discovering
    /// `flatpak install` got OOM-killed mid-bootstrap.
    fn launch_extra_args(self) -> &'static [&'static str] {
        match self {
            Kind::Lxc => &[],
            Kind::Vm => &[
                "--vm",
                "-c",
                "limits.cpu=2",
                "-c",
                "limits.memory=4GiB",
                "--device",
                "root,size=20GiB",
            ],
        }
    }
}

/// `forge lxc` entry point.
pub fn lxc(name: Option<String>, no_shell: bool, ip: Option<String>) -> Result<()> {
    bootstrap(Kind::Lxc, name, no_shell, ip)
}

/// `forge vm` entry point.
pub fn vm(name: Option<String>, no_shell: bool, ip: Option<String>) -> Result<()> {
    bootstrap(Kind::Vm, name, no_shell, ip)
}

/// Shared bootstrap path for both LXC and VM.
fn bootstrap(
    kind: Kind,
    name_arg: Option<String>,
    no_shell: bool,
    ip_arg: Option<String>,
) -> Result<()> {
    let name = name_arg.unwrap_or_else(|| kind.default_name().to_string());
    let ip = ip_arg.unwrap_or_else(|| kind.default_ip().to_string());

    preflight(&name)?;
    let binary = resolve_forge_binary()?;

    println!(
        "==> launching {} '{}' from images:debian/13 (IP {})",
        match kind {
            Kind::Lxc => "LXC",
            Kind::Vm => "VM",
        },
        name,
        ip
    );

    // Step 1: incus launch.
    //
    // We deliberately do NOT follow this with
    // `incus config device override eth0 ipv4.address=…` and
    // `incus restart`. An earlier draft did, on the theory that
    // registering the static IP with incusbr0's dnsmasq was needed
    // so the gateway DNS would accept queries from the leased
    // address. But we *also* write 1.1.1.1 / 9.9.9.9 to the
    // guest's /etc/resolv.conf below, which bypasses the gateway
    // dnsmasq entirely — so the override gave us nothing the in-
    // guest DNS swap doesn't already cover.
    //
    // Removing the override let us drop the `incus restart` step
    // too. That's load-bearing: VM restart on this host hangs for
    // 6+ minutes per test run (ACPI shutdown timing in the
    // upstream image), which made `forge vm` effectively unusable.
    // No restart, no hang. The static IP we write to the guest's
    // networkd config below works for outbound routing on its own
    // — the bridge is happy to forward packets from arbitrary
    // source IPs in its subnet, only the dnsmasq DNS port (UDP/53
    // back to the gateway) was filtered for unleased addresses.
    let mut launch = Command::new("incus");
    launch.args(["launch", "images:debian/13", &name, "--quiet"]);
    // For VMs this appends `--vm` plus resource defaults
    // (`limits.cpu=2`, `limits.memory=4GiB`, `root,size=20GiB`).
    // For LXCs it's a no-op — LXCs share the host kernel + memory
    // pool so per-instance CPU / memory / disk limits would be
    // surprising knobs to flip without the user asking. See
    // `Kind::launch_extra_args` for the full rationale per knob.
    launch.args(kind.launch_extra_args());
    run(&mut launch).context("launching the guest")?;

    // Wait for the guest agent + systemd dbus to be ready.
    wait_for_agent(&name, kind.agent_timeout())?;

    // Step 3: figure out the in-guest interface name (eth0 for LXC,
    // enp5s0 for VM) by listing /sys/class/net inside the guest.
    let iface = probe_interface(&name)?;
    println!("==> applying static network config (interface {iface})");
    let net_script = format!(
        r#"
set -e
rm -f /etc/systemd/network/eth0.network /etc/systemd/network/enp5s0.network
cat > /etc/systemd/network/00-static.network <<EOF
[Match]
Name={iface}

[Network]
Address={ip}/24
Gateway=10.160.110.1
IPv6AcceptRA=yes
EOF
systemctl restart systemd-networkd
sleep 2
"#
    );
    run(Command::new("incus").args(["exec", &name, "--", "bash", "-c", &net_script]))
        .context("writing in-guest networkd config")?;

    // Step 4: replace /etc/resolv.conf with public DNS. We stop
    // resolved first so it doesn't try to rewrite the file mid-write
    // and so it stops shadowing queries via 127.0.0.53.
    println!("==> writing /etc/resolv.conf with public DNS (1.1.1.1, 9.9.9.9)");
    let resolv_script = r#"
set -e
systemctl stop systemd-resolved 2>/dev/null || true
rm -f /etc/resolv.conf
cat > /etc/resolv.conf <<EOF
nameserver 1.1.1.1
nameserver 9.9.9.9
EOF
"#;
    run(Command::new("incus").args(["exec", &name, "--", "bash", "-c", resolv_script]))
        .context("writing in-guest /etc/resolv.conf")?;

    // Step 5: apt + useradd + sudoers + linger. Single bash -c so
    // a failure in any step aborts the whole bootstrap (set -e).
    println!("==> bootstrapping august (UID 1000, NOPASSWD sudo, linger)");
    let user_script = r#"
set -e
apt-get update -qq 2>&1 | tail -1
apt-get install -y -qq sudo ca-certificates 2>&1 | tail -1
useradd -m -s /bin/bash -u 1000 august
echo "august ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/august
chmod 440 /etc/sudoers.d/august
loginctl enable-linger august
"#;
    run(Command::new("incus").args(["exec", &name, "--", "bash", "-c", user_script]))
        .context("bootstrapping in-guest user `august`")?;

    // Step 6: push forge binary.
    println!("==> pushing {} → /home/august/forge", binary.display());
    run(Command::new("incus").args([
        "file",
        "push",
        &binary.to_string_lossy(),
        &format!("{name}/home/august/forge"),
        "--mode=755",
        "--quiet",
    ]))
    .context("pushing forge binary into the guest")?;
    run(Command::new("incus").args([
        "exec",
        &name,
        "--",
        "chown",
        "august:august",
        "/home/august/forge",
    ]))
    .context("chowning forge binary to august")?;

    // Step 7: handoff.
    if no_shell {
        println!();
        println!("==> ready. enter the guest manually with:");
        println!(
            "      incus exec {name} --user 1000 --group 1000 --cwd /home/august \\\n         \
             --env HOME=/home/august --env USER=august \\\n         \
             --env XDG_RUNTIME_DIR=/run/user/1000 \\\n         \
             -- bash"
        );
        println!();
        println!("    then run forge:    ./forge setup desktop");
        println!("    cleanup when done: incus delete --force {name}");
        return Ok(());
    }

    println!();
    println!("==> dropping into a shell as august. you're inside the guest now.");
    println!("    run forge:        ./forge setup desktop");
    println!("    leave shell:      exit");
    println!("    delete the guest: incus delete --force {name}     (run from host)");
    println!();

    // Replace this process with `incus exec ... -- bash` so the
    // user gets a clean interactive session. `exec` semantics are
    // important here — without it the parent stays waiting for the
    // child and ^C / ^D handling gets weird.
    use std::os::unix::process::CommandExt;
    let err = Command::new("incus")
        .args([
            "exec",
            &name,
            "--user",
            "1000",
            "--group",
            "1000",
            "--cwd",
            "/home/august",
            "--env",
            "HOME=/home/august",
            "--env",
            "USER=august",
            "--env",
            "XDG_RUNTIME_DIR=/run/user/1000",
            "--",
            "bash",
        ])
        .exec();
    // `exec()` only returns on failure (otherwise this process is
    // gone). Surface the error so the user knows what blew up.
    Err(anyhow!("failed to exec incus shell handoff: {err}"))
}

/// Pre-conditions: incus is on PATH and the named instance doesn't
/// already exist. Each failure carries a specific remediation hint.
fn preflight(name: &str) -> Result<()> {
    if which("incus").is_none() {
        anyhow::bail!(
            "incus is not on PATH. Install the virt stack first:\n  \
             forge setup server"
        );
    }

    let list_out = Command::new("incus")
        .args(["list", "--format", "csv"])
        .output()
        .context("listing existing incus instances")?;
    let stdout = String::from_utf8_lossy(&list_out.stdout);
    for line in stdout.lines() {
        if let Some(first) = line.split(',').next() {
            if first == name {
                anyhow::bail!(
                    "incus instance '{name}' already exists. To recreate from scratch:\n  \
                     incus delete --force {name}"
                );
            }
        }
    }

    Ok(())
}

/// Find the forge binary to push into the guest.
///
/// Preference order:
///   1. `~/linux-config/target/release/forge` if it exists. This is
///      the working-copy build — running `cargo build --release -p
///      forge` and then `forge lxc` should always test the
///      just-built code, even if `which forge` resolves to a stale
///      `~/.cargo/bin/forge` (the gotcha called out in CLAUDE.md's
///      "forge binary in sync" note).
///   2. The currently-running binary (`current_exe()`). Predictable
///      fallback when the working copy isn't where it's expected.
fn resolve_forge_binary() -> Result<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        let release = PathBuf::from(home).join("linux-config/target/release/forge");
        if release.is_file() {
            return Ok(release);
        }
    }
    std::env::current_exe().context("resolving the running forge binary path")
}

/// Wait for the guest to be ready for the rest of the bootstrap.
///
/// Two-stage check, both required:
///
///   1. `incus exec NAME -- echo` succeeds. This proves the lxd-agent
///      socket is up and exec'ing into the guest works.
///   2. `incus exec NAME -- systemctl is-system-running` runs without
///      "Failed to connect to system bus" on stderr. The agent socket
///      comes up earlier than systemd's dbus, so without this second
///      gate we race and hit
///      `Failed to connect to system scope bus via local transport`
///      on the first `systemctl restart systemd-networkd` call.
///      `is-system-running` is the right probe because it's
///      passive (no side effects) but uses the same dbus path the
///      restart will use; we don't care about its exit code (it
///      returns non-zero for "starting" or "degraded" states), only
///      that dbus responded at all.
fn wait_for_agent(name: &str, deadline: Duration) -> Result<()> {
    use std::io::Write;
    print!("==> waiting for guest agent + systemd dbus");
    std::io::stdout().flush().ok();
    let started = std::time::Instant::now();
    let mut last_progress = started;
    loop {
        // Stage 1: agent socket alive.
        let agent_ok = Command::new("incus")
            .args(["exec", name, "--", "echo", "agent-up"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if agent_ok {
            // Stage 2: systemd's dbus reachable. We look at stderr
            // for the specific "Failed to connect ... bus" pattern
            // emitted by systemctl when PID 1's bus isn't yet
            // listening — exit code alone is ambiguous because
            // is-system-running returns 1 for "degraded" / 4 for
            // "maintenance" even when dbus is fine.
            let probe = Command::new("incus")
                .args(["exec", name, "--", "systemctl", "is-system-running"])
                .output();
            if let Ok(o) = probe {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.contains("Failed to connect") {
                    println!(" ready ({}s)", started.elapsed().as_secs());
                    return Ok(());
                }
            }
        }

        if started.elapsed() > deadline {
            println!(" timeout");
            anyhow::bail!(
                "guest agent / systemd for '{name}' didn't come up within {}s — try \
                 `incus list {name}` to inspect state",
                deadline.as_secs()
            );
        }

        // Visible progress so the user can tell the wait is alive.
        // A dot every 2s of polling is the right cadence: frequent
        // enough that you don't doubt the process is running,
        // sparse enough that a 90s VM boot doesn't paint a wall of
        // text. We emit the dot from the wait loop (not the inner
        // poll) so the output reflects elapsed wall time rather
        // than per-attempt count.
        if last_progress.elapsed() >= Duration::from_secs(2) {
            print!(".");
            std::io::stdout().flush().ok();
            last_progress = std::time::Instant::now();
        }
        thread::sleep(Duration::from_secs(1));
    }
}

/// Probe the in-guest network interface name. LXCs use `eth0`
/// (incus's profile name passes straight through), VMs use
/// `enp5s0` or similar (the kernel renames virtio-net based on
/// PCI topology). We list /sys/class/net and pick the first
/// non-loopback entry — works for both, no per-kind branching
/// needed.
fn probe_interface(name: &str) -> Result<String> {
    let out = Command::new("incus")
        .args([
            "exec",
            name,
            "--",
            "bash",
            "-c",
            "ls /sys/class/net | grep -v '^lo$' | head -1",
        ])
        .output()
        .context("probing in-guest network interface name")?;
    if !out.status.success() {
        anyhow::bail!(
            "failed to probe in-guest interface name (exit {}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let iface = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if iface.is_empty() {
        anyhow::bail!("in-guest /sys/class/net is empty (no non-loopback interfaces)");
    }
    Ok(iface)
}

/// Run a command, propagating its exit status and stderr into an
/// anyhow error if it fails. We let stdin/stdout/stderr inherit so
/// progress output (apt download lines, incus image pull progress
/// bars) reaches the user in real time.
fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| {
        let exe = cmd.get_program().to_string_lossy().to_string();
        format!("spawning {exe}")
    })?;
    if !status.success() {
        anyhow::bail!(
            "{} failed with status {}",
            cmd.get_program().to_string_lossy(),
            status
        );
    }
    Ok(())
}

/// Minimal `which` — returns the first matching executable on PATH.
/// Avoids pulling in the `which` crate just for one preflight check.
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata()
        .map(|m| m.is_file() && (m.permissions().mode() & 0o111) != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Kind::default_name` and `Kind::default_ip` must be
    /// distinct between LXC and VM so the two scratch commands
    /// don't collide on either name or IP if the user runs both
    /// without arguments. A regression that returns the same
    /// default for both would cause the second invocation to fail
    /// preflight with "instance already exists" or to step on the
    /// first guest's network lease.
    #[test]
    fn default_name_and_ip_differ_between_lxc_and_vm() {
        assert_ne!(Kind::Lxc.default_name(), Kind::Vm.default_name());
        assert_ne!(Kind::Lxc.default_ip(), Kind::Vm.default_ip());
    }

    /// VM agent timeout must be strictly greater than LXC's. VMs
    /// boot a kernel + cloud-init before `lxd-agent` is reachable;
    /// LXCs are nearly instant. A test pinning the inequality
    /// catches a future "let me unify these" refactor that would
    /// either time out VMs (if it picked the LXC budget) or waste
    /// 90+ seconds on every LXC bootstrap (if it picked VM's).
    #[test]
    fn vm_agent_timeout_exceeds_lxc() {
        assert!(Kind::Vm.agent_timeout() > Kind::Lxc.agent_timeout());
    }

    /// VM launch arg list must include `--vm` AND pin all three
    /// resource overrides: `limits.cpu=2`, `limits.memory=4GiB`,
    /// `root,size=20GiB`. Each is asserted individually so a
    /// regression that drops one is named in the failure message
    /// rather than collapsed into a vague "resource args wrong".
    ///
    /// LXC's launch arg list must be empty — LXCs share the host
    /// kernel and memory pool, so per-instance CPU/memory/disk
    /// limits would be surprising knobs to set per-launch on a
    /// system container.
    #[test]
    fn vm_launch_args_set_resource_defaults() {
        let vm_args = Kind::Vm.launch_extra_args();
        assert!(
            vm_args.contains(&"--vm"),
            "VM launch must pass `--vm` to incus launch; got {vm_args:?}"
        );
        for required in [
            "limits.cpu=2",
            "limits.memory=4GiB",
            "root,size=20GiB",
        ] {
            assert!(
                vm_args.contains(&required),
                "VM launch must pin `{required}` — incus's default is too lean for the \
                 forge-setup workload (apt + flatpak + binary downloads). Got {vm_args:?}"
            );
        }
        assert!(
            Kind::Lxc.launch_extra_args().is_empty(),
            "LXC must not pin per-instance resource limits — LXCs share the host kernel \
             and memory pool, so resource overrides are a surprising default. Got {:?}",
            Kind::Lxc.launch_extra_args()
        );
    }

    /// `which` returns `Some` for ubiquitous commands like `sh`
    /// (POSIX-mandated, present on every Linux box) and `None` for
    /// gibberish. Sanity check on the PATH search path so a future
    /// refactor that breaks the iteration silently isn't a runtime
    /// surprise.
    #[test]
    fn which_finds_sh_and_misses_gibberish() {
        assert!(which("sh").is_some(), "sh must be on PATH");
        assert!(
            which("definitely-not-a-real-binary-xyz123").is_none(),
            "fake binary name must not resolve"
        );
    }
}
