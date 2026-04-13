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
    checked(&[
        "sudo",
        "DEBIAN_FRONTEND=noninteractive",
        "apt-get",
        "update",
        "-y",
    ])?;
    Ok(())
}

/// Install one package via `sudo apt-get install -y`. Non-interactive —
/// requires passwordless sudo or a previously-authenticated session.
/// Surfaces apt's stderr on failure via `checked`, so errors like
/// "Unable to locate package" aren't hidden behind a bare exit code.
pub fn apt_install(pkg: &str) -> Result<()> {
    checked(&[
        "sudo",
        "DEBIAN_FRONTEND=noninteractive",
        "apt-get",
        "install",
        "-y",
        "--no-install-recommends",
        pkg,
    ])?;
    Ok(())
}

/// Uninstall one package (teardown path). Uses `remove` rather than
/// `purge` so dotfiles the user may have customized are preserved;
/// `forge teardown --full` would use `purge` + `autoremove`.
pub fn apt_remove(pkg: &str) -> Result<()> {
    checked(&[
        "sudo",
        "DEBIAN_FRONTEND=noninteractive",
        "apt-get",
        "remove",
        "-y",
        pkg,
    ])?;
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
