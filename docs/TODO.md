# Project TODO

[Documentation index](README.md)

Updated 2026-10-15. Keep this file current as work is completed or priorities
change. Record reusable lessons in [LESSONS.md](LESSONS.md).

Unfinished work only, including optional and deferred tasks. Hardware tasks
are grouped at the end because they cannot be verified in the development VM.
`[target-machine]` marks checks that need the production installation.

## Publishing and fresh-machine verification

- [x] Test the published bootstrap on a fresh Debian 13 x86_64 VM: packages,
  themes, sudo prompts, cleanup after success, reruns and operation after the
  checkout is gone. Found and fixed the Yazi filetype key rename, the status
  panel closing on focus loss, setup skipping template re-renders, and Zen's
  profile move to `~/.config/zen`.
- [ ] Repeat the bootstrap on the replacement laptop and on a Raspberry Pi,
  and exercise cleanup after a failed step. The aarch64 path (Yazi aarch64
  build, no VM stack) has only been checked offline.
- [ ] First real setup run with the firefoxconfig package: confirm `dpkg -i`,
  the startup cache clear and the Yazi picker from Firefox on the laptop.
- [ ] When the firefoxconfig `.deb` moves off GitHub releases, add `url` to its
  `releases.toml` entry; `scripts/releases` check and bump still read GitHub.
  The default source now points at the GitHub repository; if the repository
  is private, the raw URL in [installation](installation.md#remote-bootstrap)
  will not work anonymously.
- [x] Pin Yazi, Zen and the Nerd Font to reviewed releases with recorded
  digests in `releases.toml`; `scripts/releases check` reports newer upstream
  releases and `bump` records them for review.

## Security

- [ ] Finish system-health verification: the live daily timer run and a real
  installer receipt. Run the file picker through selection and cancellation.
  See [system health](sysreport.md).
- [x] Verify OpenSnitch prompts after a fresh setup and login: prompts appear
  on the VM. Connection history stays off because the daemon has no database
  path, which the health report shows as "History off".
- [ ] Decide whether OpenSnitch should ship a default-deny daemon config and a
  database path for connection history. Today Debian's default allows
  unmatched connections, so everything setup downloads before the first login
  passes without a prompt.
- [x] Verify a real unattended-upgrade dry run and an update notification: the
  dry run passed on the VM and the notifier announced the dev-tools install.
  A notification for a real unattended upgrade is still to be seen.
- [ ] Finish privileged verification of sudoers, libvirt permissions and failed
  update services. See the
  [privilege review](security-audit.md#privilege-and-update-review--2026-10-08).
- [ ] Reconcile the installed lazy.nvim revision with the repo lock during an
  intentional plugin maintenance session.
- [ ] Deferred container cleanup: keep Incus, its guest support and LXCFS
  untouched. Any later Netavark/Podman cleanup needs a privileged workload
  check; preserve data and avoid blanket purge or autoremove.
- [ ] Add Healthchecks check-ins to detect missed job runs.

## Desktop

The vision and reference options are in the [desktop plan](desktop-plan.md).

- [ ] Finish the visual consistency pass: Sway borders are 2 px while Fuzzel
  and Mako use 1 px; GTK dialogs use Sans 10 and Adwaita icons while Fuzzel
  requests hicolor; Qt has no theme target; named Neovim and Yazi themes can
  drift from palette edits. Compare everything under one palette before
  changing settings. Contrast figures are in [appearance](appearance.md).
- [ ] Try SwayFX for rounded corners, blur and dimming, keeping the config
  compatible.
- [ ] Match movement and close keys across Sway, Neovim, Yazi, zsh and the
  browser.
- [ ] Explore the Sway features in the plan, then polish boot, login and
  locking.
- [ ] Verify the [touchpad gestures](window-behavior.md#touchpad-gestures-and-status-panel)
  on hardware.
- [ ] Complete shell verification after installing fzf-tab: Tab selection and
  cancellation, previews and completion-based suggestions. See
  [shell](shell.md#verification).
- [ ] Decide whether browser Markdown preview is useful beyond Space+mr. If
  wanted, install and verify its helper. See [editor](editor.md#markdown-view).
- [ ] Consider optional locale and personal bar modules.

## Ongoing conventions

Implementation rules, not tasks:

- Keep scripts in the dotfiles repo and install user commands into
  `~/.local/bin`. Anything run as root must live in a root-owned location.
- Prefer Bash for glue and Bun TypeScript once a new script has real logic.
  Existing scripts do not need a blanket rewrite.
- Preserve Incus, guest support and LXCFS; avoid blanket purge or autoremove.

## Hardware and later laptop setup

Configuration and offline checks can be prepared in the VM; complete these on
the laptop or with the intended devices.

- [ ] [target-machine] Boot with the disk passphrase only, then auto-login into
  Sway. Keep swaylock for idle and suspend. Develop the flow in an encrypted VM;
  verify it on the production installation.
- [ ] Configure the Restic target and password, run a backup and restore test,
  then enable the timer. See [backups](backups.md).
- [x] VPN: `vpn-menu` toggles WireGuard profiles through NetworkManager and
  Tailscale, fails off, and `vpn-off.service` disconnects at login. Pending: a
  real WireGuard peer and a Tailscale login on the laptop.
- [ ] Rebuild dictation if needed. Its binaries, service and shortcuts were
  removed; model data is backed up.
- [ ] Fix audio on the production XPS: check the exact model, audio devices,
  firmware and PipeWire logs on the laptop.
- [ ] Verify brightness controls and keys on the XPS after a reboot with the
  brightness permissions in place.
- [ ] Verify volume keys, volume indication and speaker and headphone output.
- [ ] Configure and test fingerprint enrollment and authentication with the
  intended laptop or external reader.
- [ ] Exercise Wi-Fi authentication with a real adapter and access point.
- [ ] Verify locking before suspend and after resume, lid-close behavior and
  guest connectivity after resume.
- [ ] Recheck KeePassXC's Linux fingerprint quick unlock before choosing an
  upgrade; stable was 2.7.12 on 2026-10-08, the Debian package is 2.7.10.
- [ ] Evaluate a USB fingerprint reader or YubiKey for use with the lid
  closed. If a YubiKey becomes the database's second factor, test it and
  verify backup recovery.

Use `./scripts/check --full` for the current suite. Results and limits are in
[testing](testing.md).
