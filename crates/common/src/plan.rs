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

use crate::role::PostCmd;

/// One discrete thing the executor will do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Run `apt-get update` once up front. The planner emits this as the
    /// very first action whenever the plan will install any package, so
    /// that per-package installs can reuse the refreshed cache without
    /// re-updating each time.
    AptUpdate,

    /// Install a Debian package. Skipped at execute time if already
    /// installed. One per apt call so errors point at the specific
    /// package that broke.
    InstallPackage(String),

    /// Uninstall a Debian package (used by teardown). Skipped if the
    /// package isn't installed in the first place.
    RemovePackage(String),

    /// Write `contents` to `$HOME/<rel>` atomically, only if different.
    WriteFile {
        rel: PathBuf,
        contents: Vec<u8>,
        mode: Option<u32>,
    },

    /// Remove `$HOME/<rel>` if it exists. Used by teardown. Silently
    /// succeeds if the file is already gone.
    RemoveFile(PathBuf),

    /// Download `url` to `dest` (relative to `$HOME`) and optionally
    /// chmod to `mode`. Skipped when the destination already exists
    /// unless `--refresh-downloads` is passed.
    ///
    /// When `extract` is set, the URL is an archive — downloaded to a
    /// temp dir, extracted, and the file matching the glob pattern is
    /// moved to `dest`. Supports `.tar.gz` and `.zip`.
    Download {
        url: String,
        dest: PathBuf,
        mode: Option<u32>,
        extract: Option<String>,
    },

    /// Run a shell command after dotfiles are in place. If `check` is
    /// set, run it first and skip the command if the check exits 0.
    /// Runs through `sh -c` so shell features work.
    PostCommand(PostCmd),
}

/// The full plan for one `forge setup <role>` invocation.
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

    /// A terse summary of how many of each action kind the plan contains.
    pub fn summary(&self) -> PlanSummary {
        let mut s = PlanSummary::default();
        for a in &self.actions {
            match a {
                Action::AptUpdate => s.apt_updates += 1,
                Action::InstallPackage(_) => s.packages += 1,
                Action::RemovePackage(_) => s.package_removals += 1,
                Action::WriteFile { .. } => s.files += 1,
                Action::RemoveFile(_) => s.file_removals += 1,
                Action::Download { .. } => s.downloads += 1,
                Action::PostCommand(_) => s.commands += 1,
            }
        }
        s
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanSummary {
    pub apt_updates: usize,
    pub packages: usize,
    pub package_removals: usize,
    pub files: usize,
    pub file_removals: usize,
    pub downloads: usize,
    pub commands: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role::PostCmd;

    #[test]
    fn summary_counts_each_kind() {
        let mut p = Plan::new();
        p.push(Action::AptUpdate);
        p.push(Action::InstallPackage("zsh".into()));
        p.push(Action::WriteFile {
            rel: PathBuf::from(".zshrc"),
            contents: b"x".to_vec(),
            mode: None,
        });
        p.push(Action::Download {
            url: "http://example/nvim".into(),
            dest: PathBuf::from(".local/bin/nvim"),
            mode: Some(0o755),
            extract: None,
        });
        p.push(Action::PostCommand(PostCmd::Simple("chsh".into())));
        let s = p.summary();
        assert_eq!(s.apt_updates, 1);
        assert_eq!(s.packages, 1);
        assert_eq!(s.files, 1);
        assert_eq!(s.downloads, 1);
        assert_eq!(s.commands, 1);
    }

    #[test]
    fn teardown_summary_counts_removals() {
        let mut p = Plan::new();
        p.push(Action::RemoveFile(PathBuf::from(".zshrc")));
        p.push(Action::RemovePackage("zsh".into()));
        let s = p.summary();
        assert_eq!(s.file_removals, 1);
        assert_eq!(s.package_removals, 1);
    }

    #[test]
    fn empty_plan_has_zero_summary() {
        assert_eq!(Plan::new().summary(), PlanSummary::default());
    }
}
