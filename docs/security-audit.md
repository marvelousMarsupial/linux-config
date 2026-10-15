# Security audit — 2026-10-07

## Current status

[Documentation index](README.md)

This document preserves the initial restricted audit and its dated follow-ups.
It is evidence from those sessions, not a description of the host today. Use
[testing](testing.md) for the current verification status and [TODO](TODO.md)
for open items. Since the audit, setup installs the firewall and the APT policy
itself; the clipboard watcher and idle-lock findings were fixed in the repo.

## Initial result and scope

The most important unresolved findings at the original audit were clipboard persistence, missing inactivity locking, pending security updates, and uncertainty about the VM's effective firewall and obsolete services. No matching secrets were found in the pattern scan. That is not proof that the repository or host is free of secrets or vulnerabilities.

Reviewed all repository scripts, desktop startup, password/clipboard flow, Neovim plugin installation, user services, permissions, package requirements, and readable host configuration. Applied safe repository fixes and copied them to the live user configuration. No packages were installed, no firewall rules or remote services were changed, no clipboard entries were deleted, and no remote session was interrupted.

The tool environment has an isolated process/network view and no privileged systemd, netlink, sudo, or libvirt access. Empty `/proc/net/tcp` tables in that environment do not demonstrate that the desktop VM has no listening ports. Some system ownership metadata is remapped. Privileged settings and host exposure remain unverified.

## Findings

| Priority | Finding | Status |
| --- | --- | --- |
| High if credentials are copied | Sway captures clipboard changes into persistent cliphist history. Installed pass copies ordinary clipboard text and only restores/clears the clipboard later; it does not delete the cliphist entry. The installed wl-clipboard cannot reliably label secrets for the watcher. Ten retained entries still permit credential persistence. No actual password content was inspected or claimed found. | Resolved 2026-10-08 by disabling automatic capture in the repo and installed test-VM config, removing the history shortcut and future-install package requirement. Existing history was not inspected or erased; old entries may still contain secrets. Sandbox restrictions prevented stopping/verifying a running watcher; log out/in to apply the startup change. |
| High maintenance priority | Cached APT metadata lists 183 upgradable packages, including 63 whose candidate listings include a security repository. Examples include Flatpak, bubblewrap, jq, media libraries, and Incus. These counts are candidates, not counts of CVEs, and metadata was not refreshed. | Needs privileged package update review. Unattended upgrade settings and enabled timers exist but do not establish that updates succeeded. |
| Medium | The desktop locks before suspend but has no inactivity timeout. A logged-in desktop can remain accessible indefinitely. Fullscreen applications also inhibit idle. | Configured 2026-10-08: 30-minute idle timeout, retained synchronous pre-suspend locking and removed blanket fullscreen inhibition in repo and installed VM config. Application-requested inhibitors remain respected. Logout/login and live idle/unlock/suspend checks are pending; sandbox restrictions prevent live activation/verification. |
| Medium, exposure unknown | `/etc/nftables.conf` is an old Forge placeholder with no filtering rules. Other tools may have loaded rules; the effective ruleset could not be read. NAT is not a substitute for a host exposure review. | Verify loaded rules and required remote/libvirt traffic before designing a firewall. No network changes made. |
| Medium, active state unknown | Enabled service/socket links remain for Incus, LXCFS, and Netavark despite their removal from future repository installs. An enabled link is not proof of a running or remotely exposed service. | Static review completed 2026-10-08; see the container-service review below. Workload inventory, live state and retirement remain pending due to sandbox restrictions. Guest agent and access path preserved. |
| Medium supply-chain risk | Markdown browser preview’s build hook downloads an external helper without a digest check and probes an existing helper by executing it. | Mitigated 2026-10-08: browser plugin disabled and automatic install hook removed from repo and installed config. In-buffer rendering retained. Existing helper files preserved and unverified; restart Neovim to apply. |
| Medium supply-chain risk | setup-editor downloaded and executed Neovim without checking a fixed digest. HTTPS alone did not compare the asset to an independently reviewed expected value. | Fixed: pinned official v0.12.2 x86_64/arm64 SHA-256 digests; checksum failure aborts before extraction or execution. Version overrides require an explicit digest. |
| Medium supply-chain risk | lazy.nvim bootstrap selected a moving branch rather than the repository's lock entry. | Fixed for new bootstraps: clone without checkout, then detach at the validated 40-character lock commit. The installed manager differs from the repo lock; the other 26 installed plugins match. Existing plugin code was not replaced during the audit. |
| Residual supply-chain risk | Markdown preview's build hook downloads a prebuilt helper without a separately pinned digest. Optional Mason installs also fetch tools, and plugin code executes as the user. Pinning the manager does not audit every plugin or transitive download. | Documented; no claim of a complete dependency/CVE audit. Consider replacing that build path when preview is rebuilt. |
| Medium if used on an untrusted network | The `serve` alias bound Python's unauthenticated HTTP server to all interfaces, sharing the current directory with reachable peers. | Fixed: binds only to 127.0.0.1; intentional remote sharing requires an explicit command. |
| Low | Network aliases used implicit HTTP, and alias reminders included command expansions in notifications. Custom aliases can embed credentials. | Fixed: HTTPS with timeouts; notifications contain only the alias name and a generic body. |
| Low / defense in depth | zsh sourced the runtime dbus-env file as shell code. The normal runtime directory is already private and the file is user-controlled; no cross-user exploit was demonstrated. | Fixed: import only the bus address as data, require owner match and reject a symlink. Tests prove injected commands do not execute. |
| Low / defense in depth | Backup directories inherited a shared umask, and many repository config files were group-writable/executable unnecessarily. Home is already 0700, which reduces exposure. | Fixed: private installer/setup umask, backup root 0700, normalized repository config/script permissions. Existing backup trees are protected by the private root. |
| Low / defense in depth | Untrusted file modelines and project-local editor config broaden the editor's input attack surface. | Explicitly disabled modelines/exrc. A behavioral test opens a modeline and verifies it does not change options. |
| Low / defense in depth | Markdown preview relied on upstream defaults for its bind address and stayed open after leaving Markdown. | Explicit loopback-only configuration and automatic close. This is not a fix for a demonstrated public listener; upstream already defaults to localhost. |

