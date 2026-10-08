# Current state

Checked 2026-10-07 on the Debian 13 VM.
See [TODO](TODO.md) for work still needed on the XPS.

| Area | Current state |
| --- | --- |
| User files | Installer dry run shows no changes. Local profiles and backups are kept. |
| Sway | Responds to IPC. Virtual display is active. New tty1 logins send output to the journal. Current session output was not saved. |
| Locking (2026-10-08) | Repo and installed config lock after 30 minutes idle and before suspend. Blanket fullscreen inhibition removed; application-requested inhibitors respected. Logout/login and live idle/unlock/suspend checks pending; sandbox cannot access live Sway. |
| Session services | Session target, IPC monitor, kanshi and update timer are active. Logout and suspend still need a live test. |
| Clipboard (2026-10-08) | Automatic history capture disabled in repo and installed config; history shortcut removed. Logout/login still needed: the sandbox cannot access live Sway/user services to stop or verify any existing watcher. Existing history was not inspected or erased. Normal copying remains available through wl-clipboard. |
| Notifications | One Mako daemon runs under systemd. Duplicate Sway startup was removed. |
| Foot | Running. Socket override fixes a startup ordering cycle. |
| Updates | APT timers are active. The manual update run finished. No pending upgrades at this check. The repo's named APT policy file is still absent from /etc. Existing local settings permit Debian maintenance updates and disable automatic reboot. |
| Kernel | Running 6.12.74+deb13+1-amd64. Installed 6.12.111+deb13-amd64 needs a reboot. Old kernel kept as fallback. |
| Update notices | Timer active and recent checks succeeded. Delivery after a future update still needs confirmation. |
| Firewall | nftables enabled and active. Earlier live incoming and outgoing probes passed. |
| OpenSnitch | Machine-local trial only. Daemon and UI active. Added an allow rule for systemd-networkd and restarted the daemon. UI reconnected. |
| Audio | VM has no sound card. PipeWire has a dummy output. XPS audio problems need investigation on the laptop. |
| Brightness | Screen-only controls and backup shortcuts installed. Permission rules installed and user added to video group. Reboot and test on the XPS. VM has no backlight. |
| Accessibility | Missing at-spi2-core installed. Accessibility bus responds. |
| ALSA rules | Local override fixes Debian's duplicate label. udev verification passes. Remove override when Debian fixes its packaged rule. |
| Storage | Cache cleanup lowered disk use from 92 to 87 percent. About 2.6 GB free. Container data and backups kept. |
| Memory | About 2.5 GB available at inspection. No logged out of memory or disk I/O errors found. |
| Editor | Known HTML save test failure. Installed lazy.nvim revision still needs reconciliation with the lock. |
| File tools | file installed. ipython3 missing. Interactive Yazi checks pending. |

System and user managers report no failed units after the fixes.
The full suite ran 97 tests. 96 passed and the known HTML save test failed.
Syntax and theme checks passed. See [testing](testing.md) for limits.

OpenSnitch restart logged shutdown warnings. Its next startup completed and the
UI connected. This does not prove every network flow works.
