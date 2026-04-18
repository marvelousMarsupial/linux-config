//! `forge dev` — rootless podman dev containers.
//!
//! Design goal: one command, fast feedback, identical shell environment
//! to the host. `forge dev up <name>` is the main entry point — it does
//! whatever's needed (create if missing, start if stopped, attach).
//!
//! Images are described in `dev-images.toml` (embedded). Each image lists
//! default volumes and ports, so `forge dev up rust` is enough — you
//! don't have to remember CLI flags.

use anyhow::{anyhow, Result};
use forge_common::{
    dev_config::DevImagesFile,
    paths::expand_tilde,
    podman::{self, Volume},
};
use owo_colors::OwoColorize;

use crate::cli::DevCmd;

/// The embedded dev-images.toml, compiled into the binary.
const DEV_IMAGES_TOML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../config/dev-images.toml"));

pub fn dispatch(sub: DevCmd) -> Result<()> {
    match sub {
        DevCmd::Build { image } => build(&image),
        DevCmd::Up { name } => up(&name),
        DevCmd::Ls => ls(),
        DevCmd::Attach { name } => attach(&name),
        DevCmd::Exec { name, cmd } => exec(&name, &cmd),
        DevCmd::Down { name } => down(&name),
        DevCmd::Rm { name } => rm(&name),
    }
}

fn images() -> Result<DevImagesFile> {
    DevImagesFile::from_toml(DEV_IMAGES_TOML)
}

fn build(image: &str) -> Result<()> {
    let imgs = images()?;
    let img = imgs
        .find(image)
        .ok_or_else(|| anyhow!("unknown image `{image}`"))?;
    cliclack::intro(format!("forge dev build {}", image.bold()))?;
    let spinner = cliclack::spinner();
    spinner.start(format!("podman build -f {}", img.containerfile));
    let status = std::process::Command::new("podman")
        .args(["build", "-f", &img.containerfile, "-t", &img.name, "."])
        .status()?;
    if !status.success() {
        spinner.error("build failed");
        anyhow::bail!("podman build failed");
    }
    spinner.stop(format!("built {}", img.name));
    cliclack::outro("done")?;
    Ok(())
}

/// The one-command creature comfort: create + start + shell.
/// - If the container doesn't exist, create it with the image's defaults.
/// - If it exists but is stopped, start it.
/// - Either way, drop into a zsh inside it.
fn up(name: &str) -> Result<()> {
    let imgs = images()?;
    cliclack::intro(format!("forge dev up {}", name.bold()))?;

    if !podman::exists(name) {
        // Default to the "base" image unless `name` itself matches a
        // known image — this lets `forge dev up rust` do the expected
        // thing without an explicit `--image` flag.
        let img = imgs.find(name).or_else(|| imgs.find("base")).ok_or_else(
            || anyhow!("no dev images defined; add one to dev-images.toml"),
        )?;

        let bare_name = podman::bare_name(name);
        let mut volumes: Vec<Volume> = Vec::new();

        // Named volumes: `{name}` placeholder is replaced with the
        // container's bare name so each container gets its own volume.
        for v in &img.default_volumes.named {
            let vol_name = v.name.replace("{name}", bare_name);
            volumes.push(Volume::Named {
                volume: vol_name,
                container: v.container.clone(),
            });
        }

        // Bind mounts: expand `~` in host paths so podman sees absolutes.
        for v in &img.default_volumes.bind {
            let host = expand_tilde(&v.host)?;
            volumes.push(Volume::Bind {
                host: host.to_string_lossy().into(),
                container: v.container.clone(),
            });
        }

        cliclack::log::info(format!("creating from image {}", img.name))?;
        podman::create(name, &img.name, &volumes, &img.default_ports, img.ssh_agent)?;
    } else if !podman::running(name) {
        podman::start(name)?;
    }

    cliclack::outro(format!("attached to {}", name.green()))?;
    podman::shell(name)
}

fn ls() -> Result<()> {
    let listings = podman::list()?;
    cliclack::intro("dev containers".bold().to_string())?;
    if listings.is_empty() {
        cliclack::log::info("(none)")?;
    }
    for l in listings {
        cliclack::log::info(format!("{} · {} · {}", l.name, l.state, l.image))?;
    }
    cliclack::outro("")?;
    Ok(())
}

fn attach(name: &str) -> Result<()> {
    if !podman::exists(name) {
        anyhow::bail!("container `{name}` does not exist");
    }
    if !podman::running(name) {
        podman::start(name)?;
    }
    podman::shell(name)
}

fn exec(name: &str, cmd: &[String]) -> Result<()> {
    if cmd.is_empty() {
        anyhow::bail!("usage: forge dev exec <name> -- <cmd> [args...]");
    }
    let q = podman::qualified(name);
    let mut args = vec!["exec".to_string(), "-it".to_string(), q];
    args.extend(cmd.iter().cloned());
    let status = std::process::Command::new("podman").args(&args).status()?;
    if !status.success() {
        anyhow::bail!("exec exited with {status}");
    }
    Ok(())
}

fn down(name: &str) -> Result<()> {
    podman::stop(name)?;
    cliclack::log::info(format!("stopped {name}"))?;
    Ok(())
}

fn rm(name: &str) -> Result<()> {
    podman::remove(name, true)?;
    cliclack::log::info(format!("removed {name}"))?;
    Ok(())
}
