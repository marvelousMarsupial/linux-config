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

/// The embedded dotfile tree. Rooted at `dotfiles/` so paths inside
/// the `Dir` are already relative to `$HOME` (e.g. `.zshrc`, not
/// `dotfiles/.zshrc`).
static DOTFILES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../dotfiles");

/// The embedded `roles.toml`, loaded as a `&str` so the TOML parser can
/// consume it without any runtime IO.
pub const ROLES_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/roles.toml"
));

/// The theme targets / scheme overrides config.
pub const THEMES_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../themes/config.toml"
));

/// Embedded base16 scheme YAML files. Keyed by bare filename
/// (`kanagawa.yaml`), values are the raw YAML text.
pub static SCHEMES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../themes/schemes");

/// Embedded Tera templates (`foot.ini.j2` etc.).
pub static TEMPLATES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../themes/templates");

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

#[cfg(test)]
mod tests {
    use super::*;
    use forge_common::theme::ThemesConfig;
    use forge_common::{build_plan, Role, RolesFile};

    /// The embedded roles.toml must parse without errors. Catches typos,
    /// missing fields, and TOML syntax issues at test time rather than
    /// at runtime on a fresh machine.
    #[test]
    fn embedded_roles_toml_parses() {
        RolesFile::from_toml(ROLES_TOML).expect("roles.toml should parse");
    }

    /// Every role must resolve (no broken extends chains) and produce a
    /// non-empty plan when combined with the embedded dotfiles.
    #[test]
    fn all_roles_resolve_and_plan() {
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        for role in Role::all() {
            let cfg = roles
                .resolved(*role)
                .unwrap_or_else(|_| panic!("{role} should resolve"));
            let plan = build_plan(&cfg, &EmbeddedDotfiles)
                .unwrap_or_else(|_| panic!("{role} should produce a plan"));
            assert!(!plan.actions.is_empty(), "{role} plan should not be empty");
        }
    }

    /// The embedded themes.toml must parse and all referenced templates
    /// must exist in the embedded templates dir.
    #[test]
    fn embedded_themes_config_valid() {
        let cfg = ThemesConfig::from_toml(THEMES_TOML).expect("themes.toml should parse");
        for target in &cfg.targets {
            assert!(
                TEMPLATES.get_file(&target.template).is_some(),
                "template {} referenced in themes.toml but not embedded",
                target.template
            );
        }
    }

    /// Every embedded scheme must parse as valid base16 YAML.
    #[test]
    fn all_embedded_schemes_parse() {
        for name in scheme_names() {
            load_scheme(&name).unwrap_or_else(|| panic!("scheme {name} should parse"));
        }
    }

    /// Dotfiles tree should have at least the core files. Catches an
    /// accidental glob removal or rename that would otherwise only
    /// manifest as a missing file on a freshly-provisioned box.
    ///
    /// The `.local/bin/*` entries below are bound to sway keybinds:
    /// without them, the desktop role ships with broken keybindings
    /// (`$mod+w` for wifi-menu, `$mod+i` for status-panel) and the
    /// user gets no feedback when they press the key.
    #[test]
    fn embedded_dotfiles_has_core_files() {
        let files = EmbeddedDotfiles.list();
        let has = |p: &str| files.iter().any(|f| f.to_str() == Some(p));
        assert!(has(".zshrc"), "missing .zshrc");
        assert!(has(".tmux.conf"), "missing .tmux.conf");
        assert!(has(".config/nvim/init.lua"), "missing nvim init.lua");
        assert!(has(".config/starship.toml"), "missing starship.toml");
        // wifi-menu is bound to $mod+w in sway and is the primary way
        // to join a wifi network on this setup; status-panel is bound
        // to $mod+i and is the user-facing system status popup.
        assert!(has(".local/bin/wifi-menu"), "missing wifi-menu");
        assert!(has(".local/bin/status-panel"), "missing status-panel");
        // yazi keymap.toml carries the M / gm bindings for browsing
        // mounted USB drives; the mount.yazi plugin (vendored from
        // yazi-rs/plugins) is what `M` invokes — both must ship or the
        // bind silently does nothing.
        assert!(has(".config/yazi/keymap.toml"), "missing yazi keymap.toml");
        assert!(
            has(".config/yazi/plugins/mount.yazi/main.lua"),
            "missing mount.yazi plugin"
        );
        // kanshi config drives monitor hotplug behavior (laptop screen
        // off when docked, etc.); monitor-info is the helper script
        // that emits a kanshi-syntax block from currently-connected
        // displays. Without the config, kanshi exits at startup with
        // "no profiles" and monitor hotplug stops working.
        assert!(has(".config/kanshi/config"), "missing kanshi config");
        assert!(
            has(".local/bin/monitor-info"),
            "missing monitor-info helper"
        );
    }

