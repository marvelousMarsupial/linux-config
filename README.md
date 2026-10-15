# linux-config

Personal Debian 13 workstation configs and small scripts: Sway, Foot, Fuzzel,
Neovim, Yazi, zsh and six shared Base16 palettes. VM management uses libvirt
directly. The default package list keeps development tools minimal.

## Intended installation flow

The goal is to keep this setup in a hosted Git repository (GitHub or equivalent),
run a single curl command on a fresh Debian 13 x86_64 workstation, and leave no
local checkout behind. Installed applications and configs remain independent of
the downloaded source. `scripts/bootstrap` implements temporary checkout cleanup;
the current changes must be committed and published before remote use works.
See [remote bootstrap](docs/installation.md#remote-bootstrap) for commands and limits.

## Start here

Read [installation](docs/installation.md) before applying the setup. It covers
packages, separately installed applications, config backups and local settings.
Finish the [short manual checklist](docs/manual.md) after setup.
See [current state](docs/state.md) for checked repository/host differences and
[systemd](docs/systemd.md), [updates](docs/updates.md) and
[firewall](docs/firewall.md) for system integration.
Use [editor setup](docs/editor.md) for Neovim and optional language tools.
See [shell](docs/shell.md) for prefix history, suggestions, fuzzy completion and
the Git prompt.
See [notifications](docs/notifications.md) and [font/operator display](docs/font-preview.md)
for the current desktop shortcuts and typography.

For a fresh Debian 13 x86_64 workstation, run as your normal
user from the checkout:

```sh
./scripts/setup --dry-run
./scripts/setup
```

Setup installs Debian packages, missing Yazi/Zen/font releases, the pinned Yazi
portal backend, fzf-tab, Neovim/plugins, user configs, initial colors, group permissions,
zsh login shell, incoming firewall and the automatic-update policy. Review required
incoming access in [the firewall guide](docs/firewall.md) before setup. It requires sudo and network
access. It includes IPython with NumPy/Matplotlib/SciPy, KeePassXC/OpenSnitch dark
defaults and the Yazi upload/save picker with fullscreen handling. It does not
install Obsidian or perform development-VM cleanup. Log out/in or reboot afterward. See [installation](docs/installation.md)
for details and the system settings that remain manual.

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
tmux, VPN and dictation implementations remain in Git history.
