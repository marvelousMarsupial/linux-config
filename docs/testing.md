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
2026-10-07: **90 tests passed, none skipped**. Tests requiring Neovim and installed plugins can
be skipped on a machine without them; inspect the output before claiming coverage.

## Automated coverage

| Area | What is exercised |
| --- | --- |
| Installation | Dry runs, repeated copies, permissions, private backups, destination and backup-path symlinks, collisions, mode repairs, preserved local settings and named retirement. |
| Themes | Six palettes; generated Lua/TOML and native Foot/Fuzzel validation; previews and rejected templates/palettes. |
| Neovim | Eight filetypes, Markdown/text wrapping, saves, plugins, language/formatter configuration, actual LSP fixture attachment/diagnostics/hover/definition, external formatter execution, modeline rejection and removed note automation. |
| Bat | Rounded fitted borders, ANSI colors, Unicode, width limits, multiple/empty/large files, pipelines and PTY pager selection/quoting. |
| Desktop helpers | All 17 commands: recorded menu actions, display layouts/escaping, status parsing, battery warning/reset loop, session startup ordering, launcher/Zen argument forwarding, screenshots, notes and missing commands. |
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
  manual locking and locking before suspend.
- Test Yazi preview and file opening with `file` installed; optional formats need
  their own preview dependencies.
- Test real Wi-Fi authentication and libvirt operations.
- Verify language-server attachment only on a machine with optional tools installed.
- Test a clean installation with working network access and privileges.
- Follow the host checks in [the security audit](security-audit.md).

The sandbox cannot certify the host's running services, firewall or listening
ports. Stubbed/offline tests do not establish real network, power or installation
behavior. Track outstanding checks in [TODO](TODO.md).
