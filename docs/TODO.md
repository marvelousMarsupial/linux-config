# Project TODO

Updated: 2026-10-08. Keep this file current as work is completed or priorities change. Record reusable lessons in [LESSONS.md](LESSONS.md).

## Security — next priorities

- [x] Disabled automatic clipboard-history capture in the repo and installed test-VM config; removed the history shortcut and cliphist package requirement. Existing history has not been inspected or erased. Normal clipboard copying remains configured. Log out/in to stop any existing watcher; sandbox access could not verify or stop it.
- [x] Configured swaylock after 30 minutes idle and before suspend in repo and installed VM config. Removed blanket fullscreen idle inhibition. Log out/in to activate; live idle, unlock and suspend checks remain pending below because sandbox access is restricted.
- [x] Installed pending APT updates. The package audit is clean. Reboot to use the new kernel.
- [ ] Install the exact repo APT policy. On 2026-10-08, the native APT parser verified the repo policy merged with a temporary copy of installed settings: exactly four Debian origins, daily updates, weekly cleanup, no automatic reboot/removal. Installation and privileged dry run remain blocked by sandbox root restrictions; see [updates](updates.md). Debian update timers were previously verified enabled.
- [ ] Correct the editor smoke test’s unchanged-code assertion when a formatter is installed. Latest full run: 96 passed, 1 failed, no skips.
- [x] Inspected the effective firewall/listeners and installed boot-enabled nftables host filtering. Live incoming/outgoing tests passed; see [firewall](firewall.md).
- [ ] Review obsolete Incus/LXCFS/Netavark services on the disposable VM. Preserve the Incus guest agent and remote access until migration is complete.
- [ ] Verify sudoers, libvirt permissions, and failed update services with privileged access.
- [ ] Review Markdown preview's unverified helper download; replace or verify it when rebuilding that feature.
- [ ] Reconcile the installed lazy.nvim revision with the repo lock during an intentional plugin maintenance session. The remaining 26 installed plugin revisions matched at audit time.

Details, evidence, caveats, and commands: [security audit](security-audit.md).

## Functionality and migration

- [ ] Fix audio on the production XPS. Check its exact model, audio devices, firmware and PipeWire logs on the laptop. The VM has no sound card and cannot verify this.
- [ ] Test brightness keys on the production XPS after installing the brightness permissions and rebooting.
- [ ] Grow the VM disk if it will stay in use. Safe package caches were cleared. Container data and backups were kept.

- [x] Confirmed `file` is installed on the live VM; interactive Yazi preview checks remain below.
- [ ] Confirm Yazi works with representative text, code, JSON, images and PDFs; install optional preview dependencies only when wanted.
- [ ] Install IPython on the live VM if still missing; it is already included in future installs.
- [ ] Exercise Wi-Fi authentication interactively. Offline tests do not verify NetworkManager's password exchange.
- [ ] Test Sway logout/relogin, 30-minute idle locking (including a fullscreen terminal), password unlock, suspend locking, and running guest connectivity. Session target, IPC monitor, kanshi and notification timer are now active; libvirt’s default network is active.
- [ ] Validate a clean installation on the replacement machine; offline setup tests use stubs for privileged/network operations.
- [ ] Review and commit the current changes, then push when requested before discarding the VM.
- [ ] Rebuild dictation later only if needed. Its implementation, binaries, service and shortcuts have been removed; model data is backed up.
- [ ] Rebuild VPN support with the fuzzel toggle described below. Repo integrations are currently removed; the active network was not changed.

## Desktop and security roadmap

- [ ] Boot with the disk passphrase only, then auto-login into Sway. Keep swaylock for idle and suspend.
- [ ] Use Yazi as Zen's file picker through the portal backend.
- [ ] Set up KeePassXC 2.8 with fingerprint quick unlock and a short lock timeout. Use the AppImage until Debian ships it.
- [ ] Run a Polkit agent in Sway.
- [ ] Evaluate an optional USB fingerprint reader or YubiKey for use when the laptop lid is closed.
- [ ] Put KeePassXC in the Sway scratchpad behind a hotkey.
- [ ] Add a key file or YubiKey as the database's second factor, with a backup in the safe.
- [ ] Run desktop jobs as systemd user units.
- [ ] Add a job failure hook that notifies the desktop and ntfy.
- [ ] Add Healthchecks check-ins to detect missed job runs.
- [ ] Show a failed-units count in the bar.
- [ ] Add a fuzzel VPN toggle using NetworkManager or a narrow sudo rule; optionally add a kill switch.
- [ ] Keep scripts in the dotfiles repo and install user commands into `~/.local/bin`. Anything run as root must live in a root-owned location.
- [ ] Use Bash for glue and Bun TypeScript once a script has real logic.

## Completed

- [x] Removed duplicate Mako startup. One notification daemon runs under systemd.
- [x] Fixed the Foot socket startup ordering cycle.
- [x] Installed the missing accessibility bus and corrected the Debian ALSA rule label.
- [x] Cleared safe APT and npm caches. VM disk use dropped from 92 to 87 percent.
- [x] Allowed systemd-networkd in the machine-local OpenSnitch trial. The daemon restarted and the UI reconnected.
- [x] Added journal logging for future Sway logins.

- [x] Added and installed user notification files for completed APT upgrades, including deduplication, delivery retries and reboot reminders. User timer is now active and its service succeeded. The exact repo APT policy still needs installation.

- [x] Added explicit automatic Debian stable/security update policy and activation/monitoring instructions; the manual update run succeeded with existing local settings. The exact repo policy is not yet installed.

- [x] Bound the Sway systemd target to an IPC monitor, made kanshi a direct dependency, scoped failure resets and documented session checks. The services are active. Logout and suspend checks remain pending.


- [x] Split tests by area and expanded from 33 to 90 behavior tests, including real offline LSP/formatter exchanges, session startup, menu actions, setup download boundaries and documentation links.
- [x] Fixed regressions exposed by tests: backup symlink escape, invalid compositor responses creating notes, standard Wi-Fi connection type/profile parsing, truncated MOTD input, multiline Bluetooth names and partial monitor-profile output.

- [x] Organized setup, editor, appearance, script, testing and security docs behind one index; removed stale counts, dead aliases, unused wallpaper and retired build artifacts.

- [x] Simplified to configs/scripts with direct libvirt tools; retired Forge and container-management integration.
- [x] Removed tmux, Obsidian integration, dictation and VPN integration from future installs.
- [x] Preserved shared palettes, terminal/desktop contrast, rounded bat output and a dollar prompt.
- [x] Refined existing desktop appearance without adding programs, plugins or services.
- [x] Fixed the application-search placeholder leaking into the info popup and other menus. The placeholder now belongs only to the application-launcher wrapper.
- [x] Added installer backups, atomic per-file copies, migration retirement and path checks.
- [x] Audited scripts and added cancellation/failure regression checks.
- [x] Added pinned Neovim checksums and plugin-manager bootstrap, private backup permissions, safe runtime environment parsing and editor safeguards.
- [x] Restricted the local HTTP-server alias to loopback, switched network aliases to HTTPS, and removed command expansions from alias notifications.

Use `./scripts/check --full` for the current suite. Latest results and limits are
recorded in [testing](testing.md); installed-machine differences are in
[current state](state.md).
