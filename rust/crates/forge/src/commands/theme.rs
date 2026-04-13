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
        ThemeCmd::Apply { name } => do_apply(&name, false),
        ThemeCmd::Preview { name } => do_apply(&name, true),
    }
}

fn list() -> Result<()> {
    let current = current_slug().unwrap_or_default();
    cliclack::intro("themes".bold().to_string())?;
    for s in scheme_names() {
        let marker = if s == current { "*" } else { " " };
        cliclack::log::info(format!("{marker} {s}"))?;
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

    // In preview mode, write to a temp dir so the user can see what
    // would change without touching live files. For apply, write to $HOME.
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

    // Run reload hooks only for files that actually changed. If nothing
    // changed, reruns are free — a huge UX win for "theme apply X" called
    // from scripts.
    for hook in &report.hooks {
        cliclack::log::info(format!("hook: {hook}"))?;
        let _ = std::process::Command::new("sh").arg("-c").arg(hook).status();
    }

    // Record in legacy path `~/.cache/theme/current` for tools that read it.
    let cache = home_dir.join(".cache/theme/current");
    write_atomic(&cache, name.as_bytes(), None)?;

    // Record in forge state for `forge status`.
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
