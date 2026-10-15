# Project TODO

[Documentation index](README.md)

Updated 2026-10-15. Keep this file current as work is completed or priorities
change. Record reusable lessons in [LESSONS.md](LESSONS.md).

Unfinished work only, including optional and deferred tasks. Hardware tasks
are grouped at the end because they cannot be verified in the development VM.
`[target-machine]` marks checks that need the production installation.

## Publishing and fresh-machine verification

- [ ] Test the published bootstrap on a fresh Debian 13 x86_64 machine:
  packages, themes, sudo prompts, cleanup after success and failure, and
  operation after the checkout is gone. Repeat on the replacement laptop.
  The default source now points at the GitHub repository; if the repository
  is private, the raw URL in [installation](installation.md#remote-bootstrap)
  will not work anonymously.
- [ ] Decide whether Yazi, Zen and the Nerd Font should be pinned to reviewed
  release versions with recorded digests, like Neovim, instead of taking the
  latest release and trusting the digest GitHub publishes alongside it. The
  current check proves download integrity, not that the version was reviewed.

## Security

- [ ] Finish system-health verification: the live daily timer run and a real
  installer receipt. Run the file picker through selection and cancellation.
  See [system health](sysreport.md).
- [ ] Verify OpenSnitch prompts and connection collection after a fresh setup
  and login. Setup provisions matching daemon and UI sockets.
- [ ] Decide whether OpenSnitch should ship a default-deny daemon config.
  Today Debian's default allows unmatched connections, so everything setup
  downloads before the first login passes without a prompt.
- [ ] Verify a real unattended-upgrade dry run and a later update notification
  on the target machine. See [updates](updates.md).
- [ ] Finish privileged verification of sudoers, libvirt permissions and failed
  update services. See the
  [privilege review](security-audit.md#privilege-and-update-review--2026-10-08).
- [ ] Reconcile the installed lazy.nvim revision with the repo lock during an
  intentional plugin maintenance session.
- [ ] Install and validate the scoped
  [T3 Code firewall rule](firewall.md#t3-code-on-the-private-network) with
  privileges; verify that a paired device on `10.160.110.0/24` can connect.
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
- [ ] Set up optional VPN support with a Fuzzel toggle using NetworkManager or
  a narrow sudo rule; consider a kill switch. Preserve the active network.
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
