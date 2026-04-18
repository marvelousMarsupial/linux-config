//! Persistent "what is currently installed" state at `~/.config/forge/state.json`.
//!
//! Used for idempotency: `forge setup` reads this to skip steps that already
//! match the target state, `forge status` reads it to report, `forge teardown`
//! reads it to know what to undo.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{atomic::write_atomic, role::Role};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct State {
    pub role: Option<Role>,
    pub theme: Option<String>,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub binary_version: Option<String>,
}

impl State {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let parsed: Self = serde_json::from_str(&s)
            .with_context(|| format!("parsing state file {}", path.display()))?;
        Ok(parsed)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_vec_pretty(self).context("serializing state")?;
        write_atomic(path, &json, Some(0o644))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_missing_returns_default() {
        let d = tempdir().unwrap();
        let p = d.path().join("state.json");
        let s = State::load(&p).unwrap();
        assert_eq!(s, State::default());
    }

    #[test]
    fn roundtrip() {
        let d = tempdir().unwrap();
        let p = d.path().join("state.json");
        let s = State {
            role: Some(Role::Desktop),
            theme: Some("kanagawa".into()),
            applied_at: Some("2026-04-13T15:00:00Z".into()),
            binary_version: Some("0.1.0".into()),
        };
        s.save(&p).unwrap();
        assert_eq!(State::load(&p).unwrap(), s);
    }

    #[test]
    fn save_is_idempotent() {
        let d = tempdir().unwrap();
        let p = d.path().join("state.json");
        let s = State {
            role: Some(Role::Server),
            ..Default::default()
        };
        s.save(&p).unwrap();
        let mtime1 = std::fs::metadata(&p).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        // Saving identical state should not rewrite the file (write_atomic skips).
        s.save(&p).unwrap();
        let mtime2 = std::fs::metadata(&p).unwrap().modified().unwrap();
        assert_eq!(mtime1, mtime2);
    }
}
