# linux-config

Personal Debian 13 workstation configs and small scripts: Sway, Foot, Fuzzel,
Neovim, Yazi, zsh and six shared Base16 palettes. One font everywhere,
CaskaydiaCove Nerd Font. VM management uses libvirt directly. The default
package list keeps development tools minimal.

## Start here

Read [installation](docs/installation.md) before applying the setup. It covers
packages, separately installed applications, config backups and local settings.
Finish the [manual checklist](docs/manual.md) after setup.

For a fresh Debian 13 x86_64 workstation, run as your normal user from the
checkout:

```sh
./scripts/setup --dry-run
./scripts/setup
```

Setup installs Debian packages, missing Yazi/Zen/font releases, the pinned Yazi
portal backend, fzf-tab, Neovim and plugins, user configs, initial colors, group
permissions, the zsh login shell, the incoming firewall and the automatic-update
policy. It requires sudo and network access. Review required incoming access in
[the firewall guide](docs/firewall.md) before running it. Log out and back in,
or reboot, afterward.

The same setup can run without a local checkout through
[remote bootstrap](docs/installation.md#remote-bootstrap), which downloads a
temporary checkout and removes it when setup finishes.

For config-only updates, use `./scripts/install --dry-run` and `./scripts/install`.
The config installer backs up changed files and preserves local monitor profiles,
plugin locks and generated colors. Run `./scripts/check --full` to verify the repo.

## Repository layout

| Path | Contents |
| --- | --- |
| `dotfiles/` | User configs, desktop commands, theme templates and assets. |
| `scripts/` | Install, check and explicit editor/shell setup commands. |
| `packages.txt` | Debian desktop package requirements. |
| `system/` | APT/firewall configuration applied by setup; optional lid and udev settings. |
| `examples/` | Hardware-specific reference configurations. |
| `tests/` | Offline integration and editor smoke checks. |
| `docs/` | Setup, usage, maintenance, security and project tracking. |

See the [documentation index](docs/README.md), [TODO](docs/TODO.md) and
[security audit](docs/security-audit.md). Retired Forge, container-management,
tmux, VPN, dictation and font-preview implementations remain in Git history.
