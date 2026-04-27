//! Debian package state queries.
//!
//! We install packages one at a time (not batched) so errors point at a
//! specific package, and so `is_installed()` can short-circuit each one
//! before we pay the apt roundtrip. Slightly slower than `apt install a b c`
//! but far easier to debug, and zero cost on reruns when everything is
//! already installed.

use anyhow::Result;
use std::process::Command;

use crate::run::checked;

/// Returns `&["sudo"]` when running as a normal user, or `&[]` when
/// already root. Keeps apt commands working in both contexts: a user's
/// desktop (needs sudo) and a container/VM (already root, no sudo installed).
fn maybe_sudo() -> &'static [&'static str] {
    // $EUID is set by bash/zsh; `id -u` is the POSIX fallback. Checking
    // the env var is cheaper than spawning a process on every apt call.
    let is_root = std::env::var("EUID")
        .unwrap_or_else(|_| {
            Command::new("id")
                .arg("-u")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        })
        .trim()
        .parse::<u32>()
        .map(|uid| uid == 0)
        .unwrap_or(false);
    if is_root {
        &[]
    } else {
        &["sudo"]
    }
}

/// Return `true` if a dpkg package is installed *and* in the "ok installed"
/// state. Packages that are half-configured, half-removed, or only unpacked
/// return `false` so we re-run apt and force them into a healthy state.
pub fn is_installed(pkg: &str) -> bool {
    // dpkg-query -W -f='${Status}' returns lines like "install ok installed"
    // for a healthy package, or exits non-zero if the package is unknown.
    let out = Command::new("dpkg-query")
        .args(["-W", "-f=${Status}", pkg])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.trim() == "install ok installed"
        }
        _ => false,
    }
}

/// Refresh apt package lists. Called once per `forge setup` run by the
/// executor so individual package installs see a fresh cache without
/// paying the refresh cost N times.
pub fn apt_update() -> Result<()> {
    let mut cmd: Vec<&str> = maybe_sudo().to_vec();
    cmd.extend([
        "env",
        "DEBIAN_FRONTEND=noninteractive",
        "apt-get",
        "update",
        "-y",
    ]);
    checked(&cmd)?;
    Ok(())
}

/// Install one package via apt-get. Non-interactive — requires passwordless
/// sudo (or root). Surfaces apt's stderr on failure via `checked`.
pub fn apt_install(pkg: &str) -> Result<()> {
    let mut cmd: Vec<&str> = maybe_sudo().to_vec();
    cmd.extend([
        "env",
        "DEBIAN_FRONTEND=noninteractive",
        "apt-get",
        "install",
        "-y",
        "--no-install-recommends",
        pkg,
    ]);
    checked(&cmd)?;
    Ok(())
}

/// Mark a package as manually installed.
///
/// Called by the executor for every `InstallPackage` action — including
/// the "already installed" path — so the package is protected from
/// `apt autoremove` even if it originally landed on disk as a transitive
/// dependency of something else.
///
/// # The bug this exists to prevent
///
/// We hit this exact failure mode after switching whisper.cpp from
/// host-build to vendored binaries: `libopenblas0` was already on disk
/// because `libopenblas-dev` (the old build dep) had pulled it in as a
/// dep with the auto-installed flag set. The `desktop` role's package
/// list was updated to include `libopenblas0` directly, but the
/// executor's `is_installed` short-circuit meant `apt-get install
/// libopenblas0` never ran — and so apt's auto-installed flag stayed
/// set. The next `apt autoremove` swept libopenblas0 out from under
/// whisper-server. Re-marking the package manually here closes that
/// window: even if the package is already installed, we tell apt
/// "the user wants this regardless of what depends on it."
///
/// `apt-mark manual` is a true no-op (and very fast) when the flag is
/// already set, so calling it on every install path is cheap.
pub fn apt_mark_manual(pkg: &str) -> Result<()> {
    let mut cmd: Vec<&str> = maybe_sudo().to_vec();
    cmd.extend(["apt-mark", "manual", pkg]);
    checked(&cmd)?;
    Ok(())
}

/// Uninstall one package (teardown path). Uses `remove` rather than
/// `purge` so dotfiles the user may have customized are preserved.
pub fn apt_remove(pkg: &str) -> Result<()> {
    let mut cmd: Vec<&str> = maybe_sudo().to_vec();
    cmd.extend([
        "env",
        "DEBIAN_FRONTEND=noninteractive",
        "apt-get",
        "remove",
        "-y",
        pkg,
    ]);
    checked(&cmd)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// We can't unit-test `is_installed` against a fixed known package
    /// portably, but we *can* assert that a definitely-bogus name returns
    /// `false` and doesn't panic.
    #[test]
    fn bogus_package_not_installed() {
        assert!(!is_installed("definitely-not-a-real-package-xyz-123"));
    }

    /// And the empty string shouldn't crash dpkg-query's output parsing.
    #[test]
    fn empty_name_not_installed() {
        assert!(!is_installed(""));
    }
}
