# Scripts

[Documentation index](README.md)

User commands live in `dotfiles/.local/bin/` and are copied into `~/.local/bin`
with executable permissions. Repository maintenance commands live in `scripts/`.

## Repository scripts

| Script | Responsibility |
| --- | --- |
| scripts/setup | Fresh Debian 13 setup on x86_64 or aarch64: packages, firewall, verified app releases, fzf-tab, portal build, editor, user files, initial theme, host settings, OpenSnitch socket and health timer. Dry run performs no privileged or network changes. Refuses to run as root. |
| scripts/bootstrap | Downloads a temporary checkout, forwards `--dry-run` and `--dev-tools` to setup, and removes the checkout afterward, including on failure. HTTPS sources only. |
| scripts/install | Atomic per-file copies with backups, a named retirement list, bytecode exclusion, symlink boundary checks, and merges of only the dark-theme key for KeePassXC and OpenSnitch. Renders the theme hook path and desktop entries for the destination user. Not a transaction across the whole home. |
| scripts/install-receipt | Records the source commit, modified-checkout flag, completion date and install scope after a successful installation. |
| scripts/setup-editor | Installs Neovim 0.12.2 with a pinned SHA-256, copies user files, installs and checks plugins; `--dev-tools` adds compilers, Node, language servers, formatters and parsers; `--skip-apt` skips the package step. |
| scripts/setup-shell | Installs fzf-tab v1.2.0 after checking the pinned Git commit; dry run and alternate home supported; backs up replaced copies. |
| scripts/editor-tools.lua | Explicit optional tool and parser installs; exits nonzero when verification fails. |
| scripts/check | Syntax, TOML and six palette renders; `--full` adds the offline suite. Skips zsh and Neovim checks with a message when they are absent. |

## User commands

| Command | Responsibility |
| --- | --- |
| app-dark-mode, keepassxc, opensnitch-ui | Apply the dark-theme key before launching the packaged app. The OpenSnitch wrapper uses the user service's runtime socket by default. |
| appearance-menu | Compact GTK palette-dot picker with a glass switch; Super+G. |
| bat | Runs batcat one column narrower and closes its grid on the right with rounded corners; piped output, an explicit pager and paging flags go straight to batcat. |
| battery-monitor | Validates thresholds and polling interval; tolerates notification failures; no repeated warnings after a critical one. |
| brightness | Changes screen backlights in 10 percent steps; shows failures; never falls back to keyboard LEDs. |
| desktop-appearance | Publishes the dark desktop and GTK preferences at Sway start and reload. |
| glass-mode | Switches desktop and Zen glass together, re-applies the palette, tolerates a missing notification daemon. |
| keepass-toggle | Super+P launches KeePassXC, focuses its window or toggles it into the scratchpad. |
| keybinds | Static shortcut help. Super+Shift+/ launches it, hides it in the scratchpad when focused, or shows it again. |
| launcher | Quoted argument forwarding; keeps inherited XDG data directories. |
| motd | Read-only login summary with bounded optional commands. |
| notes-picker, notes-toggle | Cancellation is harmless; notes stay inside the notes directory; unique private filenames. |
| notifications | Searchable Mako history through busctl, jq and Fuzzel. |
| power-menu | Only listed actions run; cancelled menus do nothing. |
| restic-home | Optional encrypted home backup with private per-machine config and explicit init. |
| screenshot | Unique private names; cancellation creates no file; failed captures are cleaned up. |
| status-panel | Read-only clock, network and power popup with bounded queries; no controls. |
| sway-popup-guard | Reveals popups during fullscreen, restores fullscreen on close, adjusts Foot opacity on floating changes. |
| sway-session | Explicit environment import, active-session guard, starts the session target. |
| sysreport | Python health report: private static HTML with eight tiles, folded command output and 14-report retention; reads the OpenSnitch database directly and read-only; runs from a daily timer. |
| terminal | Super+Enter: leaves fullscreen, opens a centered floating Foot; a normal terminal otherwise. |
| theme | Validates palettes, renders everything before writing, replaces files atomically, runs hooks without a shell. |
| update-notify | Detects completed APT upgrades, baselines existing history, deduplicates and retries delivery. |
| wallpaper-texture | Generates the palette-colored wallpaper; dots by default; solid color when glass is off. |
| wifi-menu | Preserves saved profiles on failure; passwords go through nmcli's stdin; bounded connection timeout. |
| zen, zen-theme-sync | Browser launcher; copies generated colors into the default Zen profile while preserving custom CSS and preferences. |

The integration suite uses temporary homes and command stubs for disruptive
actions. Wi-Fi authentication, VM launches, session and power changes,
privileged installation and live service behavior still need interactive
verification. See [testing](testing.md) and the [security audit](security-audit.md).

Retired implementations remain in Git history. The installer's retirement
list removes their old user files with backups; it does not manage old system
services.
