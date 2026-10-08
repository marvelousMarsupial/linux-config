# Testing

[Documentation index](README.md)

Run from the repository root:

```sh
./scripts/check
./scripts/check --full
./scripts/install --dry-run
```

The basic check validates shell/Python/Lua syntax, TOML, and all six palette renders.
The full check also runs the offline integration suite. Latest full run on
2026-10-07 with unrestricted access: **97 tests run, 96 passed, 1 failed,
none skipped**. The editor smoke test expects unchanged HTML on save, but an
available formatter rewrites that fixture. The assertion needs to distinguish
code formatting from prose preservation; the product deliberately formats code
on save. Syntax, TOML, Lua and six palette renders passed. Tests requiring Neovim and installed plugins can
be skipped on a machine without them; inspect the output before claiming coverage.

## Automated coverage

| Area | What is exercised |
| --- | --- |
| Installation | Dry runs, repeated copies, permissions, private backups, destination and backup-path symlinks, collisions, mode repairs, preserved local settings and named retirement. |
| Themes | Six palettes; generated Lua/TOML and native Foot/Fuzzel validation; previews and rejected templates/palettes. |
| Neovim | Eight filetypes, Markdown/text wrapping, saves, plugins, language/formatter configuration, actual LSP fixture attachment/diagnostics/hover/definition, external formatter execution, modeline rejection and removed note automation. |
| Bat | Rounded fitted borders, ANSI colors, Unicode, width limits, multiple/empty/large files, pipelines and PTY pager selection/quoting. |
| Desktop helpers | Menu and background commands: recorded menu actions, display layouts/escaping, status parsing, battery warning/reset loop, session startup ordering, launcher/Zen argument forwarding, screenshots, notes, screen-only brightness controls and visible brightness failures. |
| Systemd / updates | Real swaymsg subscription and disconnect against a disposable IPC socket; session startup failure paths; completed APT upgrade detection, notification baseline, deduplication and delivery retry. |
| Wi-Fi | Escaped SSIDs and saved profiles, strongest 15 networks, cached credentials, authentication retry, cancellation, timeouts and exact picker selections. |
| Shell/Yazi | Remote terminfo option validation, alias reminder privacy and prefix matching, and Yazi editor argument forwarding. |
| Setup | Default/optional branches, local verified release installation, existing-install backup, rejected version/digest/architecture, failed binary/plugin validation and temporary-file cleanup. |
| Security | Runtime environment injection and symlinks, private backups under a shared umask, editor safeguards and download rejection. |

## Working on tests

Tests are grouped by behavior in `tests/test_*.py`; `support.py` and
`command_support.py` provide shared imports and isolated command fixtures.
To run a focused area:

```sh
python3 -B -m unittest discover -s tests -p 'test_wifi.py' -v
```

Use temporary homes and explicit command stubs for external actions. The suite
never intentionally changes the live power state, network, clipboard or services.
Neovim uses installed plugins with temporary config/cache/state; its LSP server
and JSON formatter fixtures run locally and require no development tools.
Editor download tests generate local archives and stub curl/sudo; they exercise
checksum, extraction and installation logic without fetching a release.
Documentation links are checked too. No extra testing packages are required.

Coverage describes observable behavior, not exhaustive branch coverage or a
security guarantee. Native Foot/Fuzzel parsers are used when available; Neovim
and bat checks report skips when their runtime prerequisites are missing.

A bad template fails before theme writes begin; filesystem failures can still leave
a multi-file theme apply incomplete. Atomic replacement is per file.

## Manual verification

- Open a new Foot window and inspect the launcher, info popup, notifications,
  rounded bat output and the selected theme.
- Reload Sway with Super+Shift+c and verify window focus, layouts, monitor profiles,
  manual locking, 30-minute idle locking and locking before suspend.
- Test Yazi preview and file opening with `file` installed; optional formats need
  their own preview dependencies.
- Test real Wi-Fi authentication and libvirt operations.
- Test XPS audio and brightness on the laptop. The VM has no sound card or backlight.
- Log out and back in to confirm Sway journal logging and single Mako startup.
- Verify language-server attachment only on a machine with optional tools installed.
- Test a clean installation with working network access and privileges.
- Follow the host checks in [the security audit](security-audit.md).

The automated suite does not certify live network, power or installation
behavior. Separate host checks and the firewall network-namespace probes are
recorded in [current state](state.md) and [firewall](firewall.md). Restricted
sessions may skip the IPC test when local sockets are prohibited. Track
outstanding checks in [TODO](TODO.md).

## Clipboard-history removal — 2026-10-08

Removed automatic capture, its shortcut/help entry and the cliphist package
requirement. Repo and installed Sway/keybind help files match. Normal clipboard
tools remain configured. `./scripts/check --full`: 152 tests, 150 passed, two
skipped (GTK display and local IPC unavailable). Syntax, TOML and six theme
renders passed. Sway validation could not start because the sandbox prohibits
Wayland sockets, including with a headless backend. Live watcher shutdown and
clipboard behavior could not be checked; log out/in to apply the startup change.
Existing clipboard history was not inspected or erased.

## Idle and suspend locking — 2026-10-08

Repo and installed Sway configs match. Checked that exactly one startup watcher
uses a 1800-second idle timeout and `-w`, with both idle and before-sleep commands
invoking `swaylock -f`; executed both shell commands against an isolated lock
stub to verify argument forwarding. Removed the blanket fullscreen inhibitor.
`./scripts/check --full`: 152 tests, 150 passed, two skipped (GTK display and
local IPC unavailable). Syntax, TOML and six theme renders passed.
Live activation and idle/password-unlock/suspend behavior remain unverified
because this sandbox cannot access Sway or the user service manager. Log out/in
and follow the checks in [systemd](systemd.md) before relying on automatic locks.

## APT policy pre-install check — 2026-10-08

Copied installed `apt.conf.d` into a temporary directory and overlaid both repo
APT files. Used `APT_CONFIG` to select that directory before native `apt-config`
loading. Confirmed exactly four intended origin patterns and every explicit
periodic/reboot/removal setting. No system files changed. Installation and the
privileged unattended-upgrade dry run remain blocked by sandbox restrictions.
The idle timeout was changed to 1800 seconds in repo and installed Sway config;
files match. Basic syntax, TOML and six theme checks passed.
