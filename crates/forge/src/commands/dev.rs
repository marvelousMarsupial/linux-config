//! `forge dev` — rootless podman dev containers, one per project.
//!
//! There is exactly one dev image (defined in the embedded
//! `dev-container.toml`, built from the embedded
//! `containers/base.Containerfile`). Every container created by
//! `forge dev up <name>` is just a long-lived shell environment for
//! one project: same shell, same editor, same AI tooling baked into
//! the image; per-project toolchains layered on top and stored in the
//! named volume so they survive container recreation.
//!
//! Design goal: continuity. `forge dev up myapp` is the main entry —
//! creates the container if missing, starts it if stopped, attaches a
//! zsh, and you pick up where you left off.
//!
//! # Build context portability
//!
//! `forge dev build` materialises the Containerfile and the dotfiles
//! tree into a tempdir at runtime (both are embedded in the binary at
//! compile time) and runs `podman build` against that tempdir. That
//! means the binary alone is enough to build the dev image — you can
//! `scp forge user@box:` to a totally fresh machine and `forge dev
//! build` works from any cwd, no repo clone required.

use anyhow::{anyhow, Context, Result};
use forge_common::{
    dev_config::DevContainer,
    paths::expand_tilde,
    planner::DotfileSource,
    podman::{self, Volume},
};
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use crate::cli::DevCmd;
use crate::embed::{EmbeddedDotfiles, BASE_CONTAINERFILE};

/// The embedded `dev-container.toml`, compiled into the binary.
const DEV_CONTAINER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/dev-container.toml"
));

pub fn dispatch(sub: DevCmd) -> Result<()> {
    // Single up-front check: every `forge dev` subcommand shells out to
    // `podman`. If it isn't installed, the underlying `Command::status()`
    // surfaces a bare `No such file or directory (os error 2)` that
    // gives the user no useful hint — replace it with one that names
    // the missing dependency and points at the install command.
    if !podman::installed() {
        anyhow::bail!(
            "podman is not on PATH. Install it first:\n  \
             forge setup dev\n  \
             # or, manually:\n  \
             sudo apt install podman uidmap slirp4netns fuse-overlayfs"
        );
    }

    match sub {
        DevCmd::Build => build(),
        DevCmd::Up { name } => up(&name),
        DevCmd::Ls => ls(),
        DevCmd::Attach { name } => attach(&name),
        DevCmd::Exec { name, cmd } => exec(&name, &cmd),
        DevCmd::Down { name } => down(&name),
        DevCmd::Rm { name } => rm(&name),
    }
}

/// Parse the embedded config. Done at the top of every command rather
/// than at startup because the toml is tiny and parsing is essentially
/// free, and it keeps the failure mode (a malformed embedded toml)
/// localised to the command that actually needs it.
fn container_config() -> Result<DevContainer> {
    DevContainer::from_toml(DEV_CONTAINER_TOML)
}

/// Build the dev image from the embedded Containerfile + dotfiles.
///
/// The flow:
///   1. Create a tempdir (auto-cleaned on `Drop`).
///   2. Write the embedded Containerfile to `<tempdir>/Containerfile`.
///   3. Walk the embedded dotfiles tree and dump every file to
///      `<tempdir>/dotfiles/<rel>`. The Containerfile's
///      `COPY --chown=dev:dev dotfiles/...` directives resolve
///      against this, since the build context we hand podman is the
///      tempdir itself.
///   4. Invoke `podman build -f <tempdir>/Containerfile -t <tag> <tempdir>`.
///   5. The tempdir is dropped (and removed) at the end of the scope.
///
/// This is what makes `forge dev build` work from any cwd: the binary
/// alone carries everything podman needs as build inputs.
fn build() -> Result<()> {
    let cfg = container_config()?;
    cliclack::intro(format!("forge dev build {}", cfg.image_tag.bold()))?;

    let tempdir = TempDir::new().context("creating tempdir for build context")?;
    let context = tempdir.path();

    let prep = cliclack::spinner();
    prep.start("preparing build context");
    materialize_build_context(context)?;
    prep.stop("build context ready");

    let containerfile_path = context.join("Containerfile");
    let spinner = cliclack::spinner();
    spinner.start(format!("podman build -t {}", cfg.image_tag));
    let status = std::process::Command::new("podman")
        .args([
            "build",
            "-f",
            &containerfile_path.to_string_lossy(),
            "-t",
            &cfg.image_tag,
            &context.to_string_lossy(),
        ])
        .status()
        .context("invoking `podman build`")?;
    if !status.success() {
        spinner.error("build failed");
        anyhow::bail!("podman build failed");
    }
    spinner.stop(format!("built {}", cfg.image_tag));
    cliclack::outro("done")?;
    Ok(())
}

