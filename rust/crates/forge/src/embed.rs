//! Compile-time-embedded assets: dotfiles tree and roles.toml.
//!
//! Using `include_dir!` means the finished binary has everything it needs
//! to provision a machine from scratch — no repo clone, no network fetch,
//! no config path lookup. Copy the binary onto a container or VM and it
//! Just Works.
//!
//! The tradeoff is that editing a dotfile requires a rebuild. For iterative
//! development we'll eventually add a `--dotfiles-dir` flag that reads from
//! disk instead, but embedded is the default and the shipping behavior.

use include_dir::{include_dir, Dir, DirEntry};
use std::path::{Path, PathBuf};

use forge_common::planner::DotfileSource;

/// The embedded dotfile tree. Rooted at `rust/dotfiles/` so paths inside
/// the `Dir` are already relative to `$HOME` (e.g. `.zshrc`, not
/// `dotfiles/.zshrc`).
static DOTFILES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../dotfiles");

/// The embedded `roles.toml`, loaded as a `&str` so the TOML parser can
/// consume it without any runtime IO.
pub const ROLES_TOML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../config/roles.toml"));

/// Thin adapter that lets the planner treat the embedded `Dir` as a
/// `DotfileSource`. Two non-trivial pieces here: the recursive walker
/// (`include_dir` gives you `entries()` but doesn't flatten) and the
/// lookup by relative path (we normalize the caller's path against the
/// `Dir`'s own rooted paths before comparing).
pub struct EmbeddedDotfiles;

impl EmbeddedDotfiles {
    fn walk(dir: &'static Dir<'static>, out: &mut Vec<PathBuf>) {
        for entry in dir.entries() {
            match entry {
                DirEntry::File(f) => out.push(f.path().to_path_buf()),
                DirEntry::Dir(d) => Self::walk(d, out),
            }
        }
    }
}

impl DotfileSource for EmbeddedDotfiles {
    fn list(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        Self::walk(&DOTFILES, &mut out);
        out
    }

    fn read(&self, rel: &Path) -> Option<Vec<u8>> {
        DOTFILES.get_file(rel).map(|f| f.contents().to_vec())
    }
}
