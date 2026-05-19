//! Parsed `dev-container.toml`.
//!
//! Flat config — there is exactly one dev image, used by every
//! `forge dev up <name>`. Plain data structs (no behavior) so the forge
//! binary reads them once at startup and command handlers receive a
//! borrowed slice. The previously-paired `vm-profiles.toml` /
//! `VmProfilesFile` was retired with the rest of the `forge vm` stack.

use anyhow::{Context, Result};
use serde::Deserialize;

/// The (single) dev container image config.
///
/// Backs `forge dev up <name>`. The fields are flat at the top level of
/// `dev-container.toml` deliberately — there is only one image in the
/// system, so wrapping it in an `[[image]]` array would be ceremony
/// around a list of length one.
///
/// Note: there is no `containerfile` field. The Containerfile is
/// embedded into the binary at compile time (see `embed::BASE_CONTAINERFILE`)
/// and materialised to a tempdir at `forge dev build` time, so the build
/// runs from any cwd without needing the repo on disk. Image-as-data
/// rather than image-as-disk-path is what makes the binary truly
/// self-contained.
#[derive(Debug, Clone, Deserialize)]
pub struct DevContainer {
    /// Image tag applied at build time and used as the `FROM` reference
    /// at container-create time. Namespaced (e.g. `forge-dev`) so it
    /// doesn't collide with unrelated images on the host.
    pub image_tag: String,
    /// Volumes attached to every container. The named-volume entry is
    /// what makes per-project state persistent across `forge dev rm`.
    #[serde(default)]
    pub default_volumes: Volumes,
    /// Forward the host's `$SSH_AUTH_SOCK` into the container. The agent
    /// holds the keys — the container only gets a socket to request
    /// signatures, never the key material itself.
    #[serde(default)]
    pub ssh_agent: bool,
}

impl DevContainer {
    /// Parse the embedded `dev-container.toml` text.
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("parsing dev-container.toml")
    }
}

/// Volume configuration: named volumes for persistent data (code, home dir)
/// and bind mounts for host files.
///
/// Named volumes use `{name}` as a placeholder for the container name, so
/// each container gets its own volume (e.g. `myapp-home` for container `myapp`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Volumes {
    #[serde(default)]
    pub named: Vec<NamedVolume>,
    #[serde(default)]
    pub bind: Vec<BindMount>,
}

/// A podman named volume. Persists across container restarts and removals.
/// Delete with `podman volume rm <name>` for a clean slate.
#[derive(Debug, Clone, Deserialize)]
pub struct NamedVolume {
    pub name: String,
    pub container: String,
}

/// A host path bind-mounted into the container.
///
/// `readonly` defaults to `false` (read-write) so an unmarked entry
/// gives the container write access to the host path — matching
/// podman's own default for `-v host:container` without a mode
/// suffix. Set `readonly = true` for safe-by-default mounts of host
/// system files like `/etc/localtime` and `/etc/timezone`, where
/// the container should only ever read the host's value, never
/// silently mutate it.
#[derive(Debug, Clone, Deserialize)]
pub struct BindMount {
    pub host: String,
    pub container: String,
    #[serde(default)]
    pub readonly: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flat dev-container.toml shape — top-level fields, optional
    /// `[default_volumes]` table — must round-trip through serde.
    /// `containerfile` is intentionally absent; the Containerfile is
    /// embedded in the binary, not loaded from disk.
    #[test]
    fn parse_dev_container() {
        let src = r#"
image_tag = "forge-dev"
ssh_agent = true

[default_volumes]
named = [{ name = "{name}-home", container = "/home/dev" }]
bind = [{ host = "~/.ssh", container = "/home/dev/.ssh:ro" }]
"#;
        let c = DevContainer::from_toml(src).unwrap();
        assert_eq!(c.image_tag, "forge-dev");
        assert!(c.ssh_agent);
        assert_eq!(c.default_volumes.named.len(), 1);
        assert_eq!(c.default_volumes.named[0].name, "{name}-home");
        assert_eq!(c.default_volumes.bind.len(), 1);
    }

    /// `[default_volumes]` is optional. Without it the struct should
    /// still parse, with an empty `Volumes` default. Surfaces here so a
    /// future minimal config doesn't silently regress.
    #[test]
    fn dev_container_default_volumes_optional() {
        let src = r#"
image_tag = "forge-dev"
"#;
        let c = DevContainer::from_toml(src).unwrap();
        assert!(c.default_volumes.named.is_empty());
        assert!(c.default_volumes.bind.is_empty());
        assert!(!c.ssh_agent);
    }

    /// Missing required fields must surface as a parse error rather
    /// than a silently-defaulted empty string — that would let a typo
    /// in `image_tag` produce a buildable-but-untagged image.
    #[test]
    fn dev_container_missing_required_fields_errors() {
        let src = r#"ssh_agent = true"#;
        assert!(DevContainer::from_toml(src).is_err());
    }

    /// Bind mounts default `readonly = false` when the field is
    /// omitted, matching podman's own default for plain
    /// `-v host:container` (no mode suffix). The explicit
    /// `readonly = true` form must round-trip.
    ///
    /// This matters because the dev-container.toml's TZ bind mounts
    /// (`/etc/localtime`, `/etc/timezone`) MUST stay read-only —
    /// under rootless podman + `--userns keep-id` a writable mount
    /// would let the container mutate host system files. If a
    /// future serde rename / refactor breaks the readonly bool
    /// silently, this assertion catches it.
    #[test]
    fn dev_container_bind_readonly_field_roundtrips() {
        let src = r#"
image_tag = "forge-dev"

[default_volumes]
bind = [
    { host = "/etc/localtime", container = "/etc/localtime", readonly = true },
    { host = "/repo",          container = "/repo" },
]
"#;
        let c = DevContainer::from_toml(src).unwrap();
        assert_eq!(c.default_volumes.bind.len(), 2);

        let tz = c
            .default_volumes
            .bind
            .iter()
            .find(|b| b.host == "/etc/localtime")
            .expect("/etc/localtime bind must round-trip");
        assert!(
            tz.readonly,
            "explicit `readonly = true` must round-trip — without it the TZ bind mounts \
             become writable under --userns keep-id"
        );

        let rw = c
            .default_volumes
            .bind
            .iter()
            .find(|b| b.host == "/repo")
            .expect("/repo bind must round-trip");
        assert!(
            !rw.readonly,
            "omitted `readonly` must default to false (matches podman's default for plain \
             `-v host:container`)"
        );
    }
}
