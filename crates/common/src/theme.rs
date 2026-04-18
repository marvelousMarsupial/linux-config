//! Base16 color scheme loader and template renderer.
//!
//! A "scheme" is a base16 palette (16 named colors `base00`..`base0F`)
//! loaded from a YAML file. A "target" is one file we render from a Tera
//! template (e.g. `tmux-colors.conf.j2` → `~/.config/tmux/colors.conf`)
//! using the scheme's colors as template variables. Applying a theme is:
//!
//!   1. Load the scheme YAML, extract base00..base0F
//!   2. For each target, render its template with the palette as context
//!   3. Write the rendered output to the target path atomically
//!   4. Run any reload hook the target specifies (e.g. `swaymsg reload`)
//!
//! Template rendering uses Tera, which speaks the same `{{ var }}` /
//! `{% if %}` syntax as Jinja2 with a few filter renames. The existing
//! `.j2` templates work with minimal edits.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use crate::atomic::write_atomic;

/// A parsed base16 scheme. All 16 colors are required; missing any is an
/// error. Values are stored as 6-char hex strings *without* the leading
/// `#` so templates can choose to emit `#{{ base00 }}` or `0x{{ base00 }}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheme {
    pub scheme: String,
    #[serde(default)]
    pub author: String,
    pub base00: String,
    pub base01: String,
    pub base02: String,
    pub base03: String,
    pub base04: String,
    pub base05: String,
    pub base06: String,
    pub base07: String,
    pub base08: String,
    pub base09: String,
    #[serde(rename = "base0A")]
    pub base0a: String,
    #[serde(rename = "base0B")]
    pub base0b: String,
    #[serde(rename = "base0C")]
    pub base0c: String,
    #[serde(rename = "base0D")]
    pub base0d: String,
    #[serde(rename = "base0E")]
    pub base0e: String,
    #[serde(rename = "base0F")]
    pub base0f: String,
}

impl Scheme {
    pub fn from_yaml(s: &str) -> Result<Self> {
        serde_yaml::from_str(s).context("parsing base16 scheme YAML")
    }

    /// Build a Tera context with every base16 color as a top-level variable
    /// plus `scheme` and `author` for template headers.
    pub fn into_context(&self, extra: &BTreeMap<String, String>) -> TeraContext {
        let mut ctx = TeraContext::new();
        ctx.insert("scheme", &self.scheme);
        ctx.insert("author", &self.author);
        ctx.insert("base00", &self.base00);
        ctx.insert("base01", &self.base01);
        ctx.insert("base02", &self.base02);
        ctx.insert("base03", &self.base03);
        ctx.insert("base04", &self.base04);
        ctx.insert("base05", &self.base05);
        ctx.insert("base06", &self.base06);
        ctx.insert("base07", &self.base07);
        ctx.insert("base08", &self.base08);
        ctx.insert("base09", &self.base09);
        ctx.insert("base0A", &self.base0a);
        ctx.insert("base0B", &self.base0b);
        ctx.insert("base0C", &self.base0c);
        ctx.insert("base0D", &self.base0d);
        ctx.insert("base0E", &self.base0e);
        ctx.insert("base0F", &self.base0f);
        // Defaults for template variables that may not be in every scheme's
        // metadata. Tera requires variables to exist (unlike Jinja2), so
        // templates using `{% if nvim_plugin %}` need a falsy default.
        ctx.insert("nvim_plugin", "");
        ctx.insert("nvim_colorscheme", "");
        ctx.insert("yazi_flavor", "");
        // Scheme-specific overrides replace the defaults above.
        for (k, v) in extra {
            ctx.insert(k, v);
        }
        ctx
    }
}

/// One render target: which template to use, where to write the output,
/// and optional reload commands.
///
/// `hook` runs a shell command (e.g. `pkill -SIGUSR1 foot`).
/// `restart` names a process to kill so it restarts with new config
/// (e.g. `"Zen Browser"` → `killall "Zen Browser"`).
#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub name: String,
    pub template: String,
    pub output: String,
    #[serde(default)]
    pub hook: Option<String>,
    #[serde(default)]
    pub restart: Option<String>,
}

/// Scheme metadata overrides from `themes.toml`. Arbitrary key/value pairs
/// exposed to templates as extra context. Lets you say "when rendering
/// catppuccin-mocha, set `nvim_plugin = folke/tokyonight.nvim`".
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SchemeMeta {
    #[serde(flatten)]
    pub fields: BTreeMap<String, String>,
}

/// The whole `themes.toml` document.
#[derive(Debug, Clone, Deserialize)]
pub struct ThemesConfig {
    #[serde(default)]
    pub scheme_meta: BTreeMap<String, SchemeMeta>,
    pub targets: Vec<Target>,
}

impl ThemesConfig {
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("parsing themes.toml")
    }
}