## Secret scan and positive checks

- Scanned 4,127 non-.git/non-bytecode working files at scan time and 601 unique blobs across 61 reachable Git commits. The working-file count includes ignored files; it is not the number of tracked project files.
- Looked for common PEM/OpenSSH private key headers, GitHub token forms, AWS access key IDs, and credential-bearing HTTP URLs. No matches. No secret values were printed. Arbitrary passwords, other token formats, unreachable objects, remote branches not fetched, and external accounts were not covered.
- Home is 0700, runtime directory 0700, shell history 0600, and cliphist directory 0700. These permissions reduce access by other users but do not protect against malware running as this user.
- Repository scripts use argument lists/quoted arguments rather than inserting SSIDs or file names into shell commands. Theme hooks no longer invoke a shell.
- No repo remote-access listener configuration, blanket sudoers rule, or privileged installer daemon was found. The live sudoers fragment could not be read.
- Libvirt is retained intentionally. Full read/write management access is privileged; the README's libvirt group instructions should be used only for trusted administrators. No broad Polkit allow rule was added.
- Existing optional package/service configuration does not automatically alter network, firewall, or host authentication settings.

## Validation

At the initial audit, `./scripts/check --full` covered 33 passing tests plus syntax, TOML, Lua, and six theme renders. Security regression checks include shell-command injection in runtime environment data, rejected runtime symlinks, backup permissions under umask 002, checksum failure before extraction/execution, and editor modeline behavior. Startup bootstrap network success and the pinned upstream downloads were not exercised end-to-end in this sandbox.

## Host follow-up from an unrestricted terminal

Review output before taking disruptive actions:

```sh
sudo apt update
apt list --upgradable
sudo nft list ruleset
sudo ss -lntup
systemctl --failed
systemctl status unattended-upgrades apt-daily-upgrade.timer
systemctl list-units --all 'incus*' 'lxc*' 'netavark*'
sudo -l
```

Then install appropriate updates. A package upgrade can restart services or require a reboot, so schedule it around remote access. Do not purge the Incus guest agent as part of removing unused host/container tooling.

## Sources

