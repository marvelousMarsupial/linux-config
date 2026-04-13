//! Action plan produced by the setup planner.
//!
//! The planner is a **pure function**: given a role config and a source of
//! dotfile contents, it returns a `Plan` (a sequence of `Action`s) without
//! touching the system. The executor is the only thing that mutates state.
//!
//! This split gives us three nice properties:
//!   1. Dry-run is free — we just build the plan and print it.
//!   2. Unit tests can assert "this input produces these actions" without
//!      needing root, apt, or a temp directory.
//!   3. Each action is idempotent on its own (skip-if-done), so reruns are
//!      cheap and partial failures resume cleanly.

use std::path::PathBuf;

/// One discrete thing the executor will do.
///
/// The executor consumes actions in order. Ordering matters for only two
/// reasons: packages must be installed before post-commands that depend on
/// them (chsh needs zsh installed), and dotfiles go before post-commands
/// that read them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Install a Debian package. Skipped at execute time if already installed.
    /// One per apt call so errors point at the specific package that broke.
    InstallPackage(String),

    /// Write `contents` to `$HOME/<rel>` atomically, only if different.
    /// `rel` is a relative path like `.zshrc` or `.config/foot/foot.ini`.
    WriteFile {
        rel: PathBuf,
        contents: Vec<u8>,
        mode: Option<u32>,
    },

    /// Run a shell command after dotfiles are in place. Used for things like
    /// `chsh -s /bin/zsh`. Runs through `sh -c` so shell features work.
    PostCommand(String),
}

/// The full plan for one `forge setup <role>` invocation. Lives in memory
/// only; never serialized. Printed by `--dry-run`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    pub actions: Vec<Action>,
}

impl Plan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, a: Action) {
        self.actions.push(a);
    }

    /// A terse human-readable summary of how many of each action kind the
    /// plan contains. Used at the top of `--dry-run` output.
    pub fn summary(&self) -> PlanSummary {
        let mut s = PlanSummary::default();
        for a in &self.actions {
            match a {
                Action::InstallPackage(_) => s.packages += 1,
                Action::WriteFile { .. } => s.files += 1,
                Action::PostCommand(_) => s.commands += 1,
            }
        }
        s
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanSummary {
    pub packages: usize,
    pub files: usize,
    pub commands: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_each_kind() {
        let mut p = Plan::new();
        p.push(Action::InstallPackage("zsh".into()));
        p.push(Action::InstallPackage("tmux".into()));
        p.push(Action::WriteFile {
            rel: PathBuf::from(".zshrc"),
            contents: b"x".to_vec(),
            mode: None,
        });
        p.push(Action::PostCommand("chsh -s /bin/zsh".into()));
        let s = p.summary();
        assert_eq!(s.packages, 2);
        assert_eq!(s.files, 1);
        assert_eq!(s.commands, 1);
    }

    #[test]
    fn empty_plan_has_zero_summary() {
        let p = Plan::new();
        assert_eq!(p.summary(), PlanSummary::default());
    }
}
