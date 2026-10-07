# Security audit — 2026-10-07

## Result and scope

The most important unresolved findings are clipboard persistence, missing inactivity locking, pending security updates, and uncertainty about the VM's effective firewall and obsolete services. No matching secrets were found in the pattern scan. That is not proof that the repository or host is free of secrets or vulnerabilities.

Reviewed all repository scripts, desktop startup, password/clipboard flow, Neovim plugin installation, user services, permissions, package requirements, and readable host configuration. Applied safe repository fixes and copied them to the live user configuration. No packages were installed, no firewall rules or remote services were changed, no clipboard entries were deleted, and no remote session was interrupted.

The tool environment has an isolated process/network view and no privileged systemd, netlink, sudo, or libvirt access. Empty `/proc/net/tcp` tables in that environment do not demonstrate that the desktop VM has no listening ports. Some system ownership metadata is remapped. Privileged settings and host exposure remain unverified.

## Findings

| Priority | Finding | Status |
| --- | --- | --- |
| High if credentials are copied | Sway captures clipboard changes into persistent cliphist history. Installed pass copies ordinary clipboard text and only restores/clears the clipboard later; it does not delete the cliphist entry. The installed wl-clipboard cannot reliably label secrets for the watcher. Ten retained entries still permit credential persistence. No actual password content was inspected or claimed found. | User preference pending: disable automatic capture, or retain history with this risk. Existing history was not erased. |
| High maintenance priority | Cached APT metadata lists 183 upgradable packages, including 63 whose candidate listings include a security repository. Examples include Flatpak, bubblewrap, jq, media libraries, and Incus. These counts are candidates, not counts of CVEs, and metadata was not refreshed. | Needs privileged package update review. Unattended upgrade settings and enabled timers exist but do not establish that updates succeeded. |
| Medium | The desktop locks before suspend but has no inactivity timeout. A logged-in desktop can remain accessible indefinitely. Fullscreen applications also inhibit idle. | User preference pending for a 10-minute timeout. Manual and suspend locking preserved. |
| Medium, exposure unknown | `/etc/nftables.conf` is an old Forge placeholder with no filtering rules. Other tools may have loaded rules; the effective ruleset could not be read. NAT is not a substitute for a host exposure review. | Verify loaded rules and required remote/libvirt traffic before designing a firewall. No network changes made. |
| Medium, active state unknown | Enabled service/socket links remain for Incus, LXCFS, and Netavark despite their removal from future repository installs. An enabled link is not proof of a running or remotely exposed service. | Review and retire unused host components later. Preserve the Incus guest agent and the current remote-access path. |
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

`./scripts/check --full` covers 33 passing tests plus syntax, TOML, Lua, and six theme renders. Security regression checks include shell-command injection in runtime environment data, rejected runtime symlinks, backup permissions under umask 002, checksum failure before extraction/execution, and editor modeline behavior. Startup bootstrap network success and the pinned upstream downloads were not exercised end-to-end in this sandbox.

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

## Test follow-up — 2026-10-07

Expanded regression testing found that a symlinked installer backup directory
could escape the destination home before the normal file path checks ran. The
installer now validates the resolved backup parent before mkdir/chmod/copy.
A regression asserts rejection, no outside writes and preservation of the
original config. Malformed Sway responses now stop notes-toggle before creating
a note. See [testing](testing.md) for the expanded 90-test suite and its limits.