- [Neovim official release asset digests](https://github.com/neovim/neovim/releases/expanded_assets/v0.12.2)
- [swayidle timeout, lock and suspend semantics](https://github.com/swaywm/swayidle/blob/master/swayidle.1.scd)
- [cliphist persistent clipboard history](https://github.com/sentriz/cliphist)
- [Markdown preview binding and lifetime settings](https://github.com/iamcco/markdown-preview.nvim)
- [Libvirt privilege/access-control model](https://libvirt.org/aclpolkit.html)

Clipboard/password behavior was additionally checked directly against the installed `/usr/bin/pass` and wl-clipboard manual, not inferred only from upstream documentation.

## Glass mode review, 2026-10-07

Reviewed the glass scripts, theme renderer and templates, Zen CSS and selected
live preferences, running Zen arguments and environment, listeners,
permissions, clipboard capture and lock configuration. A scoped review, not a
host penetration test or dependency audit.

- **Low, privacy tradeoff:** translucent surfaces can reveal terminal or
  document content in screen shares or to someone nearby. Blur is not
  redaction. The generated swaylock configuration stays solid.
- **Low, corrected:** review screenshots in `/tmp` were mode 0664; they and
  the test metadata are now 0600, and test profile directories 0700.
- The two running Zen processes had no remote-debugging arguments or
  sandbox-disabling environment, and no Zen TCP listener was present. These are
  targeted checks, not proof of every runtime preference.
- `zen-themes.json` was empty: no downloaded mod runs for glass. CSS imports
  are local, the scripts fetch no remote code and use no shell evaluation, and
  the switch changes only three appearance preferences, leaving web-page
  transparency off. No sandbox, TLS or certificate preference is touched.
- The CSS hides no identity, permission or tracking control. Local CSS must
  stay trusted: restyling browser UI can disguise security indicators, but any
  process running as this user could already edit these files.

The clipboard and idle-lock findings noted during this review are the ones in
the table above and were fixed on 2026-10-08.

## Test follow-up — 2026-10-07

Expanded regression testing found that a symlinked installer backup directory
could escape the destination home before the normal file path checks ran. The
installer now validates the resolved backup parent before mkdir/chmod/copy.
A regression asserts rejection, no outside writes and preservation of the
original config. Malformed Sway responses now stop notes-toggle before creating
a note. The suite subsequently grew further; see [testing](testing.md) for current results and limits.

## Firewall follow-up — 2026-10-07

Unrestricted access subsequently allowed inspection of the actual machine. The
old nftables config was empty and effective filtering consisted of libvirt's
rules with accepting host input policies. Installed and enabled the dedicated
IPv4/IPv6 host firewall and a service override preserving other tables. Live
incoming/outgoing tests passed, DNS/HTTPS worked, and Incus agent/libvirt stayed
active. This resolves the firewall uncertainty above; the other original audit
findings retain their stated scope. See [firewall](firewall.md) for verification
limits, installed files and recovery commands.

## Container service review — 2026-10-08

Read the installed units, `/etc/systemd/system` startup links and dpkg dependency
metadata. The repo no longer includes Incus, LXCFS, Podman or Netavark integration.
No system settings, packages, services, networking or container data were changed.

| Component | Installed evidence and implication |
| --- | --- |
| Incus host | `incus` and `incus-base` 6.0.4-2+deb13u11 are installed. `incus.socket`, `incus-user.socket` and `incus-startup.service` have startup links. Either socket can activate a host daemon; stopping only `incus.service` would not retire activation. The host daemon requires LXCFS. |
| Incus guest support | `incus-agent` is a separate installed package. Its service uses a guest virtio-port condition and does not require the Incus host daemon. Preserve it. Also preserve the separately enabled `incus-growpart.service`, which grows the VM root partition. |
| LXCFS | 6.0.4-1+deb13u1 is installed with a multi-user startup link. Check Incus/LXC workloads before retiring it. |
| Netavark | 1.14.0-2 is installed. DHCP proxy service/socket and firewalld reload helper have startup links. Podman 5.4.2+ds1-2+b2 depends on Netavark; these helpers are not proof that Podman networking is unused. |
| Stored data | Incus and system/user container storage directories remain. Incus instance/storage directories are unreadable here. No instance count or unused-data claim is supported. |
| Access and libvirt | Libvirt service/socket links remain and must be preserved. `openssh-server` is not installed; do not assume SSH is the current access or recovery path. The host access path could not be established from this sandbox. |

The system manager rejects this session's D-Bus connection. Its isolated process
and device views cannot establish running services, guests, listeners or the
presence of the VM's agent channel. Startup links establish configuration only.
No running workload inventory was obtained.

User instruction after this review: leave Incus unharmed. Keep Incus and its
LXCFS dependency untouched. The following is a read-only inventory for a future
host review, not authorization to retire Incus.

From an unrestricted terminal on the VM, inspect:

```sh
systemctl list-units --all 'incus*' 'lxc*' 'netavark*' 'podman*'
systemctl list-unit-files 'incus*' 'lxc*' 'netavark*' 'podman*'
sudo incus list
sudo incus storage list
sudo podman ps --all
podman ps --all
systemctl --user list-units --all 'podman*' 'container*'
```

Incus and Podman queries can activate their daemons; perform these only during
the intentional host review. Check other users' workloads and Quadlet definitions
as applicable. Confirm the current connection and a working recovery route before
stopping anything. Keep Incus, its guest support and LXCFS unchanged under the current user
instruction. Any later Netavark/Podman cleanup needs its own workload check.
Review package removal separately, including its simulated dependency changes,
and avoid blanket purge/autoremove.

## Privilege and update review — 2026-10-08

No permissions, services, package settings or Incus components were changed.
The requested privileged verification remains incomplete.

- `/etc/sudoers` and `/etc/sudoers.d/<user>` exist with mode 0440 but cannot be
  read here. No claim about passwordless sudo or allowed commands is supported.
  Ownership appears remapped in this sandbox and cannot certify host ownership.
- Account metadata lists the desktop user in `libvirt` and `kvm`. The packaged
  `/usr/share/polkit-1/rules.d/60-libvirt.rules` grants
  `org.libvirt.unix.manage` to the libvirt group without a password prompt.
  Treat this group as administrative access, as already intended by the repo.
- Installed libvirtd socket units and visible sockets use mode 0666. Readable
  `libvirtd.conf` has no active overrides; its comments document Polkit as the
  read/write default for this build. Open socket permissions alone do not grant
  unauthenticated management under [Polkit authentication](https://libvirt.org/auth.html#unix-socket-policykit-auth).
  Local `/etc/polkit-1/rules.d` and protected `qemu.conf` are unreadable, and even
  a read-only virsh connection is sandbox-denied. Effective authorization remains
  unverified; do not change socket modes based only on this restricted view.
- Both packaged APT services have `ConditionACPower=true`; a battery-related
  skipped run is not itself a failed upgrade. The system manager is inaccessible,
  unattended-upgrade logs are unreadable and the system journal is hidden.
  `journalctl` returning no entries here is not evidence of healthy services.

From an unrestricted VM terminal, complete the read-only checks:

```sh
sudo visudo -c
sudo -l
sudo cat /etc/sudoers /etc/sudoers.d/*
id "$USER"
sudo ls -l /run/libvirt/libvirt-sock* /dev/kvm
sudo grep -vE '^[[:space:]]*(#|$)' /etc/libvirt/libvirtd.conf /etc/libvirt/qemu.conf
sudo find /etc/polkit-1/rules.d -maxdepth 1 -type f -exec cat {} +
virsh --connect qemu:///system list --all
sudo systemctl --failed --no-pager
sudo systemctl show apt-daily.service apt-daily-upgrade.service unattended-upgrades.service -p ActiveState -p SubState -p Result -p ExecMainStatus -p ConditionResult
sudo journalctl -b -u apt-daily.service -u apt-daily-upgrade.service -u unattended-upgrades.service --no-pager
sudo tail -n 50 /var/log/unattended-upgrades/unattended-upgrades.log
```

Run the virsh query as the desktop user, without sudo, to check the intended management
access. Review all sudoers includes, including any referenced by the main file,
and local Polkit rules before drawing a conclusion. Preserve failed-unit evidence;
do not reset failures or restart services merely to make the status look clean.
