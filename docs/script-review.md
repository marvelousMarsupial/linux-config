# Script review — 2026-10-07

Reviewed all 18 user scripts and all four repository setup/check scripts. `scripts/check --full` passes 30 tests plus shell/Python/Lua syntax, TOML parsing, and six palette renders.

| Scripts | Review and resulting behavior |
| --- | --- |
| bat | Arguments forwarded without a shell; piped output preserved; large frames spill to private temporary storage. Unicode width remains an approximation for complex emoji sequences. |
| battery-monitor | Validates percentage thresholds and polling interval; rejects impossible readings; notification failures are tolerated; a critical warning no longer leads to a redundant low warning next poll. |
| keybinds | Static help display; removed obsolete shortcuts. |
| launcher | Quoted argument forwarding; preserves inherited XDG data directories. |
| monitor-info | Bounded Sway query, readable invalid-response error, escaped generated monitor descriptions, fallback modes for disabled outputs, refuses nonexistent modes. Only prints configuration. |
| motd | Read-only system information; bounded optional commands. |
| notes-picker | Cancellation is harmless; verifies existing Markdown files remain inside the notes directory; handles shell-special characters in home paths. |
| notes-toggle | Queries Sway before creating files; checks JSON with jq; creates unique notes with private permissions instead of overwriting timestamp collisions. |
| pass-picker | Cancellation is harmless; rejects options/path traversal and nonexistent entries; opens pass without retaining a terminal after completion. Password decryption remains pass's responsibility. |
| power-menu | Only explicit listed actions run; cancelled/failed menus perform no power action. |
| screenshot | Unique private output names; cancellation creates no image; failed captures are cleaned up; clipboard failure leaves a successfully saved image. |
| status-osd | Read-only information, bounded optional commands, handles colons in network profile names. |
| status-panel | Read-only display, bounded data queries; dictation controls removed. |
| sway-session | Explicit environment import rather than importing all variables; active-session guard; systemctl failures stop startup. Live service behavior requires desktop testing. |
| theme | Validates palette mappings/colors; renders before writing; atomic per-file replacement with temporary cleanup; reload hooks execute argument lists without a shell. A filesystem failure can still interrupt a multi-file apply; this is not a transactional update of all targets. |
| vpn | Restricts interface names, including indexed selections, preventing arbitrary config path selection; up/down/status failures propagate. Root WireGuard configs remain trusted executable configuration for wg-quick. |
| zen | Small quoted launcher for a separately installed browser. |
| scripts/install | Atomic per-file copies, backups and explicit retirement list; refuses directory symlinks leading outside the destination home; skips bytecode and preserves local settings. Not a transaction across the entire home. |
| scripts/setup-editor | Explicit package/development branches, private download directory and cleanup, validates release override, checks editor version/plugins, avoids following a directory symlink when replacing the launcher. Downloads official release assets over HTTPS; no independent signature verification is implemented. |
| scripts/editor-tools.lua | Explicit optional installs, verifies tool and parser installation and exits nonzero on failure. |
| scripts/check | Offline syntax/render checks; full suite uses temporary homes and command stubs for disruptive actions. |
| wifi-menu | Preserves saved profiles on authentication failure and cancellation; passwords use nmcli's prompted stdin instead of process arguments; only displayed choices can connect; connection timeout is bounded. |

The tests exercise failure and cancellation paths without actual power, network, or VM changes. Wi-Fi password exchange, privilege escalation, fresh network downloads, and live Sway/systemd integration still need interactive testing. A code review and test suite cannot establish that every possible runtime condition is safe.

VPN support was subsequently removed at user request: the helper, aliases, package requirement, and status integrations are retired. Active network connections were not changed. The VPN-specific test was removed. Yazi diagnostics found the required `file` tool missing on the live VM; it is now in the package manifest, but requires installation from an unrestricted terminal.
