//! Canonical paths: $HOME, config dir, state file.

use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn home() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("could not determine $HOME"))
}

pub fn config_dir() -> Result<PathBuf> {
    Ok(home()?.join(".config/forge"))
}

pub fn state_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("state.json"))
}

/// Expand a leading `~` in `s` to `$HOME`. Non-leading tildes are left alone.
pub fn expand_tilde(s: &str) -> Result<PathBuf> {
    if let Some(rest) = s.strip_prefix("~/") {
        Ok(home()?.join(rest))
    } else if s == "~" {
        home()
    } else {
        Ok(PathBuf::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_slash() {
        let home = home().unwrap();
        assert_eq!(expand_tilde("~/foo").unwrap(), home.join("foo"));
    }

    #[test]
    fn expand_tilde_bare() {
        assert_eq!(expand_tilde("~").unwrap(), home().unwrap());
    }

    #[test]
    fn expand_no_tilde() {
        assert_eq!(expand_tilde("/etc/foo").unwrap(), PathBuf::from("/etc/foo"));
    }

    #[test]
    fn expand_non_leading_tilde() {
        assert_eq!(
            expand_tilde("/tmp/~foo").unwrap(),
            PathBuf::from("/tmp/~foo")
        );
    }
}
