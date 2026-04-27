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

}
