//! Remote provisioning: push the forge binary into a container or VM and
//! run `forge setup <role>` inside it.
//!
//! This is the glue behind `--target=container:<name>` and `--target=vm:<name>`.
//! The workflow is: resolve the running binary's path, copy it into the
//! target, then exec `forge setup <role>` inside. The target gets a
//! fully self-contained binary with embedded dotfiles, so no repo clone
//! or network access is needed inside the instance.
//!
//! Container targets use `podman cp` + `podman exec`. VM targets use
//! `incus file push` + `incus exec`. Both start the instance if it's
//! stopped so the user doesn't have to think about state.

use anyhow::{Context, Result};
use forge_common::{incus, podman, Role};
use owo_colors::OwoColorize;

const REMOTE_BIN: &str = "/usr/local/bin/forge";

/// Push forge into a podman container and run `forge setup <role>`.
///
/// Starts the container if stopped. Fails if the container doesn't exist
/// — use `forge dev up` to create it first.
pub fn setup_in_container(name: &str, role: Role) -> Result<()> {
    if !podman::exists(name) {
        anyhow::bail!("container `{name}` does not exist — create it with `forge dev up {name}`");
    }
    if !podman::running(name) {
        podman::start(name)?;
    }

    let q = podman::qualified(name);
    let self_path = std::env::current_exe().context("resolving own binary path")?;

    cliclack::intro(format!("forge setup {} → container {}", role.bold(), name.bold()))?;

    let spinner = cliclack::spinner();
    spinner.start("copying forge binary");
    let src = self_path.to_string_lossy();
    forge_common::checked(&["podman", "cp", &src, &format!("{q}:{REMOTE_BIN}")])
        .context("podman cp")?;
    spinner.stop("copied");

    cliclack::log::info(format!("running forge setup {role}"))?;
    let status = std::process::Command::new("podman")
        .args(["exec", "-it", &q, REMOTE_BIN, "setup", role.as_str()])
        .status()
        .context("podman exec")?;
    if !status.success() {
        anyhow::bail!("provisioning exited with {status}");
    }

    cliclack::outro(format!("{name} provisioned with {}", role.green()))?;
    Ok(())
}

/// Push forge into an incus VM/container and run `forge setup <role>`.
///
/// Starts the instance if stopped. Fails if it doesn't exist — use
/// `forge vm create` to launch it first.
pub fn setup_in_vm(name: &str, role: Role) -> Result<()> {
    if !incus::exists(name) {
        anyhow::bail!("instance `{name}` does not exist — create it with `forge vm create`");
    }

    let q = incus::qualified(name);
    let self_path = std::env::current_exe().context("resolving own binary path")?;

    cliclack::intro(format!("forge setup {} → vm {}", role.bold(), name.bold()))?;

    let spinner = cliclack::spinner();
    spinner.start("pushing forge binary");
    incus::push_file(name, &self_path, REMOTE_BIN)?;
    spinner.stop("pushed");

    cliclack::log::info(format!("running forge setup {role}"))?;
    let status = std::process::Command::new("incus")
        .args(["exec", &q, "--", REMOTE_BIN, "setup", role.as_str()])
        .status()
        .context("incus exec")?;
    if !status.success() {
        anyhow::bail!("provisioning exited with {status}");
    }

    cliclack::outro(format!("{name} provisioned with {}", role.green()))?;
    Ok(())
}
