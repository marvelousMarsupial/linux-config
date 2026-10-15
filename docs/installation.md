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

The dry run performs read-only discovery and prints a plan without downloads,
sudo or file changes. A normal run, in order:

- Installs `packages.txt` through APT. This includes IPython with NumPy,
  Matplotlib and SciPy, Chromium, KeePassXC, OpenSnitch and its UI theme
  library, Starship, and the portal broker with its GTK and wlr backends.
- Validates and installs the incoming firewall and its service override, backs
  up previous files, then enables and reloads nftables. A failed check stops
  setup. Review the [firewall policy](firewall.md) before running setup,
  especially over a remote connection.
- Installs missing Yazi/ya, Zen and CaskaydiaCove Nerd Font from their latest
  official GitHub releases. The published SHA-256 digest is required and checked
  before extraction; a missing digest or mismatch stops setup.
- Calls `setup-shell` to install the pinned fzf-tab plugin. See [shell](shell.md).
- Builds termfilechooser at the pinned upstream commit from temporary sources
  and installs it into `/usr`. Build dependencies remain installed. A marker
  avoids repeating the build when that revision is present.
- Calls `setup-editor` for Neovim and plugins, which calls `install` for user
  files. Existing applications are reused; setup is not an application updater.
- Applies the summer-dusk palette on a fresh home. Existing theme selections,
  generated colors and local monitor profiles are preserved. Paths are rendered
  for the installing user.
- Adds the user to video, libvirt and kvm, selects zsh as the login shell,
  generates the English UTF-8 locale if missing, installs the APT update policy
  with backups, and enables libvirtd and Debian's update timers.
- Configures the OpenSnitch daemon socket for the installing user and enables
  the daemon. Sway starts the matching UI service at login.
- Enables the daily health-report timer, or creates its enablement link when no
  user bus is available.
- Writes an installation receipt once every step has succeeded.

Use `./scripts/setup --dev-tools` to also install optional editor development
tools, and `--backups` to include Restic. Setup stops on failure; fix the
reported step and rerun. It is repeatable, not transactional: completed steps
stay applied. Changed user files have the config installer's backups, replaced
application files have dated backups, and no packages are purged. No running
desktop or browser is restarted.

Log out and back in, or reboot, then launch Zen normally. The script does not
configure disk encryption, firmware, new monitor layouts, lid policy, VM
networks/storage or hardware-specific drivers. Those are covered below or in
the [manual checklist](manual.md).

## Remote bootstrap

`scripts/bootstrap` clones one branch or tag into a private directory under
`/tmp`, runs `scripts/setup`, then deletes only that temporary directory. It
also cleans up on failure and on interruption. A power loss or SIGKILL can
leave temporary files behind. It never deletes an existing working checkout.
Installed files are copies, so no checkout is needed afterward.

The default source is `https://github.com/marvelousMarsupial/linux-config.git`,
branch `main`. From a fresh machine with Bash, curl, git, Python 3, sudo and CA
certificates installed:

```sh
curl -fsSL https://raw.githubusercontent.com/marvelousMarsupial/linux-config/main/scripts/bootstrap | bash
```

Pass `--dry-run` or `--dev-tools` after `bash` to forward them to setup. To use
another host or a release tag, set the two environment variables; fetch the
bootstrap from the same tag you select:

```sh
curl -fsSL <raw-bootstrap-url> | env \
  LINUX_CONFIG_REPOSITORY=https://github.com/OWNER/linux-config.git \
  LINUX_CONFIG_REF=v1.0.0 bash
```

The bootstrap and the checkout both execute code from the chosen source; use a
source you trust. Sudo asks for your password through the terminal. If the
repository is private, clone it with credentials and run `./scripts/setup`
from the checkout instead.

Offline tests cover piped execution, argument forwarding, failure exit codes
and cleanup. A real run on a fresh machine is still pending; see [TODO](TODO.md).

## Config-only installation

For user config changes without package or system setup:

```sh
./scripts/install --dry-run
./scripts/install
```

The installer copies files into your home, backs up replaced files under
`~/.local/state/linux-config/backups/`, and skips identical files. It does not
install packages, change services, remove software or alter networking. Run it
again after editing the repo. Existing monitor profiles, Neovim plugin locks and
generated color files are preserved. `--home /path` installs into another home
for testing. Copies are independent of the checkout.

The installer also retires a named list of files from removed integrations,
with the same backups. It does not uninstall packages or stop system services.

After installing or changing user units:

```sh
systemctl --user daemon-reload
```

Log out of Sway and back in after changing units; see [systemd](systemd.md).
Press Super+Shift+c to reload Sway after config changes. Sway starts on tty1
from the zsh login shell; run `sway` manually for an initial session.

## Shell setup

