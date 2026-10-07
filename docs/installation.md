# Installation

[Documentation index](README.md)

This setup targets a personal Debian 13 workstation. Review commands and package
requirements before applying them. Existing software is not automatically removed.

## Desktop packages and user configs

Start with Debian 13 and your normal user account. Review the package list:

```sh
sudo apt-get update
sed '/^#/d; /^$/d' packages.txt | xargs sudo apt-get install --
./scripts/install --dry-run
./scripts/install
~/.local/bin/theme apply summer-dusk
```

The installer copies files into your home, backs up replaced files under
`~/.local/state/linux-config/backups/`, and skips identical files. It does
not install packages, change services, remove software, or alter networking.
Run it again after editing the repo. Existing monitor profiles, Neovim plugin
locks, and generated color files are preserved. `--home /path` installs into
an alternate home for testing. Copies are independent of the checkout.

For a fresh desktop, enable the existing user monitor service explicitly:

```sh
systemctl --user daemon-reload
systemctl --user enable kanshi.service
```

Open a new terminal after installation and press Super+Shift+c to reload Sway.
Choose zsh as your login shell with `chsh -s /bin/zsh` if desired. Sway starts
on tty1; run `sway` manually for an initial session.

## Separately installed applications

Install Yazi/ya, Starship, Zen and CaskaydiaCove Nerd Font separately using your
preferred source. Zen's wrapper expects `~/.local/zen/zen`. The launcher includes
separately installed Flatpak application entries. No application binaries or fonts
are bundled by this repository.

Run `./scripts/setup-editor` for Neovim and its plugins; see [editor setup](editor.md)
for the optional development tools. Complete this before using the editor config.
Debian calls bat/fd `batcat` and `fdfind`; the local wrapper and shell aliases handle
these names. Optional fnm integration uses `~/.local/share/fnm`.

Yazi requires `file`, which is in `packages.txt`. Optional preview tools include
`poppler-utils` for PDFs, `ffmpeg` for videos and `7zip` for archives.
See [Yazi dependencies](https://yazi-rs.github.io/docs/installation/).

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
Use it for machine-specific bindings or output rules. Edit your kanshi
profiles locally; [`examples/kanshi-laptop.conf`](../examples/kanshi-laptop.conf) preserves the previous
physical laptop/monitor layout as a reference. The default kanshi config
contains no hardware assumptions.

`system/` mirrors selected `/etc` files for unattended apt updates and lid
behavior. Review and copy individual files with sudo only when wanted.
They are not automatically applied. Networking and remote access remain
under your control.

## Updating an existing installation

Edit source files under `dotfiles/`, run `./scripts/check --full`, inspect
`./scripts/install --dry-run`, then run `./scripts/install`. Copies in your home
are independent of the checkout; copy useful live experiments back into the repo
before reinstalling. Render themes after editing templates. Reload Sway and reopen
applications as described in [appearance](appearance.md).

The installer backs up and retires a named list of files from removed integrations.
It does not uninstall packages, stop old system services or change active networking.
Current security/migration follow-up lives in [TODO](TODO.md).
