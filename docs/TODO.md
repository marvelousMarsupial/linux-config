# Project TODO

Updated: 2026-10-15. Keep this file current as work is completed or priorities change. Record reusable lessons in [LESSONS.md](LESSONS.md).

This list contains unfinished work only, including optional and deferred tasks.
Hardware tasks are grouped below because their final verification cannot be done
in this VM. Other tasks may still need an interactive session, network access or
privileges unavailable in the current sandbox. `[target-machine]` marks checks
needed on the production/replacement installation.

## Security — next priorities

- [ ] Finish system-health verification: warning-free ShellCheck, live daily timer schedule/automatic run, and a real installer receipt. Run the updated file picker through selection and cancellation; install UPower on the current system. See [system health](sysreport.md).
- [ ] Verify OpenSnitch prompts and connection collection after a fresh setup and login. Setup now provisions matching daemon/UI sockets and the session UI service; offline provisioning tests pass.

- [ ] Verify a real unattended-upgrade dry run and later update notification on the target machine. Setup installs the exact APT policy; both policy files matched the development VM during the latest read-only check. See [updates](updates.md).
- [ ] Deferred container cleanup: user requested Incus be left unharmed. Keep Incus, its guest support and LXCFS dependency untouched. Any later Netavark/Podman cleanup needs a privileged workload/access check; preserve data and avoid blanket purge/autoremove.
- [ ] Finish privileged verification of sudoers, libvirt permissions and failed update services. Partial review 2026-10-08 confirms august is in libvirt/kvm and the packaged Polkit rule grants libvirt management; live authentication, protected sudoers/local rules and system update logs remain inaccessible from the sandbox. See [review](security-audit.md#privilege-and-update-review--2026-10-08).
- [ ] Reconcile the installed lazy.nvim revision with the repo lock during an intentional plugin maintenance session. The remaining 26 installed plugin revisions matched at audit time.

Details, evidence, caveats, and commands: [security audit](security-audit.md).

## Desktop and security roadmap (optional enhancements)

### Desktop vision

Make Debian, Sway, foot, fuzzel, nvim, yazi and Zen feel like one deliberate
system while staying minimal. Decisions, reference options and implementation
ideas are recorded in the [desktop plan](desktop-plan.md).

Priority order:

1. [ ] Finish the visual [theme consistency review](theme-review.md). Foot now uses FiraCode Nerd Font; reconcile GTK, fuzzel, Mako and status typography, icons, borders and named nvim/Yazi themes.
2. [ ] Try SwayFX and unify typography and geometry.
3. [ ] Match movement and other common keys across tools.
4. [ ] Explore the Sway features in the plan, then polish boot, login and locking. Verify the new [touchpad gestures](desktop-status.md#touchpad-and-status-toggle) on hardware.

- [ ] Catch apps that ignore the theme: GTK, Qt, cursor, icons, portal dark preference and Zen.
- [ ] Consider optional locale and personal bar modules.
- [ ] Install and validate the scoped [T3 Code firewall rule](firewall.md#t3-code-on-the-private-network) with privileges; verify that a paired device on `10.160.110.0/24` can connect. The repository policy is updated; the live firewall is not.

### Other desktop and security work

- [ ] Complete shell verification after downloading fzf-tab with `./scripts/setup-shell`: Tab selection/cancellation, directory/file previews and completion-based suggestions. Prefix Up/Down is user-confirmed; interactive prefix/Ctrl+R tests pass. See [shell](shell.md#verification).

- [ ] [target-machine] Boot with the disk passphrase only, then auto-login into Sway. Keep swaylock for idle and suspend. The boot/login flow can be developed in an encrypted VM; verify it on the production installation.
- [ ] Add Healthchecks check-ins to detect missed job runs.

## Ongoing conventions

These are implementation rules, not unfinished tasks:

- Keep scripts in the dotfiles repo and install user commands into `~/.local/bin`. Anything run as root must live in a root-owned location.
- Prefer Bash for glue and Bun TypeScript once a new script has real logic. Existing scripts do not require a blanket rewrite.
- Preserve Incus, guest support and LXCFS; avoid blanket purge/autoremove.

## Hardware and later laptop setup

Configuration and offline checks can be prepared in the VM; complete these checks
on the laptop or with the intended devices.

Publishing and optional features below are deferred. Current work is preparation;
do not publish the repo as part of it.

- [ ] Choose the final hosted repository/raw bootstrap URL, then commit and publish when ready.
- [ ] Test the published curl installer on fresh Debian 13 x86_64: packages, themes, sudo prompts, cleanup after success/failure, and operation after deleting the checkout. Repeat on the replacement laptop before discarding the VM.
- [ ] Set up optional VPN support with a fuzzel toggle using NetworkManager or a narrow sudo rule; consider a kill switch. Preserve the active network.
- [ ] Rebuild dictation if needed. Its binaries, service and shortcuts were removed; model data is backed up.
- [ ] Decide whether browser Markdown preview is useful beyond Space+mr. If wanted, install and verify its helper.
- [ ] Configure the Restic target and password, run a backup and restore test, then enable its timer. The integration is ready; this VM has no target. See [backups](backups.md).

- [ ] Fix audio on the production XPS. Check its exact model, audio devices, firmware and PipeWire logs on the laptop. The VM has no sound card and cannot verify this.
- [ ] Verify brightness controls and brightness keys on the production XPS after installing the brightness permissions and rebooting.
- [ ] Verify volume up/down and mute keys, volume indication and actual speaker/headphone volume on the production XPS.
- [ ] Configure and test fingerprint enrollment and authentication with the intended laptop or external reader.
- [ ] Exercise Wi-Fi authentication interactively with a real Wi-Fi adapter and access point. Offline tests do not verify NetworkManager's password exchange; this VM's virtual Ethernet cannot verify it.
- [ ] Verify locking before suspend and after resume on the production laptop, including lid-close behavior and running guest connectivity after resume. VM checks cannot establish laptop sleep/wake behavior.
- [ ] Recheck released KeePassXC support for Linux fingerprint quick unlock before choosing an upgrade; stable is 2.7.12 as of 2026-10-08, VM package is 2.7.10. Five-minute lock defaults are prepared; fingerprint enrollment and unlock require the intended reader.
- [ ] Evaluate an optional USB fingerprint reader or YubiKey for use when the laptop lid is closed. Test the chosen device on the laptop.
- [ ] If choosing a YubiKey as the database's second factor, configure and test it with the physical key and verify backup recovery.

Use `./scripts/check --full` for the current suite. Latest results and limits are
recorded in [testing](testing.md); installed-machine differences are in
[current state](state.md).
