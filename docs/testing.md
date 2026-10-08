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
2026-10-08 in the restricted session: **193 tests run, 191 passed, two skipped**
(GTK display routing and local IPC unavailable). Syntax, TOML, Lua and six
palette renders passed. The editor smoke test now preserves entire prose files
exactly while allowing code formatting. Deterministic external HTML/JSON fixtures
verify formatting on save and the resulting disk contents, even without optional
formatters installed. This resolves the prior unchanged-HTML assertion failure.
Tests requiring Neovim and installed plugins can be skipped on a machine without
them; inspect the output before claiming coverage.

Remote bootstrap checks exercise piped execution, argument forwarding, setup/clone
failure exit codes and temporary checkout cleanup without network or sudo. A
focused follow-up also verified fresh monitor installation and preservation of
subsequent local edits. The published URL and real sudo prompts remain untested.

## Automated coverage

| Area | What is exercised |
| --- | --- |
| Installation | Dry runs, repeated copies, permissions, private backups, destination and backup-path symlinks, collisions, mode repairs, preserved local settings, selective dark-theme merges and named retirement. |
| KeePassXC / app themes | First launch, focus and scratchpad show/hide; invalid IPC; launch-time dark settings preserve other app preferences and remain repeatable. |
| Themes | Six palettes; generated Lua/TOML and native Foot/Fuzzel validation; previews and rejected templates/palettes. |
| Neovim | Eight filetypes, Markdown/text wrapping, saves, plugins, language/formatter configuration, actual LSP fixture attachment/diagnostics/hover/definition, external formatter execution, modeline rejection, disabled unverified browser preview, retained in-buffer Markdown shortcut and removed note automation. |
| Bat | Rounded fitted borders, ANSI colors, Unicode, width limits, multiple/empty/large files, pipelines and PTY pager selection/quoting. |
| Desktop helpers | Menu and background commands: recorded menu actions, display layouts/escaping, status parsing, battery warning/reset loop, session startup ordering, launcher/Zen argument forwarding, screenshots, notes, screen-only brightness controls and visible brightness failures. |
| Systemd / updates | Real swaymsg subscription and disconnect against a disposable IPC socket; session startup failure paths; completed APT upgrade detection, notification baseline, deduplication and delivery retry. |
| Portals | Offline Foot/Yazi wrapper checks for literal filenames, single/multiple selection, cancellation and directory fallback cleanup; Zen launcher environment. The user confirmed a live single-file upload and fullscreen picker reveal; other dialog/backend paths still require live checks in [portals](portals.md). |
| Wi-Fi | Escaped SSIDs and saved profiles, strongest 15 networks, cached credentials, authentication retry, cancellation, timeouts and exact picker selections. |
| Shell/Yazi | Real interactive Zsh prefix history and Ctrl+R insertion, both arrow formats and editing modes; plugin installer dry runs/backups/repeats and commit rejection; remote terminfo option validation, alias reminder privacy and prefix matching, and Yazi editor argument forwarding. |
| Desktop setup | Side-effect-free dry run, verified release download/rejection, archive traversal/link rejection, reuse of installed apps, replacement rollback and destination-user paths. Real APT/source builds and fresh-machine behavior need host testing. |
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

- In a new terminal, verify Tab selection/cancellation, directory/file previews
  and completion-based grey suggestions after downloading fzf-tab. Prefix
  Up/Down is user-confirmed. See [shell verification](shell.md#verification).

- Restart `sway-popup-guard.service`, toggle Foot with Super+Shift+Space, and check
  60% floating opacity versus the tiled theme opacity. Reopen an upload picker
  and check its background too. Offline tests verify transitions and PTY delivery.
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

## Current live verification boundaries — 2026-10-08

The user ran full setup on the development VM and rebooted. Installed package,
config and policy checks and a working Yazi upload do not verify a fresh target
installation of the latest revision. Automated setup tests use local release
fixtures and mocked privilege/network actions. Sandbox sudo, D-Bus, Sway and
local socket restrictions still limit live checks.

Confirmed by the user: a Zen upload opens Yazi, and opening the picker while
fullscreen exits fullscreen so the floating picker is visible. This behavior is
intentional and must be preserved. Picker tests also cover exit-before-launch and
restoration sequencing offline. Live save/overwrite, multiple/directory selection,
fullscreen restoration after selection/cancel and screen sharing remain pending.

The repo configures GTK/Zen dark preferences and KeePassXC/OpenSnitch launch-time
dark settings, including OpenSnitch's theme dependency. App appearance, database
pairing/autofill, idle/suspend locks, notifications and hardware behavior require
live checks. Old dated audit results remain in [security audit](security-audit.md);
they are not current assertions about running services.

## Polkit session integration

Setup includes mate-polkit. Sway launches the agent directly so its process
belongs to the desktop login session, rather than the user manager's session.
The old user unit is retired with a backup by config installation. On 2026-10-08 the user confirmed the graphical authentication test works after
applying the direct Sway startup fix. See
[systemd](systemd.md#graphical-authentication). The former user service ran on the
VM but the user's pkexec test reported no authentication agent.

Desktop service migration (2026-10-08): full offline suite passed, 175 tests,
2 skips. Battery monitor, swayidle and udiskie now use session-scoped user units.
Unit verification reported a sandbox SO_PASSCRED restriction; live activation
requires logout/relogin to replace the previous directly started processes.

Optional Restic integration (2026-10-08): full offline suite passed, 181 tests,
2 skips. Backup tests use an isolated home and stubbed Restic, covering missing
configuration, private credential permissions, source/exclusion arguments, local
repository recursion (including symlinks), explicit init/restore and error status.
Setup dry-run with --backups includes Restic without enabling its timer. No real
repository backup/restore was run: this VM has no Restic binary or configured target.
# Journal fixes — 2026-10-08

The session launcher now queries failed session services before resetting them;
unloaded services are left alone. The Yazi picker leaves an empty result on normal
file cancellation. Terminal failures still propagate, and fullscreen restoration
is unchanged. The 19 command tests and six portal tests passed after these changes.

UPower was added to the setup package list. Installing it on the current VM and
checking live service state require host privileges unavailable in the sandbox.
Bluetooth availability and GTK/GNOME/Wayland compatibility messages remain visible;
no journal filters or log deletion were added. Live file selection/cancellation
still needs verification in the desktop session.
