//! `forge vm` — incus VM / system container management.
//!
//! Profiles live in `vm-profiles.toml` (embedded). Each profile names an
//! image to launch, whether it's a container or VM, and a role to
//! provision inside it. `forge vm create <profile>` launches, waits for
//! boot, pushes the forge binary onto the instance, and runs
//! `forge setup <role>` in it.

use anyhow::{anyhow, Result};
use forge_common::{
    dev_config::{VmKind, VmProfilesFile},
    incus::{self, InstanceKind},
};
use owo_colors::OwoColorize;
use std::time::Duration;

use crate::cli::VmCmd;

const VM_PROFILES_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/vm-profiles.toml"
));

pub fn dispatch(sub: VmCmd) -> Result<()> {
    match sub {
        VmCmd::Create { profile } => create(&profile),
        VmCmd::Ls => ls(),
        VmCmd::Start { name } => {
            incus::start(&name)?;
            Ok(())
        }
        VmCmd::Stop { name } => {
            incus::stop(&name)?;
            Ok(())
        }
        VmCmd::Shell { name } => incus::shell(&name),
    }
}

fn profiles() -> Result<VmProfilesFile> {
    VmProfilesFile::from_toml(VM_PROFILES_TOML)
}

fn create(profile_name: &str) -> Result<()> {
    let ps = profiles()?;
    let p = ps
        .find(profile_name)
        .ok_or_else(|| anyhow!("unknown profile `{profile_name}`"))?;

    cliclack::intro(format!("forge vm create {}", profile_name.bold()))?;

    if incus::exists(&p.name) {
        anyhow::bail!("instance `forge-{}` already exists", p.name);
    }

    let kind = match p.kind {
        VmKind::Vm => InstanceKind::Vm,
        VmKind::Container => InstanceKind::Container,
    };

    let spinner = cliclack::spinner();
    spinner.start(format!("launching {}", p.image));
    incus::launch(&p.name, &p.image, kind)?;
    spinner.stop("launched");

    let spinner = cliclack::spinner();
    spinner.start("waiting for network");
    if !incus::wait_ready(&p.name, Duration::from_secs(120)) {
        spinner.error("boot timeout");
        anyhow::bail!("instance did not come up within 120s");
    }
    spinner.stop("ready");

    // Push this binary onto the instance so it can provision itself.
    let self_path = std::env::current_exe()?;
    let spinner = cliclack::spinner();
    spinner.start("pushing forge binary");
    incus::push_file(&p.name, &self_path, "/usr/local/bin/forge")?;
    spinner.stop("pushed");

    // Run `forge setup <role>` inside the instance.
    cliclack::log::info(format!("provisioning role {}", p.role))?;
    let q = incus::qualified(&p.name);
    let status = std::process::Command::new("incus")
        .args([
            "exec",
            &q,
            "--",
            "/usr/local/bin/forge",
            "setup",
            &p.role,
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("provisioning exited with {status}");
    }

    cliclack::outro(format!("{} ready", p.name.green()))?;
    Ok(())
}

fn ls() -> Result<()> {
    let listings = incus::list()?;
    cliclack::intro("vms".bold().to_string())?;
    if listings.is_empty() {
        cliclack::log::info("(none)")?;
    }
    for l in listings {
        cliclack::log::info(format!(
            "{} · {} · {} · {}",
            l.name, l.state, l.ipv4, l.kind
        ))?;
    }
    cliclack::outro("")?;
    Ok(())
}
