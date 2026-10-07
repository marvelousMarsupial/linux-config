# Scripts

[Documentation index](README.md)

User commands live in `dotfiles/.local/bin/`; repository maintenance commands live
in `scripts/`. The installer copies user commands into `~/.local/bin` with executable
permissions. There are 17 user commands and four maintenance scripts.

| Scripts | Review and resulting behavior |
| --- | --- |
| bat | Arguments forwarded without a shell; piped output preserved; large frames spill to private temporary storage. Unicode width remains an approximation for complex emoji sequences. |
| battery-monitor | Validates percentage thresholds and polling interval; rejects impossible readings; notification failures are tolerated; a critical warning no longer leads to a redundant low warning next poll. |
| keybinds | Static help display; removed obsolete shortcuts. |
| launcher | Quoted argument forwarding; preserves inherited XDG data directories; application-only placeholder. |
| monitor-info | Bounded Sway query, readable invalid-response error, escaped generated monitor descriptions, fallback modes for disabled outputs, refuses nonexistent modes. Only prints configuration. |
| motd | Read-only system information; bounded optional commands. |
| notes-picker | Cancellation is harmless; verifies existing Markdown files remain inside the notes directory; handles shell-special characters in home paths. |
| notes-toggle | Queries Sway before creating files; checks JSON with jq; creates unique notes with private permissions instead of overwriting timestamp collisions. |
| pass-picker | Cancellation is harmless; rejects options/path traversal and nonexistent entries; opens pass without retaining a terminal after completion. Password decryption remains pass's responsibility. |
| power-menu | Only explicit listed actions run; cancelled/failed menus perform no power action. |
| screenshot | Unique private output names; cancellation creates no image; failed captures are cleaned up; clipboard failure leaves a successfully saved image. |
| status-osd | Read-only information, bounded optional commands, handles colons in network profile names. |
| status-panel | Read-only display, bounded data queries; no interactive controls. |
| sway-session | Explicit environment import rather than importing all variables; active-session guard; systemctl failures stop startup. Live service behavior requires desktop testing. |
| theme | Validates palette mappings/colors; renders before writing; atomic per-file replacement with temporary cleanup; reload hooks execute argument lists without a shell. A filesystem failure can still interrupt a multi-file apply; this is not a transactional update of all targets. |
| zen | Small quoted launcher for a separately installed browser. |
| scripts/install | Atomic per-file copies, backups and explicit retirement list; refuses directory symlinks leading outside the destination home; skips bytecode and preserves local settings. Not a transaction across the entire home. |
| scripts/setup-editor | Explicit package/development branches, private download directory and cleanup, validates release override, checks editor version/plugins, avoids following a directory symlink when replacing the launcher. Downloads official release assets over HTTPS; verifies pinned SHA-256 digests before extraction. |
| scripts/editor-tools.lua | Explicit optional installs, verifies tool and parser installation and exits nonzero on failure. |
| scripts/check | Offline syntax/render checks; full suite uses temporary homes and command stubs for disruptive actions. |
| wifi-menu | Preserves saved profiles on authentication failure and cancellation; passwords use nmcli's prompted stdin instead of process arguments; only displayed choices can connect; connection timeout is bounded. |

## Review and test boundaries

The integration suite uses temporary homes and command stubs for disruptive actions.
Wi-Fi authentication, VM launches, session/power changes, privileged installation
and live service behavior still require interactive verification. See
[testing](testing.md) for coverage and [security](security-audit.md) for findings.

Retired implementations remain in Git history. The installer has a backed-up
migration list for their old user files; it does not manage old system services.
