# Project TODO

Updated: 2026-10-07. Keep this file current as work is completed or priorities change. Record reusable lessons in [LESSONS.md](LESSONS.md).

## Security — next priorities

- [ ] Decide whether to disable automatic clipboard-history capture or rebuild it with reliable secret exclusion. Current pass copies can persist in cliphist. Existing history has not been inspected or erased.
- [ ] Decide whether to add a 10-minute inactivity lock. Current behavior locks manually and before suspend; fullscreen apps inhibit idle.
- [ ] Refresh APT metadata, review and install updates from an unrestricted terminal. Audit found 183 cached upgrade candidates, 63 with security-repository candidates; these are not CVE counts.
- [ ] Inspect the effective firewall and listeners with `sudo nft list ruleset` and `sudo ss -lntup`. The old Forge nftables file is empty; actual loaded rules are unknown.
- [ ] Review obsolete Incus/LXCFS/Netavark services on the disposable VM. Preserve the Incus guest agent and remote access until migration is complete.
- [ ] Verify sudoers, libvirt permissions, and failed update services with privileged access.
- [ ] Review Markdown preview's unverified helper download; replace or verify it when rebuilding that feature.
- [ ] Reconcile the installed lazy.nvim revision with the repo lock during an intentional plugin maintenance session. The remaining 26 installed plugin revisions matched at audit time.

Details, evidence, caveats, and commands: [security audit](security-audit.md).

## Functionality and migration

- [ ] Install `file` on the live VM and retest Yazi previews and file opening. It is already included in future installs.
- [ ] Confirm Yazi works with representative text, code, JSON, images and PDFs; install optional preview dependencies only when wanted.
- [ ] Install IPython on the live VM if still missing; it is already included in future installs.
- [ ] Exercise Wi-Fi authentication interactively. Offline tests do not verify NetworkManager's password exchange.
- [ ] Test live Sway/systemd/libvirt behavior outside the sandbox, including locking before suspend.
- [ ] Validate a clean installation on the replacement machine; offline setup tests use stubs for privileged/network operations.
- [ ] Review and commit the current changes, then push when requested before discarding the VM.
- [ ] Rebuild dictation later only if needed. Its implementation, binaries, service and shortcuts have been removed; model data is backed up.
- [ ] Rebuild VPN support later. Repo integrations are removed; the active network was not changed.

## Completed

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

Use `./scripts/check --full` for the current test suite. The expanded suite passed 90 tests with none skipped; live host verification remains separate.