Zsh provides prefix history on Up/Down, grey history and completion suggestions,
fzf search on Ctrl+R, and a searchable Tab menu with previews. Full setup
installs the pinned fzf-tab plugin. For an existing installation:

```sh
./scripts/setup-shell
./scripts/install
```

Open a new terminal afterward. Plugin installation requires GitHub access.
Without fzf-tab, standard Zsh completion remains available. See [shell](shell.md).

## Screen brightness

The brightness keys change the screen in 10 percent steps. Super+F11 dims it
and Super+F12 brightens it. Debian needs `brightness-udev` and membership in the
video group; full setup handles both, and the config-only installer does not.
Reboot after setup to apply the group and device permissions.

On the laptop, run these from a terminal in Sway:

```sh
brightnessctl --class=backlight --list
~/.local/bin/brightness up
```

If the command works but the keys do not, try the keys with Fn. Fn+Escape
toggles Fn lock on supported Dell keyboards. The Super shortcuts avoid Fn lock.
A VM usually has no backlight control. If the laptop also reports none, check
its panel type before changing kernel settings.

## Separately installed applications

Setup installs missing Yazi/ya, Zen and CaskaydiaCove Nerd Font from official
releases. For config-only installs, provide those yourself. Zen's wrapper
expects `~/.local/zen/zen`. Flatpak is not installed by the package list; if
you add it yourself, the launcher already lists its applications. No
application binaries or fonts are bundled by this repository.

Zen's config installer installs uBlock Origin, Dark Reader, Surfingkeys and
KeePassXC-Browser through browser policies on the next browser startup, skips
the welcome screen and sets dark defaults before the first launch. See
[browser](browser.md).

Run `./scripts/setup-editor` for Neovim and its plugins; see [editor](editor.md)
for the optional development tools. Debian calls bat and fd `batcat` and
`fdfind`; the local wrapper and shell aliases handle those names. Optional fnm
integration uses `~/.local/share/fnm`.

Yazi requires `file`, which is in `packages.txt`. Optional preview tools include
`poppler-utils` for PDFs, `ffmpeg` for videos and `7zip` for archives. See
[Yazi dependencies](https://yazi-rs.github.io/docs/installation/).

## Virtual machines

Use virt-manager, virsh, virt-install and virt-viewer directly. The package
list includes the libvirt/QEMU tools, UEFI firmware and SPICE support. Setup
enables libvirtd and adds the user to libvirt and kvm; log out and back in
after group changes. Configure networks and storage through virt-manager.
This repo does not create, start or delete guests.

Full libvirt management access is administrative access. See the
[security audit](security-audit.md) before widening it.

## Local and system configuration

`~/.config/sway/local.conf` is optional and never installed by the repo. Use it
for machine-specific bindings or output rules.

Fresh installs receive the saved laptop/LG monitor profiles: curved landscape on
the left, portrait ultrawide on the right, laptop panel off when docked.
Existing kanshi configs are preserved. Profiles match on monitor description, so
connector names can change without affecting them.
[`examples/kanshi-laptop.conf`](../examples/kanshi-laptop.conf) keeps the
earlier physical layout as a reference.

`system/` mirrors selected `/etc` files. Full setup applies the two APT policy
files, the nftables policy and its service override. Lid and udev changes stay
manual:

- `system/systemd/logind.conf.d/lid.conf` suspends on lid close, including on
  external power, and ignores lid close while docked. Review it for the machine,
  then copy it into `/etc/systemd/logind.conf.d/` during a planned reboot.
- `system/udev/rules.d/90-alsa-restore.rules` fixes a duplicate udev label in
  Debian alsa-utils 1.2.14-1 and keeps sound state restore enabled. See
  [Debian bug 1104719](https://bugs.debian.org/1104719). For that package
  version only:

  ```sh
  sudo install -Dm644 system/udev/rules.d/90-alsa-restore.rules /etc/udev/rules.d/90-alsa-restore.rules
  sudo udevadm control --reload-rules
  ```

  Remove the override once Debian ships a corrected rule.

## Updating an existing installation

Edit source files under `dotfiles/`, run `./scripts/check --full`, inspect
`./scripts/install --dry-run`, then run `./scripts/install`. Copies in your home
are independent of the checkout; copy useful live experiments back into the repo
before reinstalling. Render themes after editing templates, then reload Sway and
reopen applications as described in [appearance](appearance.md).

## Receipts and health report

Successful config installation and full setup write a receipt to
`~/.local/state/linux-config/install-info` with the source commit, a
modified-checkout flag, the date and the install scope. Full setup records
completion only after every step succeeds. Setup also enables the daily
`sysreport.timer`; a config-only install copies the files but does not activate
the timer. See [system health](sysreport.md).

## Optional backups

`./scripts/setup --backups` also installs Restic. Scheduling stays disabled
until configured on each machine; see [backups](backups.md).
