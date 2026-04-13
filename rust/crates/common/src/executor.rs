//! Executes a `Plan` against the live system.
//!
//! Every action is idempotent: `WriteFile` short-circuits when the existing
//! content is identical, `InstallPackage` skips when `dpkg-query` says the
//! package is already installed, and `PostCommand` is the one kind that
//! always runs (since arbitrary shell commands can't be inspected safely
//! before running). If a post-command needs idempotency, it must implement
//! its own check — e.g. `grep -q pattern /file || echo >> /file`.
//!
//! The executor stops at the first error and bubbles it up with context,
//! so reruns pick up where they left off thanks to the per-action skip
//! checks.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::atomic::write_atomic;
use crate::packages::{apt_install, is_installed};
use crate::plan::{Action, Plan};

/// What each action produced. Used by the executor's caller to print
/// human summaries ("15 packages: 12 already installed, 3 installed").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResult {
    /// The action ran and changed system state.
    Changed,
    /// The action checked and found nothing to do.
    AlreadyDone,
}

/// Run every action in order, under `home` as the destination root.
/// Returns a parallel vector of results so callers can report a tally.
pub fn execute(plan: &Plan, home: &Path) -> Result<Vec<ActionResult>> {
    let mut results = Vec::with_capacity(plan.actions.len());
    for action in &plan.actions {
        let r = apply_one(action, home)?;
        results.push(r);
    }
    Ok(results)
}

fn apply_one(action: &Action, home: &Path) -> Result<ActionResult> {
    match action {
        Action::InstallPackage(pkg) => {
            if is_installed(pkg) {
                Ok(ActionResult::AlreadyDone)
            } else {
                apt_install(pkg)
                    .with_context(|| format!("installing package `{pkg}`"))?;
                Ok(ActionResult::Changed)
            }
        }
        Action::WriteFile { rel, contents, mode } => {
            let dest = home.join(rel);
            let changed = write_atomic(&dest, contents, *mode)
                .with_context(|| format!("writing dotfile {}", dest.display()))?;
            Ok(if changed {
                ActionResult::Changed
            } else {
                ActionResult::AlreadyDone
            })
        }
        Action::PostCommand(cmd) => {
            let status = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .status()
                .with_context(|| format!("spawning post-command `{cmd}`"))?;
            if !status.success() {
                anyhow::bail!("post-command `{cmd}` exited with {status}");
            }
            Ok(ActionResult::Changed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::Plan;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// WriteFile creates a new file and reports Changed the first time,
    /// AlreadyDone the second time.
    #[test]
    fn write_file_is_idempotent() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::WriteFile {
            rel: PathBuf::from(".zshrc"),
            contents: b"hello\n".to_vec(),
            mode: None,
        });

        let r1 = execute(&plan, d.path()).unwrap();
        assert_eq!(r1, vec![ActionResult::Changed]);
        let r2 = execute(&plan, d.path()).unwrap();
        assert_eq!(r2, vec![ActionResult::AlreadyDone]);
    }

    /// A WriteFile deep under $HOME creates the parent chain.
    #[test]
    fn write_file_creates_parents() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::WriteFile {
            rel: PathBuf::from(".config/foot/foot.ini"),
            contents: b"x".to_vec(),
            mode: None,
        });
        execute(&plan, d.path()).unwrap();
        assert!(d.path().join(".config/foot/foot.ini").exists());
    }

    /// A WriteFile that edits an existing file reports Changed.
    #[test]
    fn write_file_detects_change() {
        let d = tempdir().unwrap();
        std::fs::write(d.path().join(".zshrc"), b"old").unwrap();
        let mut plan = Plan::new();
        plan.push(Action::WriteFile {
            rel: PathBuf::from(".zshrc"),
            contents: b"new".to_vec(),
            mode: None,
        });
        let r = execute(&plan, d.path()).unwrap();
        assert_eq!(r, vec![ActionResult::Changed]);
    }

    /// PostCommand success path — using `true` which always exits 0.
    #[test]
    fn post_command_success() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand("true".into()));
        let r = execute(&plan, d.path()).unwrap();
        assert_eq!(r, vec![ActionResult::Changed]);
    }

    /// PostCommand failure path — using `false` which always exits 1.
    #[test]
    fn post_command_failure_bubbles_up() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand("false".into()));
        let err = execute(&plan, d.path()).unwrap_err();
        assert!(err.to_string().contains("exited"));
    }

    /// A PostCommand failure halts the plan so later actions don't run.
    #[test]
    fn failure_stops_execution() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand("false".into()));
        plan.push(Action::WriteFile {
            rel: PathBuf::from(".should-not-exist"),
            contents: b"x".to_vec(),
            mode: None,
        });
        assert!(execute(&plan, d.path()).is_err());
        assert!(!d.path().join(".should-not-exist").exists());
    }
}
