# Installation

[Documentation index](README.md)

This setup targets a personal Debian 13 workstation. Review commands and package
requirements before applying them. Existing software is not automatically removed.

## One-command setup

Use Debian 13 **x86_64** and your normal user account with sudo access. Python 3,
git and sudo must already be available to obtain the checkout and run setup.
From the repository root:

```sh
./scripts/setup --dry-run
./scripts/setup
```

The same command supports this VM and a fresh installation. The dry run performs
read-only discovery and prints a plan without downloads, sudo or file changes.
A normal run:

- Installs IPython with its recommended packages and explicitly includes NumPy, Matplotlib and SciPy.
- Installs Chromium, KeePassXC, OpenSnitch UI and its theme library; config installation sets KeePassXC and OpenSnitch to Dark while preserving other existing settings. Setup configures the OpenSnitch daemon socket for the installing user's UID and Sway starts the matching UI service.
- Installs `packages.txt` through APT, including Starship and the portal broker,
  GTK and wlr backends.
- Installs missing Yazi/ya, Zen and CaskaydiaCove Nerd Font from their official
  latest stable GitHub releases. Published SHA-256 digests are required and
  checked before extraction; missing digests or mismatches stop setup.
- Builds termfilechooser at the pinned upstream commit using temporary sources
  and installs it into `/usr`. Build dependencies remain installed. A marker
  avoids repeating the build when that revision and its files are present.
- Calls `setup-editor` for Neovim and plugins, and `install` for user files.
  Existing applications are reused; setup is not a general application updater.
- Applies summer-dusk on a fresh home. Existing theme selections/colors and
  local monitor profiles are preserved. Paths are rendered for the destination
  user rather than assuming `/home/august`.
- Adds the user to video/libvirt/kvm, selects zsh as the login shell, generates
  the configured English UTF-8 locale if missing, and enables libvirtd.
- Backs up changed APT policy files under `/var/backups/linux-config/`, installs
  the repo update policy, and enables Debian's update timers.
