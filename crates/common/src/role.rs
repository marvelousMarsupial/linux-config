//! Role definitions loaded from `roles.toml`.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Server,
    Dev,
    Desktop,
    RemoteDesktop,
    NetBox,
    AttackBox,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Server => "server",
            Role::Dev => "dev",
            Role::Desktop => "desktop",
            Role::RemoteDesktop => "remote-desktop",
            Role::NetBox => "net-box",
            Role::AttackBox => "attack-box",
        }
    }

    pub fn all() -> &'static [Role] {
        &[
            Role::Server,
            Role::Dev,
            Role::Desktop,
            Role::RemoteDesktop,
            Role::NetBox,
            Role::AttackBox,
        ]
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "server" => Ok(Role::Server),
            "dev" => Ok(Role::Dev),
            "desktop" => Ok(Role::Desktop),
            "remote-desktop" => Ok(Role::RemoteDesktop),
            "net-box" => Ok(Role::NetBox),
            "attack-box" => Ok(Role::AttackBox),
            other => Err(anyhow!("unknown role: {other}")),
        }
    }
}

/// One role's configuration as authored in `roles.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RoleConfig {
    #[serde(default)]
    pub extends: Option<Role>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub dotfiles: Vec<String>,
    #[serde(default)]
    pub downloads: Vec<Download>,
    #[serde(default)]
    pub post: Vec<PostCmd>,
    /// Commands to run during teardown (reverse of `post`). Since arbitrary
    /// shell commands can't be auto-reversed, the role author writes the
    /// inverse explicitly here.
    #[serde(default)]
    pub teardown_post: Vec<PostCmd>,
}

/// A post-install command.
///
/// Two forms are accepted in `roles.toml`:
///
/// ```toml
/// post = [
///     "chsh -s /bin/zsh",                              # bare string = always run
///     { run = "ufw enable", check = "ufw status | grep -q active" }  # skip if check exits 0
/// ]
/// ```
///
/// The checked form lets us make post-commands idempotent without
/// teaching the executor about ufw/systemctl/git specifics — the role
/// author writes the check.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PostCmd {
    Simple(String),
    Checked {
        run: String,
        #[serde(default)]
        check: Option<String>,
    },
}

