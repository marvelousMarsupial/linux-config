//! Parsed `dev-images.toml` and `vm-profiles.toml`.
//!
//! Both files describe lookups: given a name, find the config to use for
//! building / launching. Kept as plain data structs (no behavior) so the
//! forge binary reads them once at startup and all command handlers
//! receive a borrowed slice.

use anyhow::{Context, Result};
use serde::Deserialize;

/// One dev-container image definition.
#[derive(Debug, Clone, Deserialize)]
pub struct DevImage {
    pub name: String,
    pub containerfile: String,
    #[serde(default)]
    pub default_volumes: Volumes,
    #[serde(default)]
    pub default_ports: Vec<u16>,
    /// Forward the host's SSH agent socket into the container. The agent
    /// holds the keys — the container only gets a socket to request
    /// signatures, never the key material itself.
    #[serde(default)]
    pub ssh_agent: bool,
}

/// Volume configuration: named volumes for persistent data (code, home dir)
/// and bind mounts for host files (.ssh, .gitconfig).
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
#[derive(Debug, Clone, Deserialize)]
pub struct BindMount {
    pub host: String,
    pub container: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DevImagesFile {
    #[serde(default, rename = "image")]
    pub images: Vec<DevImage>,
}

impl DevImagesFile {
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("parsing dev-images.toml")
    }
    pub fn find(&self, name: &str) -> Option<&DevImage> {
        self.images.iter().find(|i| i.name == name)
    }
}

/// One VM / system-container profile.
#[derive(Debug, Clone, Deserialize)]
pub struct VmProfile {
    pub name: String,
    pub image: String,
    pub kind: VmKind,
    pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VmKind {
    Vm,
    Container,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VmProfilesFile {
    #[serde(default, rename = "profile")]
    pub profiles: Vec<VmProfile>,
}

impl VmProfilesFile {
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("parsing vm-profiles.toml")
    }
    pub fn find(&self, name: &str) -> Option<&VmProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dev_images() {
        let src = r#"
[[image]]
name = "base"
containerfile = "containers/base.Containerfile"
default_ports = [3000, 8080]

[image.default_volumes]
named = [{ name = "{name}-home", container = "/home/dev" }]
bind = [{ host = "~/.ssh", container = "/home/dev/.ssh:ro" }]
"#;
        let f = DevImagesFile::from_toml(src).unwrap();
        assert_eq!(f.images.len(), 1);
        let img = f.find("base").unwrap();
        assert_eq!(img.default_volumes.named.len(), 1);
        assert_eq!(img.default_volumes.bind.len(), 1);
        assert_eq!(img.default_volumes.named[0].name, "{name}-home");
        assert_eq!(img.default_ports, vec![3000, 8080]);
    }

    #[test]
    fn find_returns_none_for_missing() {
        let f = DevImagesFile { images: vec![] };
        assert!(f.find("nope").is_none());
    }

    #[test]
    fn parse_vm_profiles() {
        let src = r#"
[[profile]]
name = "net-box"
image = "images:debian/13"
kind = "vm"
role = "net-box"
"#;
        let f = VmProfilesFile::from_toml(src).unwrap();
        let p = f.find("net-box").unwrap();
        assert_eq!(p.kind, VmKind::Vm);
        assert_eq!(p.role, "net-box");
    }

    #[test]
    fn vm_kind_deserializes_lowercase() {
        let c = toml::from_str::<VmProfile>(
            r#"name="x"
image="i"
kind="container"
role="dev""#,
        )
        .unwrap();
        assert_eq!(c.kind, VmKind::Container);
    }
}
