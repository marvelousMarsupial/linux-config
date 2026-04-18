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
use crate::packages::{apt_install, apt_remove, apt_update, is_installed};
use crate::plan::{Action, Plan};
use crate::role::PostCmd;

/// What each action produced. Used by the executor's caller to print
/// human summaries ("15 packages: 12 already installed, 3 installed").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResult {
    /// The action ran and changed system state.
    Changed,
    /// The action checked and found nothing to do.
    AlreadyDone,
}

/// Options that change executor behavior without altering the plan itself.
#[derive(Debug, Clone, Default)]
pub struct ExecOpts {
    /// When true, re-download files even if the destination already exists.
    pub refresh_downloads: bool,
}

/// Run every action in order, under `home` as the destination root.
/// Returns a parallel vector of results so callers can report a tally.
pub fn execute(plan: &Plan, home: &Path) -> Result<Vec<ActionResult>> {
    execute_with(plan, home, &ExecOpts::default())
}

/// Like `execute`, but with explicit options (e.g. `--refresh-downloads`).
pub fn execute_with(plan: &Plan, home: &Path, opts: &ExecOpts) -> Result<Vec<ActionResult>> {
    let mut results = Vec::with_capacity(plan.actions.len());
    for action in &plan.actions {
        let r = apply_one(action, home, opts)?;
        results.push(r);
    }
    Ok(results)
}

fn apply_one(action: &Action, home: &Path, opts: &ExecOpts) -> Result<ActionResult> {
    match action {
        Action::AptUpdate => {
            apt_update().context("refreshing apt package lists")?;
            Ok(ActionResult::Changed)
        }
        Action::InstallPackage(pkg) => {
            if is_installed(pkg) {
                Ok(ActionResult::AlreadyDone)
            } else {
                apt_install(pkg)
                    .with_context(|| format!("installing package `{pkg}`"))?;
                Ok(ActionResult::Changed)
            }
        }
        Action::RemovePackage(pkg) => {
            if !is_installed(pkg) {
                Ok(ActionResult::AlreadyDone)
            } else {
                apt_remove(pkg)
                    .with_context(|| format!("removing package `{pkg}`"))?;
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
        Action::RemoveFile(rel) => {
            let dest = home.join(rel);
            if dest.exists() || dest.symlink_metadata().is_ok() {
                std::fs::remove_file(&dest)
                    .with_context(|| format!("removing file {}", dest.display()))?;
                Ok(ActionResult::Changed)
            } else {
                Ok(ActionResult::AlreadyDone)
            }
        }
        Action::Download { url, dest, mode, extract } => {
            let abs = home.join(dest);
            if abs.exists() && !opts.refresh_downloads {
                return Ok(ActionResult::AlreadyDone);
            }
            if let Some(parent) = abs.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            match extract {
                Some(pattern) => download_and_extract(url, &abs, pattern)?,
                None => download_raw(url, &abs)?,
            }
            if let Some(m) = mode {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&abs, std::fs::Permissions::from_mode(*m))?;
            }
            Ok(ActionResult::Changed)
        }
        Action::PostCommand(cmd) => apply_post(cmd),
    }
}

/// Download a URL directly to a destination file.
fn download_raw(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .with_context(|| format!("spawning curl for {url}"))?;
    if !status.success() {
        anyhow::bail!("downloading {url} failed with status {status}");
    }
    Ok(())
}

/// Download an archive and extract it. Two modes based on `pattern`:
///
/// - `"tree:<N>"` — extract entire archive to `dest` directory, stripping
///   N leading path components (like `tar --strip-components`).
/// - Anything else — glob pattern for a single file to pull out and place
///   at `dest` (e.g. `"starship"`, `"*/yazi"`).
fn download_and_extract(url: &str, dest: &Path, pattern: &str) -> Result<()> {
    if let Some(n) = pattern.strip_prefix("tree:") {
        let strip: u32 = n.parse().with_context(|| format!("invalid strip count in `{pattern}`"))?;
        return download_and_extract_tree(url, dest, strip);
    }
    download_and_extract_file(url, dest, pattern)
}

/// Extract an entire archive to `dest` as a directory, stripping `strip`
/// leading path components. Used for tools like nvim that need their
/// sibling lib/ and share/ directories alongside the binary.
fn download_and_extract_tree(url: &str, dest: &Path, strip: u32) -> Result<()> {
    let tmp = tempfile::tempdir().context("creating temp dir")?;
    let archive = tmp.path().join("archive");
    download_raw(url, &archive)?;

    std::fs::create_dir_all(dest)
        .with_context(|| format!("creating {}", dest.display()))?;

    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        let status = Command::new("tar")
            .arg("xzf").arg(&archive)
            .arg("-C").arg(dest)
            .arg(&format!("--strip-components={strip}"))
            .status()
            .context("extracting tar.gz")?;
        if !status.success() {
            anyhow::bail!("tar extraction failed for {url}");
        }
    } else if url.ends_with(".zip") {
        // unzip doesn't have --strip-components; extract to tmp then move.
        let inner = tmp.path().join("inner");
        let status = Command::new("unzip")
            .arg("-q").arg(&archive).arg("-d").arg(&inner)
            .status()
            .context("extracting zip")?;
        if !status.success() {
            anyhow::bail!("unzip failed for {url}");
        }
        // Walk `strip` levels deep to find the real root, then move contents.
        let mut root = inner.clone();
        for _ in 0..strip {
            let entries: Vec<_> = std::fs::read_dir(&root)?.collect::<Result<Vec<_>, _>>()?;
            if entries.len() == 1 && entries[0].path().is_dir() {
                root = entries[0].path();
            } else {
                break;
            }
        }
        for entry in std::fs::read_dir(&root)? {
            let e = entry?;
            let target = dest.join(e.file_name());
            std::fs::rename(e.path(), &target)
                .with_context(|| format!("moving {} to {}", e.path().display(), target.display()))?;
        }
    } else {
        anyhow::bail!("unknown archive format for {url}");
    }
    Ok(())
}