impl PostCmd {
    pub fn run(&self) -> &str {
        match self {
            PostCmd::Simple(s) => s,
            PostCmd::Checked { run, .. } => run,
        }
    }
    pub fn check(&self) -> Option<&str> {
        match self {
            PostCmd::Simple(_) => None,
            PostCmd::Checked { check, .. } => check.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Download {
    pub name: String,
    pub url: String,
    pub dest: String,
    /// How to extract from an archive. Two forms:
    ///
    /// - Glob pattern (e.g. `"starship"`, `"*/yazi"`) — extract one
    ///   matching file and place it at `dest`.
    /// - `"tree:<N>"` (e.g. `"tree:1"`) — extract the entire archive to
    ///   `dest` as a directory, stripping N leading path components.
    ///   Use this for tools like nvim that need sibling dirs (lib/, share/).
    ///
    /// Without this field, the URL is downloaded directly to `dest`.
    #[serde(default)]
    pub extract: Option<String>,
    /// File mode as octal digits written in decimal (e.g. `755` in TOML
    /// means `rwxr-xr-x`). Converted to a real mode by `chmod_mode()`.
    #[serde(default)]
    pub chmod: Option<u32>,
}

impl Download {
    /// Convert the TOML `chmod` field (octal digits written as a decimal
    /// integer, e.g. 755) to a Unix mode suitable for `set_permissions`.
    pub fn chmod_mode(&self) -> Option<u32> {
        self.chmod.map(|v| {
            let s = v.to_string();
            u32::from_str_radix(&s, 8).unwrap_or(v)
        })
    }
}

/// Parsed `roles.toml` top-level document.
#[derive(Debug, Clone, Deserialize)]
pub struct RolesFile {
    #[serde(flatten)]
    pub roles: BTreeMap<Role, RoleConfig>,
}

impl RolesFile {
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("parsing roles.toml")
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let s =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Self::from_toml(&s)
    }

    /// Resolve a role including its transitive `extends` chain, with the
    /// base role's steps running before the child's.
    pub fn resolved(&self, role: Role) -> Result<RoleConfig> {
        let mut chain: Vec<Role> = Vec::new();
        let mut current = Some(role);
        while let Some(r) = current {
            if chain.contains(&r) {
                return Err(anyhow!("cycle in role extends at {r}"));
            }
            chain.push(r);
            let cfg = self
                .roles
                .get(&r)
                .ok_or_else(|| anyhow!("role {r} not defined in roles.toml"))?;
            current = cfg.extends;
        }

        // Apply base-first so leaf overrides/appends come last.
        let mut merged = RoleConfig::default();
        for r in chain.iter().rev() {
            let cfg = &self.roles[r];
            merged.packages.extend(cfg.packages.iter().cloned());
            merged.dotfiles.extend(cfg.dotfiles.iter().cloned());
            merged.downloads.extend(cfg.downloads.iter().cloned());
            merged.post.extend(cfg.post.iter().cloned());
            merged
                .teardown_post
                .extend(cfg.teardown_post.iter().cloned());
        }
        merged.extends = None;
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[server]
packages = ["zsh", "tmux"]
dotfiles = ["zsh", "tmux"]
post = ["chsh -s /bin/zsh"]

[desktop]
extends = "server"
packages = ["sway", "foot"]
dotfiles = ["sway", "foot"]

[remote-desktop]
extends = "desktop"
packages = ["tigervnc-standalone-server"]
"#;

    #[test]
    fn parses_sample() {
        let f = RolesFile::from_toml(SAMPLE).unwrap();
        assert_eq!(f.roles.len(), 3);
    }

    #[test]
    fn resolves_chain() {
        let f = RolesFile::from_toml(SAMPLE).unwrap();
        let desktop = f.resolved(Role::Desktop).unwrap();
        assert_eq!(desktop.packages, vec!["zsh", "tmux", "sway", "foot"]);
        assert_eq!(desktop.dotfiles, vec!["zsh", "tmux", "sway", "foot"]);
        assert_eq!(desktop.post.len(), 1);
        assert_eq!(desktop.post[0].run(), "chsh -s /bin/zsh");
    }

    #[test]
    fn post_cmd_parses_simple_and_checked() {
        let src = r#"
[server]
post = [
    "chsh -s /bin/zsh",
    { run = "ufw enable", check = "ufw status | grep -q active" },
]
"#;
        let f = RolesFile::from_toml(src).unwrap();
        let s = f.resolved(Role::Server).unwrap();
        assert_eq!(s.post.len(), 2);
        assert_eq!(s.post[0].run(), "chsh -s /bin/zsh");
        assert_eq!(s.post[0].check(), None);
        assert_eq!(s.post[1].run(), "ufw enable");
        assert_eq!(s.post[1].check(), Some("ufw status | grep -q active"));
    }

    #[test]
    fn resolves_three_deep() {
        let f = RolesFile::from_toml(SAMPLE).unwrap();
        let rd = f.resolved(Role::RemoteDesktop).unwrap();
        assert_eq!(
            rd.packages,
            vec!["zsh", "tmux", "sway", "foot", "tigervnc-standalone-server"]
        );
    }

    #[test]
    fn role_roundtrip() {
        for r in Role::all() {
            assert_eq!(Role::from_str(r.as_str()).unwrap(), *r);
        }
    }

    #[test]
    fn unknown_role_errors() {
        assert!(Role::from_str("nope").is_err());
    }

    #[test]
    fn missing_role_errors() {
        let f = RolesFile::from_toml("[server]\npackages = []").unwrap();
        assert!(f.resolved(Role::Desktop).is_err());
    }

    #[test]
    fn cycle_detection() {
        // Serde won't let us build a cycle via TOML deserialization easily,
        // but we can verify the resolver fails fast if somehow given one.
        let src = r#"
[server]
extends = "dev"

[dev]
extends = "server"
"#;
        let f = RolesFile::from_toml(src).unwrap();
        let err = f.resolved(Role::Server).unwrap_err();
        assert!(err.to_string().contains("cycle"));
    }
}