/// Dump the embedded Containerfile + dotfiles into `target` so podman
/// can use it as a build context.
///
/// Layout produced:
/// ```text
/// <target>/
///   Containerfile
///   dotfiles/
///     .zshrc
///     .config/nvim/...
///     .local/bin/...
///     ...
/// ```
///
/// The dotfiles tree is dumped in full rather than filtered to only
/// what the Containerfile `COPY`s — the embedded tree is small (~few
/// MiB of text), the cost of an over-broad context is negligible
/// compared to the apt + curl steps that dominate image build time,
/// and a future Containerfile addition won't silently fail because we
/// forgot to whitelist a path here.
fn materialize_build_context(target: &Path) -> Result<()> {
    fs::write(target.join("Containerfile"), BASE_CONTAINERFILE)
        .context("writing Containerfile to build context")?;

    let dotfiles_root = target.join("dotfiles");
    let src = EmbeddedDotfiles;
    for rel in src.list() {
        let dst = dotfiles_root.join(&rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating dir {}", parent.display()))?;
        }
        let bytes = src
            .read(&rel)
            .ok_or_else(|| anyhow!("embedded dotfile vanished mid-walk: {}", rel.display()))?;
        fs::write(&dst, bytes)
            .with_context(|| format!("writing {} to build context", dst.display()))?;
    }
    Ok(())
}

/// The one-command creature comfort: create + start + shell.
/// - If the container doesn't exist, create it from the single dev image
///   with the embedded volume / port / ssh-agent defaults.
/// - If it exists but is stopped, start it.
/// - Either way, drop into a zsh inside it via `podman exec -it`.
fn up(name: &str) -> Result<()> {
    let cfg = container_config()?;
    cliclack::intro(format!("forge dev up {}", name.bold()))?;

    if !podman::exists(name) {
        // Bail with an actionable message before `podman run` would
        // otherwise fail with podman's cryptic "short-name … did not
        // resolve to an alias and no unqualified-search registries
        // are defined" error. That message is what podman emits when
        // the local image is missing and it falls through to a
        // registry lookup — it never mentions `forge dev build`,
        // which is what the user actually needs to run.
        if !podman::image_exists(&cfg.image_tag) {
            anyhow::bail!(
                "image `{}` is not built yet. From the workspace root:\n  \
                 forge dev build\n  \
                 # ~5–10 min the first time (downloads neovim, yazi, claude-code, \
                 opencode, uv, fnm, plus the Debian base layer)",
                cfg.image_tag
            );
        }

        let bare_name = podman::bare_name(name);
        let mut volumes: Vec<Volume> = Vec::new();

        // Named volumes: `{name}` placeholder is replaced with the
        // container's bare name so each project gets its own volume.
        for v in &cfg.default_volumes.named {
            let vol_name = v.name.replace("{name}", bare_name);
            volumes.push(Volume::Named {
                volume: vol_name,
                container: v.container.clone(),
            });
        }

        // Bind mounts: expand `~` in host paths so podman sees absolutes.
        // Empty by default — the dev container is intentionally
        // self-contained, with all state in the named volume.
        for v in &cfg.default_volumes.bind {
            let host = expand_tilde(&v.host)?;
            volumes.push(Volume::Bind {
                host: host.to_string_lossy().into(),
                container: v.container.clone(),
            });
        }

        cliclack::log::info(format!("creating from image {}", cfg.image_tag))?;
        podman::create(
            name,
            &cfg.image_tag,
            &volumes,
            &cfg.default_ports,
            cfg.ssh_agent,
        )?;
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
