//! Builds a `Plan` from a `RoleConfig` plus a source of dotfile contents.
//!
//! This module is deliberately agnostic about *where* the dotfiles come
//! from: it takes a `DotfileSource` trait. In production the source is an
//! `include_dir::Dir` embedded in the binary, but in tests we pass an
//! in-memory `Vec<(path, contents)>` so planner behavior can be verified
//! without touching the embedded assets or a temp directory.
//!
//! Glob semantics follow `globset`: `*` matches one path segment, `**`
//! matches any number of segments. The patterns in `roles.toml` are
//! matched against the dotfile tree's relative paths (e.g. `.zshrc` or
//! `.config/foot/foot.ini`). Duplicates across an `extends` chain are
//! deduplicated — the same dotfile is never written twice in one plan.

use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSetBuilder};
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::plan::{Action, Plan};
use crate::role::RoleConfig;

/// A source of dotfile bytes, keyed by relative path under `$HOME`.
///
/// Kept as a trait (not a concrete type) so tests can supply a toy map
/// while production uses `include_dir::Dir`.
pub trait DotfileSource {
    /// List every file this source exposes, as paths relative to `$HOME`.
    fn list(&self) -> Vec<PathBuf>;
    /// Read the bytes of a specific file. Returns `None` if `rel` isn't
    /// in `list()`. The planner never calls this for a path it didn't get
    /// from `list()`, so `None` is treated as a programmer bug, not a
    /// user error.
    fn read(&self, rel: &std::path::Path) -> Option<Vec<u8>>;
}

/// In-memory implementation used by unit tests.
#[derive(Default)]
pub struct InMemoryDotfiles(pub Vec<(PathBuf, Vec<u8>)>);

impl DotfileSource for InMemoryDotfiles {
    fn list(&self) -> Vec<PathBuf> {
        self.0.iter().map(|(p, _)| p.clone()).collect()
    }
    fn read(&self, rel: &std::path::Path) -> Option<Vec<u8>> {
        self.0
            .iter()
            .find(|(p, _)| p == rel)
            .map(|(_, b)| b.clone())
    }
}