/// Render one template string against `ctx`, returning the output bytes.
/// Exposed separately from file-writing so tests can render in-memory.
pub fn render(template_name: &str, template_src: &str, ctx: &TeraContext) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template(template_name, template_src)
        .with_context(|| format!("compiling template {template_name}"))?;
    tera.render(template_name, ctx)
        .with_context(|| format!("rendering template {template_name}"))
}

/// Expand `~/` at the start of a path using the given home dir. A thin
/// helper so we don't need `crate::paths::expand_tilde`'s `$HOME` fallback
/// (tests can pass a temp dir).
pub fn expand_home(s: &str, home: &Path) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        home.join(rest)
    } else if s == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(s)
    }
}

/// Apply a scheme to every target: render, write, and return the set of
/// files that actually changed plus the hooks that should be run.
pub fn apply(
    scheme: &Scheme,
    cfg: &ThemesConfig,
    templates: &dyn TemplateSource,
    home: &Path,
) -> Result<ApplyReport> {
    // Gather scheme-specific metadata fields (empty map if not defined).
    let meta = cfg
        .scheme_meta
        .get(&scheme_slug(&scheme.scheme))
        .cloned()
        .unwrap_or_default();
    let ctx = scheme.into_context(&meta.fields);

    let mut changed: Vec<PathBuf> = Vec::new();
    let mut hooks: Vec<String> = Vec::new();
    let mut restarts: Vec<String> = Vec::new();

    for target in &cfg.targets {
        let src = templates
            .read(&target.template)
            .ok_or_else(|| anyhow!("template not found: {}", target.template))?;
        let out = render(&target.template, &src, &ctx)?;
        let dest = expand_home(&target.output, home);
        if write_atomic(&dest, out.as_bytes(), None)? {
            changed.push(dest);
            if let Some(h) = &target.hook {
                hooks.push(h.clone());
            }
            if let Some(r) = &target.restart {
                if !restarts.contains(r) {
                    restarts.push(r.clone());
                }
            }
        }
    }

    Ok(ApplyReport { changed, hooks, restarts })
}

/// Applying a theme returns the list of files written, hooks to run, and
/// processes to restart. Running hooks/restarts is the caller's job —
/// the common crate stays side-effect free other than filesystem writes.
#[derive(Debug, Clone, Default)]
pub struct ApplyReport {
    pub changed: Vec<PathBuf>,
    pub hooks: Vec<String>,
    /// Process names to `killall` so they restart with new config.
    /// Deduplicated (e.g. two zen targets produce one restart).
    pub restarts: Vec<String>,
}

/// Normalize a scheme name to its slug (lowercased, no spaces) for
/// `scheme_meta` lookups. `"Kanagawa"` → `"kanagawa"`.
fn scheme_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

/// Source of Tera template text, keyed by template filename.
/// Keeps the theme module agnostic about whether templates come from an
/// embedded `include_dir` or a temp directory in tests.
pub trait TemplateSource {
    fn read(&self, name: &str) -> Option<String>;
}

/// In-memory template source used by tests.
#[derive(Default)]
pub struct InMemoryTemplates(pub BTreeMap<String, String>);

