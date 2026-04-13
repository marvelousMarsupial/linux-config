# Linux Config

Workstation provisioning system. Ansible + Stow + Base16 theming.

## Quick Start

```bash
# Fresh install
./deploy/install.sh desktop

# Or via forge
forge setup server        # Base CLI
forge setup desktop       # + Sway GUI
forge status              # Check current
```

## System

| Topic | Description |
|-------|-------------|
| [[forge]] | CLI tool for provisioning |
| [[roles]] | What gets installed (server/dev/desktop) |
| [[stow]] | Dotfile management via symlinks |
| [[theme]] | Base16 color schemes across all apps |

## Development

| Topic | Description |
|-------|-------------|
| [[dev-containers]] | Isolated dev environments (podman) |
| [[dev-containers-manual]] | Manual Dockerfile setup |
| [[dev-services]] | Coder, Jupyter, CloudBeaver |
| [[podman]] | Rootless container runtime |
| [[incus]] | System containers and VMs |

## Security

| Topic | Description |
|-------|-------------|
| [[net-box]] | Network monitoring (blue team) |
| [[attack-box]] | CTF/pentesting (red team) |

## Tools

| Topic | Description |
|-------|-------------|
| [[nvim]] | Neovim config and plugins |
| [[tmux]] | Terminal multiplexer |
| [[keybindings]] | All keyboard shortcuts |
| [[obsidian-nvim]] | Note-taking in nvim |

## Key Paths

| Path | Contents |
|------|----------|
| `~/linux-config/` | This repo |
| `~/linux-config/bin/` | CLI tools (forge, vpn, etc) |
| `~/linux-config/stow/` | Dotfiles source |
| `~/linux-config/ansible/` | Playbooks and roles |
| `~/linux-config/vault/` | These docs |
| `~/.cache/forge/role` | Current role cache |

## Architecture

```
server (base CLI)
├── dev           → + dev services, build tools
├── desktop       → + Sway GUI, flatpaks
│   ├── remote-desktop → + VNC/Sunshine
│   └── attack-box     → + CTF/pentesting (via remote-desktop)
└── net-box       → + network analysis
```

## Daily Notes

`<leader>ot` in nvim opens today's note in `vault/daily/`.