- Installs the Polkit authentication agent and starts it directly from Sway
  for graphical password prompts after login. See [systemd](systemd.md#graphical-authentication).
- Reloads the user service manager when a user bus is available.

Use `./scripts/setup --dev-tools` to also install optional editor development
tools. Setup stops on failure; fix the reported step and rerun. It is repeatable,
not transactional: completed steps stay applied. Changed user files have the
config installer's backups, replaced local application files have dated backups,
and no packages are purged. No running desktop or browser is restarted.

Log out and back in, or reboot, then launch Zen normally. Verify file dialogs,
screen sharing and screen locking in the live session. The script does not
configure disk encryption, firmware, new monitor layouts, firewall rules, lid policy,
VM networks/storage or hardware-specific drivers. Keep those machine decisions
in the sections below. Incus and its data are not explicitly modified or removed.

## Remote bootstrap

Goal: host the repository on GitHub or an equivalent service, download and run
setup with one command, and automatically discard the downloaded checkout. No
checkout is needed afterward: installed files are copies, not links into it.

`scripts/bootstrap` clones the selected branch/tag into a private directory under
`/tmp`, runs `scripts/setup`, then deletes only that temporary directory. It also
cleans up on failure and handled interruption. A power loss or SIGKILL can leave
temporary files behind. It never deletes an existing working checkout. Completed
installation steps and backups remain if setup fails; fix the error and rerun.

Before remote use, commit and publish the current changes and choose the final
repository and raw bootstrap URL. The current remote is
`https://git.rat/August/linux-config.git`, branch `main`; no GitHub repository has
been selected. Once published, substitute the real raw-file URL below:

```sh
curl -fsSL <raw-bootstrap-url> | bash
```

To select another host or a published release tag:

```sh
curl -fsSL <raw-bootstrap-url> | env \
  LINUX_CONFIG_REPOSITORY=https://github.com/OWNER/linux-config.git \
  LINUX_CONFIG_REF=v1.0.0 bash
```

These are templates, not published install URLs. The bootstrap and checkout both
execute code from the chosen source; use a source you trust. For a release,
fetch the bootstrap from the same tag selected by `LINUX_CONFIG_REF`.
Run as the desktop user, with Bash, curl, git, Python 3, sudo and CA certificates
already installed. Sudo may request your password through the terminal.

To test the bootstrap from this checkout after publishing:

```sh
./scripts/bootstrap --dry-run
./scripts/bootstrap
```

`--dry-run` still downloads and removes a temporary checkout; the setup stage
only prints its plan. `--dev-tools` is forwarded to setup. Offline tests cover
cleanup and argument forwarding; a fresh-machine network/privileged installation
remains required before calling the remote flow verified.

## Config-only installation

For user config changes without package/system setup:

```sh
./scripts/install --dry-run
./scripts/install
```

The installer copies files into your home, backs up replaced files under
`~/.local/state/linux-config/backups/`, and skips identical files. It does
not install packages, change services, remove software, or alter networking.
Run it again after editing the repo. Existing monitor profiles, Neovim plugin
locks, and generated color files are preserved. `--home /path` installs into
an alternate home for testing. Copies are independent of the checkout.

After installing or changing user units, reload the user manager:

```sh
systemctl --user daemon-reload
```

The Sway session target pulls in kanshi automatically; no enable command is
required. Log out of Sway and back in after changing these units. See
[systemd setup](systemd.md) for lifecycle details and checks.

Open a new terminal after installation and press Super+Shift+c to reload Sway.
Choose zsh as your login shell with `chsh -s /bin/zsh` if desired. Sway starts
on tty1; run `sway` manually for an initial session.

## Screen brightness

The brightness keys change the screen in 10 percent steps.
Super F11 dims it. Super F12 brightens it.

Debian needs `brightness-udev` and membership in the video group.
The package is in `packages.txt`. Add your user once:

```sh
sudo usermod -aG video "$USER"
```

Reboot after setup to apply the group and device permissions.
The full setup command handles groups; the config-only installer does not.

On the XPS run these from a terminal in Sway:

```sh
brightnessctl --class=backlight --list
~/.local/bin/brightness up
```

If the command works but the keys do not, try the keys with Fn.
Fn Escape changes Fn lock on supported Dell keyboards.
The backup shortcuts avoid Fn lock.

A VM usually has no screen backlight control.
If the XPS also reports no control, check its model and panel type.
An OLED panel may need a different method. Do not change kernel settings
until the hardware is known.

## Separately installed applications

The setup command installs Starship through APT and missing Yazi/ya, Zen and
CaskaydiaCove Nerd Font from official releases. For config-only installs, provide
those applications yourself. Zen's wrapper expects `~/.local/zen/zen`. The launcher includes
separately installed Flatpak application entries. No application binaries or fonts
are bundled by this repository.

Zen's config installer installs uBlock Origin, Dark Reader, Surfingkeys and
KeePassXC-Browser through browser policies on the next browser startup. It also
skips the welcome screen and sets dark defaults before the first launch. See
[browser setup](browser.md) for verification, KeePassXC pairing and importing
the tracked Surfingkeys settings.

Run `./scripts/setup-editor` for Neovim and its plugins; see [editor setup](editor.md)
for the optional development tools. Complete this before using the editor config.
Debian calls bat/fd `batcat` and `fdfind`; the local wrapper and shell aliases handle
these names. Optional fnm integration uses `~/.local/share/fnm`.

Yazi requires `file`, which is in `packages.txt`. Optional preview tools include
`poppler-utils` for PDFs, `ffmpeg` for videos and `7zip` for archives.
See [Yazi dependencies](https://yazi-rs.github.io/docs/installation/).

## Desktop portals

Follow [Sway portals and Yazi file dialogs](portals.md) for backend installation,
Zen integration, activation and live verification.

## Firewall

See [host firewall](firewall.md) for the nftables policy and required service
override. Review the target machine’s remote-access and VM networking needs
before installing system rules.

## Automatic updates

Follow [automatic Debian updates](updates.md) to install the project policy and
enable Debian’s APT timers. It applies stable maintenance and security updates
without automatic reboots. The user-config installer does not activate it.

## Virtual machines

Use virt-manager, virsh, virt-install, and virt-viewer directly. The package
list includes the libvirt/QEMU tools, UEFI firmware, and SPICE support.
For a local hypervisor, configure the service and your access explicitly:

```sh
sudo usermod -aG libvirt,kvm "$USER"
sudo systemctl enable --now libvirtd
virsh --connect qemu:///system list --all
```

Log out and back in after changing groups. Configure networks and storage
through virt-manager as needed. Remote libvirt connections also work through
virt-manager. This repo does not create, start, or delete guests.

Only trusted administrators should have full libvirt management access. See the
[security audit](security-audit.md) before configuring privileged access.

## Local and system configuration

`~/.config/sway/local.conf` is optional and never installed by the repo.
Use it for machine-specific bindings or output rules. Fresh installs receive your saved laptop/LG monitor profiles: curved landscape
on the left, portrait ultrawide on the right, vertically centered, with the laptop
panel off when docked. Existing kanshi configs are preserved. Edit your kanshi
profiles locally; [`examples/kanshi-laptop.conf`](../examples/kanshi-laptop.conf) preserves the previous
physical laptop/monitor layout as a reference. The installed default is the saved layout in that example, matched by monitor
description so connector changes do not affect the external display identities.

`system/` mirrors selected `/etc` files for unattended apt updates and lid
behavior. Review and copy individual files with sudo only when wanted.
Full setup applies the two APT policy files only; the other files stay manual. The nftables host policy and its required
service override also live under `system/`; follow [firewall](firewall.md) before
applying them. Networking and remote access remain under your control.

## Updating an existing installation

Edit source files under `dotfiles/`, run `./scripts/check --full`, inspect
`./scripts/install --dry-run`, then run `./scripts/install`. Copies in your home
are independent of the checkout; copy useful live experiments back into the repo
before reinstalling. Render themes after editing templates. Reload Sway and reopen
applications as described in [appearance](appearance.md).

The installer backs up and retires a named list of files from removed integrations.
It does not uninstall packages, stop old system services or change active networking.
Current security/migration follow-up lives in [TODO](TODO.md).

## Debian 13 ALSA rule fix

Debian alsa-utils 1.2.14-1 has a duplicate udev label.
The project copy fixes the label and keeps sound state restore enabled.
This fixes the warning. It does not prove XPS audio works.
See [Debian bug 1104719](https://bugs.debian.org/1104719).

For that package version only:

```sh
sudo install -Dm644 system/udev/rules.d/90-alsa-restore.rules /etc/udev/rules.d/90-alsa-restore.rules
sudo udevadm control --reload-rules
```

Remove the local override once Debian supplies a corrected rule.

## Optional backups

`./scripts/setup --backups` also installs Restic. Scheduling remains disabled
until configured separately on each machine; see [encrypted home backups](backups.md).

## System health installation

Setup installs the health-report dependencies, copies `sysreport` and its user
units, and enables the daily timer. With no user bus it creates an enablement link
for the next login. A config-only install copies the files but does not activate
the timer; see [system health](sysreport.md) for commands.

Successful config installation and full setup write an installation receipt to
`~/.local/state/linux-config/install-info`. Full setup records its completion only
after all setup steps succeed. The receipt and installed copies do not depend on
keeping the source checkout. OpenSnitch's matching daemon/UI sockets are
provisioned by setup; optional persistent database storage is described in the report guide.