impl TemplateSource for InMemoryTemplates {
    fn read(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const KANAGAWA: &str = r#"
scheme: "Kanagawa"
author: "Tommaso"
base00: "1f1f28"
base01: "16161d"
base02: "223249"
base03: "727169"
base04: "c8c093"
base05: "dcd7ba"
base06: "938aa9"
base07: "363646"
base08: "c34043"
base09: "ffa066"
base0A: "e6c384"
base0B: "76946a"
base0C: "6a9589"
base0D: "7e9cd8"
base0E: "957fb8"
base0F: "d27e99"
"#;

    #[test]
    fn parses_scheme() {
        let s = Scheme::from_yaml(KANAGAWA).unwrap();
        assert_eq!(s.scheme, "Kanagawa");
        assert_eq!(s.base00, "1f1f28");
        assert_eq!(s.base0a, "e6c384");
    }

    #[test]
    fn missing_color_errors() {
        let bad = "scheme: X\nbase00: \"000000\"\n"; // only 1 of 16
        assert!(Scheme::from_yaml(bad).is_err());
    }

    #[test]
    fn renders_template_with_base_colors() {
        let s = Scheme::from_yaml(KANAGAWA).unwrap();
        let ctx = s.into_context(&BTreeMap::new());
        let out = render("t", "fg=#{{ base05 }} bg=#{{ base00 }}", &ctx).unwrap();
        assert_eq!(out, "fg=#dcd7ba bg=#1f1f28");
    }

    #[test]
    fn tera_if_branches_work() {
        let s = Scheme::from_yaml(KANAGAWA).unwrap();
        let mut extra = BTreeMap::new();
        extra.insert("mode".into(), "dark".into());
        let ctx = s.into_context(&extra);
        let tmpl = "{% if mode == 'dark' %}dark{% else %}light{% endif %}";
        assert_eq!(render("t", tmpl, &ctx).unwrap(), "dark");
    }

    #[test]
    fn expand_home_tilde_slash() {
        let h = PathBuf::from("/tmp/h");
        assert_eq!(expand_home("~/foo", &h), PathBuf::from("/tmp/h/foo"));
    }

    #[test]
    fn expand_home_bare_tilde() {
        let h = PathBuf::from("/tmp/h");
        assert_eq!(expand_home("~", &h), h);
    }

    #[test]
    fn expand_home_absolute() {
        let h = PathBuf::from("/tmp/h");
        assert_eq!(expand_home("/etc/x", &h), PathBuf::from("/etc/x"));
    }

    #[test]
    fn apply_writes_files_and_returns_report() {
        let d = tempdir().unwrap();
        let scheme = Scheme::from_yaml(KANAGAWA).unwrap();
        let mut templates = InMemoryTemplates::default();
        templates
            .0
            .insert("tmux.j2".into(), "bg=#{{ base00 }}".into());
        templates
            .0
            .insert("foot.j2".into(), "fg=#{{ base05 }}".into());

        let cfg = ThemesConfig {
            scheme_meta: BTreeMap::new(),
            targets: vec![
                Target {
                    name: "tmux".into(),
                    template: "tmux.j2".into(),
                    output: "~/tmux".into(),
                    hook: Some("tmux source".into()),
                    restart: None,
                },
                Target {
                    name: "foot".into(),
                    template: "foot.j2".into(),
                    output: "~/foot".into(),
                    hook: None,
                    restart: None,
                },
            ],
        };

        let report = apply(&scheme, &cfg, &templates, d.path()).unwrap();
        assert_eq!(report.changed.len(), 2);
        assert_eq!(report.hooks, vec!["tmux source".to_string()]);
        assert_eq!(std::fs::read_to_string(d.path().join("tmux")).unwrap(), "bg=#1f1f28");
        assert_eq!(std::fs::read_to_string(d.path().join("foot")).unwrap(), "fg=#dcd7ba");
    }

    #[test]
    fn apply_is_idempotent_on_second_run() {
        let d = tempdir().unwrap();
        let scheme = Scheme::from_yaml(KANAGAWA).unwrap();
        let mut templates = InMemoryTemplates::default();
        templates.0.insert("x.j2".into(), "bg={{ base00 }}".into());
        let cfg = ThemesConfig {
            scheme_meta: BTreeMap::new(),
            targets: vec![Target {
                name: "x".into(),
                template: "x.j2".into(),
                output: "~/x".into(),
                hook: Some("reload".into()),
                restart: None,
            }],
        };
        let r1 = apply(&scheme, &cfg, &templates, d.path()).unwrap();
        assert_eq!(r1.changed.len(), 1);
        let r2 = apply(&scheme, &cfg, &templates, d.path()).unwrap();
        assert_eq!(r2.changed.len(), 0); // unchanged — skip
        assert!(r2.hooks.is_empty()); // no hooks run when nothing changed
    }

    #[test]
    fn scheme_meta_fields_reach_template() {
        let d = tempdir().unwrap();
        let scheme = Scheme::from_yaml(KANAGAWA).unwrap();
        let mut templates = InMemoryTemplates::default();
        templates
            .0
            .insert("x.j2".into(), "{{ nvim_plugin }}".into());
        let mut scheme_meta = BTreeMap::new();
        let mut meta = SchemeMeta::default();
        meta.fields
            .insert("nvim_plugin".into(), "folke/tokyonight.nvim".into());
        scheme_meta.insert("kanagawa".into(), meta);
        let cfg = ThemesConfig {
            scheme_meta,
            targets: vec![Target {
                name: "x".into(),
                template: "x.j2".into(),
                output: "~/x".into(),
                hook: None,
                restart: None,
            }],
        };
        apply(&scheme, &cfg, &templates, d.path()).unwrap();
        assert_eq!(
            std::fs::read_to_string(d.path().join("x")).unwrap(),
            "folke/tokyonight.nvim"
        );
    }

    #[test]
    fn restart_field_collected_and_deduped() {
        let d = tempdir().unwrap();
        let scheme = Scheme::from_yaml(KANAGAWA).unwrap();
        let mut templates = InMemoryTemplates::default();
        templates.0.insert("a.j2".into(), "a".into());
        templates.0.insert("b.j2".into(), "b".into());
        let cfg = ThemesConfig {
            scheme_meta: BTreeMap::new(),
            targets: vec![
                Target {
                    name: "zen".into(),
                    template: "a.j2".into(),
                    output: "~/a".into(),
                    hook: None,
                    restart: Some("Zen Browser".into()),
                },
                Target {
                    name: "zen-flatpak".into(),
                    template: "b.j2".into(),
                    output: "~/b".into(),
                    hook: None,
                    restart: Some("Zen Browser".into()),
                },
            ],
        };
        let report = apply(&scheme, &cfg, &templates, d.path()).unwrap();
        assert_eq!(report.restarts, vec!["Zen Browser"]);
    }
}
