//! Atomic file writes: write to a temp file in the target's dir, then rename.
//!
//! This gives us two properties we rely on for idempotency:
//!   1. Partial writes can never clobber the destination.
//!   2. If the new content is identical to existing content, we skip the
//!      write entirely, returning `false` so callers can report "unchanged".

use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Write `contents` to `path`, atomically and only if changed.
/// Returns `Ok(true)` if the file was written (changed or new), `Ok(false)` if
/// the existing file already had identical contents.
pub fn write_atomic(path: &Path, contents: &[u8], mode: Option<u32>) -> Result<bool> {
    if let Ok(existing) = fs::read(path) {
        if existing == contents {
            if let Some(m) = mode {
                ensure_mode(path, m)?;
            }
            return Ok(false);
        }
    }

    let parent = path
        .parent()
        .with_context(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating parent dir {}", parent.display()))?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("creating temp file in {}", parent.display()))?;
    tmp.write_all(contents)
        .context("writing temp contents")?;
    tmp.flush().ok();

    if let Some(m) = mode {
        fs::set_permissions(tmp.path(), fs::Permissions::from_mode(m))
            .with_context(|| format!("chmod temp file to {m:o}"))?;
    }

    tmp.persist(path)
        .map_err(|e| anyhow::anyhow!("renaming temp to {}: {}", path.display(), e))?;
    Ok(true)
}

fn ensure_mode(path: &Path, mode: u32) -> Result<()> {
    let cur = fs::metadata(path)?.permissions().mode() & 0o7777;
    if cur != (mode & 0o7777) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn writes_new_file() {
        let d = tempdir().unwrap();
        let p = d.path().join("f");
        assert!(write_atomic(&p, b"hello", None).unwrap());
        assert_eq!(fs::read(&p).unwrap(), b"hello");
    }

    #[test]
    fn skips_identical_content() {
        let d = tempdir().unwrap();
        let p = d.path().join("f");
        write_atomic(&p, b"hello", None).unwrap();
        assert!(!write_atomic(&p, b"hello", None).unwrap());
    }

    #[test]
    fn overwrites_different_content() {
        let d = tempdir().unwrap();
        let p = d.path().join("f");
        write_atomic(&p, b"one", None).unwrap();
        assert!(write_atomic(&p, b"two", None).unwrap());
        assert_eq!(fs::read(&p).unwrap(), b"two");
    }

    #[test]
    fn creates_parent_dirs() {
        let d = tempdir().unwrap();
        let p = d.path().join("a/b/c/f");
        assert!(write_atomic(&p, b"x", None).unwrap());
        assert!(p.exists());
    }

    #[test]
    fn sets_mode_on_new_file() {
        let d = tempdir().unwrap();
        let p = d.path().join("f");
        write_atomic(&p, b"#!/bin/sh\n", Some(0o755)).unwrap();
        let m = fs::metadata(&p).unwrap().permissions().mode() & 0o7777;
        assert_eq!(m, 0o755);
    }

    #[test]
    fn fixes_mode_on_identical_file() {
        let d = tempdir().unwrap();
        let p = d.path().join("f");
        write_atomic(&p, b"x", Some(0o644)).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(!write_atomic(&p, b"x", Some(0o644)).unwrap());
        let m = fs::metadata(&p).unwrap().permissions().mode() & 0o7777;
        assert_eq!(m, 0o644);
    }
}
