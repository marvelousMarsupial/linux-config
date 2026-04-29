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

/// The dev container's Containerfile, embedded as a string at compile
/// time. Materialised to a tempdir at `forge dev build` time so the
/// build can run from any cwd — the binary alone is enough to build
/// the dev container, no repo clone needed. See
/// `commands::dev::materialize_build_context` for the dump logic.
pub const BASE_CONTAINERFILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../containers/base.Containerfile"
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

    /// The sway config must NOT autostart kanshi directly anymore.
    ///
    /// Lifetime: kanshi is now started by systemd via the user unit
    /// `~/.config/systemd/user/kanshi.service`, which is pulled in by
    /// `sway-session.target`. The session script (`~/.local/bin/
    /// sway-session`) starts that target after sway-IPC has been
    /// imported — that's what closes the subscribe-before-ready
    /// race that produced the boot-time wedge (kanshi 1.7.0 ending
    /// up with a dead wlr-output-management subscription, no
    /// profile applied, fuzzel rendering off-screen).
    ///
    /// If a future refactor adds `exec $$bin/kanshi`, `exec_always
    /// $$bin/kanshi-restart`, or any other direct-from-sway kanshi
    /// invocation back into the sway config, we end up with two
    /// kanshi processes (the systemd one and the sway-spawned
    /// one) racing for the wlr-output-management subscription —
    /// exactly the original race we just retired the bash helper
    /// to avoid. Catch that at test time.
    ///
    /// We do allow `kanshi` to appear in *comments* (the file
    /// explains the design), so we walk uncommented `exec` /
    /// `exec_always` lines specifically.
    #[test]
    fn sway_does_not_autostart_kanshi_directly() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(bytes).expect("sway config should be utf8");

        let offending = cfg.lines().map(str::trim_start).find(|l| {
            if l.starts_with('#') {
                return false;
            }
            // Match either `exec ` (trailing space) or `exec_always`
            // — both are sway's autostart directives.
            let is_exec = l.starts_with("exec ") || l.starts_with("exec_always");
            // Whitelist `exec $bin/sway-session` — that's the systemd
            // session bring-up, which has the word "session" in it
            // and doesn't reference kanshi at all.
            is_exec && (l.contains(" kanshi") || l.contains("/kanshi"))
        });
        assert!(
            offending.is_none(),
            "sway config must not autostart kanshi directly — the systemd unit \
             (~/.config/systemd/user/kanshi.service, pulled in by sway-session.target) \
             owns kanshi's lifecycle now. Found offending line: {offending:?}"
        );
    }

    /// The sway config must autostart the `sway-session` script. The
    /// script:
    ///   * exports `XDG_CURRENT_DESKTOP=sway` / `XDG_SESSION_TYPE=wayland`
    ///   * pushes `WAYLAND_DISPLAY` / `SWAYSOCK` / etc into dbus and
    ///     systemd via `dbus-update-activation-environment` and
    ///     `systemctl --user import-environment`
    ///   * starts `sway-session.target`, which `BindsTo=graphical-
    ///     session.target` and pulls in `WantedBy=sway-session.target`
    ///     units (kanshi.service in this repo)
    ///
    /// Without this `exec`, sway-session.target never starts on
    /// login, kanshi.service's dependencies are never met, and
    /// kanshi never runs — the launcher / monitor layout stay
    /// broken even though every unit file is in place. The whole
    /// systemd-orchestrated startup hinges on this one line.
    #[test]
    fn sway_autostarts_session_script() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(bytes).expect("sway config should be utf8");

        let line = cfg.lines().map(str::trim_start).find(|l| {
            !l.starts_with('#') && l.starts_with("exec ") && l.contains("sway-session")
        });
        assert!(
            line.is_some(),
            "sway config must include `exec $$bin/sway-session` to bring up the systemd \
             user session — without it, sway-session.target never starts and kanshi.service \
             (along with anything else WantedBy=sway-session.target) silently never runs"
        );
    }

    /// The `sway-session` script (`~/.local/bin/sway-session`) must
    /// ship with the three pieces that justify its existence:
    ///   * a shebang (so the planner marks it executable)
    ///   * a call into dbus + systemd to import compositor env vars
    ///     (`import-environment` is the load-bearing systemd one;
    ///     `dbus-update-activation-environment` covers dbus services)
    ///   * the `systemctl --user start sway-session.target` invocation
    ///     that pulls in kanshi.service and friends
    ///
    /// We substring-check the script source rather than evaluating
    /// it because evaluating bash from Rust would mean spawning a
    /// shell during `cargo test` — overkill when the assertions we
    /// care about are about the script's *shape*, not its runtime
    /// behavior.
    #[test]
    fn sway_session_script_ships_correctly() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".local/bin/sway-session"))
            .expect("sway-session script must be embedded under .local/bin/");
        let s = String::from_utf8(bytes).expect("sway-session must be utf8");

        assert!(
            s.starts_with("#!"),
            "sway-session must start with a shebang so the planner marks it executable"
        );

        // import-environment is what makes systemd-managed services
        // see WAYLAND_DISPLAY / SWAYSOCK / DISPLAY. Drop it and
        // kanshi.service starts with no compositor env and can't
        // talk to wlr-output-management.
        assert!(
            s.contains("systemctl --user import-environment"),
            "sway-session must `systemctl --user import-environment` so user-systemd services \
             inherit WAYLAND_DISPLAY / SWAYSOCK / DISPLAY — without that, kanshi.service \
             starts with no compositor env"
        );

        // dbus-update-activation-environment covers dbus-activated
        // services (xdg-desktop-portal, mako on some distros, etc).
        // The script tolerates the helper being absent (it's gated
        // on `command -v`), so we substring-check the call rather
        // than asserting strict presence.
        assert!(
            s.contains("dbus-update-activation-environment"),
            "sway-session must call dbus-update-activation-environment so dbus-activated \
             services see the compositor env (file pickers / portals / tray apps)"
        );

        // The target start is the load-bearing line. Without it
        // sway-session.target never activates and nothing
        // WantedBy=sway-session.target gets pulled in.
        assert!(
            s.contains("systemctl --user start") && s.contains("sway-session.target"),
            "sway-session must `systemctl --user start sway-session.target` to activate the \
             target that pulls in kanshi.service (and anything else WantedBy that target)"
        );

        // Defense against a regression where someone "simplifies"
        // by removing the already-active short-circuit: that would
        // overwrite a parallel sway session's env on a nested
        // launch. Substring on `is-active` covers any reasonable
        // form of the check.
        assert!(
            s.contains("is-active") && s.contains("sway-session.target"),
            "sway-session must short-circuit when sway-session.target is already active so a \
             nested or parallel sway session doesn't clobber the running one's env vars"
        );
    }

    /// The desktop role must ship the kanshi systemd user unit. The
    /// unit file is what replaces the retired `kanshi-restart` bash
    /// helper — it gives systemd ownership of kanshi's lifecycle
    /// (start ordering vs graphical-session.target, restart on
    /// crash, clean stop on session exit).
    ///
    /// Asserted contents:
    ///   * ExecStart points at the `kanshi` binary
    ///   * Restart= is set (so a kanshi crash mid-session gets
    ///     respawned without manual intervention)
    ///   * After=graphical-session.target — without this, the unit
    ///     can race the compositor at startup and we're back to the
    ///     subscribe-before-ready bug
    ///   * WantedBy=sway-session.target — what makes
    ///     `systemctl --user enable kanshi.service` symlink the
    ///     unit into the target's `.wants/` so it actually starts
    ///     when the target activates
    ///
    /// We don't pin Type= or RestartSec= because reasonable values
    /// vary; locking them in would block legitimate tweaks.
    #[test]
    fn desktop_role_ships_kanshi_systemd_unit() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/systemd/user/kanshi.service"))
            .expect("kanshi.service must ship under .config/systemd/user/ — without it the \
                     desktop role's kanshi-via-systemd setup is incomplete");
        let s = String::from_utf8(bytes).expect("kanshi.service must be utf8");

        assert!(
            s.contains("ExecStart=") && s.contains("kanshi"),
            "kanshi.service must declare an ExecStart that runs the kanshi binary"
        );
        assert!(
            s.contains("Restart="),
            "kanshi.service must declare a Restart= policy so a kanshi crash mid-session \
             auto-recovers without the user having to `systemctl --user restart` it"
        );
        assert!(
            s.contains("After=graphical-session.target"),
            "kanshi.service must `After=graphical-session.target` so it can't subscribe to \
             wlr-output-management before sway-IPC is ready — that ordering is the entire \
             reason we moved kanshi off sway's `exec_always` autostart"
        );
        assert!(
            s.contains("WantedBy=sway-session.target"),
            "kanshi.service must `WantedBy=sway-session.target` so `systemctl --user enable \
             kanshi.service` actually wires it into the target that the session script starts"
        );
    }

    /// The desktop role must ship the `sway-session.target` unit so
    /// the session script has something to start. Without this
    /// target file, `systemctl --user start sway-session.target`
    /// fails with "Unit sway-session.target not found" and
    /// kanshi.service (which `WantedBy`s the target) is never
    /// pulled in.
    ///
    /// Asserted contents:
    ///   * BindsTo=graphical-session.target — required by the
    ///     systemd freedesktop convention that DE-specific targets
    ///     bind to the generic graphical-session.target so portable
    ///     services with `WantedBy=graphical-session.target` get
    ///     pulled in alongside our DE-specific ones.
    #[test]
    fn desktop_role_ships_sway_session_target() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/systemd/user/sway-session.target"))
            .expect("sway-session.target must ship under .config/systemd/user/");
        let s = String::from_utf8(bytes).expect("sway-session.target must be utf8");

        assert!(
            s.contains("BindsTo=graphical-session.target"),
            "sway-session.target must `BindsTo=graphical-session.target` per the freedesktop \
             DESKTOP_ENVIRONMENTS convention — that's what makes our DE-specific target also \
             pull in upstream services that `WantedBy=graphical-session.target`"
        );
    }

    /// The desktop role's post-commands must enable the kanshi
    /// systemd unit. `WantedBy=sway-session.target` in the unit's
    /// [Install] section only takes effect after a user-side
    /// `systemctl --user enable` (which creates the symlink under
    /// `~/.config/systemd/user/sway-session.target.wants/`); without
    /// that step the unit file is on disk but kanshi never starts
    /// when the target activates.
    ///
    /// We assert via the planner so the check goes through the same
    /// extends-resolution path as runtime, catching the case where
    /// the command exists in roles.toml but is shadowed by a parent
    /// role override.
    #[test]
    fn desktop_role_enables_kanshi_systemd_unit() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let enable = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("systemctl --user enable")
                    && p.run().contains("kanshi.service") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            enable.is_some(),
            "desktop role must include a `systemctl --user enable kanshi.service` post-command \
             — without it the unit ships but never gets symlinked into \
             sway-session.target.wants/, so kanshi never starts on login"
        );
        // Idempotency: rerunning forge shouldn't keep re-enabling.
        // The check= field is what lets the executor short-circuit.
        let enable = enable.unwrap();
        assert!(
            enable.check().is_some(),
            "kanshi.service enable post-command must carry a `check` so reruns are no-ops"
        );
    }

    /// The desktop role must provision a working audio stack: the
    /// PulseAudio API shim (`pipewire-pulse`), the ALSA userspace
    /// tools (`alsa-utils`), AND a post-command that enables the
    /// `pipewire-pulse.socket` user unit.
    ///
    /// Background: this test was added after a "system has no audio"
    /// failure mode that was confusing precisely because most signals
    /// looked correct — PipeWire was running, Wireplumber was running,
    /// the headphones sink was unmuted, and the dictation stack's
    /// `pw-record` capture worked fine. The failure was that nothing
    /// was listening on `$XDG_RUNTIME_DIR/pulse/native`, so every
    /// libpulse-speaking app (browsers, media players, basically the
    /// entire audio surface that isn't dictation) couldn't reach a
    /// server and went silent. The fix had two parts that this test
    /// pins together so neither half can regress without the other:
    ///
    ///   1. Install `pipewire-pulse` so the socket and service unit
    ///      files land on disk in the first place.
    ///   2. Enable `pipewire-pulse.socket` so libpulse clients
    ///      actually find a server to connect to. Installing the
    ///      package alone is not enough — Debian's postinst leaves
    ///      socket enablement to the user.
    ///
    /// `alsa-utils` is asserted alongside because the next time
    /// audio breaks, debugging without `alsamixer`, `aplay`,
    /// `arecord`, or `speaker-test` is miserable; the role explicitly
    /// chooses to ship them rather than expect a future operator to
    /// remember to apt-install them under pressure.
    #[test]
    fn desktop_role_provisions_audio_stack() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Both audio packages must show up as InstallPackage actions
        // in the resolved desktop plan. We collect once and assert
        // membership per package so a missing package produces a
        // targeted error message rather than a vague "audio broken".
        let installed: Vec<&str> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::InstallPackage(p) => Some(p.as_str()),
                _ => None,
            })
            .collect();
        for required in [
            "pipewire-pulse",
            "alsa-utils",
            // libspa-0.2-bluetooth is the BlueZ ↔ PipeWire bridge.
            // Without it, `bluez` and `blueman` are still installed
            // and BT pairing succeeds in the GUI, but PipeWire never
            // produces a sink/source for the paired device — silent
            // BT audio is the failure mode this package prevents.
            "libspa-0.2-bluetooth",
            // rtkit lets PipeWire request realtime priority for its
            // audio threads. Skipping it is a soft-fail (PipeWire
            // logs the error and falls back to nice 0), but the
            // resulting audio crackle/dropouts under CPU load is
            // exactly the kind of intermittent issue that's hard to
            // attribute to a missing package after the fact.
            "rtkit",
        ] {
            assert!(
                installed.contains(&required),
                "desktop role must install `{required}` for audio — without \
                 pipewire-pulse the system has no PulseAudio-protocol server \
                 and most apps go silent; without alsa-utils there's no way \
                 to inspect the hardware mixer when something does break; \
                 without libspa-0.2-bluetooth BT audio devices pair but \
                 produce no PipeWire sink; without rtkit PipeWire audio \
                 threads run at nice 0 and crackle under load"
            );
        }

        // Find the post-command that enables the user-side
        // pipewire-pulse socket. Substring match on both
        // `systemctl --user enable` and `pipewire-pulse.socket`
        // (rather than a literal whole-string compare) so future
        // hardening of the command — adding flags, redirecting
        // stderr differently, etc. — doesn't break the test.
        let enable = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("systemctl --user enable")
                    && p.run().contains("pipewire-pulse.socket") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            enable.is_some(),
            "desktop role must include a `systemctl --user enable pipewire-pulse.socket` \
             post-command — installing the package is not enough; the user-side socket \
             needs explicit enablement so libpulse-speaking apps find a server at \
             $XDG_RUNTIME_DIR/pulse/native"
        );
        // Reruns of `forge setup desktop` should be no-ops once the
        // socket is enabled and active. The check= field is what lets
        // the executor short-circuit, so its absence would mean every
        // rerun re-invokes systemctl needlessly.
        let enable = enable.unwrap();
        assert!(
            enable.check().is_some(),
            "pipewire-pulse.socket enable post-command must carry a `check` so reruns \
             are cheap no-ops"
        );

        // bluetooth.service must be enabled by an explicit post-
        // command. Installing `bluez` is necessary but not
        // sufficient — Debian's postinst doesn't reliably enable
        // the unit on minimal installs, and a disabled bluetooth
        // service means the entire BT stack (bluetoothctl, blueman,
        // PipeWire's BT bridge) silently has nothing to talk to.
        let bt_enable = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("systemctl enable")
                    && p.run().contains("bluetooth.service") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            bt_enable.is_some(),
            "desktop role must include a `systemctl enable bluetooth.service` \
             post-command — without it BT pairing fails with `Failed to connect: \
             org.bluez.Error.NotReady` even though the package is installed"
        );
        // Same idempotency requirement as every other system-service
        // enable in this role: the check field lets reruns
        // short-circuit instead of re-invoking systemctl every time.
        let bt_enable = bt_enable.unwrap();
        assert!(
            bt_enable.check().is_some(),
            "bluetooth.service enable post-command must carry a `check` so reruns are \
             cheap no-ops"
        );
    }

    /// The desktop role must install `fonts-noto-color-emoji` so
    /// every app on the system can render Unicode emoji codepoints.
    ///
    /// Without this package every emoji renders as the missing-
    /// glyph "tofu" box — visible everywhere strings carry emoji
    /// (browser tabs, mako notification bodies, chat apps,
    /// file/folder names with emoji prefixes). The failure mode
    /// is silent: the underlying string is correct, fontconfig
    /// just has nothing to map the codepoints to. We hit this
    /// exact issue once, which is why this test exists — a
    /// future packages refactor that drops the entry would
    /// otherwise reintroduce the regression with no obvious
    /// signal until someone notices a specific emoji is broken.
    #[test]
    fn desktop_role_installs_emoji_font() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let installed = plan.actions.iter().any(|a| match a {
            Action::InstallPackage(pkg) => pkg == "fonts-noto-color-emoji",
            _ => false,
        });
        assert!(
            installed,
            "desktop role plan should install `fonts-noto-color-emoji` — without it \
             every emoji codepoint in the system renders as the missing-glyph tofu box"
        );
    }

    /// The desktop role must explicitly clean up the retired bash
    /// helpers (`kanshi-restart`, `kanshi-boot-kick`) from previous
    /// deploys.
    ///
    /// The forge planner only writes files we still ship; files
    /// deleted from the dotfiles tree linger on disk forever unless
    /// we remove them. Both helpers were superseded by the systemd
    /// unit + sway-session script, and leaving them on `PATH` would
    /// be confusing — a user looking for the old `kanshi-restart`
    /// would find a stale copy next to the new design.
    ///
    /// We assert there's *some* post-command that rm's both
    /// scripts (substring rather than exact-match so future
    /// hardening like adding more cleanup targets to the same `rm`
    /// doesn't break the test). The check= guard is required so
    /// reruns are cheap once the cleanup has succeeded.
    #[test]
    fn desktop_role_cleans_up_retired_kanshi_helpers() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let cleanup = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("rm")
                    && p.run().contains("kanshi-restart")
                    && p.run().contains("kanshi-boot-kick") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            cleanup.is_some(),
            "desktop role must include a post-command that rm's both retired bash helpers \
             (kanshi-restart, kanshi-boot-kick) so previous deploys don't leave stale scripts \
             on PATH alongside the new systemd-based design"
        );
        let cleanup = cleanup.unwrap();
        assert!(
            cleanup.check().is_some(),
            "retired-helper cleanup post-command must carry a `check` so reruns skip the rm \
             once the files are gone"
        );
    }

    /// Negative assertion: the retired bash helpers must NOT ship
    /// in the embedded dotfiles tree anymore. Easy to accidentally
    /// re-add them via a stray file revival; we lock them out at
    /// build time so a forge built with either back gets caught.
    #[test]
    fn retired_kanshi_helpers_not_embedded() {
        let files = EmbeddedDotfiles.list();
        for retired in [".local/bin/kanshi-restart", ".local/bin/kanshi-boot-kick"] {
            assert!(
                !files.iter().any(|f| f.to_str() == Some(retired)),
                "retired helper `{retired}` must not be embedded — kanshi is now managed by \
                 systemd via kanshi.service; the bash helper had a fd-leak bug (kanshi \
                 inheriting fd 9 of the lockfile and pinning the flock indefinitely) that's \
                 the whole reason we moved away from it"
            );
        }
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

    /// The server role must own `/etc/nftables.conf` (so an ad-hoc
    /// edit can't squat the path) but the file must NOT declare
    /// any ruleset. We rolled back the previous `inet forge_filter`
    /// table — see the comment block above the post-cmd in
    /// `config/roles.toml` for the full rationale, but the short
    /// version is that `policy drop` on input broke rootless
    /// podman + pasta UDP DNS replies, and on a single-user
    /// laptop behind NAT the firewall was net-negative anyway.
    ///
    /// Asserted invariants:
    ///   * post-command writes `/etc/nftables.conf`
    ///   * the file is forge-managed (carries the
    ///     `# Managed by forge: intentionally empty` marker)
    ///   * the file declares NO `table` blocks (no rules, no
    ///     filter chains, no input policy)
    ///   * **negative invariant**: NO post-command anywhere in
    ///     either role contains `flush ruleset`. This is still
    ///     the load-bearing assertion — even a comment-only
    ///     /etc/nftables.conf containing `flush ruleset` would
    ///     wipe netavark / libvirt / incus tables on every
    ///     reload. The kernel's nftables subsystem is shared by
    ///     every tool that programs it; we share the file but
    ///     not the rules, and `flush ruleset` would violate that
    ///     boundary.
    #[test]
    fn server_role_provisions_host_firewall() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let firewall = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p) if p.run().contains("/etc/nftables.conf") => Some(p),
                _ => None,
            })
            .expect(
                "server role should still own /etc/nftables.conf even though we no longer \
                 ship a ruleset — the path needs to be forge-managed so an ad-hoc edit \
                 can't squat it",
            );

        let run = firewall.run();

        // The file's body must carry the marker the check greps for.
        // Without it the post-cmd's idempotency check (greps for
        // this exact string) breaks and the file gets rewritten on
        // every forge run.
        assert!(
            run.contains("# Managed by forge: intentionally empty"),
            "/etc/nftables.conf body must include the `# Managed by forge: intentionally \
             empty` marker (the post-cmd's check greps for it); got: {run}"
        );

        // The file must NOT declare any ruleset. We assert on the
        // canonical nftables keywords that introduce active rules.
        // A comment containing the word `table` is fine (we're a
        // comment-heavy file) — the assertion looks for the
        // canonical `table <family> <name> {` shape, and `policy
        // drop|accept` declarations.
        for active_keyword in [
            "table inet forge_filter",
            "policy drop;",
            "type filter hook input",
        ] {
            assert!(
                !run.contains(active_keyword),
                "/etc/nftables.conf must not contain `{active_keyword}` — the host firewall \
                 was rolled back deliberately (see config/roles.toml comment block). If a \
                 future host actually needs filtering, restore via a per-host drop-in, not \
                 here. Got: {run}"
            );
        }

        let check = firewall
            .check()
            .expect("nftables.conf post-command must carry a `check` so reruns are no-ops");
        assert!(
            check.contains("/etc/nftables.conf"),
            "nftables.conf check must verify the conf file is in place; got: {check}"
        );

        // Negative invariant: flush ruleset must not appear in
        // ANY post-command in EITHER role. Still the load-bearing
        // assertion — a single `flush ruleset` slipped in
        // anywhere wipes netavark / libvirt / incus tables on
        // every nftables reload.
        for role in [Role::Server, Role::Desktop] {
            let cfg = roles.resolved(role).unwrap();
            let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();
            for action in plan.actions.iter() {
                if let Action::PostCommand(p) = action {
                    assert!(
                        !p.run().contains("flush ruleset"),
                        "no post-command in any role may use `flush ruleset` — it wipes \
                         netavark / libvirt / incus tables along with ours, breaking every \
                         container and VM on the host. Use `nft delete table <name>` to \
                         scope your reset. Offending command in {role:?} role: {}",
                        p.run()
                    );
                }
            }
        }
    }

    /// The desktop role must install and enable
    /// `power-profiles-daemon` — the laptop power-management daemon
    /// that exposes `power-saver` / `balanced` / `performance`
    /// profiles over D-Bus. Without it the box has no
    /// `powerprofilesctl` command, no battery-vs-AC tuning beyond
    /// kernel defaults, and a future status-bar widget that talks
    /// to PPD's D-Bus name (`net.hadess.PowerProfiles`) silently
    /// no-ops. Pinned together — package + enable — so a refactor
    /// that drops one or the other gets caught at `cargo test`.
    #[test]
    fn desktop_role_provisions_power_profiles_daemon() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Package half. Without it the post-command's
        // `enable --now` errors with "Unit power-profiles-daemon
        // not found".
        assert!(
            cfg.packages.iter().any(|p| p == "power-profiles-daemon"),
            "desktop role must install the `power-profiles-daemon` apt package"
        );

        let enable = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run().contains("power-profiles-daemon.service") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "desktop role must `enable --now power-profiles-daemon.service` so PPD is up \
                 in this session AND comes back after a reboot",
            );
        assert!(
            enable.run().contains("enable --now"),
            "PPD post-command must use `enable --now` (not just `start`) so the daemon comes \
             back after a reboot; got: {}",
            enable.run()
        );
        assert!(
            enable.check().is_some(),
            "PPD post-command must carry a `check` so reruns are no-ops"
        );
    }

    /// The desktop role must hand every network device to a single
    /// manager (NetworkManager). Debian's stock split is wifi via
    /// NM and wired via ifupdown → dhcpcd-base, which made the
    /// ethernet-priority drop-in dead config because NM never
    /// claimed the wire to apply it to. Three post-commands
    /// migrate to the unified setup; this test pins all three
    /// together so a future "I'll just clean these up" edit can't
    /// re-introduce the split.
    ///
    /// Asserted invariants:
    ///   * /etc/NetworkManager/conf.d/05-ifupdown-managed.conf
    ///     writes `[ifupdown] managed=true` (overrides the
    ///     `managed=false` shipped in NetworkManager.conf)
    ///   * /etc/network/interfaces gets rewritten to a
    ///     loopback-only file marked with `# Managed by forge`
    ///     (the marker is what the post-cmd's idempotency check
    ///     greps for)
    ///   * `networking.service` is `systemctl disable --now`'d
    ///     so ifupdown / dhcpcd-base aren't holding any device
    ///     across boots
    ///   * negative invariant: no post-command writes a
    ///     MAC-derived `enx` interface name into
    ///     /etc/network/interfaces (catches a regression where
    ///     someone re-adds the brittle hardcoded line we removed)
    ///   * each of the three commands carries an idempotent `check`
    #[test]
    fn desktop_role_unifies_networking_under_nm() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Helper: find a post-command whose `run` contains the
        // given marker. We match on a unique substring per step
        // so the failure message names the missing piece.
        let find = |marker: &str| -> Option<&forge_common::role::PostCmd> {
            plan.actions.iter().find_map(|a| match a {
                Action::PostCommand(p) if p.run().contains(marker) => Some(p),
                _ => None,
            })
        };

        let managed = find("/etc/NetworkManager/conf.d/05-ifupdown-managed.conf").expect(
            "desktop role must install \
             /etc/NetworkManager/conf.d/05-ifupdown-managed.conf so NM stops ceding wired \
             devices to ifupdown",
        );
        assert!(
            managed.run().contains("[ifupdown]") && managed.run().contains("managed=true"),
            "05-ifupdown-managed.conf must contain `[ifupdown]` and `managed=true`; got: {}",
            managed.run()
        );
        assert!(
            managed.check().is_some(),
            "managed=true drop-in post-command must carry a `check` so reruns are no-ops"
        );

        let interfaces = find("/etc/network/interfaces").expect(
            "desktop role must rewrite /etc/network/interfaces to loopback-only so ifupdown \
             stops claiming wired devices via the MAC-derived hardcode",
        );
        // The new file must declare loopback. Anything else is
        // a regression of the brittle hardcoded interface block.
        assert!(
            interfaces.run().contains("iface lo inet loopback"),
            "/etc/network/interfaces rewrite must include `iface lo inet loopback`; got: {}",
            interfaces.run()
        );
        // The marker comment is load-bearing (the check greps for
        // it). If someone deletes the marker the post-command's
        // idempotency check breaks silently — the `run` keeps
        // re-overwriting the file every forge run.
        assert!(
            interfaces.run().contains("# Managed by forge"),
            "/etc/network/interfaces rewrite must include the `# Managed by forge` marker — \
             the post-command's idempotency `check` greps for it; got: {}",
            interfaces.run()
        );
        assert!(
            interfaces.check().is_some(),
            "/etc/network/interfaces rewrite post-command must carry a `check` so reruns are \
             no-ops"
        );

        let disable = find("disable --now networking.service").expect(
            "desktop role must `systemctl disable --now networking.service` so ifupdown / \
             dhcpcd-base stop competing with NM for wired devices",
        );
        assert!(
            disable.check().is_some(),
            "networking.service disable post-command must carry a `check` so reruns are no-ops"
        );

        // Negative invariant: no post-command may re-introduce
        // the MAC-derived `enx*` ifupdown declaration that the
        // original Debian installer wrote and the migration
        // removes. Catches a regression where a future change
        // pastes back the old file.
        for action in plan.actions.iter() {
            if let Action::PostCommand(p) = action {
                let run = p.run();
                if run.contains("/etc/network/interfaces") {
                    assert!(
                        !run.contains("iface enx") && !run.contains("allow-hotplug enx"),
                        "no post-command may re-introduce the MAC-derived `enx*` ifupdown block \
                         to /etc/network/interfaces; got: {run}"
                    );
                }
            }
        }
    }

    /// The desktop role must install a NetworkManager drop-in pinning
    /// ethernet ahead of wifi at the routing layer.
    ///
    /// NM's compiled-in defaults already favor wired (route-metric
    /// 100) over wireless (route-metric 600), but those defaults live
    /// only as in-code constants — there is no on-disk config to
    /// diff against, so a stray `ipv4.route-metric=200` on a wifi
    /// connection profile (or a future NM upstream change) can
    /// silently flip the preference. The drop-in at
    /// `/etc/NetworkManager/conf.d/10-ethernet-priority.conf` makes
    /// the intent explicit and reproducible across boxes.
    ///
    /// We assert: the post-command writes to that exact path,
    /// declares both `[connection-ethernet]` and `[connection-wifi]`
    /// match-blocks, sets ethernet's IPv4 route-metric to 100 and
    /// wifi's to 600 (the canonical NM defaults — pinning them is
    /// the whole point), reloads NM's config so the change applies
    /// without bouncing the daemon, and carries a `check` so reruns
    /// are no-ops.
    ///
    /// Negative invariant: the drop-in must NOT contain
    /// `connection.autoconnect-priority`. Per `nm-settings(5)` that
    /// property only chooses between profiles competing for the
    /// *same device*; setting it between an ethernet and a wifi
    /// `[connection-*]` block is dead config (cross-device
    /// activation is independent), and the actual exclusive-
    /// failover behavior the user wants is implemented by the
    /// dispatcher script the next test pins. If a future change
    /// re-introduces `autoconnect-priority` here, this assertion
    /// catches it before it ships.
    #[test]
    fn desktop_role_pins_networkmanager_ethernet_priority() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Identify the post-command by the unique conf-file path so a
        // future unrelated NM tweak won't accidentally match.
        let drop_in = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run()
                        .contains("/etc/NetworkManager/conf.d/10-ethernet-priority.conf") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "desktop role should install an NM drop-in pinning ethernet over wifi at \
                 /etc/NetworkManager/conf.d/10-ethernet-priority.conf",
            );

        let run = drop_in.run();

        // Both match blocks must be present — without one of them the
        // drop-in would only pin half of the priority pair, leaving
        // the other side at NM's compile-time default and undermining
        // the whole reproducibility argument for shipping the file.
        assert!(
            run.contains("[connection-ethernet]"),
            "NM drop-in must contain a [connection-ethernet] section; got: {run}"
        );
        assert!(
            run.contains("[connection-wifi]"),
            "NM drop-in must contain a [connection-wifi] section; got: {run}"
        );
        assert!(
            run.contains("match-device=type:ethernet"),
            "NM drop-in must scope the ethernet section by `match-device=type:ethernet`; got: {run}"
        );
        assert!(
            run.contains("match-device=type:wifi"),
            "NM drop-in must scope the wifi section by `match-device=type:wifi`; got: {run}"
        );

        // Pin the canonical NM defaults: 100 for wired (preferred,
        // low metric wins), 600 for wireless. If someone "improves"
        // these numbers without thinking, this test catches it
        // before the change ships.
        assert!(
            run.contains("ipv4.route-metric=100"),
            "NM drop-in must pin ethernet ipv4.route-metric=100 (preferred default route); got: {run}"
        );
        assert!(
            run.contains("ipv4.route-metric=600"),
            "NM drop-in must pin wifi ipv4.route-metric=600 (fallback default route); got: {run}"
        );

        // Reload picks up conf.d changes without bouncing NM, which
        // matters for live updates: the user keeps their current
        // wifi association while the new defaults apply to the next
        // activation.
        assert!(
            run.contains("nmcli general reload conf"),
            "NM drop-in post-command must reload NM config so the new defaults take effect \
             without restarting the daemon; got: {run}"
        );
        assert!(
            drop_in.check().is_some(),
            "NM drop-in post-command must carry a `check` so reruns are no-ops"
        );

        // Negative invariant: cross-device autoconnect-priority is
        // dead config (per nm-settings(5) it only ranks profiles on
        // the same device), so the drop-in must not pin it. The
        // actual "ethernet preferred" exclusive-failover behavior
        // is implemented by the dispatcher script the next test
        // covers — keep these two concerns split.
        assert!(
            !run.contains("connection.autoconnect-priority"),
            "NM drop-in must NOT pin `connection.autoconnect-priority` between ethernet and \
             wifi — cross-device autoconnect-priority is a no-op (nm-settings(5)); the \
             dispatcher script handles exclusive failover instead. Got: {run}"
        );

        // Migration guard: when forge runs against a host that has
        // a previous version of this drop-in on disk (one with the
        // deprecated `autoconnect-priority` lines), a plain
        // `test -f` check would let the old file linger forever
        // because the path still exists. The check must include a
        // *negated* `grep -q autoconnect-priority` against the
        // file so forge rewrites it when the deprecated body is
        // detected. If a future edit drops this guard the
        // migration regresses silently — this assertion catches
        // that before it ships.
        let check = drop_in.check().expect("drop-in must carry a check (asserted above)");
        assert!(
            check.contains("! grep -q autoconnect-priority"),
            "NM drop-in `check` must include `! grep -q autoconnect-priority` so forge \
             rewrites the file on hosts where a previous version's `autoconnect-priority` \
             lines are still on disk. Got: {check}"
        );
    }

    /// The desktop role must install a NetworkManager dispatcher
    /// script that gives the host genuine "ethernet when plugged
    /// in, wifi otherwise" behavior — the user-visible laptop UX.
    ///
    /// Why a dispatcher rather than just the route-metric drop-in:
    /// kernel route-metric prefers ethernet correctly (NM defaults
    /// already do this), but `getaddrinfo()` / `connect()` source-
    /// address selection per RFC 6724 picks the lower-ifindex
    /// interface for same-subnet destinations, which on a host
    /// where wifi and ethernet share a subnet can be wifi even when
    /// ethernet has the lower metric. Result: app-level outbound
    /// binds use the wifi IP and reply traffic follows the source
    /// address back over wifi. The dispatcher fixes this
    /// structurally by removing the wifi connection (and its IP)
    /// while ethernet is up, and reconnecting wifi when ethernet
    /// goes away. See the post-cmd's block comment in
    /// `config/roles.toml` for the full rationale.
    ///
    /// We assert:
    ///   * a post-command writes
    ///     `/etc/NetworkManager/dispatcher.d/10-wifi-when-no-ethernet`
    ///   * the body filters on `en*`/`eth*` at the top (loop
    ///     prevention — without this the wifi-up event from our
    ///     own `device connect` recurses)
    ///   * both `up)` and `down)` action branches are present
    ///   * the `up` branch calls `nmcli device disconnect` (the
    ///     canonical "stop using this device, but leave the
    ///     radio on so the user can override manually" command)
    ///   * the `down` branch calls `nmcli device connect` (lifts
    ///     the manually-disconnected flag so NM autoconnect can
    ///     pick the highest-priority eligible wifi profile)
    ///   * the `down` branch short-circuits if any other ethernet
    ///     is still active (matches `802-3-ethernet` in
    ///     `connection show --active`) — a docked laptop with two
    ///     wired interfaces shouldn't un-disable wifi when one
    ///     cable is unplugged
    ///   * the post-command `chmod 0755`s the file (NM silently
    ///     skips dispatcher scripts that aren't 0755 — a confusing
    ///     failure mode logged at debug level only)
    ///   * the post-command carries a `check` so reruns are no-ops
    #[test]
    fn desktop_role_installs_wifi_failover_dispatcher() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Identify the post-command by the unique dispatcher path
        // so a future unrelated NM change won't accidentally match.
        let dispatcher = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run().contains(
                        "/etc/NetworkManager/dispatcher.d/10-wifi-when-no-ethernet",
                    ) =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "desktop role should install an NM dispatcher script at \
                 /etc/NetworkManager/dispatcher.d/10-wifi-when-no-ethernet for exclusive \
                 ethernet-preferred failover",
            );

        let run = dispatcher.run();

        // Loop prevention: the script must filter on en*/eth* before
        // doing anything, otherwise the wifi-up event NM fires when
        // we re-enable wifi recurses through this same script and
        // we end up in a tight enable/disable cycle.
        assert!(
            run.contains("en*|eth*"),
            "dispatcher script must filter on `en*|eth*` at the top to avoid recursing on \
             wifi-up events triggered by its own `device connect`; got: {run}"
        );

        // Both action branches must be present. Without `up)` the
        // "ethernet preferred" half does nothing; without `down)`
        // wifi never comes back when the cable is unplugged.
        assert!(
            run.contains("  up)"),
            "dispatcher script must handle the `up)` action (disconnects wifi when ethernet \
             comes up); got: {run}"
        );
        assert!(
            run.contains("  down)"),
            "dispatcher script must handle the `down)` action (reconnects wifi when ethernet \
             goes away); got: {run}"
        );

        // The two nmcli verbs that drive the actual behavior. We
        // deliberately use `device disconnect/connect` rather than
        // `radio off/on` so the wifi card keeps scanning and the
        // user can manually override the dispatcher with
        // `nmcli device connect <wifi>` if they want both
        // interfaces active at once.
        assert!(
            run.contains("nmcli device disconnect"),
            "dispatcher script's `up` branch must call `nmcli device disconnect` to drop the \
             wifi IP while ethernet is up; got: {run}"
        );
        assert!(
            run.contains("nmcli device connect"),
            "dispatcher script's `down` branch must call `nmcli device connect` to reactivate \
             wifi when the last ethernet leaves; got: {run}"
        );

        // The down-branch short-circuit: if another ethernet
        // connection is still active, don't un-disable wifi. This
        // pattern matches the connection.show --active output for
        // 802-3-ethernet so a docked laptop with two wired
        // interfaces behaves correctly when only one cable is
        // unplugged. Asserting on the literal substring keeps a
        // future edit from accidentally removing the guard.
        assert!(
            run.contains("802-3-ethernet"),
            "dispatcher script must short-circuit the `down` branch when another \
             802-3-ethernet connection is still active (multi-ethernet docking case); got: {run}"
        );

        // chmod 0755 is mandatory: NM silently skips dispatcher
        // scripts whose mode is not exactly 0755 (logged at debug
        // only). A common failure mode is "I wrote the script and
        // nothing happens" — this assertion guards against an
        // edit that drops the chmod.
        assert!(
            run.contains("chmod 0755"),
            "dispatcher script post-command must `chmod 0755` the file — NM silently skips \
             dispatcher scripts that aren't 0755; got: {run}"
        );

        assert!(
            dispatcher.check().is_some(),
            "dispatcher script post-command must carry a `check` so reruns are no-ops"
        );
    }

    /// The server role must install `unattended-upgrades` AND ship the
    /// two apt drop-ins that actually turn it on
    /// (`/etc/apt/apt.conf.d/20auto-upgrades` for the periodic flags
    /// and `/etc/apt/apt.conf.d/52unattended-upgrades-local.conf` for
    /// the local policy). Without both halves, the package installs
    /// quietly and the apt-daily-upgrade.timer fires every day to do
    /// nothing — which is exactly the "looks configured, isn't" trap
    /// we're guarding against here.
    ///
    /// We assert:
    ///   * package present
    ///   * 20auto-upgrades drop-in writes all four periodic keys
    ///     (Update-Package-Lists, Download-Upgradeable-Packages,
    ///     Unattended-Upgrade, AutocleanInterval) — losing any one of
    ///     them silently degrades the upgrade behavior, so they're
    ///     pinned as a set
    ///   * 52unattended-upgrades-local.conf opts in to the stable +
    ///     updates suites (Origins-Pattern), pins Automatic-Reboot
    ///     false, and sets MinimalSteps true
    ///   * both drop-ins carry a `check` so reruns are no-ops
    #[test]
    fn server_role_provisions_unattended_upgrades() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Package half. Without this, neither drop-in matters because
        // there's no /usr/bin/unattended-upgrade for apt-daily-upgrade
        // to invoke.
        assert!(
            cfg.packages.iter().any(|p| p == "unattended-upgrades"),
            "server role must install the `unattended-upgrades` package; without it the apt \
             drop-ins below have nothing to drive",
        );

        let periodic = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run().contains("/etc/apt/apt.conf.d/20auto-upgrades") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "server role should install /etc/apt/apt.conf.d/20auto-upgrades so the \
                 apt-daily-upgrade.timer actually runs unattended-upgrade",
            );

        let periodic_run = periodic.run();
        for key in [
            "APT::Periodic::Update-Package-Lists \"1\"",
            "APT::Periodic::Download-Upgradeable-Packages \"1\"",
            "APT::Periodic::Unattended-Upgrade \"1\"",
            "APT::Periodic::AutocleanInterval \"7\"",
        ] {
            assert!(
                periodic_run.contains(key),
                "20auto-upgrades drop-in must set `{key}` so the daily upgrade flow stays \
                 fully configured; got: {periodic_run}"
            );
        }
        assert!(
            periodic.check().is_some(),
            "20auto-upgrades drop-in post-command must carry a `check` so reruns are no-ops"
        );

        let local = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run()
                        .contains("/etc/apt/apt.conf.d/52unattended-upgrades-local.conf") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "server role should install \
                 /etc/apt/apt.conf.d/52unattended-upgrades-local.conf with the local policy \
                 overrides on top of the package's shipped 50unattended-upgrades",
            );

        let local_run = local.run();
        // Origins: opt in to stable + updates suites so point-release
        // fixes land alongside security updates. The shipped
        // 50unattended-upgrades is security-only — without these
        // appends, the box would still skip every minor point release.
        assert!(
            local_run.contains("codename=${distro_codename},label=Debian"),
            "local drop-in must add the Debian stable suite to Origins-Pattern; got: {local_run}"
        );
        assert!(
            local_run.contains("codename=${distro_codename}-updates"),
            "local drop-in must add the Debian -updates suite to Origins-Pattern; got: {local_run}"
        );
        // Automatic-Reboot false is also the stock default, but
        // pinning it stops a future package upgrade from silently
        // flipping the value (the user prefers to choose reboot
        // windows themselves).
        assert!(
            local_run.contains("Unattended-Upgrade::Automatic-Reboot \"false\""),
            "local drop-in must pin Automatic-Reboot=false; got: {local_run}"
        );
        // MinimalSteps makes a Ctrl-C / power-loss during the upgrade
        // resumable on the next run instead of leaving a half-
        // configured package; load-bearing for an unattended flow.
        assert!(
            local_run.contains("Unattended-Upgrade::MinimalSteps \"true\""),
            "local drop-in must enable MinimalSteps=true so interrupted runs resume cleanly; \
             got: {local_run}"
        );
        assert!(
            local.check().is_some(),
            "52unattended-upgrades-local.conf post-command must carry a `check` so reruns are \
             no-ops"
        );
    }

    /// The desktop role must install a system-level systemd timer that
    /// runs `flatpak update` daily as root, so the system-installed
    /// flatpaks (Zen Browser, Obsidian) keep themselves current
    /// without manual intervention. The pair lives at
    /// `/etc/systemd/system/flatpak-update.{service,timer}` (system
    /// units, not user units, because system flatpak installs can
    /// only be updated by root).
    ///
    /// We assert: both unit files are written, the service runs the
    /// expected `flatpak update` command, the timer fires daily and
    /// is `Persistent=true` (so a missed run on a powered-off laptop
    /// runs once on next boot), the post-command reloads systemd and
    /// `enable --now`s the timer, and it carries a `check` so reruns
    /// are no-ops.
    #[test]
    fn desktop_role_provisions_flatpak_update_timer() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Match by the unique timer path so the test isn't fooled by
        // an unrelated future edit that touches some other systemd
        // unit.
        let timer = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run()
                        .contains("/etc/systemd/system/flatpak-update.timer") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "desktop role should install the flatpak-update systemd timer pair under \
                 /etc/systemd/system/flatpak-update.{service,timer}",
            );

        let run = timer.run();

        // Both unit files must be written — service alone won't fire
        // on a schedule, timer alone has nothing to invoke.
        assert!(
            run.contains("/etc/systemd/system/flatpak-update.service"),
            "post-command must write the flatpak-update.service unit; got: {run}"
        );
        assert!(
            run.contains("/etc/systemd/system/flatpak-update.timer"),
            "post-command must write the flatpak-update.timer unit; got: {run}"
        );

        // Service body must run flatpak update non-interactively.
        // `-y --noninteractive` is the documented combination that
        // suppresses both prompts and progress UI for unattended use.
        assert!(
            run.contains("flatpak update -y --noninteractive"),
            "service must invoke `flatpak update -y --noninteractive`; got: {run}"
        );

        // Timer body: daily, persistent (catch up after suspend /
        // power-off), randomized for fleet etiquette even though this
        // box is one-of-one.
        assert!(
            run.contains("OnCalendar=daily"),
            "timer must fire daily; got: {run}"
        );
        assert!(
            run.contains("Persistent=true"),
            "timer must be Persistent=true so missed runs catch up after suspend / shutdown; \
             got: {run}"
        );

        // Reload + enable+start. Without daemon-reload, systemd
        // doesn't see the freshly-written units; without `enable
        // --now`, the timer is on disk but not scheduled.
        assert!(
            run.contains("systemctl daemon-reload"),
            "post-command must `systemctl daemon-reload` after writing the units; got: {run}"
        );
        assert!(
            run.contains("systemctl enable --now flatpak-update.timer"),
            "post-command must `enable --now flatpak-update.timer` so the timer is both \
             persistent across boots and active in the current session; got: {run}"
        );

        assert!(
            timer.check().is_some(),
            "flatpak-update timer post-command must carry a `check` so reruns are no-ops"
        );
    }

    /// `flatpak-update.service` runs `flatpak update` as root because
    /// system-scope flatpak installs live under /var/lib/flatpak. The
    /// root authority is necessary; the rest of the privileges that
    /// come with it are not. This test pins the systemd sandbox
    /// directives that contain the blast radius if flatpak (or any of
    /// its deps) ever has a privilege-escalation bug.
    ///
    /// The directives we require are all classic systemd hardening
    /// tools that work with `flatpak update` specifically:
    ///   * NoNewPrivileges — kills the setuid escape route
    ///   * PrivateTmp — isolates /tmp from shared-tmp attacks
    ///   * ProtectSystem=strict + ReadWritePaths=/var/lib/flatpak —
    ///     the kernel refuses any write outside the OSTree repo
    ///   * ProtectHome=read-only — system update has no business
    ///     writing $HOME
    ///   * ProtectKernelTunables / ProtectKernelModules /
    ///     ProtectControlGroups / ProtectClock — block kernel-state
    ///     modification syscalls flatpak never needs
    ///   * LockPersonality / RestrictRealtime — close two more
    ///     niche escape routes
    ///
    /// We do NOT assert RestrictNamespaces — flatpak's update path
    /// may use bwrap (and thus namespaces) for triggers, and
    /// over-constraining that breaks updates without a clear
    /// security win.
    ///
    /// If a future edit deletes any of the asserted directives the
    /// test fails — the goal is to make weakening this service a
    /// loud, deliberate decision instead of a silent simplification.
    #[test]
    fn desktop_role_hardens_flatpak_update_service() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Same matcher as desktop_role_provisions_flatpak_update_timer
        // — find the post-command that writes the unit pair.
        let timer = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run()
                        .contains("/etc/systemd/system/flatpak-update.service") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "desktop role should install flatpak-update.service alongside the timer; if \
                 this expect fires the provisions test will too",
            );

        let run = timer.run();

        // Each directive is asserted individually so the failure
        // message names the specific knob someone removed, not a
        // vague "hardening missing" blob.
        for directive in [
            "NoNewPrivileges=yes",
            "PrivateTmp=yes",
            "ProtectSystem=strict",
            "ReadWritePaths=/var/lib/flatpak",
            "ProtectHome=read-only",
            "ProtectKernelTunables=yes",
            "ProtectKernelModules=yes",
            "ProtectControlGroups=yes",
            "ProtectClock=yes",
            "LockPersonality=yes",
            "RestrictRealtime=yes",
        ] {
            assert!(
                run.contains(directive),
                "flatpak-update.service must include systemd hardening directive `{directive}`; \
                 if you intentionally removed it, update both the service definition and this \
                 test in the same commit. got: {run}"
            );
        }
    }

    /// The desktop role's system-scope flatpak post-commands must run
    /// via `sudo`, not bare.
    ///
    /// Plain `flatpak remote-add` / `flatpak install` (no `--user`,
    /// no `--system` flag) defaults to the system installation —
    /// /var/lib/flatpak, root-owned. The non-root path goes through
    /// polkit (`org.freedesktop.Flatpak.app-install`,
    /// `…manage-installation`), which on a graphical session pops a
    /// password dialog via the user's polkit auth agent. forge runs
    /// post-commands non-interactively, so polkit returns
    /// `error: Flatpak system operation … not allowed for user` and
    /// the executor aborts the entire desktop plan.
    ///
    /// Wrapping with `sudo` short-circuits polkit (root is already
    /// authorized) and the documented NOPASSWD sudoers entry on the
    /// user's host means sudo doesn't prompt either. Same observable
    /// result, no interactive prompts.
    ///
    /// Caught by running the desktop role inside a clean Debian
    /// trixie LXC: bare `flatpak remote-add … flathub …` exited 1
    /// with `Flatpak system operation ConfigureRemote not allowed
    /// for user`. Same class of bug as the `chsh` PAM gotcha pinned
    /// by `server_role_chsh_uses_sudo`.
    ///
    /// Note: the `flatpak override --user …` post-commands lower
    /// down in the role intentionally do NOT take sudo because
    /// `--user` writes to `~/.local/share/flatpak/overrides` and
    /// never touches system state. This test only asserts the
    /// system-scope ones (no `--user` flag).
    #[test]
    fn desktop_role_system_flatpak_ops_use_sudo() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Collect every post-command that invokes flatpak. We then
        // partition by `--user` to separate the per-user override
        // commands (no sudo needed) from the system-scope remote /
        // install commands (sudo required).
        let flatpak_cmds: Vec<&str> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::PostCommand(p) if p.run().contains("flatpak ") => Some(p.run()),
                _ => None,
            })
            .collect();

        assert!(
            !flatpak_cmds.is_empty(),
            "desktop role should include at least one flatpak post-command"
        );

        // Find the system-scope ones: bare `flatpak <subcmd>` with
        // no `--user` AND no `--system` (latter would be redundant
        // but still system scope). The flatpak-update.service
        // post-cmd embeds a `flatpak update` invocation as part of
        // an ExecStart line — that runs as root inside the
        // systemd-managed unit, NOT directly via the executor, so
        // skip any line that's writing to /etc/systemd/system.
        for run in &flatpak_cmds {
            let is_system_scope = !run.contains("--user");
            let is_unit_definition = run.contains("/etc/systemd/system/flatpak-update");
            if !is_system_scope || is_unit_definition {
                continue;
            }
            assert!(
                run.starts_with("sudo "),
                "system-scope flatpak post-command must be invoked via `sudo` so polkit \
                 short-circuits and forge's non-interactive run doesn't fail with `Flatpak \
                 system operation … not allowed for user`; got `{run}`"
            );
        }
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

    /// The `server` role must ship `ncurses-term` so SSHing into the
    /// host from a foot terminal lands in a session whose TERM the
    /// remote's ncurses library actually knows about. Without this,
    /// vim / yazi / less / tmux on the remote fall back to
    /// minimal/wrong escape sequences and render badly — even though
    /// the host's foot is fine. On Debian trixie ncurses-term
    /// `Provides: foot-terminfo` (the standalone foot-terminfo apt
    /// package is now an empty transitional shim), so installing this
    /// one package is what the canonical fix looks like.
    ///
    /// Pinned here so a future role-trim can't drop the package
    /// without something failing the test, since the symptom (TUI
    /// apps look subtly wrong on a freshly-provisioned remote) is
    /// the kind of thing that gets shipped and noticed weeks later.
    #[test]
    fn server_role_provisions_foot_terminfo() {
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        assert!(
            cfg.packages.iter().any(|p| p == "ncurses-term"),
            "server role must include `ncurses-term` so SSHing into the host from a foot \
             terminal works end-to-end without manual terminfo pushing — see the \
             `fix` helper in .zsh_aliases for the manual fallback for \
             non-forge-managed remotes"
        );
    }

    /// The `server` role must ship the rootless-podman runtime stack
    /// so every host (server and desktop, the only roles, both via
    /// extends-from-server) can spin up `forge dev` containers. This
    /// is the load-bearing rationale for keeping container packages
    /// in the base role — dev pods are universal, not opt-in.
    ///
    /// Pin every package in the runtime stack:
    /// * `podman` — the container runtime itself
    /// * `uidmap` — newuidmap/newgidmap setuid helpers; rootless
    ///   containers fail at create time without them
    /// * `passt` — provides `pasta`, the rootless network stack
    ///   podman 5+ uses by default. Without it, `podman build` /
    ///   `podman run` error out at namespace setup with `could not
    ///   find pasta, the network namespace can't be configured` —
    ///   we hit that exact failure mode the first time we tried
    ///   `forge dev build` on a freshly-provisioned trixie host,
    ///   which is what motivated pinning `passt` in this test.
    /// * `slirp4netns` — older rootless network stack, kept as a
    ///   fallback for the few podman commands that still want it
    /// * `fuse-overlayfs` — overlay storage driver that works
    ///   without root; without it podman falls back to vfs and
    ///   copies the whole image on every layer
    /// * `nftables` — ships the `nft` CLI that netavark (podman's
    ///   network backend on trixie) shells out to for firewall /
    ///   NAT rule programming. Without it, `podman build` and
    ///   `podman run` fail deep inside the first `RUN` step with
    ///   "netavark: nftables error: unable to execute nft: No
    ///   such file or directory" — confusing because the failure
    ///   appears after image pull, not at podman invocation
    ///   time. We hit this on the freshly-provisioned incus VM
    ///   that motivated the test loop.
    ///
    /// Dropping any one of these would leave a host where forge
    /// looks installed but `forge dev` silently fails. This test is
    /// the gate.
    #[test]
    fn server_role_provisions_container_runtimes() {
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();

        for pkg in [
            "podman",
            "uidmap",
            "passt",
            "slirp4netns",
            "fuse-overlayfs",
            "nftables",
        ] {
            assert!(
                cfg.packages.iter().any(|p| p == pkg),
                "server role must include `{pkg}` so every machine that extends it (desktop) \
                 ends up with the dev-container runtime stack on disk; dropping it would \
                 leave `forge dev` silently broken"
            );
        }
    }

    /// The desktop role must NOT install a Rust toolchain on the host.
    /// Forge is rebuilt inside a dev container (`forge dev up forge-build`)
    /// and the binary is `podman cp`'d to the host — this keeps the
    /// desktop free of ~/.cargo, ~/.rustup, and the few hundred MiB
    /// they pull in. Pinned as a negative invariant so a future edit
    /// (or a stale apt package install) can't silently re-add rustup
    /// to the host's setup path.
    ///
    /// We check both the post-command list (no `sh.rustup.rs`, no
    /// `rustup component add`) AND the apt package list (no `rustc` /
    /// `cargo` / `rustfmt` / `clippy` from the Debian repository).
    #[test]
    fn desktop_role_does_not_install_rust_toolchain() {
        use forge_common::plan::Action;
        use forge_common::role::PostCmd;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let post_cmds: Vec<&PostCmd> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::PostCommand(p) => Some(p),
                _ => None,
            })
            .collect();
        for forbidden_substr in ["sh.rustup.rs", "rustup component add", "rustup-init"] {
            assert!(
                !post_cmds.iter().any(|p| p.run().contains(forbidden_substr)),
                "desktop role must NOT have a post-command containing `{forbidden_substr}` — \
                 the host stays Rust-free; forge gets rebuilt in a dev container and copied over"
            );
        }

        let installed: Vec<String> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::InstallPackage(p) => Some(p.clone()),
                _ => None,
            })
            .collect();
        for forbidden_pkg in ["rustc", "cargo", "rustfmt"] {
            assert!(
                !installed.iter().any(|p| p == forbidden_pkg),
                "desktop role must NOT install apt package `{forbidden_pkg}` — Rust toolchain \
                 lives in dev containers, not on the host"
            );
        }
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
    /// * runtime packages: `wtype` (the only thing dictate strictly
    ///   needs at runtime — wl-clipboard is no longer used since
    ///   we type the transcription directly rather than pasting it)
    /// * dynamic-link runtime packages for the vendored whisper-server
    ///   binary: `libopenblas0`, `libgomp1` (inherited from `server`).
    ///   These replace the previous `build-essential` + `cmake` +
    ///   `libopenblas-dev` build-toolchain pin, since we now ship a
    ///   prebuilt static-libwhisper whisper-server in the dotfile tree.
    /// * a download for a whisper.cpp GGML model
    /// * vendored `whisper-server` and `whisper-cli` binaries embedded
    ///   in the dotfile tree under `.local/bin/`. The planner's ELF
    ///   detection writes them executable on `forge setup`.
    /// * NO `git clone` / `cmake build` / `ln -sf` post-commands.
    ///   Building from source on each host was the old shape — keeping
    ///   the negative assertion catches an accidental revival.
    #[test]
    fn desktop_role_provisions_whisper_dictation_stack() {
        use forge_common::plan::Action;
        use forge_common::role::PostCmd;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Desktop).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Runtime packages required for the dictation flow.
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
        for required in ["wtype", "libopenblas0", "libgomp1"] {
            assert!(
                installed.iter().any(|p| p == required),
                "desktop role must install `{required}` for dictation runtime; \
                 installed = {installed:?}"
            );
        }
        // Negative pin: the build chain must NOT be there. We ship a
        // prebuilt whisper-server now; if a future edit re-introduces
        // build-essential / libopenblas-dev under the assumption that
        // dictation needs to build from source, it'd silently grow
        // every host's footprint by the build chain. Catch it here.
        for forbidden in ["build-essential", "libopenblas-dev"] {
            assert!(
                !installed.iter().any(|p| p == forbidden),
                "desktop role must NOT install `{forbidden}` — whisper.cpp is now vendored \
                 prebuilt at dotfiles/.local/bin/whisper-server, no host build needed"
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

        // Vendored binaries: walk the dotfile WriteFile actions and
        // confirm both whisper-server and whisper-cli are scheduled
        // to land at ~/.local/bin/ AND that the planner has marked
        // them executable (mode 0o755 — driven by the ELF magic
        // detection in planner.rs). Mode is what makes them
        // runnable; without it the dictate keybind silently fails
        // on first press.
        for binary in ["whisper-server", "whisper-cli"] {
            let action = plan
                .actions
                .iter()
                .find(|a| matches!(a, Action::WriteFile { rel, .. } if rel.ends_with(binary)))
                .unwrap_or_else(|| {
                    panic!(
                        "desktop role plan must include a WriteFile for `{binary}`; \
                         WriteFile rels = {:?}",
                        plan.actions
                            .iter()
                            .filter_map(|a| match a {
                                Action::WriteFile { rel, .. } =>
                                    Some(rel.display().to_string()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    )
                });
            match action {
                Action::WriteFile { mode, contents, rel } => {
                    assert_eq!(
                        *mode,
                        Some(0o755),
                        "{} must be marked executable (mode 0o755) by the ELF detection in \
                         planner.rs; mode was {:?}",
                        rel.display(),
                        mode
                    );
                    assert!(
                        contents.starts_with(b"\x7fELF"),
                        "vendored {binary} should be an ELF binary; first 4 bytes were {:?}",
                        &contents[..contents.len().min(4)]
                    );
                }
                _ => unreachable!(),
            }
        }

        // Negative pin: no whisper.cpp clone / build / symlink
        // post-commands. The vendored binaries replace all of them.
        let post_cmds: Vec<&PostCmd> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::PostCommand(p) => Some(p),
                _ => None,
            })
            .collect();
        for forbidden_substr in ["git clone https://github.com/ggml-org/whisper.cpp", "cmake --build", "ln -sf $HOME/.local/src/whisper.cpp"] {
            assert!(
                !post_cmds.iter().any(|p| p.run().contains(forbidden_substr)),
                "desktop role must NOT include a post-command containing `{forbidden_substr}` — \
                 whisper.cpp is vendored prebuilt now, the build chain has been retired"
            );
        }
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
        // Substring-check the subcommand dispatch so a refactor
        // that accidentally drops a user-facing action gets caught.
        // `cancel` is bound to Mod+Ctrl+s, `stop-server` is the
        // CLI escape hatch for freeing RAM, and `toggle-server` is
        // invoked by the status-panel popup when the user clicks
        // the `stt:` line — losing any of them silently breaks a
        // user-visible flow.
        for sub in ["cancel", "stop-server", "toggle-server"] {
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

        // Regression guard: the dictate script MUST collapse newlines
        // out of the whisper transcription before piping it to `wtype
        // -`. wtype faithfully types `\n` bytes as real Return
        // keypresses, so a multi-sentence transcription containing
        // embedded newlines would auto-submit the prompt at every
        // sentence boundary in any line-submitted REPL (Claude Code,
        // shells, chat inputs). The fix is a `tr '\n\r' '  '` stage
        // in transcribe(); if a future refactor drops it we want
        // `cargo test` to fail loudly rather than the user
        // rediscovering the bug by losing dictated text mid-prompt.
        assert!(
            dictate_str.contains("tr '\\n\\r' '  '"),
            "dictate script must collapse newlines out of the whisper transcription before typing it via wtype — otherwise multi-sentence prompts auto-submit at every line break"
        );
    }

    /// status-panel must show the whisper-server (STT) state and
    /// dispatch a panel-selection on the `stt:` line to
    /// `dictate toggle-server`.
    ///
    /// Why: the user wanted both pieces — visible on/off state at
    /// Mod+i AND a one-click toggle from inside the popup, with
    /// the underlying state machine ("first use starts it, stays
    /// running until off, off-stays-off until on or used") backed
    /// by the existing dictate lazy-start. If a future refactor
    /// drops the `is-active` lookup, the line silently becomes
    /// stale; if the dispatch loses the `stt:` prefix match, the
    /// click does nothing. We assert all three pieces (state
    /// readout, prefix dispatch, dictate invocation) so any
    /// regression fails `cargo test` instead of presenting as
    /// "the panel just doesn't toggle anymore".
    #[test]
    fn status_panel_renders_and_toggles_stt_state() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".local/bin/status-panel"))
            .expect("status-panel script must be embedded under .local/bin/");
        let s = String::from_utf8(bytes).expect("status-panel must be utf8");

        // Readout: the panel queries the unit state via systemctl
        // is-active. We check the exact argv so a swap to e.g.
        // probing the loopback port (which would re-introduce the
        // `activating` flicker we deliberately avoid) trips the
        // assertion and forces a conscious decision.
        assert!(
            s.contains("\"systemctl\", \"--user\", \"is-active\", \"whisper-server.service\""),
            "status-panel must read whisper-server state via `systemctl --user is-active whisper-server.service`"
        );

        // Both rendered states must exist as literal strings —
        // `get_stt` returns one of them based on the unit state.
        // Pinning both literals catches a typo / case-change that
        // would render fine but break the prefix dispatch (e.g.
        // `STT: on` capitalized would no longer match the
        // `stt:` STT_LINE_PREFIX below).
        assert!(
            s.contains("\"stt: on\""),
            "status-panel must emit `stt: on` when whisper-server is active"
        );
        assert!(
            s.contains("\"stt: off\""),
            "status-panel must emit `stt: off` when whisper-server is inactive"
        );

        // Dispatch wiring: there must be a constant matching the
        // `stt:` prefix used by both the rendered lines and the
        // selection check, and the selection branch must call
        // `dictate toggle-server` when that prefix matches. Two
        // separate substring checks because the prefix and the
        // dispatch live in different parts of the file and could
        // drift independently.
        assert!(
            s.contains("STT_LINE_PREFIX = \"stt:\""),
            "status-panel must define `STT_LINE_PREFIX = \"stt:\"` so the rendered line and the selection check share one source of truth"
        );
        assert!(
            s.contains("startswith(STT_LINE_PREFIX)"),
            "status-panel main() must dispatch on `result.stdout.startswith(STT_LINE_PREFIX)` to act only on stt-line selections"
        );
        assert!(
            s.contains("[\"dictate\", \"toggle-server\"]"),
            "status-panel must call `dictate toggle-server` when the user picks the stt: line"
        );
    }

    /// obsidian.nvim's workspace specs must be filtered by on-disk
    /// existence before being handed to setup().
    ///
    /// Why: obsidian.nvim resolves every declared workspace path
    /// with `Path.new(path):resolve { strict = true }` (see
    /// workspace.lua:79). A non-existent path throws, and the user
    /// sees a multi-line stack trace popup *every time they open a
    /// note*. We hit this when ~/linux-config/docs hadn't been
    /// created but was still listed in the workspaces table; the
    /// fix is to filter the spec list at plugin-load time so a
    /// missing directory is silently absent instead of fatal.
    ///
    /// The test asserts the structural shape of the fix (opts is
    /// a function, isdirectory is the gate, table.insert builds
    /// the filtered list) rather than evaluating the lua, because
    /// evaluating lua from Rust would mean pulling in mlua just
    /// for one config-shape test.
    #[test]
    fn obsidian_workspaces_filtered_by_existence() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/nvim/lua/plugins/obsidian.lua"))
            .expect("obsidian.lua plugin spec must be embedded");
        let lua = String::from_utf8(bytes).expect("obsidian.lua must be utf8");

        // `opts` must be a function (not a static table) — a static
        // table can't filter, and a static table is exactly what
        // produced the original bug.
        assert!(
            lua.contains("opts = function()"),
            "obsidian.lua must use `opts = function()` so workspace specs can be filtered at load time — a static `opts = {{...}}` table can't drop missing directories and reproduces the strict-resolve crash"
        );

        // The actual filter: isdirectory must gate the table.insert.
        // Without isdirectory the function is just a fancy way to
        // produce the same un-filtered list. Without table.insert
        // there's no filtered list being built at all. Assert both.
        assert!(
            lua.contains("vim.fn.isdirectory(vim.fn.expand(spec.path))"),
            "obsidian.lua must check `vim.fn.isdirectory(vim.fn.expand(spec.path))` to drop workspace specs whose directory doesn't exist"
        );
        assert!(
            lua.contains("table.insert(workspaces, spec)"),
            "obsidian.lua must build the filtered workspaces list via table.insert in the filter loop"
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

    /// The screenshot script must ship under `.local/bin/` AND the
    /// sway config must bind both `Print` (selection) and
    /// `Shift+Print` (fullscreen) to it. The script's whole job is
    /// "save a PNG to ~/Pictures and copy the same bytes to the
    /// clipboard in one press"; missing either the script or one
    /// of the binds reverts to the older clipboard-only behavior
    /// without a clear failure signal — the bind would still fire,
    /// just into nothing useful.
    ///
    /// We assert four things:
    ///   1. The script is embedded and starts with a shebang (so
    ///      the planner marks it 0o755 via shebang detection).
    ///   2. The script implements both `select` and `full` mode
    ///      branches — substring checks rather than exact-match
    ///      so refactors (e.g. adding a `window` mode) do not
    ///      break the test.
    ///   3. The sway config binds `Print` to the script in select
    ///      mode.
    ///   4. The sway config binds `Shift+Print` to the script in
    ///      full mode.
    ///
    /// Bind assertions match the trimmed `bindsym ...` line
    /// (rather than just substring-matching the file) so a bind
    /// that ends up commented out during a refactor gets caught.
    #[test]
    fn screenshot_script_and_binds_ship_correctly() {
        let script = EmbeddedDotfiles
            .read(Path::new(".local/bin/screenshot"))
            .expect("screenshot script must be embedded under .local/bin/");
        let script_str = String::from_utf8(script).expect("screenshot script must be utf8");
        assert!(
            script_str.starts_with("#!"),
            "screenshot script must start with a shebang so the planner marks it executable"
        );
        for mode in ["select", "full"] {
            assert!(
                script_str.contains(mode),
                "screenshot script must implement the `{mode}` mode branch"
            );
        }

        let cfg_bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(cfg_bytes).expect("sway config should be utf8");

        // `Print` (no modifier) must invoke the screenshot script
        // in select mode. We require the exact `select` token in
        // the bind so a bind that calls the script with no args
        // (which would default to select) still passes — but a
        // typo'd `selct` does not.
        let print_bind = cfg.lines().map(str::trim_start).find(|l| {
            l.starts_with("bindsym Print ")
                && l.contains("screenshot")
                && l.contains("select")
        });
        assert!(
            print_bind.is_some(),
            "sway config must bind `Print` to `$bin/screenshot select` — without it \
             the user loses the save-and-copy combo on the headline screenshot key"
        );

        // `Shift+Print` is the new fullscreen bind. Same shape as
        // the select assertion, just with the `full` mode token.
        let shift_print_bind = cfg.lines().map(str::trim_start).find(|l| {
            l.starts_with("bindsym Shift+Print ")
                && l.contains("screenshot")
                && l.contains("full")
        });
        assert!(
            shift_print_bind.is_some(),
            "sway config must bind `Shift+Print` to `$bin/screenshot full` — this is \
             the user-requested fullscreen capture key"
        );

        // Negative assertion: the legacy inline-grim binds for
        // `$mod+Print` and `$mod+Shift+Print` were superseded by
        // the script (which does both save AND copy in a single
        // press) and explicitly removed at the user's request to
        // keep the bind table clean. Re-adding them would create
        // overlapping shortcuts that drift apart over time —
        // exactly the dual-source-of-truth problem the script was
        // introduced to fix.
        for legacy in ["bindsym $mod+Print ", "bindsym $mod+Shift+Print "] {
            let still_bound = cfg.lines().map(str::trim_start).any(|l| l.starts_with(legacy));
            assert!(
                !still_bound,
                "legacy screenshot bind `{legacy}` must not appear in sway config — it \
                 was deliberately removed in favor of the unified $bin/screenshot script. \
                 Re-adding it brings back the dual-source-of-truth drift the cleanup \
                 was meant to prevent."
            );
        }
    }

    /// The `vm-menu` script must ship under `.local/bin/`, start
    /// with a shebang (so the planner's shebang detection writes
    /// it 0o755), and exercise every runtime the menu claims to
    /// cover.
    ///
    /// We assert the script references all three runtime CLIs
    /// (`podman`, `incus`, `virsh`) by substring rather than
    /// exact-line matching so that future refactors that move
    /// the references into helper functions / dispatch tables
    /// don't break the test as long as the underlying coverage
    /// stays put. Same logic for the action verbs — we just
    /// check that the keywords `start`, `stop`, and `shell`
    /// appear in the source, which catches accidental deletion
    /// without forcing the test to mirror the script's exact
    /// dispatch shape.
    ///
    /// The `--help` text must mention the column glyphs; this is
    /// the user-facing legend that explains `●`/`◐`/`○`, and a
    /// future refactor that drops the legend would silently
    /// break the discoverability promise the keybind was added
    /// for.
    #[test]
    fn vm_menu_script_ships() {
        let script = EmbeddedDotfiles
            .read(Path::new(".local/bin/vm-menu"))
            .expect("vm-menu script must be embedded under .local/bin/");
        let s = String::from_utf8(script).expect("vm-menu script must be utf8");
        assert!(
            s.starts_with("#!"),
            "vm-menu script must start with a shebang so the planner's shebang \
             detection writes it executable (0o755) — without that the keybind \
             fires the script but the kernel refuses to exec it"
        );
        for cli in ["podman", "incus", "virsh"] {
            assert!(
                s.contains(cli),
                "vm-menu script must reference `{cli}` — the menu's whole job is \
                 enumerating + dispatching across all three runtimes; dropping a \
                 reference would silently lose coverage of one of them"
            );
        }
        for verb in ["start", "stop", "shell"] {
            assert!(
                s.contains(verb),
                "vm-menu script must implement the `{verb}` action — these three \
                 are the documented user-facing verbs in the action submenu"
            );
        }
        // The --help legend lives in HELP_TEXT; assert the glyph
        // characters and column names show up so a future refactor
        // that drops the legend gets caught at test time, not by
        // a confused user.
        for legend in ["●", "◐", "○", "running", "paused", "stopped"] {
            assert!(
                s.contains(legend),
                "vm-menu --help text must mention `{legend}` so users have a \
                 self-contained reference for the menu's column glyphs"
            );
        }
    }

    /// The container shell launcher must use a `command -v` probe,
    /// NOT `exec bash 2>/dev/null || exec sh`.
    ///
    /// Bash writes its `PS1` prompt to **stderr**, so the
    /// previously-shipped form silently inherited the
    /// `2>/dev/null` redirection across `exec` and rendered every
    /// prompt and readline refresh into `/dev/null` — the user
    /// saw a blank foot window with no prompt and assumed the
    /// menu was broken. The fix lives in `vm-menu`'s
    /// `SHELL_LAUNCHER` constant; this test exists to make sure
    /// nobody innocently rewrites it back into the obvious-but-
    /// wrong spelling.
    ///
    /// We assert three things:
    ///
    /// 1. The zsh probe (`command -v zsh`) is present and comes
    ///    BEFORE the bash probe in source order — this encodes
    ///    the documented preference of zsh → bash → sh, and a
    ///    refactor that swapped the order would silently downgrade
    ///    every container shell to bash even when zsh is available.
    /// 2. The bash probe (`command -v bash`) is also present, so
    ///    the middle step of the fallback chain stays intact for
    ///    images that ship bash but not zsh.
    /// 3. The buggy `exec bash 2>/dev/null` substring is NOT
    ///    present anywhere in the script — that's the prompt-
    ///    swallowing footgun the launcher was rewritten to avoid.
    #[test]
    fn vm_menu_shell_launcher_uses_command_v_probe() {
        let script = EmbeddedDotfiles
            .read(Path::new(".local/bin/vm-menu"))
            .expect("vm-menu script must be embedded under .local/bin/");
        let s = String::from_utf8(script).expect("vm-menu script must be utf8");
        let zsh_at = s.find("command -v zsh").expect(
            "vm-menu must probe for zsh with `command -v` so containers that \
             ship zsh drop into the same shell the host user runs \
             interactively; without this probe the launcher silently \
             downgrades to bash",
        );
        let bash_at = s.find("command -v bash").expect(
            "vm-menu must probe for bash with `command -v` as the middle step \
             of the zsh → bash → sh fallback chain; dropping this leaves \
             zsh-less images falling straight through to /bin/sh",
        );
        assert!(
            zsh_at < bash_at,
            "vm-menu's zsh probe must come BEFORE its bash probe in source \
             order (zsh → bash → sh). Reversing them would make every \
             container shell pick bash even when zsh is installed, which \
             defeats the point of the zsh preference."
        );
        assert!(
            !s.contains("exec bash 2>/dev/null"),
            "vm-menu must NOT use `exec bash 2>/dev/null` — bash writes its \
             PS1 prompt to stderr, so this redirection silently swallows the \
             prompt across the exec and the foot window appears to hang. \
             Use the `command -v` probes in SHELL_LAUNCHER instead."
        );
    }

    /// The sway config must bind exactly one `$mod+c` and that bind
    /// must point at `$bin/vm-menu`. We assert exactness (not just
    /// "some line mentions $mod+c") so a refactor that accidentally
    /// re-adds a duplicate / overlapping bind on the same chord —
    /// which sway resolves by last-wins, silently shadowing the
    /// vm-menu — gets caught here.
    ///
    /// We also negative-assert that `$mod+v` (the daily-use nvim
    /// bind) is NOT pointed at vm-menu: when picking the keybind we
    /// considered `$mod+v` for "virt" but rejected it because v was
    /// already a load-bearing nvim shortcut. Pinning the negative
    /// makes future "let's just move it to v" PRs explicit about
    /// what they're displacing.
    #[test]
    fn vm_menu_keybind_present() {
        let bytes = EmbeddedDotfiles
            .read(Path::new(".config/sway/config"))
            .expect("sway config should be embedded");
        let cfg = String::from_utf8(bytes).expect("sway config should be utf8");

        let matches: Vec<&str> = cfg
            .lines()
            .map(str::trim_start)
            .filter(|l| {
                l.starts_with("bindsym")
                    && l.contains("$mod+c ")
                    // Filter out chord modifiers that would also start with
                    // "$mod+c" textually (none today, but $mod+Ctrl+c could
                    // be added later and shouldn't satisfy this check).
                    && !l.contains("$mod+Ctrl+c")
                    && !l.contains("$mod+Shift+c")
                    && !l.contains("$mod+Alt+c")
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "sway config must have exactly one `$mod+c` bind (the vm-menu \
             keybind); found {}: {:?}",
            matches.len(),
            matches
        );
        assert!(
            matches[0].contains("$bin/vm-menu"),
            "the `$mod+c` bind must invoke `$bin/vm-menu`; got: {:?}",
            matches[0]
        );

        // Defensive: `$mod+v` must still belong to nvim, not vm-menu.
        // We picked $mod+c specifically because v was taken; if a
        // future PR rebinds v to vm-menu they need to make that
        // displacement explicit (and update this test).
        let nvim_bind = cfg.lines().map(str::trim_start).find(|l| {
            l.starts_with("bindsym") && l.contains("$mod+v ") && l.contains("nvim")
        });
        assert!(
            nvim_bind.is_some(),
            "$mod+v must remain bound to nvim — vm-menu was deliberately given \
             $mod+c because v was already the nvim shortcut. If you're moving \
             vm-menu to v, update this test and the keybind comment in the \
             sway config in the same change."
        );
    }

    /// The server role must install the full virt stack — incus
    /// (system containers) plus libvirt/qemu (full VMs) — and run
    /// the post-commands that wire them up.
    ///
    /// Pinned together so a future packages refactor that splits
    /// the stack across files / drops a step gets caught: the
    /// packages alone don't make incus usable (no init = no
    /// storage pool = `incus launch` fails), and the systemd
    /// enable alone doesn't help if the user isn't in the libvirt
    /// group (vm-menu's virsh calls would all 401). We assert all
    /// the moving parts together so the test has the same shape
    /// as the audio-stack and dictation-stack tests.
    ///
    /// The post-commands all carry a `check` field — without one,
    /// reruns would re-shell out to sudo for every step, and
    /// `incus admin init` would actively fail on rerun (it errors
    /// out against an already-initialized daemon). The check
    /// presence is asserted explicitly per step.
    #[test]
    fn server_role_provisions_virt_stack() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Packages must all show up as InstallPackage actions in
        // the resolved server plan. Listing them individually
        // produces a per-package failure message so an accidental
        // single-line drop is obvious in the test output.
        let installed: Vec<&str> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::InstallPackage(p) => Some(p.as_str()),
                _ => None,
            })
            .collect();
        for pkg in [
            "incus",
            "incus-client",
            "qemu-system-x86",
            // `qemu-system-modules-spice` is the SPICE display
            // backend module. Incus *always* starts VMs with
            // `-spice unix=on,…` to offer a graphical console
            // socket (for `incus console --type=vga` and for
            // virt-viewer attach), regardless of whether the
            // user asked for one. Without this module qemu
            // refuses to start with "There is no option group
            // 'spice'" and every incus VM fails on the very
            // first start. Debian splits this out of the base
            // qemu-system-x86 to keep headless installs lean,
            // but it's mandatory for our incus-on-the-host case.
            "qemu-system-modules-spice",
            "qemu-kvm",
            // `qemu-utils` ships `qemu-img`, which incus shells
            // out to in order to materialize a per-VM root disk
            // from the cached image (qcow2 inspect / convert /
            // overlay). Without it, `incus init --vm` errors
            // with "Failed reading image info … Failed to find
            // executable 'qemu-img'" — which is *especially*
            // confusing because qemu-system-x86 IS installed.
            // The two are separate packages on Debian and
            // qemu-system-x86 does not pull qemu-utils as a dep.
            "qemu-utils",
            "libvirt-daemon-system",
            "libvirt-clients",
            "bridge-utils",
            "virtinst",
            // `ovmf` is the UEFI firmware blob (OVMF_CODE.fd /
            // OVMF_VARS.fd under /usr/share/OVMF/). Without it
            // incus refuses to register the virtual-machine
            // driver at all — the daemon log shows "Unable to
            // locate a UEFI firmware" and every `incus init --vm`
            // fails with "QEMU failed to run feature checks"
            // even though qemu/kvm are installed. libvirt's
            // virt-install also silently downgrades UEFI guest
            // templates to legacy SeaBIOS without it. Treated
            // as virt-stack-critical, not optional.
            "ovmf",
        ] {
            assert!(
                installed.contains(&pkg),
                "server role must install `{pkg}` for the virt stack — without \
                 it the vm-menu (Mod+c) and CLI users lose coverage of one of \
                 the three runtimes (containers / LXCs / VMs)"
            );
        }

        // Group memberships: vm-menu shells out without sudo, so
        // the user must be in incus + libvirt at minimum, and
        // incus-admin / kvm for full functionality. We grep for
        // each group's `usermod -aG <group>` pattern individually
        // so a missing group produces a targeted error.
        let post_cmds: Vec<&str> = plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::PostCommand(p) => Some(p.run()),
                _ => None,
            })
            .collect();
        for group in ["incus-admin", "incus", "libvirt", "kvm"] {
            let needle = format!("usermod -aG {group}");
            assert!(
                post_cmds.iter().any(|cmd| cmd.contains(&needle)),
                "server role must include a `{needle} $(whoami)` post-command \
                 — without it the user can't talk to the corresponding daemon \
                 from vm-menu (which runs as the user, not root)"
            );
        }

        // libvirtd must be enabled — the vm-menu's virsh calls all
        // talk to /var/run/libvirt/libvirt-sock and the socket
        // doesn't appear until libvirtd has started at least once.
        let libvirtd_enable = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("systemctl enable")
                    && p.run().contains("libvirtd") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            libvirtd_enable.is_some(),
            "server role must include a `systemctl enable --now libvirtd` \
             post-command — without it the libvirt socket never appears and \
             every vm-menu virsh call errors out at connect time"
        );
        assert!(
            libvirtd_enable.unwrap().check().is_some(),
            "libvirtd enable post-command must carry a `check` so reruns are \
             cheap no-ops"
        );

        // virsh net-autostart on the default network: required so
        // VMs come back up after a reboot without manual
        // intervention.
        let net_autostart = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("virsh net-autostart")
                    && p.run().contains("default") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            net_autostart.is_some(),
            "server role must mark the default libvirt network autostart — \
             otherwise every VM defined on virbr0 fails to start after a host \
             reboot with `network 'default' is not active`"
        );
        assert!(
            net_autostart.unwrap().check().is_some(),
            "virsh net-autostart post-command must carry a `check` so reruns \
             skip when autostart is already configured"
        );

        // incus restart-after-ovmf: load-bearing for VM support
        // on first-run boxes. The incus package's postinst starts
        // incus.service immediately, but ovmf installs *later* in
        // the same plan (alphabetical order in roles.toml puts it
        // after `virtinst`), so the running daemon's UEFI feature
        // check has already cached "no UEFI" by the time ovmf
        // lands on disk. Without an explicit restart, the daemon
        // stays in its degraded state until the next host reboot,
        // and `incus init --vm` fails with "QEMU failed to run
        // feature checks" on an apparently-correctly-provisioned
        // host. The restart re-runs the feature check and the
        // virtual-machine driver becomes operational.
        let incus_restart = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("systemctl restart")
                    && p.run().contains("incus") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            incus_restart.is_some(),
            "server role must include a `systemctl restart incus.service` \
             post-command — without it the daemon caches its pre-OVMF \
             feature-check result and `incus init --vm` keeps failing with \
             'QEMU failed to run feature checks' until the next host reboot, \
             even though ovmf is installed"
        );
        let incus_restart = incus_restart.unwrap();
        assert!(
            incus_restart.check().is_some(),
            "incus restart post-command must carry a `check` so reruns are \
             cheap no-ops once the daemon has already been restarted after \
             ovmf landed — every forge run otherwise re-bounces incus, \
             which (a) momentarily kills any running containers' control-\
             plane access and (b) wastes seconds per run"
        );
        // The check must be a marker-file probe under
        // /var/lib/incus/, written only after a successful
        // restart. We deliberately moved away from the previous
        // /proc/<pid> mtime comparison: /proc entry mtimes are
        // not a reliable "process start time" on Linux (the
        // proc filesystem refreshes mtimes on access), so the
        // mtime check evaluated false on every rerun and the
        // restart fired every time — and on a workstation with
        // a running incus VM, that drained the VM via ACPI on
        // every forge run (~20 min hang in the worst observed
        // case). A marker file is unambiguous, requires no
        // group membership the user may not yet have, and makes
        // the post-cmd a true no-op once incus has UEFI
        // capability.
        let restart_check = incus_restart.check().unwrap();
        assert!(
            restart_check.contains("/var/lib/incus/")
                && restart_check.contains(".forge-uefi-restart-done"),
            "incus restart `check` must probe for the marker file \
             /var/lib/incus/.forge-uefi-restart-done. Do NOT regress to \
             a /proc/<pid> mtime comparison — that approach drains live \
             VMs on every forge run because /proc mtimes are not stable \
             process-start signals on Linux. Got: {restart_check:?}"
        );
        // The run command must write the marker only after a
        // successful restart — a `&&` chain (or equivalent
        // sh-level conjunction) is the only correct shape.
        // Independently running the touch would mark the host
        // as "restarted" even when the restart failed, which
        // would silently leave incus in its degraded pre-OVMF
        // state on every subsequent run.
        let restart_run = incus_restart.run();
        assert!(
            restart_run.contains("systemctl restart incus")
                && restart_run.contains(".forge-uefi-restart-done")
                && restart_run.contains("&&"),
            "incus restart `run` must chain `systemctl restart incus.service` \
             with the marker-file touch via `&&` so the marker is only \
             written when the restart succeeded — otherwise a failing \
             restart would leave the host falsely marked as 'done' and \
             every subsequent forge run would skip the restart while \
             incus stayed in its broken pre-OVMF state. Got: {restart_run:?}"
        );

        // incus admin init: required for `incus launch` to work at
        // all (no storage pool exists otherwise). The check is
        // load-bearing — `incus admin init` errors out on rerun
        // against an already-initialized daemon, so without the
        // guard every forge rerun fails.
        let incus_init = plan.actions.iter().find_map(|a| match a {
            Action::PostCommand(p)
                if p.run().contains("incus admin init") =>
            {
                Some(p)
            }
            _ => None,
        });
        assert!(
            incus_init.is_some(),
            "server role must run `incus admin init` so a fresh box ends up \
             with a storage pool + default profile + incusbr0 bridge wired up"
        );
        let incus_init = incus_init.unwrap();
        assert!(
            incus_init.check().is_some(),
            "incus admin init post-command MUST carry a `check` — running it \
             twice errors out with `existing instances or images. 'incus admin \
             init' requires an empty database`, so without the guard every \
             forge rerun fails at this step"
        );
        assert!(
            incus_init.run().contains("--storage-backend=dir"),
            "incus admin init must pin `--storage-backend=dir` rather than \
             letting --auto pick: trixie's tooling for ZFS/BTRFS can change \
             --auto's mind across releases, and we want every host using the \
             same storage shape so the docs stay predictable"
        );
    }

    /// The server role must defensively register libvirt's `default`
    /// network with libvirtd via `virsh net-define` BEFORE the
    /// subsequent `net-autostart` / `net-start` post-commands run.
    ///
    /// libvirt-daemon-system's postinst usually auto-imports the seed
    /// XML at `/etc/libvirt/qemu/networks/default.xml` into libvirtd's
    /// runtime state on first start, but not always — we caught a
    /// real host (the developer's daily driver) where the XML was on
    /// disk but `virsh net-list --all` showed no networks at all,
    /// which means `net-autostart` and `net-start` then fail with
    /// `error: failed to get network 'default'` and the entire
    /// vm-menu libvirt path is dead until someone runs `virsh
    /// net-define` by hand.
    ///
    /// This test pins three invariants together:
    ///
    ///   1. A post-command exists that calls `virsh net-define` on
    ///      the seed XML path. Without it, the post-cmd order below
    ///      assumes a registration step that doesn't exist.
    ///   2. It carries an idempotency `check` so reruns are no-ops
    ///      (otherwise every `forge setup` would shell out to sudo
    ///      to define an already-defined network for no reason).
    ///   3. It precedes `virsh net-autostart` and `virsh net-start`
    ///      in the post-cmd order. The order matters: those two
    ///      both need the network to exist in libvirtd's state, so
    ///      defining it second would still leave the first run
    ///      broken. We assert ordering by index in the action list.
    #[test]
    fn server_role_defines_libvirt_default_network() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        // Find each of the three virsh net-* post-commands by index
        // in the plan's actions. We need indices (not just refs) so
        // we can compare ordering.
        let mut define_idx = None;
        let mut autostart_idx = None;
        let mut start_idx = None;
        for (i, a) in plan.actions.iter().enumerate() {
            if let Action::PostCommand(p) = a {
                let run = p.run();
                if run.contains("virsh net-define") {
                    define_idx = Some((i, p));
                } else if run.contains("virsh net-autostart") {
                    autostart_idx = Some((i, p));
                } else if run.contains("virsh net-start") {
                    start_idx = Some((i, p));
                }
            }
        }

        let (define_i, define_cmd) = define_idx.expect(
            "server role must include a `virsh net-define …default.xml` \
             post-command so the libvirt `default` network is registered \
             with libvirtd before net-autostart / net-start run on a fresh \
             box where the package postinst didn't auto-import it",
        );
        let (autostart_i, _) = autostart_idx.expect(
            "server role must still include `virsh net-autostart default` — \
             the net-define test is layered on top, not a replacement",
        );
        let (start_i, _) = start_idx.expect(
            "server role must still include `virsh net-start default` — the \
             net-define test is layered on top, not a replacement",
        );

        // Idempotency: the define post-cmd MUST carry a check (else
        // every rerun re-shells to sudo for nothing).
        assert!(
            define_cmd.check().is_some(),
            "virsh net-define post-command must carry a `check` so reruns \
             skip the redundant sudo invocation once the network is \
             registered in libvirtd's state"
        );

        // Order: define BEFORE both autostart and start. Strict `<`
        // because they must be distinct steps; `<=` would let a
        // future "merge into one big command" refactor sneak past.
        assert!(
            define_i < autostart_i,
            "virsh net-define must precede virsh net-autostart in the \
             post-cmd order (define={define_i}, autostart={autostart_i}) — \
             autostart on an undefined network fails with `failed to get \
             network 'default'`"
        );
        assert!(
            define_i < start_i,
            "virsh net-define must precede virsh net-start in the post-cmd \
             order (define={define_i}, start={start_i}) — start on an \
             undefined network fails with `failed to get network 'default'`"
        );
    }

    /// The desktop role must install the full virt-console GUI
    /// stack — `virt-manager` (heavyweight management UI),
    /// `gir1.2-spiceclientgtk-3.0` (SPICE GTK widget bindings),
    /// and `virt-viewer` (the lightweight `remote-viewer` CLI that
    /// `incus console --type=vga` shells out to) — and none of
    /// them must appear in the server role's package list.
    ///
    /// Three packages, three failure modes if any one is dropped:
    ///
    /// * Without `virt-manager`: vm-menu's "open virt-manager"
    ///   fallback (for VMs lacking a serial pty) is dead, and
    ///   there's no GUI for inspecting libvirt's guest list.
    /// * Without `gir1.2-spiceclientgtk-3.0`: virt-manager's
    ///   window opens but the SPICE viewer pane errors with
    ///   "Namespace SpiceClientGtk not available" because the GIR
    ///   typelib is missing — Recommends of virt-manager, dropped
    ///   by our `--no-install-recommends` apt flag.
    /// * Without `virt-viewer`: `incus console <name> --type=vga`
    ///   silently fails with "remote-viewer not found" (or just
    ///   opens nothing in some incus versions). Same root cause
    ///   shape as the GIR case — incus invokes `remote-viewer`
    ///   on PATH and gives up if it's missing.
    ///
    /// All three are asserted together so a "let me trim the
    /// virt deps" pass that drops any one is loud at `cargo test`,
    /// not silent until the user hits the missing-tool path.
    /// The negative half (server must NOT install these) catches
    /// the inverse refactor — "move them to the base for
    /// symmetry" — which would add ~50 MiB of GTK / SPICE / GUI
    /// deps to every server provision.
    #[test]
    fn desktop_role_provisions_virt_console_tools() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let desktop = roles.resolved(Role::Desktop).unwrap();
        let server = roles.resolved(Role::Server).unwrap();
        let desktop_plan = build_plan(&desktop, &EmbeddedDotfiles).unwrap();

        let desktop_pkgs: Vec<&str> = desktop_plan
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::InstallPackage(p) => Some(p.as_str()),
                _ => None,
            })
            .collect();

        // Each package gets a per-package assertion so the failure
        // message names the specific missing piece, not a vague
        // "virt console stack incomplete".
        assert!(
            desktop_pkgs.contains(&"virt-manager"),
            "desktop role must install `virt-manager` — it's the SPICE/VNC \
             console fallback for VMs without a serial pty (where vm-menu's \
             `shell` action would otherwise hang at `Escape character is ^]`)"
        );
        assert!(
            desktop_pkgs.contains(&"gir1.2-spiceclientgtk-3.0"),
            "desktop role must explicitly install `gir1.2-spiceclientgtk-3.0` \
             — it's a Recommends of virt-manager, not a Depends, and forge's \
             apt invocations pass --no-install-recommends, so the GIR \
             namespace is missing at runtime and virt-manager's SPICE viewer \
             errors with `Namespace SpiceClientGtk not available` on every \
             console open"
        );
        assert!(
            desktop_pkgs.contains(&"virt-viewer"),
            "desktop role must install `virt-viewer` — it ships the \
             `remote-viewer` CLI that `incus console <name> --type=vga` \
             shells out to. Without it, opening an incus VM's SPICE display \
             fails with `remote-viewer not found` (or, in some incus versions, \
             silently opens nothing)"
        );

        // Server role's authored package list is checked directly
        // (not via build_plan) because we want to assert these
        // packages aren't even *authored* into the base, regardless
        // of how the planner deduplicates. This pins the layering
        // intent: the GUI tier is desktop-only by construction,
        // not by accident of dedupe.
        for pkg in ["virt-manager", "gir1.2-spiceclientgtk-3.0", "virt-viewer"] {
            assert!(
                !server.packages.iter().any(|p| p == pkg),
                "server role must NOT install `{pkg}` — it's part of the GUI \
                 console tier and pulls in GTK / SPICE deps that belong on \
                 workstation boxes only. The shared CLI virt stack (libvirtd, \
                 virsh, qemu) lives in `server` already; the GUI tier layers \
                 on top in `desktop`."
            );
        }
    }

    /// `dev-container.toml` is embedded into the binary by `commands::dev`
    /// the same way `roles.toml` is — via `include_str!`. A bad edit
    /// (typo in a key, missing required field, dangling containerfile
    /// path) would otherwise only surface the next time the user runs
    /// `forge dev build` or `forge dev up`. This test re-embeds the
    /// file via the same `concat!(env!("CARGO_MANIFEST_DIR"), …)`
    /// pattern the runtime uses, parses it, and asserts the invariants
    /// the dev-container subsystem depends on.
    ///
    /// Pinned invariants:
    /// * The file parses as a flat `DevContainer` (no `[[image]]`
    ///   array — that ceremony was retired when the multi-image
    ///   model collapsed to "one image per host, one container per
    ///   project"). Re-introducing the array would break parsing.
    /// * `BASE_CONTAINERFILE` (the embedded Containerfile) is non-empty
    ///   and contains a `FROM` directive. The Containerfile is now
    ///   embedded rather than referenced by path, so a typo or empty
    ///   file at the source would otherwise only surface at
    ///   `forge dev build` time on a remote box. Catch it here.
    /// * `image_tag` is non-empty AND namespaced (contains `forge`).
    ///   A blank tag would leave podman generating a random one;
    ///   a generic tag like "base" would collide with unrelated
    ///   images on the host.
    /// * `ssh_agent = true`. Forwarding the host agent is how
    ///   `git push` works inside the container without leaking
    ///   keys; flipping it to false would silently break the
    ///   "self-contained dev pod" promise.
    /// * The default named volume mounts on `/home/dev`. The base
    ///   Containerfile bakes the user's dotfiles under `/home/dev`
    ///   specifically because podman seeds the empty volume from
    ///   that path on first run; if the toml mount target drifts
    ///   away from `/home/dev` the seeding stops happening and
    ///   fresh containers come up bare.
    #[test]
    fn embedded_dev_container_toml_valid() {
        use forge_common::dev_config::DevContainer;

        const DEV_CONTAINER_TOML: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/dev-container.toml"
        ));

        let cfg = DevContainer::from_toml(DEV_CONTAINER_TOML)
            .expect("dev-container.toml should parse");

        // The Containerfile is embedded into the binary (no longer a
        // disk-relative path on the config side). Validate the embedded
        // string is non-empty and looks like a real Containerfile.
        // A typo or accidental empty file at the source would otherwise
        // surface only at `forge dev build` time.
        assert!(
            !BASE_CONTAINERFILE.is_empty(),
            "BASE_CONTAINERFILE is empty — containers/base.Containerfile failed to embed"
        );
        assert!(
            BASE_CONTAINERFILE.contains("FROM "),
            "BASE_CONTAINERFILE must contain a FROM directive; the embedded content does not \
             look like a Containerfile"
        );

        assert!(
            !cfg.image_tag.is_empty(),
            "image_tag must be non-empty — podman would otherwise auto-generate a random tag \
             and the build/run paths would diverge"
        );
        assert!(
            cfg.image_tag.contains("forge"),
            "image_tag should be namespaced with `forge` so a `podman images` listing makes \
             it obvious which images come from this project; got {:?}",
            cfg.image_tag
        );

        assert!(
            cfg.ssh_agent,
            "ssh_agent must stay true so git pushes from inside the container can reach the \
             host's ssh-agent without copying key material"
        );

        let home_volume = cfg
            .default_volumes
            .named
            .iter()
            .find(|v| v.container == "/home/dev")
            .expect(
                "dev-container.toml must mount a named volume on /home/dev — that path is \
                 where the Containerfile bakes the user's dotfiles, and podman only seeds \
                 the volume from the image when the mount target matches",
            );
        assert!(
            home_volume.name.contains("{name}"),
            "the home volume's name must contain the `{{name}}` placeholder so each container \
             gets its own volume; got {:?}",
            home_volume.name
        );
    }

    /// `config/dev-container.toml` must bind-mount the host's
    /// `/etc/localtime` into every dev container, read-only, and
    /// must NOT bind-mount `/etc/timezone`.
    ///
    /// Why `/etc/localtime`: glibc's time-formatting code reads
    /// it directly to determine the active zone (the `TZ` env
    /// var is only a fallback). Without the bind mount, the
    /// trixie-slim base image's UTC default leaks into every
    /// container — `date`, `git log --date=local`, anything that
    /// timestamps to a log, all show UTC even though the host is
    /// in a different zone.
    ///
    /// Why NOT `/etc/timezone`: that file is a Debian-specific
    /// human-readable zone name that doesn't exist on modern
    /// systemd-based hosts (this trixie laptop included).
    /// Bind-mounting a missing host path makes podman bail at
    /// container create time with `statfs /etc/timezone: no such
    /// file or directory` — the exact failure mode an earlier
    /// version of this test pinned in the wrong direction.
    ///
    /// `readonly = true` on the localtime bind is a hard
    /// requirement: under rootless podman + `--userns keep-id`
    /// the container UID maps onto the invoking host user, which
    /// on this single-user laptop is the user who actually owns
    /// `/etc/localtime`. A read-write mount would let a
    /// misbehaving container rewrite the host's zone.
    #[test]
    fn dev_container_provisions_tz_bind_mounts() {
        use forge_common::dev_config::DevContainer;

        const DEV_CONTAINER_TOML: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/dev-container.toml"
        ));

        let cfg = DevContainer::from_toml(DEV_CONTAINER_TOML)
            .expect("dev-container.toml should parse");

        let localtime = cfg
            .default_volumes
            .bind
            .iter()
            .find(|b| b.host == "/etc/localtime" && b.container == "/etc/localtime")
            .expect(
                "dev-container.toml must bind-mount `/etc/localtime` so the container \
                 inherits the host timezone instead of falling back to the trixie-slim \
                 base image's UTC default",
            );
        assert!(
            localtime.readonly,
            "bind mount of `/etc/localtime` must be `readonly = true` — under rootless \
             podman + `--userns keep-id` a writable mount would let a container mutate the \
             host's zone configuration"
        );

        // Negative invariant: `/etc/timezone` must NOT be bind-
        // mounted. That file is missing on systemd-based hosts
        // (no `/etc/timezone` on this trixie laptop) and trying
        // to bind it makes podman fail at container create with
        // `statfs /etc/timezone: no such file or directory`.
        assert!(
            !cfg.default_volumes
                .bind
                .iter()
                .any(|b| b.host == "/etc/timezone"),
            "dev-container.toml must NOT bind-mount `/etc/timezone` — the file doesn't \
             exist on modern systemd-based hosts and binding a missing host path makes \
             `podman run` bail at container create time"
        );
    }

    /// The base Containerfile must install the `batcat`→`bat`
    /// symlink at `/usr/local/bin/bat`.
    ///
    /// Debian's `bat` apt package installs the binary as
    /// `/usr/bin/batcat` (not `bat`) due to the long-standing
    /// filename clash with the `bacula-console-bat` package
    /// (Debian bug #930299, won't-fix). The user's `cat = bat` zsh
    /// alias relies on `bat` being on PATH; without the symlink,
    /// every `cat <file>` in the container fails with "command
    /// not found: bat" despite the binary being installed. The
    /// upstream `sharkdp/bat` README documents this exact symlink
    /// as the recommended Debian workaround.
    #[test]
    fn base_containerfile_installs_bat_symlink() {
        assert!(
            BASE_CONTAINERFILE.contains("ln -s /usr/bin/batcat /usr/local/bin/bat"),
            "Containerfile must symlink `/usr/bin/batcat` to `/usr/local/bin/bat` so the user's \
             `cat = bat` zsh alias works inside the container (Debian's bat package installs \
             as `batcat` due to a filename clash)"
        );
    }

    /// The base Containerfile must install lazygit from upstream.
    ///
    /// `lazygit` is bound to the `lg` zsh alias and is part of the
    /// dotfile experience. Debian trixie main has no `lazygit`
    /// package (it only ships in unstable/sid which we avoid for
    /// reproducibility), so we pull the upstream static tarball at
    /// image-build time — same pattern the Containerfile uses for
    /// nvim/yazi/uv/fnm.
    #[test]
    fn base_containerfile_installs_lazygit() {
        assert!(
            BASE_CONTAINERFILE.contains("jesseduffield/lazygit"),
            "Containerfile must install lazygit from upstream (no apt package on trixie) so the \
             `lg` zsh alias works"
        );
        assert!(
            BASE_CONTAINERFILE.contains("/usr/local/bin/lazygit"),
            "lazygit binary must land at /usr/local/bin/lazygit so it's on PATH and not \
             shadowed by the home volume"
        );
    }

    /// The base Containerfile must install git-delta + tzdata via apt.
    ///
    /// `git-delta` (binary `delta`) is in trixie main; the user can
    /// opt in via `[core] pager = delta` without forge configuring
    /// global git for them. `tzdata` ships the zoneinfo database that
    /// the `/etc/localtime` bind mount layers on top of.
    #[test]
    fn base_containerfile_installs_git_delta_and_tzdata() {
        assert!(
            BASE_CONTAINERFILE.contains("git-delta"),
            "Containerfile apt layer must install `git-delta` (the `delta` git pager binary)"
        );
        assert!(
            BASE_CONTAINERFILE.contains("tzdata"),
            "Containerfile apt layer must install `tzdata` so the host /etc/localtime bind \
             mount has a zoneinfo layout to layer on top of"
        );
    }

    /// The base Containerfile must install `file` so yazi's mime
    /// detection works inside the container.
    ///
    /// Yazi shells out to `file --mime-type <path>` whenever a
    /// directory entry's extension is ambiguous or absent (the
    /// preview pane, the `open` action, and the per-row mime
    /// rendering all consult it). On a stock `debian:trixie-slim`
    /// base image the `file` binary is NOT present — it gets
    /// transitively pulled in by heavier packages like `git`'s
    /// recommends list, but `--no-install-recommends` (which the
    /// Containerfile uses to keep the image small) suppresses that
    /// pull. The visible failure mode is yazi greeting the user
    /// with `Cannot find 'file' to detect the file's mime type` on
    /// every navigation event. Adding `file` to the apt list
    /// resolves it; the package is ~1.4 MiB installed, so the
    /// image-size cost is negligible.
    #[test]
    fn base_containerfile_installs_file_for_yazi_mime_detection() {
        // Match the package as a whitespace-delimited token so this
        // doesn't false-positive on `git-delta`, `dotfile`, etc.
        let in_apt_list = BASE_CONTAINERFILE
            .split_whitespace()
            .any(|tok| tok == "file");
        assert!(
            in_apt_list,
            "Containerfile apt layer must install `file` so yazi's `file --mime-type` shell-out \
             works (without it, yazi prints `Cannot find 'file' to detect the file's mime type` \
             on every directory listing inside the container)"
        );
    }

    /// The base Containerfile must declare `ENV COLORTERM=truecolor`
    /// so TUI apps inside the container detect 24-bit color support.
    ///
    /// Most modern TUI apps (nvim, opencode, lazygit, btop, bat)
    /// check `$COLORTERM` before the terminfo `Tc` capability.
    /// `podman exec -t` propagates `TERM` but NOT arbitrary env;
    /// without this `ENV`, the container shell ends up with
    /// `COLORTERM` unset, apps quantise their palette to the
    /// 256-color cube, and catppuccin's gradients render visibly
    /// off — the "TUI graphics are wrong" symptom we saw in
    /// practice. Pair with the `terminal-features ',foot*:RGB'`
    /// line in `dotfiles/.tmux.conf` so the truecolor escapes
    /// actually make it through tmux to foot.
    #[test]
    fn base_containerfile_sets_colorterm_truecolor() {
        assert!(
            BASE_CONTAINERFILE.contains("ENV COLORTERM=truecolor"),
            "Containerfile must declare `ENV COLORTERM=truecolor` so TUI apps inside the \
             container detect truecolor support and don't fall back to 256-color quantisation"
        );
    }

    /// The base Containerfile must install a `forge` stub at
    /// `/usr/local/bin/forge` that prints a host-only hint and exits 127.
    ///
    /// Without this, typing `forge` in the container hits zsh's
    /// command_not_found handler with a generic "command not
    /// found" + apt-index suggestion miss, which is misleading
    /// (suggesting forge is something installable). The stub
    /// intercepts and emits a one-line hint pointing at the host.
    /// Same pattern Gitpod uses for its `gp` command.
    #[test]
    fn base_containerfile_installs_forge_stub() {
        assert!(
            BASE_CONTAINERFILE.contains("/usr/local/bin/forge"),
            "Containerfile must drop a stub at `/usr/local/bin/forge` so typing `forge` inside \
             the container produces a useful host-only hint instead of a generic \
             command-not-found"
        );
        assert!(
            BASE_CONTAINERFILE.contains("host-only"),
            "the forge stub's body must mention `host-only` so the user understands the \
             command runs outside the container; got Containerfile lacking that string"
        );
    }

    /// The base Containerfile must install a `wl-copy` OSC52 shim at
    /// `/usr/local/bin/wl-copy` so the `clip` zsh alias and any
    /// tmux copy-pipe binding that shells out to `wl-copy` works
    /// inside the container.
    ///
    /// The host's `wl-copy` talks to the Wayland compositor, which
    /// doesn't exist inside a podman container. The shim emits the
    /// OSC52 escape (`ESC ] 52 ; c ; <base64> ESC \\`); foot
    /// supports OSC52 since v1.4 (default-on), and the host's
    /// tmux is configured (in dotfiles/.tmux.conf) to pass the
    /// escape through with `set-clipboard on` and
    /// `terminal-features ',foot*:clipboard'`.
    #[test]
    fn base_containerfile_installs_wl_copy_shim() {
        assert!(
            BASE_CONTAINERFILE.contains("/usr/local/bin/wl-copy"),
            "Containerfile must drop an OSC52 shim at `/usr/local/bin/wl-copy` so the `clip` \
             alias and tmux copy-pipe work inside the container (no Wayland in container)"
        );
        // The OSC52 selection identifier `52;c;` is the load-bearing
        // marker — it's what the outer terminal recognises as
        // "place this on clipboard". A future edit that drops it
        // would silently break clipboard while still creating a
        // file at the right path.
        assert!(
            BASE_CONTAINERFILE.contains("52;c;"),
            "wl-copy shim body must contain the OSC52 marker `52;c;` (the clipboard selection \
             identifier the outer terminal interprets); the file at /usr/local/bin/wl-copy \
             must emit this escape sequence to be functional"
        );
    }

    /// The base Containerfile must pre-create `/run/user/1000`
    /// (mode 0700, owned by the `dev` user) so tools that read
    /// `$XDG_RUNTIME_DIR` work inside the container.
    ///
    /// On a host, systemd-logind creates this path as a tmpfs at
    /// user login. Inside a podman container there's no logind,
    /// so without this RUN step `/run/user/1000` doesn't exist
    /// and anything that writes per-session runtime state there
    /// fails with `No such file or directory`. The visible
    /// surface was `fnm` failing to create its multishell
    /// symlink the first time the user ran `node` in a fresh
    /// dev pod, but anything XDG-RUNTIME-DIR-consuming
    /// (gpg-agent, dbus user session, certain editors' caches)
    /// hits the same wall.
    ///
    /// Mode 0700 matches what logind creates on a host — many
    /// tools refuse to use the directory if it's group/world-
    /// readable since it can hold session secrets. Ownership is
    /// `dev:dev` because the path is per-user state and the
    /// container's UID 1000 maps onto the host UID via
    /// `--userns keep-id`.
    #[test]
    fn base_containerfile_creates_xdg_runtime_dir() {
        // The directory must be created with 0700 perms and owned
        // by the dev user. We assert on the load-bearing string
        // fragments rather than the exact whitespace, since that
        // would couple to formatting.
        for marker in ["mkdir -p /run/user/1000", "chown dev:dev /run/user/1000", "chmod 700 /run/user/1000"] {
            assert!(
                BASE_CONTAINERFILE.contains(marker),
                "Containerfile must contain `{marker}` so $XDG_RUNTIME_DIR exists for the \
                 dev user inside the container — without this, fnm / gpg-agent / dbus user \
                 session all fail on first use with `No such file or directory` errors"
            );
        }
    }

    /// Negative invariant: the base Containerfile must NOT copy
    /// `dotfiles/.config/theme/` into the container.
    ///
    /// The theme tree is templates + base16 schemes consumed by
    /// `forge theme apply` on the host. Inside the container forge
    /// is intentionally unavailable (see the host-only stub
    /// asserted above), so the templates can never be rendered —
    /// they'd just sit as ~100 KiB of inert .j2 + .yaml in
    /// `/home/dev/.config/theme/`. The container consumes the
    /// *rendered* outputs (`nvim/lua/plugins/colorscheme.lua`,
    /// `yazi/theme.toml`, `tmux/colors.conf`) which DO get copied
    /// via the explicit dotfile COPYs. Re-introducing this COPY
    /// brings back the dead weight.
    #[test]
    fn base_containerfile_does_not_copy_dead_theme_dir() {
        // Match an actual COPY directive — not any mention of the path
        // string (the Containerfile contains an explanatory comment
        // referencing it). Walk every non-comment line and look for
        // a COPY whose source is the theme tree.
        for (lineno, line) in BASE_CONTAINERFILE.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }
            assert!(
                !(trimmed.starts_with("COPY") && trimmed.contains("dotfiles/.config/theme")),
                "Containerfile line {} contains `COPY ... dotfiles/.config/theme ...` — that \
                 tree is templates + schemes that only `forge theme apply` (host-only) can \
                 render; inside the container they're inert dead weight. The rendered \
                 outputs (nvim/lua/plugins/colorscheme.lua, yazi/theme.toml, etc.) are \
                 copied via the explicit per-target COPYs.\n  line: {}",
                lineno + 1,
                line,
            );
        }
    }

    /// The server role must install the `bat` symlink as a
    /// post-cmd: `ln -sf /usr/bin/batcat /usr/local/bin/bat`.
    ///
    /// Same Debian-rename gotcha as `base_containerfile_installs_bat_symlink`,
    /// but on the host. The user previously had a manual symlink
    /// at /usr/local/bin/bat — undocumented drift that would
    /// vanish on a fresh re-image. This post-cmd makes the
    /// symlink reproducible and survives `forge teardown` +
    /// re-setup.
    #[test]
    fn server_role_installs_bat_symlink() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let symlink = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p)
                    if p.run().contains("ln -sf /usr/bin/batcat /usr/local/bin/bat") =>
                {
                    Some(p)
                }
                _ => None,
            })
            .expect(
                "server role must install a post-cmd creating /usr/local/bin/bat as a symlink \
                 to /usr/bin/batcat — Debian's bat package installs the binary under the \
                 `batcat` name only",
            );

        // The check guards against forge re-running the symlink on
        // every invocation. `test -L` is intentionally stricter
        // than `test -e` — if a regular file ever ends up at this
        // path (e.g. someone copied batcat instead of symlinked),
        // forge corrects it on the next run.
        assert_eq!(
            symlink.check(),
            Some("test -L /usr/local/bin/bat"),
            "server role bat symlink post-cmd must check `test -L /usr/local/bin/bat` (link, \
             not just any file) so a copied binary or stub at that path triggers a re-symlink"
        );
    }

    /// The server role's `chsh` post-command must run via `sudo`,
    /// not bare.
    ///
    /// `chsh` is setuid-root on Debian and routes through PAM
    /// (`/etc/pam.d/chsh`) which authenticates the invoking user
    /// against `common-auth` even when changing their own shell —
    /// i.e. it interactively prompts for the user's password.
    /// forge's executor invokes post-commands non-interactively, so
    /// a bare `chsh -s /bin/zsh ...` fails with `chsh: PAM:
    /// Authentication failure` and the whole `forge setup server`
    /// run aborts mid-plan.
    ///
    /// Wrapping with `sudo` short-circuits the chsh PAM auth chain
    /// via `pam_rootok.so` (already-root → succeed without prompt),
    /// and the documented NOPASSWD sudoers entry on the user's host
    /// means sudo itself doesn't prompt either. Net effect: the
    /// post-command works the same way on a tty-attached host run,
    /// inside a fresh LXC test container, or on any minimally
    /// provisioned Debian box where the target user happens to have
    /// no password set yet.
    ///
    /// The original failure was caught by running the server role
    /// inside a clean Debian trixie LXC where `august` was created
    /// via plain `useradd` (no password). The bare `chsh` line in
    /// the post-cmd hit PAM's `common-auth`, returned exit status
    /// 1, and the executor surfaced it as
    /// `Error: post-command \`chsh -s /bin/zsh $(whoami)\` exited
    /// with exit status: 1`. This test pins the `sudo` prefix so a
    /// future "clean up the role file" pass can't silently drop it
    /// and reintroduce the regression.
    #[test]
    fn server_role_chsh_uses_sudo() {
        use forge_common::plan::Action;

        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        let chsh = plan
            .actions
            .iter()
            .find_map(|a| match a {
                Action::PostCommand(p) if p.run().contains("chsh ") => Some(p),
                _ => None,
            })
            .expect(
                "server role must include a post-command that calls `chsh` to default the \
                 user's login shell to zsh",
            );

        assert!(
            chsh.run().starts_with("sudo "),
            "the chsh post-command must be invoked via `sudo` so PAM's `chsh` stack \
             short-circuits via pam_rootok and forge's non-interactive run doesn't hang \
             on a password prompt; got `{}`",
            chsh.run()
        );

        // The check should still gate idempotency — once the user's
        // shell is zsh, reruns must be no-ops. We don't pin the exact
        // form of the check string (getent vs grep /etc/passwd vs
        // anything else that detects /bin/zsh) so a reasonable
        // future tweak doesn't break this test, but the field must
        // be present.
        assert!(
            chsh.check().is_some(),
            "the chsh post-command must carry a `check` so reruns of `forge setup server` \
             skip a redundant chsh once the user's shell is already zsh"
        );
    }

    /// `dotfiles/.tmux.conf` must declare RGB and clipboard
    /// capabilities for the host's foot terminal so 24-bit color
    /// and OSC52 clipboard escapes propagate from inner pty
    /// programs (host shells, dev container shells) up through
    /// tmux to foot.
    ///
    /// The previous config had `terminal-overrides ",xterm-256color:RGB"`
    /// — a regex match against `xterm-256color` that does NOT
    /// match `foot` or `foot-extra`, so tmux stripped truecolor
    /// escapes when running under foot. Symptom: TUI apps inside
    /// the dev container quantised their palette to 256-color
    /// even though every layer in the chain (foot, tmux,
    /// container shell) was technically truecolor-capable.
    ///
    /// This test pins the modern (tmux ≥3.2) `terminal-features`
    /// declarations for both RGB and clipboard, plus the
    /// `set-clipboard on` switch that actually enables OSC52
    /// passthrough inside tmux.
    #[test]
    fn tmux_conf_advertises_foot_truecolor_and_clipboard() {
        const TMUX_CONF: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../dotfiles/.tmux.conf"
        ));

        for marker in [
            // Truecolor passthrough for foot (covers `foot` AND
            // `foot-extra` via the glob).
            "terminal-features \",foot*:RGB\"",
            // Clipboard passthrough for foot.
            "terminal-features \",foot*:clipboard\"",
            // The actual switch that enables OSC52 forwarding.
            "set -g set-clipboard on",
        ] {
            assert!(
                TMUX_CONF.contains(marker),
                "dotfiles/.tmux.conf must contain `{marker}` — without it, truecolor and/or \
                 OSC52 clipboard escapes get stripped at the tmux boundary when running under \
                 foot, breaking dev-container TUI rendering and the `wl-copy` shim"
            );
        }

        // Negative invariant: the legacy `xterm-256color`-only
        // override must not be the SOLE truecolor declaration. We
        // still allow it to be present (covers ssh-from-an-xterm
        // sessions), but if it's the only declaration the foot
        // path is broken.
        let has_legacy_only = TMUX_CONF.contains(",xterm-256color:RGB")
            && !TMUX_CONF.contains(",foot*:RGB");
        assert!(
            !has_legacy_only,
            "dotfiles/.tmux.conf has `xterm-256color:RGB` but is missing `foot*:RGB` — under \
             foot, tmux strips truecolor escapes and dev-container TUIs quantise to 256-color"
        );
    }

    /// `themes/schemes/catppuccin-mocha.yaml` must use catppuccin's
    /// *style-guide* recommended colors for base03 (comments) and
    /// base04 (subtle text / line numbers), not the upstream
    /// catppuccin/base16 export's choice of surface1/surface2.
    ///
    /// Background: catppuccin's surface1 (#45475a) and surface2
    /// (#585b70) are part of the *structural background* palette
    /// tier, intentionally low-contrast against base00 (#1e1e2e).
    /// catppuccin's own style guide (catppuccin/catppuccin
    /// docs/style-guide.md) explicitly assigns:
    ///
    ///   * Subtle text / Line numbers  → Overlay 1  (#7f849c)
    ///   * Comments                    → Overlay 2  (#9399b2)
    ///
    /// The upstream base16 export uses surface1/surface2 anyway,
    /// which causes zsh-autosuggest (ANSI bright0 = base03), nvim
    /// comments, and TUI line numbers to render at ~1.5:1
    /// contrast — well below WCAG's 3:1 minimum, visibly
    /// unreadable. This test pins the divergence so a future
    /// "sync from upstream" doesn't silently regress the fix.
    /// `~/.config/containers/containers.conf` must ship as a
    /// forge-owned dotfile but must NOT pin `dns_servers`.
    ///
    /// Why we ship it at all: the path is reserved for future
    /// per-user podman overrides, and shipping it (even with no
    /// active overrides) keeps it forge-managed so an ad-hoc
    /// edit can't squat the path. The body documents what was
    /// previously tried and why it was the wrong fix.
    ///
    /// Why we DON'T set `dns_servers` here: a previous version of
    /// this file pinned public DNS (1.1.1.1 / 8.8.8.8) for
    /// `podman run`, but networks that block outbound DNS to
    /// anything except the gateway (the user's actual LAN, plus
    /// many hotels / coffee shops / corporate networks / captive
    /// portals) drop those queries — `forge dev up` would then
    /// fail to resolve anything. The build-side DNS issue this
    /// originally tried to solve is fixed structurally in
    /// `crates/forge/src/commands/dev.rs` instead, by passing
    /// `--dns=` flags read from the host's resolv.conf to
    /// `podman build` directly. For `podman run`, pasta's default
    /// behaviour (auto-discover from host resolv.conf) is correct
    /// on every network the host itself works on.
    ///
    /// This test pins both sides: the file SHIPS (path is
    /// forge-owned), and it does NOT set `dns_servers` (regression
    /// guard against re-introducing the public-DNS-on-filtered-
    /// LAN failure mode).
    #[test]
    fn dotfiles_ship_containers_conf_with_no_dns_override() {
        let entry = DOTFILES
            .get_file(".config/containers/containers.conf")
            .expect(
                "dotfiles tree must ship .config/containers/containers.conf so the path is \
                 forge-owned (even though we deliberately set no overrides — see the file \
                 body for the design rationale)",
            );
        let body = entry
            .contents_utf8()
            .expect("containers.conf must be UTF-8");

        // Negative invariant: the body must not have a live
        // `dns_servers` setting. We allow the *word* in comments
        // (the body documents what was tried and why), so the
        // assertion has to look for the key+`=` pattern, not just
        // the word — `# was previously dns_servers = ...` is fine.
        for (lineno, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }
            assert!(
                !trimmed.starts_with("dns_servers"),
                "containers.conf must NOT set `dns_servers` — pinning public DNS here \
                 broke `podman run` on LANs that block outbound DNS to anything except \
                 the gateway, and the build-side fix lives in \
                 crates/forge/src/commands/dev.rs instead. Got line {}: {}",
                lineno + 1,
                line
            );
        }

        // The file must be picked up by the server role's dotfile
        // glob — without this the file lives in the embedded tree
        // but never lands at $HOME/.config/containers/containers.conf
        // and forge can't claim ownership of the path.
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        use forge_common::plan::Action;
        let written = plan.actions.iter().any(|a| match a {
            Action::WriteFile { rel, .. } => rel
                .to_string_lossy()
                .ends_with(".config/containers/containers.conf"),
            _ => false,
        });
        assert!(
            written,
            "server role dotfiles glob must match \
             `.config/containers/containers.conf` so the file actually lands at \
             $HOME/.config/containers/containers.conf during forge setup. The embedded \
             entry exists but no WriteFile action targets it — the role's `dotfiles` \
             list is missing a `.config/containers/**` (or equivalent) entry."
        );
    }

    /// `dotfiles/.config/nvim/lua/plugins/hardtime.lua` must ship in
    /// the embedded tree, reference the right upstream plugin, and
    /// land at `~/.config/nvim/lua/plugins/hardtime.lua` via the
    /// server role's nvim dotfile glob.
    ///
    /// hardtime.nvim coaches the user out of inefficient motions
    /// (`jjjj` → "use `5j`"). The file's existence is half the
    /// invariant; the other half is the disabled-filetype list,
    /// which is where this user's experience lives or dies.
    ///
    /// Specifically `"markdown"` MUST be in the disabled-filetypes
    /// list. Without it, hardtime fires inside Obsidian / note
    /// buffers where line-by-line `j` movement through prose is
    /// the right motion, not a sin — at which point the user's
    /// note-taking flow gets interrupted by a "use `5j` instead"
    /// notification on every paragraph and the plugin gets
    /// disabled in frustration. We assert this single override
    /// because it's the one a future "let's also coach in
    /// markdown" tweak would silently regress.
    ///
    /// We do NOT assert on the upstream defaults (max_count,
    /// disable_mouse, hint, etc.) because those are tunable
    /// preferences, not invariants — the user might decide
    /// `max_count = 5` is gentler and still right for them. The
    /// pin is just: the plugin is wired up, points at the right
    /// repo, and excludes prose buffers.
    #[test]
    fn dotfiles_ship_hardtime_with_markdown_disabled() {
        let entry = DOTFILES
            .get_file(".config/nvim/lua/plugins/hardtime.lua")
            .expect(
                "dotfiles tree must ship .config/nvim/lua/plugins/hardtime.lua so the user's \
                 nvim picks up the m4xshen/hardtime.nvim coaching plugin via lazy.nvim's \
                 `{ import = \"plugins\" }` auto-discovery",
            );
        let body = entry
            .contents_utf8()
            .expect("hardtime.lua must be UTF-8");

        // Pin the upstream repo. A future rename / fork that drops
        // the m4xshen identifier would silently break the plugin
        // installation since lazy.nvim resolves the URL from the
        // first string in the spec table. The literal `"m4xshen/`
        // segment is unambiguous.
        assert!(
            body.contains("\"m4xshen/hardtime.nvim\""),
            "hardtime.lua must reference the m4xshen/hardtime.nvim upstream repo verbatim — \
             a fork or renamed identifier would silently fail to install. Got body:\n{body}"
        );

        // The disabled_filetypes block must contain `"markdown"`.
        // We look for the literal quoted token rather than parsing
        // Lua because (a) the surrounding context is a Lua list,
        // (b) any other appearance of the word "markdown" in the
        // file would be inside a comment, and (c) a comment with
        // the bare word doesn't satisfy the assertion since we
        // require the quotes — Lua list entries are quoted, prose
        // mentions aren't.
        assert!(
            body.contains("\"markdown\""),
            "hardtime.lua must include `\"markdown\"` in disabled_filetypes — without it the \
             plugin fires inside Obsidian / note buffers where line-by-line `j` motion \
             through prose is the right keystroke, not a sin, and the constant \"use 5j \
             instead\" notifications make the plugin unbearable for note-taking. Got \
             body:\n{body}"
        );

        // The file must be picked up by the server role's nvim
        // dotfile glob — without this the file lives in the
        // embedded tree but never lands at $HOME, and the user's
        // nvim doesn't see it. The glob is `.config/nvim/**`
        // which should match anything under that subtree.
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        use forge_common::plan::Action;
        let written = plan.actions.iter().any(|a| match a {
            Action::WriteFile { rel, .. } => rel
                .to_string_lossy()
                .ends_with(".config/nvim/lua/plugins/hardtime.lua"),
            _ => false,
        });
        assert!(
            written,
            "server role dotfiles glob must match \
             `.config/nvim/lua/plugins/hardtime.lua` so the plugin actually lands in the \
             user's nvim config tree during forge setup. The embedded entry exists but no \
             WriteFile action targets it — the role's `.config/nvim/**` glob may have been \
             narrowed."
        );
    }

    /// `dotfiles/.config/nvim/lua/plugins/notify.lua` must ship in
    /// the embedded tree, point at the right upstream repo, AND
    /// re-apply the noise filter that lives in
    /// `lua/config/options.lua`.
    ///
    /// nvim-notify replaces `vim.notify` when it loads, which would
    /// silently undo the options.lua wrapper that filters out
    /// mason / lspconfig / "not found" / "spawn" startup spam. The
    /// notify.lua plugin spec has to re-wrap the new vim.notify
    /// with the same filter; otherwise the visual upgrade lands
    /// alongside a regression where the toast UI starts showing
    /// every dropped LSP warning. This test pins the four pattern
    /// literals so a future "let me clean up the filter" pass that
    /// drops one would be caught at `cargo test`, not at the next
    /// `nvim` startup.
    ///
    /// We deliberately pin the LITERAL string tokens (e.g.
    /// `"mason"`) rather than parse the Lua: the patterns are
    /// short, the file is small, and a regex/parse approach would
    /// be overkill for what amounts to a "did you keep these four
    /// strings in there" check.
    ///
    /// We do NOT pin the visual settings (render, stages, timeout,
    /// background_colour) — those are tunable preferences a future
    /// edit should be free to change without a test-and-fix dance.
    /// The pin is just: the plugin is wired up, points at the right
    /// repo, and preserves the noise-filter behavior.
    #[test]
    fn dotfiles_ship_nvim_notify_with_filter_preserved() {
        let entry = DOTFILES
            .get_file(".config/nvim/lua/plugins/notify.lua")
            .expect(
                "dotfiles tree must ship .config/nvim/lua/plugins/notify.lua so nvim-notify \
                 is auto-discovered by lazy.nvim's `{ import = \"plugins\" }`",
            );
        let body = entry
            .contents_utf8()
            .expect("notify.lua must be UTF-8");

        assert!(
            body.contains("\"rcarriga/nvim-notify\""),
            "notify.lua must reference the rcarriga/nvim-notify upstream repo verbatim — a \
             rename or fork identifier would silently fail to install. Got body:\n{body}"
        );

        // Pin every filter pattern from lua/config/options.lua. If
        // the user adds a new pattern there but forgets to add it
        // here, the toast UI starts leaking the spam — these four
        // are the original set; new patterns should be added in
        // both places (and pinned here) in the same change.
        for pattern in ["\"mason\"", "\"lspconfig\"", "\"not found\"", "\"spawn\""] {
            assert!(
                body.contains(pattern),
                "notify.lua must re-apply the same vim.notify filter that lives in \
                 lua/config/options.lua — pattern {pattern} is missing, which means the \
                 corresponding mason/LSP startup messages will leak into the toast UI \
                 once nvim-notify replaces vim.notify on plugin load. Got body:\n{body}"
            );
        }

        // The file must be picked up by the server role's nvim
        // dotfile glob — without this the file lives in the
        // embedded tree but never lands at $HOME during forge
        // setup, and nvim never loads the spec.
        let roles = RolesFile::from_toml(ROLES_TOML).unwrap();
        let cfg = roles.resolved(Role::Server).unwrap();
        let plan = build_plan(&cfg, &EmbeddedDotfiles).unwrap();

        use forge_common::plan::Action;
        let written = plan.actions.iter().any(|a| match a {
            Action::WriteFile { rel, .. } => rel
                .to_string_lossy()
                .ends_with(".config/nvim/lua/plugins/notify.lua"),
            _ => false,
        });
        assert!(
            written,
            "server role dotfiles glob must match \
             `.config/nvim/lua/plugins/notify.lua` so the plugin actually lands in the \
             user's nvim config tree during forge setup"
        );
    }

    #[test]
    fn catppuccin_mocha_uses_overlay_for_text_tiers() {
        let entry = SCHEMES
            .get_file("catppuccin-mocha.yaml")
            .expect("themes/schemes/catppuccin-mocha.yaml must ship");
        let body = entry
            .contents_utf8()
            .expect("catppuccin-mocha.yaml must be UTF-8");

        // base03 = Overlay 2 (comments). The literal hex must
        // appear on a `base03:` line. We assert on the line shape
        // rather than just "contains 9399b2" so a future edit that
        // accidentally puts overlay2 on the wrong slot still fails.
        assert!(
            body.contains(r#"base03: "9399b2""#),
            "catppuccin-mocha.yaml base03 must be `9399b2` (Overlay 2 — comments per \
             catppuccin's style guide). The upstream base16 export uses surface1 (#45475a) \
             which is too dim for text and renders zsh-autosuggest / nvim comments \
             unreadable at ~1.5:1 contrast against base00"
        );
        assert!(
            body.contains(r#"base04: "7f849c""#),
            "catppuccin-mocha.yaml base04 must be `7f849c` (Overlay 1 — subtle text / line \
             numbers per catppuccin's style guide). The upstream base16 export uses surface2 \
             (#585b70) which is too dim for text"
        );

        // Negative invariant: the dim surface tones must NOT
        // appear on base03/base04 lines. Catches a regression
        // where someone "syncs from catppuccin/base16" and
        // re-introduces the readability bug.
        assert!(
            !body.contains(r#"base03: "45475a""#),
            "catppuccin-mocha.yaml base03 must not be `45475a` (surface1) — that's the value \
             from catppuccin/base16's upstream Mocha export, which causes the readability \
             regression this scheme deliberately diverges from"
        );
        assert!(
            !body.contains(r#"base04: "585b70""#),
            "catppuccin-mocha.yaml base04 must not be `585b70` (surface2) — same regression"
        );
    }

}
