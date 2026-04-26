//! `forge setup <role>` — the core provisioning command.
//!
//! This command turns a role name into a plan, optionally previews it, and
//! then executes it. All the real work lives in `forge_common`: this module
//! just wires things together and handles user-facing output.

use anyhow::Result;
use forge_common::{
    build_plan, build_teardown_plan, diff::unified_diff, execute, execute_with, home, state_file,
    Action, ActionResult, ExecOpts, Role, RolesFile, State,
};
use owo_colors::OwoColorize;

use crate::embed::{EmbeddedDotfiles, ROLES_TOML};

pub fn run(role: Role, dry_run: bool, show_diff: bool, refresh_downloads: bool) -> Result<()> {
    let roles = RolesFile::from_toml(ROLES_TOML)?;
    let cfg = roles.resolved(role)?;
    let plan = build_plan(&cfg, &EmbeddedDotfiles)?;
    let home_dir = home()?;

    cliclack::intro(format!("forge setup {}", role.bold()))?;

    let s = plan.summary();
    cliclack::log::info(format!(
        "plan: {} packages · {} files · {} commands",
        s.packages, s.files, s.commands
    ))?;

    if show_diff {
        print_diffs(&plan, &home_dir)?;
    }

    if dry_run {
        for a in &plan.actions {
            cliclack::log::info(describe(a))?;
        }
        cliclack::outro("dry run — no changes made".dimmed().to_string())?;
        return Ok(());
    }

    let opts = ExecOpts { refresh_downloads };
    let results = execute_with(&plan, &home_dir, &opts)?;

    // Tally per-kind for a terse final line.
    let mut changed = 0usize;
    let mut already = 0usize;
    for r in &results {
        match r {
            ActionResult::Changed => changed += 1,
            ActionResult::AlreadyDone => already += 1,
        }
    }

    // Record what we just applied so `forge status` knows and `forge
    // teardown` has something to reverse. The state file skip-check in
    // `write_atomic` means an unchanged state save is also a no-op.
    let new_state = State {
        role: Some(role),
        theme: State::load(&state_file()?).ok().and_then(|s| s.theme),
        applied_at: Some(chrono::Utc::now().to_rfc3339()),
        binary_version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };
    new_state.save(&state_file()?)?;

    cliclack::outro(format!(
        "{} changed · {} already up to date",
        changed.green(),
        already.dimmed()
    ))?;
    Ok(())
}

/// Show what the plan will do: packages, downloads, commands as a summary,
/// then a unified diff for each file against the current destination.
fn print_diffs(plan: &forge_common::Plan, home_dir: &std::path::Path) -> Result<()> {
    // Non-file actions first so the user sees the full picture.
    for a in &plan.actions {
        match a {
            Action::InstallPackage(p) => cliclack::log::info(format!("+ package {p}"))?,
            Action::Download { url, dest, .. } => {
                cliclack::log::info(format!("↓ {url} → {}", dest.display()))?
            }
            Action::PostCommand(c) => cliclack::log::info(format!("$ {}", c.run()))?,
            _ => {}
        }
    }

    // File diffs.
    for a in &plan.actions {
        if let Action::WriteFile { rel, contents, .. } = a {
            let dest = home_dir.join(rel);
            let old = std::fs::read_to_string(&dest).unwrap_or_default();
            let new = String::from_utf8_lossy(contents).into_owned();
            let d = unified_diff(&old, &new, rel.display().to_string().as_str());
            if d.is_empty() {
                cliclack::log::info(format!("{} unchanged", rel.display()))?;
            } else {
                cliclack::log::warning(format!("{}\n{d}", rel.display().bold()))?;
            }
        }
    }
    Ok(())
}

fn describe(a: &Action) -> String {
    match a {
        Action::InstallPackage(p) => format!("install {p}"),
        Action::WriteFile { rel, contents, .. } => {
            format!("write {} ({} bytes)", rel.display(), contents.len())
        }
        Action::PostCommand(c) => format!("run `{}`", c.run()),
        Action::AptUpdate => "apt-get update".into(),
        Action::RemovePackage(p) => format!("remove {p}"),
        Action::RemoveFile(rel) => format!("remove {}", rel.display()),
        Action::Download { url, dest, .. } => {
            format!("download {url} → {}", dest.display())
        }
    }
}

/// `forge switch <new>` — teardown the current role, then setup the new.
///
/// Safe ordering: we teardown first so that overlapping files get cleaned
/// before the new role writes its versions. If teardown fails, we still
/// attempt setup (the user explicitly asked to end up on `new`).
pub fn switch(new_role: Role, dry_run: bool) -> Result<()> {
    let path = state_file()?;
    let prev = State::load(&path).ok().and_then(|s| s.role);

    if let Some(old) = prev {
        if old == new_role {
            cliclack::log::info(format!("already on {old}; running setup to refresh"))?;
            return run(new_role, dry_run, false, false);
        }
        cliclack::log::info(format!("tearing down {old} → {new_role}"))?;
        teardown_role(old, dry_run)?;
    }
    run(new_role, dry_run, false, false)
}

/// `forge teardown` — remove whatever the current role installed.
/// Reads `state.json` to find the current role, builds a reverse plan
/// from its role config, executes it, and wipes the state file so the
/// machine is ready for a fresh `forge setup`.
pub fn teardown(dry_run: bool) -> Result<()> {
    let path = state_file()?;
    let state = State::load(&path).unwrap_or_default();
    let role = state
        .role
        .ok_or_else(|| anyhow::anyhow!("no role installed (state.json empty)"))?;
    teardown_role(role, dry_run)?;
    if !dry_run {
        let cleared = State {
            role: None,
            theme: state.theme,
            applied_at: None,
            binary_version: state.binary_version,
        };
        cleared.save(&path)?;
    }
    Ok(())
}

fn teardown_role(role: Role, dry_run: bool) -> Result<()> {
    let roles = RolesFile::from_toml(ROLES_TOML)?;
    let cfg = roles.resolved(role)?;
    let plan = build_teardown_plan(&cfg, &EmbeddedDotfiles)?;
    let home_dir = home()?;

    cliclack::intro(format!("forge teardown {}", role.bold()))?;
    let s = plan.summary();
    cliclack::log::info(format!(
        "plan: {} files · {} packages",
        s.file_removals, s.package_removals
    ))?;

    if dry_run {
        for a in &plan.actions {
            cliclack::log::info(describe(a))?;
        }
        cliclack::outro("dry run — no changes made".dimmed().to_string())?;
        return Ok(());
    }

    let results = execute(&plan, &home_dir)?;
    let changed = results
        .iter()
        .filter(|r| **r == ActionResult::Changed)
        .count();
    cliclack::outro(format!("{changed} removed"))?;
    Ok(())
}

pub fn diff(role: Role) -> Result<()> {
    run(role, true, true, false)
}
