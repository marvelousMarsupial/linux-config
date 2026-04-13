//! Debian package state queries.
//!
//! We install packages one at a time (not batched) so errors point at a
//! specific package, and so `is_installed()` can short-circuit each one
//! before we pay the apt roundtrip. Slightly slower than `apt install a b c`
//! but far easier to debug, and zero cost on reruns when everything is
//! already installed.

use anyhow::Result;
use std::process::Command;

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

/// Install one package via `sudo apt-get install -y`. Surfaces apt's own
/// error output on failure. Non-interactive — requires the user to have
/// passwordless sudo or have already authenticated in this session.
pub fn apt_install(pkg: &str) -> Result<()> {
    let status = Command::new("sudo")
        .args([
            "DEBIAN_FRONTEND=noninteractive",
            "apt-get",
            "install",
            "-y",
            "--no-install-recommends",
            pkg,
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("apt-get install {pkg} failed with status {status}");
    }
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
