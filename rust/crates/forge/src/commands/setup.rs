//! `forge setup <role>` — the core provisioning command.
//!
//! This command turns a role name into a plan, optionally previews it, and
//! then executes it. All the real work lives in `forge_common`: this module
//! just wires things together and handles user-facing output.

use anyhow::Result;
use forge_common::{
    build_plan, diff::unified_diff, execute, home, state_file, Action, ActionResult, Role,
    RolesFile, State,
};
use owo_colors::OwoColorize;

use crate::embed::{EmbeddedDotfiles, ROLES_TOML};

pub fn run(role: Role, dry_run: bool, show_diff: bool) -> Result<()> {
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

    // Execute for real. Each action's skip-check lives inside the executor,
    // so reruns are cheap even without inspecting the current state here.
    let results = execute(&plan, &home_dir)?;

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

/// For each `WriteFile` action, show a unified diff against the current
/// destination. Skipped files print a single "unchanged" line so the user
/// can see every file the plan touches.
fn print_diffs(plan: &forge_common::Plan, home_dir: &std::path::Path) -> Result<()> {
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
        Action::PostCommand(c) => format!("run `{c}`"),
    }
}

pub fn switch(_role: Role) -> Result<()> {
    anyhow::bail!("forge switch: not yet implemented")
}

pub fn teardown(_full: bool) -> Result<()> {
    anyhow::bail!("forge teardown: not yet implemented")
}

pub fn diff(role: Role) -> Result<()> {
    run(role, true, true)
}
