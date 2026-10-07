# Verification — 2026-10-07

Run `./scripts/check --full` from the repository. The audit passed all 25 offline integration tests, script and Lua syntax checks, TOML parsing, and renders of all six palettes. Installed Foot and Fuzzel also accepted the live configuration. A repeat installer dry run reported zero changes.

## Coverage

- Installer: dry runs, repeat installs, backups, executable permissions, symlink handling, directory collision refusal, local monitor/color/lockfile preservation, obsolete integration removal, and bytecode exclusion.
- Themes: all six palettes, native Foot/Fuzzel configuration checks, generated Lua/TOML, previews without writes, and template/palette failures without partial rendering.
- Neovim: startup with installed plugins, eight filetypes, Markdown/text wrapping, saves, completion, Telescope, language configuration, formatting configuration, and absence of Obsidian/file automation.
- Bat: rounded fitted borders, ANSI colors, Unicode, terminal width limits, multiple/empty/large files, piped output and error propagation.
- Helpers: escaped Wi-Fi SSIDs, duplicates, scan timeout and password cancellation; missing battery readings/tools; screenshot cancellation, capture and clipboard success, capture failure; dictation help without runtime changes.
- Setup: default and optional development branches using command stubs. These checks verify control flow, not real downloads or package installation.

## Repairs applied during audit

Removed Foot's undocumented SIGUSR1 reload hook: it could terminate terminals and interrupt theme application. Regenerated the live theme consistently as Summer Dusk; Foot keeps the slightly different background. Updated Yazi theme keys, made installer replacements atomic per file, added backed-up removal of retired integrations, excluded Python bytecode, handled escaped Wi-Fi fields and scan timeouts, and delayed dictation runtime setup until after help parsing. Improved Neovim bootstrap/install failure handling and added the Debian fdfind alias fallback.

Live configuration backups are under `~/.local/state/linux-config/backups/`.

## Remaining manual verification

The execution sandbox cannot access the live Sway, systemd user, or libvirt sockets. Their running behavior is not certified by these tests. Network connection changes, VM launches, power/session actions, dictation recording, and a clean privileged installation were not exercised.

IPython is included in future package installation but was absent from the live VM during this audit. WireGuard tools are now explicitly listed for the VPN helper. Language servers are optional: configuration tests do not demonstrate live server attachment when the executables are absent. Install the optional development tools to enable that workflow.

Reload Sway and open new application windows to inspect generated colors; existing terminals do not reload their palette automatically.

Tmux configuration, helper, shell function, package requirement, theme target, and Neovim multiplexer integration were subsequently removed. The installer backs up and retires the old user files.

Dictation was subsequently removed at user request, including bundled binaries, service, shortcuts, and status controls. The current suite has 24 tests after removal of the dictation help test. Live model data was moved to the configuration backup directory.
