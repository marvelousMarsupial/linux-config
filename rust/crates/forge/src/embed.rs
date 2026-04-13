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

/// The theme targets / scheme overrides config.
pub const THEMES_TOML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../themes/config.toml"));

/// Embedded base16 scheme YAML files. Keyed by bare filename
/// (`kanagawa.yaml`), values are the raw YAML text.
pub static SCHEMES: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../themes/schemes");

/// Embedded Tera templates (`foot.ini.j2` etc.).
pub static TEMPLATES: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../themes/templates");

/// Template source adapter used by `theme::apply`.
pub struct EmbeddedTemplates;
impl forge_common::theme::TemplateSource for EmbeddedTemplates {
    fn read(&self, name: &str) -> Option<String> {
        TEMPLATES
            .get_file(name)
            .and_then(|f| f.contents_utf8().map(|s| s.to_string()))
    }
}

/// Return the slugs of every embedded scheme (file stems).
pub fn scheme_names() -> Vec<String> {
    let mut out: Vec<String> = SCHEMES
        .entries()
        .iter()
        .filter_map(|e| e.as_file())
        .filter_map(|f| f.path().file_stem().and_then(|s| s.to_str()))
        .map(|s| s.to_string())
        .collect();
    out.sort();
    out
}

/// Load a scheme by slug, returning `None` if not embedded.
pub fn load_scheme(slug: &str) -> Option<forge_common::theme::Scheme> {
    let fname = format!("{slug}.yaml");
    let text = SCHEMES.get_file(&fname)?.contents_utf8()?;
    forge_common::theme::Scheme::from_yaml(text).ok()
}

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