/// Extract a single file matching a glob pattern from an archive.
fn download_and_extract_file(url: &str, dest: &Path, pattern: &str) -> Result<()> {
    let tmp = tempfile::tempdir().context("creating temp dir for extraction")?;
    let archive = tmp.path().join("archive");
    download_raw(url, &archive)?;

    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        let status = Command::new("tar")
            .arg("xzf").arg(&archive).arg("-C").arg(tmp.path())
            .status().context("extracting tar.gz")?;
        if !status.success() {
            anyhow::bail!("tar extraction failed for {url}");
        }
    } else if url.ends_with(".zip") {
        let status = Command::new("unzip")
            .arg("-q").arg(&archive).arg("-d").arg(tmp.path())
            .status().context("extracting zip")?;
        if !status.success() {
            anyhow::bail!("unzip failed for {url}");
        }
    } else {
        anyhow::bail!("unknown archive format for {url} (expected .tar.gz or .zip)");
    }

    let glob = globset::GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .with_context(|| format!("invalid extract pattern `{pattern}`"))?
        .compile_matcher();

    let mut found = None;
    for entry in walkdir(tmp.path())? {
        let rel = entry.strip_prefix(tmp.path()).unwrap_or(&entry);
        if glob.is_match(rel) {
            found = Some(entry);
            break;
        }
    }

    let src = found.ok_or_else(|| {
        anyhow::anyhow!("no file matching `{pattern}` in archive from {url}")
    })?;
    std::fs::copy(&src, dest)
        .with_context(|| format!("moving extracted file to {}", dest.display()))?;
    Ok(())
}

/// Recursively list all files under `dir`.
fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    fn walk(d: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(d).with_context(|| format!("reading {}", d.display()))? {
            let e = entry?;
            let p = e.path();
            if p.is_dir() {
                walk(&p, out)?;
            } else {
                out.push(p);
            }
        }
        Ok(())
    }
    walk(dir, &mut out)?;
    Ok(out)
}

