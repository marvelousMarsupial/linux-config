# Linux Config

Declarative, reproducible Linux workstation provisioning with Ansible. One command to deploy a complete development environment with Sway (Wayland), unified theming, and remote desktop support.

```bash
curl -sL http://git.rat/August/linux-config/raw/branch/main/deploy/install.sh | bash -s -- desktop
```

## Features

- **Role-based deployment** - Server, dev, desktop, or remote-desktop configurations
- **Unified theme system** - One command changes colors across 12+ applications
- **Dotfiles via GNU Stow** - Version-controlled configs with instant updates
- **Sway/Wayland desktop** - Keyboard-driven tiling WM with multi-monitor support
- **Remote desktop** - Headless Sway over WireGuard VPN with wayvnc
- **Dev containers** - Isolated podman environments with full tooling
- **Modern CLI tools** - Neovim, tmux, yazi, starship, eza, bat, ripgrep

## Quick Start

```bash
# Bootstrap and deploy
./deploy/install.sh desktop

# Or use forge CLI
forge setup desktop      # Install desktop role
forge status             # Show current role + theme
forge switch dev         # Switch to different role
forge teardown           # Remove current role
```

## Roles

Roles inherit from each other in a chain:

```
server (base)
├── dev         → server + build tools, podman, dev services
├── desktop     → server + sway, foot, flatpaks
│   ├── remote-desktop → desktop + wayvnc, sunshine
│   └── attack-box     → remote-desktop + pentesting tools
└── net-box     → server + network analysis
```

### Server (Base)

Core CLI environment for any Linux system.

| Component | Description |
|-----------|-------------|
| **zsh** | Shell with autosuggestions, syntax highlighting, 97+ aliases |
| **tmux** | Terminal multiplexer with vim-style navigation |
| **Neovim 0.11** | Full config with LSP, format-on-save, treesitter |
| **yazi** | Terminal file manager with image previews |
| **starship** | Cross-shell prompt |
| **theme** | Unified color scheme system (Base16) |

**Packages:** git, curl, stow, btop, bat, eza, fd-find, ripgrep, trash-cli, wireguard-tools

### Desktop

Full Wayland desktop environment (extends server).

| Component | Description |
|-----------|-------------|
| **Sway** | Tiling Wayland compositor |
| **foot** | Fast terminal emulator |
| **fuzzel** | Application launcher |
| **mako** | Notification daemon |
| **swaylock** | Screen locker |

**Flatpak Apps:** Obsidian (notes), Zen Browser, Moonlight (game streaming)

### Remote Desktop

Headless Sway accessible over VPN (extends desktop).

```bash
remote-start   # Start headless Sway + wayvnc
remote-stop    # Stop remote desktop
vpn up         # Connect WireGuard
```

## Dev Containers

Isolated development environments with podman:

```bash
forge dev create myapp python    # Create Python container
forge dev create frontend ts     # TypeScript/Node container
forge dev enter myapp            # Enter container shell
forge dev list                   # List all containers
forge dev rm myapp               # Remove container
```

**Runtimes:** `python`, `c`, `ts`, `nuxt`, `bun`, `java`, `base`

**Features:**
- Debian trixie with zsh, tmux, neovim, starship, yazi
- Full nvim config with LSPs pre-installed
- Theme system (inherits host theme)
- Format-on-save (stylua, prettier, ruff, shfmt)

**Options:**
```bash
forge dev create myapp python -p 8000           # Port forward
forge dev create myapp python -v .:/workspace   # Mount volume
```

## Theme System

Unified Base16 color scheme across all applications.

```bash
theme list                     # Available schemes
theme apply gruvbox-dark-hard  # Apply everywhere
theme current                  # Show current + color palette
```

**Available Schemes:**
- `gruvbox-dark-hard` - Warm, retro colors
- `catppuccin-mocha` - Pastel dark theme
- `kanagawa` - Japanese-inspired palette
- `tokyonight-moon` - Cool blue theme
- `onehalfdark-black` - Clean dark theme
- `summer-dusk` - Warm sunset colors

**Themed Applications:**
| App | Config File | Reload Method |
|-----|-------------|---------------|
| tmux | ~/.config/tmux/colors.conf | tmux source |
| Neovim | colorscheme.lua | Auto-reload |
| yazi | ~/.config/yazi/theme.toml | Auto |
| foot | ~/.config/foot/colors.ini | SIGUSR1 |
| Sway | ~/.config/sway/colors | swaymsg reload |
| fuzzel | ~/.config/fuzzel/colors.ini | Auto |
| mako | ~/.config/mako/config | makoctl reload |
| swaylock | ~/.config/swaylock/config | Auto |
| GTK 3/4 | ~/.config/gtk-*/gtk.css | Restart apps |

## Neovim

**LSPs:** lua_ls, ts_ls, pyright, vue_ls, tailwindcss, clangd

**Format-on-save** via conform.nvim:
| Language | Formatter |
|----------|-----------|
| Python | ruff |
| JS/TS/Vue/CSS/HTML/JSON/YAML/MD | prettier |
| Lua | stylua |
| C/C++ | clang-format |
| Shell | shfmt |

