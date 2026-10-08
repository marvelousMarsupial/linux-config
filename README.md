# linux-config

Personal Debian 13 workstation configs and small scripts: Sway, Foot, Fuzzel,
Neovim, Yazi, zsh and six shared Base16 palettes. VM management uses libvirt
directly. The default package list keeps development tools minimal.

## Start here

Read [installation](docs/installation.md) before applying the setup. It covers
packages, separately installed applications, config backups and local settings.
See [current state](docs/state.md) for checked repository/host differences and
[systemd](docs/systemd.md), [updates](docs/updates.md) and
[firewall](docs/firewall.md) for system integration.
Use [editor setup](docs/editor.md) for Neovim and optional language tools.

For an existing installation:

```sh
./scripts/check --full
./scripts/install --dry-run
./scripts/install
```

The installer copies user files with backups and preserves local monitor profiles,
plugin locks and generated colors. It also retires named files from old integrations.
It does not install packages, change services or alter networking.

## Repository layout

| Path | Contents |
| --- | --- |
| `dotfiles/` | User configs, desktop commands, theme templates and assets. |
| `scripts/` | Install, check and explicit editor setup commands. |
| `packages.txt` | Debian desktop package requirements. |
| `system/` | Optional `/etc` configuration; review and copy explicitly. |
| `examples/` | Hardware-specific reference configurations. |
| `tests/` | Offline integration and editor smoke checks. |
| `docs/` | Setup, usage, maintenance, security and project tracking. |

See the [documentation index](docs/README.md), [TODO](docs/TODO.md) and
[security audit](docs/security-audit.md). Retired Forge, container-management,
tmux, VPN and dictation implementations remain in Git history.