/// Build a plan from a resolved role config and a dotfile source.
///
/// Action order within the plan:
///   1. Packages (so everything the dotfiles and post-commands need is there)
///   2. Files (so post-commands can read them, e.g. `chsh` needs /etc/shells)
///   3. Post-commands
pub fn build_plan(cfg: &RoleConfig, src: &dyn DotfileSource) -> Result<Plan> {
    let mut plan = Plan::new();

    // --- Packages ---------------------------------------------------------
    // Deduplicate while preserving order. A later `extends` chain might name
    // the same package twice; installing it twice is fine but noisy.
    let mut seen_pkg = BTreeSet::new();
    for p in &cfg.packages {
        if seen_pkg.insert(p.clone()) {
            plan.push(Action::InstallPackage(p.clone()));
        }
    }

    // --- Dotfiles ---------------------------------------------------------
    // Compile glob patterns once. `globset` is literal about `**` vs `*`
    // semantics, which is what we want — `.config/foot/**` matches every
    // descendant of `.config/foot/` but not `.config/foot-backup/foo`.
    // `literal_separator(true)` makes `*` match within a single path segment
    // only (like gitignore / shell globs), so `.config/*` matches direct
    // children but not `.config/foot/foot.ini`. Use `**` for recursive.
    let mut builder = GlobSetBuilder::new();
    for pat in &cfg.dotfiles {
        let g = GlobBuilder::new(pat)
            .literal_separator(true)
            .build()
            .with_context(|| format!("invalid glob `{pat}`"))?;
        builder.add(g);
    }
    let globs = builder
        .build()
        .context("assembling glob set for dotfiles")?;

    // Enumerate every file in the source, keep those matching any glob,
    // deduplicate (a file matched by two overlapping globs only appears once),
    // and sort for deterministic plans (makes tests stable and diffs readable).
    let mut matched: BTreeSet<PathBuf> = BTreeSet::new();
    for rel in src.list() {
        if globs.is_match(&rel) {
            matched.insert(rel);
        }
    }
    for rel in matched {
        let contents = src
            .read(&rel)
            .ok_or_else(|| anyhow::anyhow!("dotfile vanished from source: {}", rel.display()))?;
        plan.push(Action::WriteFile {
            rel,
            contents,
            mode: None,
        });
    }

    // --- Post-commands ----------------------------------------------------
    for cmd in &cfg.post {
        plan.push(Action::PostCommand(cmd.clone()));
    }

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::Action;

    fn sample_source() -> InMemoryDotfiles {
        InMemoryDotfiles(vec![
            (PathBuf::from(".zshrc"), b"zsh!".to_vec()),
            (PathBuf::from(".tmux.conf"), b"tmux!".to_vec()),
            (
                PathBuf::from(".config/foot/foot.ini"),
                b"foot!".to_vec(),
            ),
            (
                PathBuf::from(".config/foot/colors.ini"),
                b"colors".to_vec(),
            ),
            (PathBuf::from(".config/nvim/init.lua"), b"vim".to_vec()),
            (PathBuf::from(".local/share/applications/x.desktop"), b"x".to_vec()),
        ])
    }

    fn cfg(packages: &[&str], dotfiles: &[&str], post: &[&str]) -> RoleConfig {
        RoleConfig {
            extends: None,
            packages: packages.iter().map(|s| s.to_string()).collect(),
            dotfiles: dotfiles.iter().map(|s| s.to_string()).collect(),
            downloads: vec![],
            post: post.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn packages_first_files_second_post_last() {
        let c = cfg(&["zsh"], &[".zshrc"], &["chsh"]);
        let plan = build_plan(&c, &sample_source()).unwrap();
        assert!(matches!(plan.actions[0], Action::InstallPackage(_)));
        assert!(matches!(plan.actions[1], Action::WriteFile { .. }));
        assert!(matches!(plan.actions[2], Action::PostCommand(_)));
    }

    #[test]
    fn exact_path_glob_matches_one_file() {
        let c = cfg(&[], &[".zshrc"], &[]);
        let plan = build_plan(&c, &sample_source()).unwrap();
        assert_eq!(plan.summary().files, 1);
        match &plan.actions[0] {
            Action::WriteFile { rel, contents, .. } => {
                assert_eq!(rel, &PathBuf::from(".zshrc"));
                assert_eq!(contents, b"zsh!");
            }
            _ => panic!("expected WriteFile"),
        }
    }

    #[test]
    fn recursive_glob_captures_subtree() {
        let c = cfg(&[], &[".config/foot/**"], &[]);
        let plan = build_plan(&c, &sample_source()).unwrap();
        // `.config/foot/foot.ini` + `.config/foot/colors.ini` = 2
        assert_eq!(plan.summary().files, 2);
    }

    #[test]
    fn overlapping_globs_dont_duplicate() {
        let c = cfg(&[], &[".config/foot/**", ".config/foot/foot.ini"], &[]);
        let plan = build_plan(&c, &sample_source()).unwrap();
        assert_eq!(plan.summary().files, 2); // still 2, not 3
    }

    #[test]
    fn duplicate_packages_deduped() {
        let c = cfg(&["zsh", "zsh", "tmux"], &[], &[]);
        let plan = build_plan(&c, &sample_source()).unwrap();
        assert_eq!(plan.summary().packages, 2);
    }

    #[test]
    fn invalid_glob_errors_with_pattern_in_message() {
        let c = cfg(&[], &["["], &[]);
        let err = build_plan(&c, &sample_source()).unwrap_err();
        assert!(err.to_string().contains("invalid glob"));
    }

    #[test]
    fn empty_role_produces_empty_plan() {
        let plan = build_plan(&cfg(&[], &[], &[]), &sample_source()).unwrap();
        assert!(plan.actions.is_empty());
    }

    #[test]
    fn file_order_is_deterministic() {
        let c = cfg(&[], &[".config/foot/**"], &[]);
        let plan1 = build_plan(&c, &sample_source()).unwrap();
        let plan2 = build_plan(&c, &sample_source()).unwrap();
        assert_eq!(plan1, plan2);
    }

    #[test]
    fn single_star_does_not_recurse() {
        // `.config/*` should match direct children only (but our sample has
        // `.config/foot/foot.ini` which is 2 segments deep — shouldn't match).
        let c = cfg(&[], &[".config/*"], &[]);
        let plan = build_plan(&c, &sample_source()).unwrap();
        assert_eq!(plan.summary().files, 0);
    }

    #[test]
    fn glob_matches_across_extends_deduped_via_resolver() {
        // After `RolesFile::resolved()` merges a chain, the planner sees
        // duplicates in `dotfiles` — the glob matching + BTreeSet handle it.
        let c = cfg(
            &[],
            &[".zshrc", ".zshrc", ".tmux.conf"],
            &[],
        );
        let plan = build_plan(&c, &sample_source()).unwrap();
        assert_eq!(plan.summary().files, 2);
    }
}