/// Run a post-command, honoring its optional `check` for idempotency.
/// If the check exits 0, the command is skipped.
fn apply_post(cmd: &PostCmd) -> Result<ActionResult> {
    if let Some(chk) = cmd.check() {
        let chk_status = Command::new("sh").arg("-c").arg(chk).status()?;
        if chk_status.success() {
            return Ok(ActionResult::AlreadyDone);
        }
    }
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd.run())
        .status()
        .with_context(|| format!("spawning post-command `{}`", cmd.run()))?;
    if !status.success() {
        anyhow::bail!("post-command `{}` exited with {status}", cmd.run());
    }
    Ok(ActionResult::Changed)
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
        plan.push(Action::PostCommand(PostCmd::Simple("true".into())));
        let r = execute(&plan, d.path()).unwrap();
        assert_eq!(r, vec![ActionResult::Changed]);
    }

    /// PostCommand failure path — using `false` which always exits 1.
    #[test]
    fn post_command_failure_bubbles_up() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand(PostCmd::Simple("false".into())));
        let err = execute(&plan, d.path()).unwrap_err();
        assert!(err.to_string().contains("exited"));
    }

    /// A PostCommand failure halts the plan so later actions don't run.
    #[test]
    fn failure_stops_execution() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand(PostCmd::Simple("false".into())));
        plan.push(Action::WriteFile {
            rel: PathBuf::from(".should-not-exist"),
            contents: b"x".to_vec(),
            mode: None,
        });
        assert!(execute(&plan, d.path()).is_err());
        assert!(!d.path().join(".should-not-exist").exists());
    }

    /// A post-command with a passing `check` is skipped (AlreadyDone).
    #[test]
    fn post_command_check_short_circuits() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand(PostCmd::Checked {
            run: "false".into(), // would fail if run
            check: Some("true".into()), // but check passes, so we skip
        }));
        let r = execute(&plan, d.path()).unwrap();
        assert_eq!(r, vec![ActionResult::AlreadyDone]);
    }

    /// A post-command with a failing `check` runs normally.
    #[test]
    fn post_command_check_fails_then_runs() {
        let d = tempdir().unwrap();
        let mut plan = Plan::new();
        plan.push(Action::PostCommand(PostCmd::Checked {
            run: "true".into(),
            check: Some("false".into()), // fails → must run
        }));
        let r = execute(&plan, d.path()).unwrap();
        assert_eq!(r, vec![ActionResult::Changed]);
    }

    /// RemoveFile is idempotent: twice in a row is fine, second reports AlreadyDone.
    #[test]
    fn remove_file_idempotent() {
        let d = tempdir().unwrap();
        std::fs::write(d.path().join(".zshrc"), b"x").unwrap();
        let mut plan = Plan::new();
        plan.push(Action::RemoveFile(PathBuf::from(".zshrc")));
        let r1 = execute(&plan, d.path()).unwrap();
        assert_eq!(r1, vec![ActionResult::Changed]);
        let r2 = execute(&plan, d.path()).unwrap();
        assert_eq!(r2, vec![ActionResult::AlreadyDone]);
    }

    /// Extract finds the right file from an archive using a glob pattern.
    #[test]
    fn extract_tar_gz_finds_binary() {
        let d = tempdir().unwrap();
        // Create a tar.gz with a known file inside.
        let staging = tempdir().unwrap();
        std::fs::write(staging.path().join("mybinary"), b"#!/bin/sh\necho hi").unwrap();
        let archive = d.path().join("test.tar.gz");
        let status = Command::new("tar")
            .args(["czf"])
            .arg(&archive)
            .arg("-C")
            .arg(staging.path())
            .arg("mybinary")
            .status()
            .unwrap();
        assert!(status.success());

        let dest = d.path().join("extracted");
        download_and_extract(
            &format!("file://{}", archive.display()),
            &dest,
            "mybinary",
        )
        .unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"#!/bin/sh\necho hi");
    }

    /// Extract with a glob pattern that includes a wildcard prefix.
    #[test]
    fn extract_glob_with_wildcard() {
        let d = tempdir().unwrap();
        let staging = tempdir().unwrap();
        std::fs::create_dir_all(staging.path().join("yazi-v0.4")).unwrap();
        std::fs::write(staging.path().join("yazi-v0.4/yazi"), b"yazi-bin").unwrap();
        let archive = d.path().join("test.tar.gz");
        Command::new("tar")
            .args(["czf"])
            .arg(&archive)
            .arg("-C")
            .arg(staging.path())
            .arg("yazi-v0.4")
            .status()
            .unwrap();

        let dest = d.path().join("yazi");
        download_and_extract(
            &format!("file://{}", archive.display()),
            &dest,
            "*/yazi",
        )
        .unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"yazi-bin");
    }

    /// Extract with no matching file produces a clear error.
    #[test]
    fn extract_no_match_errors() {
        let d = tempdir().unwrap();
        let staging = tempdir().unwrap();
        std::fs::write(staging.path().join("other"), b"x").unwrap();
        let archive = d.path().join("test.tar.gz");
        Command::new("tar")
            .args(["czf"])
            .arg(&archive)
            .arg("-C")
            .arg(staging.path())
            .arg("other")
            .status()
            .unwrap();

        let dest = d.path().join("nope");
        let err = download_and_extract(
            &format!("file://{}", archive.display()),
            &dest,
            "missing*",
        )
        .unwrap_err();
        assert!(err.to_string().contains("no file matching"));
    }
}