    /// The sway autostart must launch `kanshi`, the monitor hotplug
    /// daemon, via `exec_always` (not bare `exec`).
    ///
    /// `exec` only fires on initial sway startup, so a freshly-
    /// installed kanshi (or a config change to ~/.config/kanshi/config)
    /// wouldn't take effect until the user logs out and back in —
    /// confusing footgun where everything looks correct on disk but
    /// nothing changes in the live session. `exec_always` re-fires on
    /// every `swaymsg reload`, which is what makes "edit kanshi
    /// config, swaymsg reload, see new layout" work.
    ///
    /// We also assert the autostart kills any prior kanshi instance
    /// first (`pkill ... kanshi`) — Debian's kanshi package doesn't
    /// ship `kanshi.service` or `kanshictl`, so without the pkill
    /// every reload would stack a new daemon instance fighting the
    /// previous one for the wlroots IPC.
    #[test]
    fn sway_autostarts_kanshi() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(bytes).expect("sway config should be utf8");

        let line = cfg
            .lines()
            .map(str::trim_start)
            .find(|l| l.contains("kanshi") && !l.starts_with('#'));
        let line = line.expect("sway config should autostart kanshi");

        assert!(
            line.starts_with("exec_always"),
            "kanshi autostart must use `exec_always` (not bare `exec`) so it survives \
             `swaymsg reload`; got: {line}"
        );
        assert!(
            line.contains("pkill") && line.contains("kanshi"),
            "kanshi autostart must `pkill` any prior instance before relaunching to \
             avoid stacked daemons; got: {line}"
        );
    }

    /// The desktop role must install `kanshi`. Without it the autostart
    /// line in the sway config silently fails to launch the daemon and
    /// the kanshi config goes unused. Asserts the package shows up in
    /// the resolved desktop plan as an `InstallPackage` action.
    #[test]
    fn desktop_role_installs_kanshi() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let installed = plan.actions.iter().any(|a| match a {
            Action::InstallPackage(pkg) => pkg == "kanshi",
            _ => false,
        });
        assert!(
            installed,
            "desktop role plan should install the `kanshi` apt package"
        );
    }

    /// The embedded kanshi config must define at least one `profile {`
    /// block — kanshi exits with "no profiles" if the file is empty or
    /// only comments, which would silently disable monitor hotplug
    /// management on the live system.
    #[test]
    fn kanshi_config_has_at_least_one_profile() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/kanshi/config"))
            .expect("kanshi config should be embedded");
        let cfg = String::from_utf8(bytes).expect("kanshi config should be utf8");

        // We look for an *uncommented* `profile ... {` line.
        let has_active_profile = cfg.lines().any(|l| {
            let trimmed = l.trim_start();
            !trimmed.starts_with('#') && trimmed.starts_with("profile ")
        });
        assert!(
            has_active_profile,
            "kanshi config should contain at least one uncommented `profile ... {{` block"
        );
    }

    /// The kanshi config must define both a `docked` profile (laptop +
    /// externals, with eDP-1 disabled) and a `laptop` profile (laptop
    /// alone). Without the docked profile, kanshi never blanks the
    /// laptop screen when externals are connected — which is the whole
    /// point of this config. Without the laptop profile, undocking
    /// leaves you with no profile that matches the eDP-only state and
    /// kanshi falls back to whatever sway last applied (typically
    /// nothing useful).
    ///
    /// We also assert the docked profile actually says
    /// `output eDP-1 disable` so a future edit can't accidentally
    /// re-enable the panel while leaving the profile name intact.
    #[test]
    fn kanshi_config_defines_docked_and_laptop_profiles() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/kanshi/config"))
            .expect("kanshi config should be embedded");
        let cfg = String::from_utf8(bytes).expect("kanshi config should be utf8");

        let active_profile_lines: Vec<&str> = cfg
            .lines()
            .map(str::trim_start)
            .filter(|l| !l.starts_with('#') && l.starts_with("profile "))
            .collect();
        assert!(
            active_profile_lines
                .iter()
                .any(|l| l.starts_with("profile docked")),
            "kanshi config must declare a `docked` profile; got: {active_profile_lines:?}"
        );
        assert!(
            active_profile_lines
                .iter()
                .any(|l| l.starts_with("profile laptop")),
            "kanshi config must declare a `laptop` profile; got: {active_profile_lines:?}"
        );

        // Walk into the docked block and make sure eDP-1 is explicitly
        // disabled there. Naive substring search is enough — the
        // docked block is the only place we'd write that line.
        let mut in_docked = false;
        let mut docked_disables_edp = false;
        for raw in cfg.lines() {
            let line = raw.trim();
            if line.starts_with("profile docked") {
                in_docked = true;
                continue;
            }
            if in_docked && line.starts_with('}') {
                break;
            }
            if in_docked
                && !line.starts_with('#')
                && line.contains("eDP-1")
                && line.contains("disable")
            {
                docked_disables_edp = true;
            }
        }
        assert!(
            docked_disables_edp,
            "kanshi `docked` profile must contain `output eDP-1 disable` so the laptop screen turns off when docked"
        );
    }

    /// The desktop role must install a systemd-logind drop-in pinning
    /// `HandleLidSwitchDocked=ignore` (and the related lid handlers).
    /// Defaults already happen to match what we want, but the values
    /// ship commented-out in /etc/systemd/logind.conf — a stray edit
    /// could silently break "close the lid while docked, keep
    /// working" without anything else changing.
    ///
    /// We assert the post-command writes the drop-in to
    /// /etc/systemd/logind.conf.d/sway-lid.conf and reloads logind,
    /// and that it carries an idempotency `check` so reruns don't
    /// keep rewriting the same file.
    #[test]
    fn desktop_role_pins_logind_lid_behavior() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let drop_in = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run().contains("logind.conf.d") && p.run().contains("sway-lid.conf") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect("desktop role should install a logind lid-switch drop-in");

        // The drop-in must explicitly set HandleLidSwitchDocked=ignore
        // so docked + lid close stays a no-op even if a future
        // systemd version flips the default.
        assert!(
            drop_in.run().contains("HandleLidSwitchDocked=ignore"),
            "logind drop-in must set HandleLidSwitchDocked=ignore; got: {}",
            drop_in.run()
        );
        // And reload logind so the drop-in takes effect without a
        // logout (otherwise it'd only apply on next login).
        assert!(
            drop_in.run().contains("systemctl reload systemd-logind"),
            "logind drop-in post-command must reload systemd-logind so the new values take effect immediately; got: {}",
            drop_in.run()
        );
        assert!(
            drop_in.check().is_some(),
            "logind drop-in post-command must carry a `check` so reruns are no-ops"
        );
    }

    /// The sway autostart for udiskie must use `--no-tray`, not `--tray`.
    /// This setup has no system tray daemon, so `udiskie --tray` (the
    /// default flag in older configs) silently fails to register the
    /// daemon — the user plugs in a USB and nothing happens. We assert
    /// the *string* in the embedded sway config rather than parsing it,
    /// because the sway config format isn't structured and parsing it
    /// just to check one line would be overkill.
    ///
    /// Also asserts `--automount` and `--notify` are present so the user
    /// gets the expected "drive mounted at /run/media/.../<label>" pop.
    #[test]
    fn sway_autostarts_udiskie_without_tray() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(bytes).expect("sway config should be utf8");

        // Match the autostart `exec udiskie ...` line specifically — the
        // sway config also contains `exec udiskie-umount -a` on the
        // $mod+u keybind, which would falsely match a naive
        // `contains("exec udiskie")` check.
        let line = cfg
            .lines()
            .map(str::trim_start)
            .find(|l| l.starts_with("exec udiskie ") || l == &"exec udiskie")
            .expect("sway config should autostart `exec udiskie ...`");
        assert!(
            line.contains("--no-tray"),
            "udiskie autostart must use --no-tray (no system tray on this setup); got: {line}"
        );
        assert!(
            line.contains("--automount"),
            "udiskie autostart must request --automount; got: {line}"
        );
        assert!(
            line.contains("--notify"),
            "udiskie autostart must request --notify; got: {line}"
        );
    }

    /// The desktop role must enable NetworkManager. wifi-menu (Mod+w)
    /// checks `systemctl is-active NetworkManager` first thing and bails
    /// out with a "NetworkManager not running" notification if the
    /// service isn't active, so a fresh desktop install that doesn't
    /// enable the daemon would leave wifi unreachable from the keybind.
    ///
    /// We assert both that *some* post-command enables NetworkManager
    /// (substring match on the relevant tokens — gives roles.toml a bit
    /// of authoring flexibility) and that the post-command carries a
    /// `check`, so reruns are cheap no-ops instead of pointlessly
    /// restarting the daemon.
    #[test]
    fn desktop_role_enables_networkmanager() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let nm = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run().contains("NetworkManager")
                        && p.run().contains("systemctl")
                        && p.run().contains("enable") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect("desktop role should systemctl-enable NetworkManager");
        assert!(
            nm.check().is_some(),
            "NetworkManager enable post-command must have a `check` for idempotency"
        );
    }

    /// The dev role must install rustup (so clippy / rustfmt /
    /// rust-analyzer are available on the host, matching the toolchain
    /// in `containers/rust.Containerfile`). Both the rustup-init step
    /// and the `component add clippy rustfmt` step must carry a
    /// `check` so reruns are cheap no-ops — without those checks, every
    /// `forge setup dev` rerun would re-download the rustup installer
    /// and reapply components even when they're already installed.
    ///
    /// Asserting on `sh.rustup.rs` and `rustup component add` rather
    /// than the full command string keeps the test stable against
    /// minor flag tweaks while still catching accidental regressions
    /// (e.g. someone replacing rustup with a Debian apt package and
    /// breaking clippy availability).
    #[test]
    fn dev_role_installs_rustup_with_clippy_and_rustfmt() {
        use forge_common::plan::Action;
        use forge_common::role::PostCmd;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Dev).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let post_cmds: Vec<&PostCmd> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::PostCommand(p) => Some(p),
                _ => None,
            })
            .collect();

        let init = post_cmds
            .iter()
            .find(|p| p.run().contains("sh.rustup.rs"))
            .expect("dev role should install rustup via sh.rustup.rs");
        assert!(
            init.check().is_some(),
            "rustup-init post-command must have a `check` for idempotency"
        );

        let components = post_cmds
            .iter()
            .find(|p| p.run().contains("rustup component add"))
            .expect("dev role should add clippy + rustfmt components");
        assert!(
            components.run().contains("clippy") && components.run().contains("rustfmt"),
            "component-add must include both clippy and rustfmt, got: {}",
            components.run()
        );
        assert!(
            components.check().is_some(),
            "rustup component-add must have a `check` for idempotency"
        );
    }

    /// The desktop role provisions the local Whisper dictation stack
    /// triggered by Mod+s. There are several moving parts and they
    /// have to land together — the `dictate` script is useless without
    /// the model file, the model is useless without the binary, and
    /// the binary is useless without wtype/OpenBLAS. Asserting all of
    /// that here means a fresh `forge setup desktop` either ships a
    /// working dictation keybind or fails the test suite, never the
    /// confusing middle state where Mod+s pops a notification about a
    /// missing component on the user's first dictation press.
    ///
    /// Specifically we assert the role has:
    ///   * runtime packages: `wtype` (the only thing dictate strictly
    ///     needs at runtime — wl-clipboard is no longer used since
    ///     we type the transcription directly rather than pasting it)
    ///   * build packages: `build-essential`, `cmake`, `libopenblas-dev`
    ///   * a download for a whisper.cpp GGML model
    ///   * a post-command that clones whisper.cpp (with a `check`)
    ///   * a post-command that builds it with -DGGML_BLAS=1 (with a `check`)
    ///   * a post-command that symlinks the built binaries onto PATH
    #[test]
    fn desktop_role_provisions_whisper_dictation_stack() {
        use forge_common::plan::Action;
        use forge_common::role::PostCmd;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Runtime + build packages required for the dictation flow.
        // We collect the install actions once and substring-match on
        // them rather than re-deriving the resolved package list, so
        // the test catches an accidental package removal regardless of
        // whether it was deleted from `desktop` or from a parent role.
        let installed: Vec<String> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::InstallPackage(p) => Some(p.clone()),
                _ => None,
            })
            .collect();
        for required in ["wtype", "build-essential", "cmake", "libopenblas-dev"] {
            assert!(
                installed.iter().any(|p| p == required),
                "desktop role must install `{required}` for dictation; installed = {installed:?}"
            );
        }

        // Model download. We don't pin the exact filename on disk
        // because the URL is the load-bearing part — if someone
        // changes `dest` to keep models elsewhere, the systemd unit's
        // --model arg can be updated to match. The URL substring
        // ensures we're still pulling a whisper.cpp GGML English model
        // (any of small/base/medium with optional quant suffix).
        let model_dl = plan
            .actions
            .iter()
            .find(|a| matches!(a, Action::Download { url, .. } if url.contains("ggml-") && url.contains(".en") && url.ends_with(".bin")));
        assert!(
            model_dl.is_some(),
            "desktop role must download a whisper.cpp .en GGML model; actions = {:?}",
            plan.actions
                .iter()
                .filter_map(|a| match a {
                    Action::Download { url, .. } => Some(url.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        );

        let post_cmds: Vec<&PostCmd> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::PostCommand(p) => Some(p),
                _ => None,
            })
            .collect();

        // git clone — substring on the upstream repo URL so a
        // mirror/fork rename doesn't slip through silently.
        let clone = post_cmds
            .iter()
            .find(|p| p.run().contains("git clone") && p.run().contains("whisper.cpp"))
            .expect("desktop role must clone whisper.cpp");
        assert!(
            clone.check().is_some(),
            "whisper.cpp clone post-command must have a `check` so reruns skip the network fetch"
        );

        // cmake build — assert the OpenBLAS flag is present, since
        // dropping it silently regresses encoder throughput by a
        // measurable amount (and there's no other signal that it
        // happened — the binary still works, just slower).
        let build = post_cmds
            .iter()
            .find(|p| p.run().contains("cmake --build") && p.run().contains("whisper.cpp"))
            .expect("desktop role must build whisper.cpp via cmake");
        assert!(
            build.run().contains("-DGGML_BLAS=1"),
            "whisper.cpp build must enable OpenBLAS via -DGGML_BLAS=1; got: {}",
            build.run()
        );
        assert!(
            build.check().is_some(),
            "whisper.cpp build post-command must have a `check` so reruns are no-ops"
        );

        // Symlink binaries into ~/.local/bin so the dictate script
        // and the systemd unit can reference them by short path.
        let link = post_cmds
            .iter()
            .find(|p| p.run().contains("ln -sf") && p.run().contains("whisper-server"))
            .expect("desktop role must symlink whisper-server into ~/.local/bin");
        assert!(
            link.run().contains("whisper-cli"),
            "symlink post-command should also expose whisper-cli for ad-hoc use; got: {}",
            link.run()
        );
        assert!(
            link.check().is_some(),
            "symlink post-command must have a `check` so reruns are no-ops"
        );
    }

    /// The dictate script must ship as an embedded executable and
    /// the whisper-server systemd unit must ship without an
    /// [Install] section. The latter is load-bearing — adding
    /// [Install] would make `systemctl --user enable` succeed, and
    /// a future "let me just enable this so it survives reboot"
    /// edit would silently waste ~1.7 GiB of RAM on every login by
    /// loading the medium.en model before the user has pressed
    /// any keybind.
    #[test]
    fn dictation_dotfiles_ship_correctly() {
        let dictate = EmbeddedDotfiles
            .read(Path::new(".local/bin/dictate"))
            .expect("dictate script must be embedded under .local/bin/");
        let dictate_str = String::from_utf8(dictate).expect("dictate script must be utf8");
        assert!(
            dictate_str.starts_with("#!"),
            "dictate script must start with a shebang so the planner marks it executable"
        );
        // Substring-check the subcommand dispatch so a refactor that
        // accidentally drops `cancel` or `stop-server` (both bound to
        // user-facing actions) gets caught.
        for sub in ["cancel", "stop-server"] {
            assert!(
                dictate_str.contains(sub),
                "dictate script must implement the `{sub}` subcommand"
            );
        }

        let unit = EmbeddedDotfiles
            .read(Path::new(".config/systemd/user/whisper-server.service"))
            .expect("whisper-server.service must be embedded under .config/systemd/user/");
        let unit_str = String::from_utf8(unit).expect("whisper-server.service must be utf8");
        // Look for an actual `[Install]` section header — a line that
        // is exactly that string after trimming. A bare substring
        // check would falsely trip on the inline docstring at the top
        // of the unit file that explains *why* there's no Install
        // section, which is the comment we most want to keep.
        let has_install_section = unit_str.lines().map(str::trim).any(|l| l == "[Install]");
        assert!(
            !has_install_section,
            "whisper-server.service must NOT have an [Install] section — the unit is lazy-started by dictate, enabling it at boot would defeat the point"
        );
        // The unit and the dictate script must agree on the port.
        // 8127 is the chosen value; if you move it, change both.
        assert!(
            unit_str.contains("--port 8127"),
            "whisper-server unit must bind --port 8127 to match the dictate script"
        );
        assert!(
            dictate_str.contains("8127"),
            "dictate script must reference port 8127 to match the systemd unit"
        );

        // The whisper-server unit's `--model` arg must reference the
        // exact GGML file the desktop role downloads. A mismatch
        // (e.g. someone changes the role to ggml-base.en.bin but
        // leaves the unit pointing at ggml-medium.en.bin) would let
        // `forge setup desktop` succeed silently and then make every
        // dictation press fail with "no such file" only at first
        // press — exactly the late-binding failure mode this test
        // file exists to prevent.
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let model_filename = cfg
            .downloads
            .iter()
            .find_map(|d| {
                let last = d.url.rsplit('/').next().unwrap_or("");
                if last.starts_with("ggml-") && last.ends_with(".bin") {
                    Some(last.to_string())
                } else {
                    None
                }
            })
            .expect("desktop role must declare a ggml-*.bin model download");
        assert!(
            unit_str.contains(&model_filename),
            "whisper-server unit's --model arg must reference `{model_filename}` (the file downloaded by the role); unit:\n{unit_str}"
        );
    }

    /// Sway must bind Mod+s to dictate-toggle and Mod+Ctrl+s to
    /// dictate-cancel. Without these binds the rest of the dictation
    /// stack is dead weight — there's no other way to invoke the
    /// script. We assert the *bindsym lines* exist (not just that the
    /// substrings appear somewhere in the file) to catch the failure
    /// mode where the binds get commented out during a refactor.
    #[test]
    fn sway_binds_dictate_toggle_and_cancel() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(bytes).expect("sway config should be utf8");

        let toggle = cfg
            .lines()
            .map(str::trim_start)
            .find(|l| l.starts_with("bindsym") && l.contains("$mod+s ") && l.contains("dictate"));
        assert!(
            toggle.is_some(),
            "sway config must bind $mod+s to the dictate script"
        );

        let cancel = cfg.lines().map(str::trim_start).find(|l| {
            l.starts_with("bindsym") && l.contains("$mod+Ctrl+s") && l.contains("dictate cancel")
        });
        assert!(
            cancel.is_some(),
            "sway config must bind $mod+Ctrl+s to `dictate cancel`"
        );
    }
}
