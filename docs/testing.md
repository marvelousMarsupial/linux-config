# Testing

[Documentation index](README.md)

Run from the repository root:

```sh
./scripts/check
./scripts/check --full
./scripts/install --dry-run
```

The basic check validates shell, Python and Lua syntax, TOML, and renders all
six palettes in a temporary home. It needs `python3-jinja2` and `python3-yaml`;
zsh and Neovim checks are skipped with a message when those are not installed.
The full check also runs the offline integration suite. To run one area:

```sh
python3 -B -m unittest discover -s tests -p 'test_wifi.py' -v
```

Tests are grouped by behavior in `tests/test_*.py`; `support.py` and
`command_support.py` provide shared imports and isolated command fixtures.

## What the suite covers

| Area | What is exercised |
| --- | --- |
| Installation | Dry runs, repeated copies, permissions, private backups, destination and backup-path symlinks, collisions, mode repairs, preserved local settings, selective dark-theme merges, named retirement and rendering for another username. |
| Setup | Default and optional branches, local verified release installation, existing-install backup, rejected version, digest and architecture, failed binary or plugin validation, temporary-file cleanup. Setup refuses to run as root, so these tests need a normal user. |
| Remote bootstrap | Piped execution, argument forwarding, setup and clone failure exit codes, temporary checkout cleanup. |
| Themes and glass | Six palettes; generated Lua and TOML; native Foot and Fuzzel validation when installed; previews; rejected templates and palettes; Zen profile sync, import order and glass toggling. |
| Neovim | Eight filetypes, Markdown and text wrapping, saves, plugins, LSP fixture attachment, diagnostics, hover and definition, external formatter execution, modeline rejection, disabled browser preview, operator symbol toggling. Save tests compare prose files exactly and use deterministic external fixtures. |
| Bat | Rounded corners and a right edge on bat-shaped input, ANSI colors, Unicode widths, pass-through of plain text, piped output, pager override and exit status; per-file frame widths when batcat is installed. |
| Health report | Escaping, section failure isolation, private modes, retention, demo page, service tile states, a read-only OpenSnitch database and the install receipt. |
| Desktop helpers | Recorded menu actions, display layouts and escaping, status parsing, battery warning and reset loop, session startup ordering, launcher and Zen argument forwarding, screenshots, notes, brightness controls and visible brightness failures, the appearance picker, the notification viewer and the status toggle. |
| Popup guard | Tree-snapshot tests for reveal, delayed visibility, moves and focus, startup recovery, multiple popups, restoration, native dialogs, hidden workspaces, explicit fullscreen priority and Foot opacity delivery over a PTY. |
| Systemd and updates | Real swaymsg subscription and disconnect against a disposable IPC socket (skipped where local sockets are prohibited); session startup failure paths; APT upgrade detection, notification baseline, deduplication and delivery retry. |
| Portals | Foot and Yazi wrapper checks for literal filenames, single and multiple selection, cancellation, directory fallback cleanup, fullscreen exit before launch and restoration sequencing; Zen launcher environment. |
| KeePassXC | First launch, focus, scratchpad show and hide, invalid IPC, launch-time dark settings preserving other preferences. |
| Wi-Fi | Escaped SSIDs and saved profiles, strongest 15 networks, cached credentials, authentication retry, cancellation, timeouts and exact picker selections. |
| Shell and Yazi | Real interactive zsh prefix history and Ctrl+R insertion in both arrow formats and editing modes; fzf-tab installer dry runs, backups, repeats and commit rejection; remote terminfo helper validation; alias reminder privacy; Yazi editor argument forwarding. |
| Backups | Isolated home and stubbed Restic: missing configuration, private credential permissions, source and exclusion arguments, local repository recursion, explicit init and restore, error status. |
| Security | Runtime environment file injection and symlinks, private backups under a shared umask, editor safeguards, download rejection. |
| Project | Minimal package list, retired files absent, every local documentation link resolves. |

Tests use temporary homes and command stubs for network, power and privileged
actions. The suite never changes the live power state, network, clipboard or
services. Passing it does not prove that a real action works, that a listening
service is absent, or that the host is secure. Atomic replacement is per file;
a filesystem failure can still leave a multi-file theme apply incomplete.

## Manual verification

Most live checks are collected in the [manual checklist](manual.md). The
remaining items here need an interactive session the sandbox cannot provide:

- After installing fzf-tab, verify Tab selection and cancellation, previews and
  completion-based grey suggestions. Prefix Up/Down is user-confirmed.
- Restart the popup guard, toggle Foot with Super+Shift+Space and check 60%
  floating opacity versus untinted tiled windows. Reopen an upload picker.
- Reload Sway and verify focus, layouts, monitor profiles, manual locking,
  30-minute idle locking and locking before suspend.
- Test Yazi preview and file opening; optional formats need their own tools.
- Test real Wi-Fi authentication and libvirt operations.
- Test audio, brightness and volume keys on the laptop.
- Log out and back in to confirm Sway journal logging and a single Mako.
- Verify language-server attachment only on a machine with `--dev-tools`.
- Run a clean installation with network access and privileges.
- Follow the host checks in the [security audit](security-audit.md).

## Verification status

The user ran full setup on the development VM and rebooted. Read-only checks
confirmed required packages, installed user files, both APT policy files, the
shell, group memberships, locale and portal backend files. The user confirmed
the Zen upload through Yazi, the fullscreen exit and floating picker, the
notification viewer, the graphical authentication prompt, OpenSnitch collection
and prefix history. Those checks predate later changes and do not establish a
fresh-machine installation of the current revision.

Not yet verified anywhere: a real remote bootstrap run, a fresh installation of
the current revision, live idle and suspend locking, save and overwrite through
the picker, screen sharing, physical multi-monitor behavior, and everything in
the hardware section of [TODO](TODO.md). Dated host findings stay in the
[security audit](security-audit.md); they are not statements about running
services today.