**Key bindings:**
| Key | Action |
|-----|--------|
| `<leader>cf` | Format buffer |
| `<leader>ca` | Code actions |
| `gd` | Go to definition |
| `K` | Hover docs |
| `<leader>ff` | Find files |
| `<leader>fg` | Live grep |

**Notes integration:** Files in `~/vault/*.md` have zen mode (no statusline).

## Keybindings

### Sway (Super = Mod key)

Run `keybinds` or press `Super+?` for the full cheatsheet.

**Essential:**
| Key | Action |
|-----|--------|
| `Super+Return` | Terminal |
| `Super+Space` | App launcher |
| `Super+q` | Close window |
| `Super+f` | Fullscreen |
| `Super+Shift+Space` | Toggle floating |

**Navigation (vim-style):**
| Key | Action |
|-----|--------|
| `Super+h/j/k/l` | Focus left/down/up/right |
| `Super+Shift+h/j/k/l` | Move window |
| `Super+1-9` | Go to workspace |

**Apps:**
| Key | Action |
|-----|--------|
| `Super+b` | Browser (Zen) |
| `Super+n` | New note |
| `Super+Shift+n` | Search notes |
| `Super+e` | Files (yazi) |
| `Super+p` | Passwords (pass) |
| `Super+v` | Editor (nvim) |

**System:**
| Key | Action |
|-----|--------|
| `Super+Escape` | Lock screen |
| `Super+x` | Power menu |
| `Super+i` | Status info |
| `Super+w` | WiFi menu |
| `Super+Shift+c` | Reload Sway |

### Tmux

| Key | Action |
|-----|--------|
| `Alt+h/j/k/l` | Navigate panes |
| `Alt+Enter` | Vertical split |
| `Alt+s` | Horizontal split |
| `Alt+q` | Close pane |
| `Alt+z` | Zoom pane |
| `F12` | Toggle nested session |

## Shell Aliases

97+ aliases defined in `~/.zsh_aliases`. Highlights:

```bash
# Navigation
..          # cd ..
-           # cd to previous

# Modern replacements
ls          # eza (colored, icons)
cat         # bat (syntax highlighting)

# Git
gs          # git status
ga          # git add
gc          # git commit
gp          # git push

# Safety
rm          # trash-put (moves to trash)
```

## Project Structure

```
linux-config/
├── ansible/                  # Ansible playbooks and roles
│   └── roles/
│       ├── server/           # Base CLI tools
│       ├── dev/              # Development tools
│       ├── desktop/          # Sway desktop
│       └── remote-desktop/   # VNC + WireGuard
├── bin/                      # ALL scripts (in PATH)
│   ├── forge                 # Main CLI tool
│   ├── theme                 # Color scheme switcher
│   ├── vpn                   # WireGuard helper
│   ├── wifi-menu             # WiFi selector
│   ├── power-menu            # Power/logout menu
│   ├── status-panel          # System status popup
│   ├── notes-toggle          # Toggle notes window
│   ├── notes-picker          # Search notes
│   ├── keybinds              # Keybinding reference
│   ├── remote-start          # Start headless Sway
│   └── ...
├── stow/                     # Dotfiles (symlinked to ~)
│   ├── zsh/                  # Shell config
│   ├── tmux/                 # Terminal multiplexer
│   ├── nvim/                 # Neovim config
│   ├── yazi/                 # File manager
│   ├── starship/             # Prompt
│   ├── theme/                # Theme system
│   ├── sway/                 # Window manager
│   ├── foot/                 # Terminal
│   ├── fuzzel/               # Launcher
│   ├── gtk-3.0/              # GTK3 settings
│   └── gtk-4.0/              # GTK4 settings
├── vault/                    # Documentation
└── deploy/
    └── install.sh            # Bootstrap script
```

## Obsidian Integration

obsidian.nvim configured for `~/vault/`:

| Key | Action |
|-----|--------|
| `gf` | Follow [[wiki-link]] |
| `<leader>of` | Follow link |
| `<leader>os` | Search notes |
| `<leader>oq` | Quick switch |
| `<leader>ot` | Today's daily note |
| `<leader>ch` | Toggle checkbox |

## Dotfiles with Stow

GNU Stow creates symlinks from `stow/` to `~`:

```bash
cd ~/linux-config/stow
stow -t ~ zsh       # Creates ~/.zshrc → stow/zsh/.zshrc
stow -R zsh         # Re-stow (update)
stow -D zsh         # Remove symlinks
```

All stow packages create **directory symlinks** - editing in `stow/` updates configs instantly.

## Troubleshooting

**Sway config won't reload:**
```bash
chown -R august:august /home/august/linux-config
sway -C -c ~/.config/sway/config
```

**Theme not applying:**
```bash
theme current              # Check current theme
theme apply gruvbox-dark-hard  # Re-apply
```

## Design Philosophy

- **Everything in Ansible** - Fully reproducible
- **All scripts in bin/** - One location
- **Directory symlinks** - Simple stow structure
- **Wayland-only** - No X11
- **Keyboard-driven** - Vim keys everywhere
- **Working > perfect** - Ship it

## Requirements

- Debian/Ubuntu-based Linux (tested on Debian 13)
- Root access for initial setup

## License

MIT
