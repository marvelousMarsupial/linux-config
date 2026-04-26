//! `forge theme` — list, show, apply, preview color schemes.
//!
//! Everything is embedded: schemes as YAML in the binary, templates as
//! Tera `.j2` files, and the `themes.toml` meta config. `apply` writes
//! rendered configs into `$HOME` via `forge_common::theme::apply`, runs
//! any reload hooks for files that actually changed, and records the
//! current scheme in both `~/.cache/theme/current` (for other tools) and
//! `~/.config/forge/state.json` (for `forge status`).

use anyhow::{anyhow, Result};
use forge_common::{
    atomic::write_atomic,
    home, state_file,
    theme::{apply, ThemesConfig},
    State,
};
use owo_colors::OwoColorize;

use crate::cli::ThemeCmd;
use crate::embed::{load_scheme, scheme_names, EmbeddedTemplates, THEMES_TOML};

pub fn dispatch(sub: ThemeCmd) -> Result<()> {
    match sub {
        ThemeCmd::List => list(),
        ThemeCmd::Current => current(),
        ThemeCmd::Apply { name } => do_apply(&resolve_name(&name)?, false),
        ThemeCmd::Preview { name } => do_apply(&resolve_name(&name)?, true),
    }
}

/// Accept either a scheme name or a 1-based number (matching `forge theme list`).
fn resolve_name(input: &str) -> Result<String> {
    if let Ok(n) = input.parse::<usize>() {
        let names = scheme_names();
        if n == 0 || n > names.len() {
            anyhow::bail!("theme number {n} out of range (1–{})", names.len());
        }
        Ok(names[n - 1].clone())
    } else {
        Ok(input.to_string())
    }
}

fn list() -> Result<()> {
    let current = current_slug().unwrap_or_default();
    let names = scheme_names();
    cliclack::intro("themes".bold().to_string())?;
    for (i, s) in names.iter().enumerate() {
        let marker = if *s == current { "*" } else { " " };
        cliclack::log::info(format!("{}{} {s}", i + 1, marker))?;
    }
    cliclack::outro("")?;
    Ok(())
}

fn current() -> Result<()> {
    match current_slug() {
        Some(s) => println!("{s}"),
        None => println!("(none)"),
    }
    Ok(())
}

fn do_apply(name: &str, preview: bool) -> Result<()> {
    let scheme = load_scheme(name).ok_or_else(|| {
        anyhow!(
            "scheme `{name}` not found; available: {}",
            scheme_names().join(", ")
        )
    })?;
    let cfg = ThemesConfig::from_toml(THEMES_TOML)?;
    let home_dir = home()?;

    let verb = if preview { "preview" } else { "apply" };
    cliclack::intro(format!("forge theme {verb} {}", name.bold()))?;

    let target_home = if preview {
        tempfile::tempdir()?.keep()
    } else {
        home_dir.clone()
    };

    let report = apply(&scheme, &cfg, &EmbeddedTemplates, &target_home)?;

    if preview {
        cliclack::log::info(format!(
            "rendered into {} ({} files)",
            target_home.display(),
            report.changed.len()
        ))?;
        cliclack::outro("preview only — no live changes")?;
        return Ok(());
    }

    for f in &report.changed {
        cliclack::log::info(format!("wrote {}", f.display()))?;
    }

    for hook in &report.hooks {
        cliclack::log::info(format!("hook: {hook}"))?;
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(hook)
            .status();
    }

    for proc in &report.restarts {
        cliclack::log::info(format!("restart: {proc}"))?;
        let _ = std::process::Command::new("killall").arg(proc).status();
    }

    let cache = home_dir.join(".cache/theme/current");
    write_atomic(&cache, name.as_bytes(), None)?;

    let path = state_file()?;
    let mut state = State::load(&path).unwrap_or_default();
    state.theme = Some(name.to_string());
    state.save(&path)?;

    cliclack::outro(format!(
        "{} changed · applied {}",
        report.changed.len().green(),
        name
    ))?;
    Ok(())
}

fn current_slug() -> Option<String> {
    let path = state_file().ok()?;
    let s = State::load(&path).ok()?;
    s.theme
}
