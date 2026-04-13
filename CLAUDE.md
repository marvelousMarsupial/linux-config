# CLAUDE.md

AI onboarding for this Linux dotfiles and workstation provisioning project.

## CRITICAL: File Ownership

The repo lives at `/home/august/linux-config` and is **owned by august**.

**After editing files, ALWAYS fix ownership:**
```bash
chown -R august:august /home/august/linux-config
```

If august can't read configs (sway reload fails, etc.), this is usually the cause.

## Quick Start

```bash
# Deploy a role to user "august"
./deploy/install.sh desktop

# Or via forge CLI
forge setup desktop
forge status
forge teardown
```

**Available roles:** `server`, `dev`, `desktop`, `remote-desktop`, `net-box`, `attack-box`

## Fresh System Install

On a fresh system, `install.sh`:
1. Installs ansible and git
2. Clones repo to `$HOME/linux-config`
3. Runs `forge setup <role>`

Ansible then:
1. Creates user `august` with zsh, sudo access
2. Handles repo location (clones to user home, or symlinks from root if run as root)
3. Removes conflicting dotfiles
4. Stows all configs
5. Applies theme

## Project Structure

```
linux-config/
├── ansible/
│   ├── playbook.yml          # Main entry (accepts role= variable)
│   ├── inventory.yml         # Defines august user + host settings
│   ├── group_vars/all.yml    # Global vars (theme, gitea URL)
│   └── roles/
│       ├── server/           # Base: bash, zsh, tmux, nvim, yazi, starship, theme
│       ├── dev/              # + build tools, nodejs, lazygit, podman services
│       ├── desktop/          # + sway, foot, fuzzel, mako, flatpaks
│       ├── remote-desktop/   # + wayvnc, Sunshine
│       ├── net-box/          # + network analysis tools
│       └── attack-box/       # + CTF/pentesting (Burp, Wireshark, etc)
├── bin/                      # ALL scripts (in PATH via zshrc)
│   ├── forge                 # Main CLI (setup/teardown/status)
│   ├── forge_lib/            # Python library for forge
│   ├── theme                 # Theme switcher (base16 colors)
│   ├── vpn                   # WireGuard helper
│   ├── wifi-menu             # WiFi selector ($mod+w)
│   ├── status-panel          # System status popup ($mod+i)
│   ├── status-osd            # Status OSD notifications
│   ├── power-menu            # Power/logout menu ($mod+x)
│   ├── keybinds              # Keybinding reference ($mod+?)
│   ├── notes-toggle          # Toggle notes window ($mod+n)
│   ├── notes-picker          # Search notes ($mod+Shift+n)
│   ├── pass-picker           # Password store picker ($mod+p)
│   ├── remote-start          # Start headless Sway + VNC
│   ├── remote-stop           # Stop remote desktop
│   ├── tmux-keys             # Tmux keybinding reference
│   ├── battery-monitor       # Background battery daemon
│   ├── launcher              # Fuzzel wrapper
│   └── motd                  # Message of the day
├── stow/                     # Dotfiles (ALL directory symlinks)
│   ├── bash/                 # → ~/.bashrc
│   ├── zsh/                  # → ~/.zshrc, ~/.zsh_aliases
│   ├── tmux/                 # → ~/.tmux.conf
│   ├── nvim/                 # → ~/.config/nvim/
│   ├── yazi/                 # → ~/.config/yazi/
│   ├── starship/             # → ~/.config/starship.toml
│   ├── theme/                # → ~/.config/theme/
│   ├── sway/                 # → ~/.config/sway/
│   ├── foot/                 # → ~/.config/foot/
│   ├── fuzzel/               # → ~/.config/fuzzel/
│   ├── gtk-3.0/              # → ~/.config/gtk-3.0/
│   ├── gtk-4.0/              # → ~/.config/gtk-4.0/
│   └── desktop-entries/      # → ~/.local/share/applications/
├── vault/                    # Documentation (Obsidian-compatible)
└── deploy/
    └── install.sh            # Bootstrap script
```

## Stow Packages

All configs symlinked via GNU Stow. **Never edit files in ~/.config** - edit in `stow/`.

**Server role stows:** `bash zsh tmux yazi nvim starship theme`
**Desktop role adds:** `sway foot fuzzel gtk-3.0 gtk-4.0 desktop-entries`

### Symlink Structure

All stow packages create **directory symlinks** (entire dir is one symlink):

```
~/.config/nvim      → linux-config/stow/nvim/.config/nvim
~/.config/yazi      → linux-config/stow/yazi/.config/yazi
~/.config/theme     → linux-config/stow/theme/.config/theme
~/.config/sway      → linux-config/stow/sway/.config/sway
~/.config/foot      → linux-config/stow/foot/.config/foot
~/.config/fuzzel    → linux-config/stow/fuzzel/.config/fuzzel
~/.config/gtk-3.0   → linux-config/stow/gtk-3.0/.config/gtk-3.0
~/.config/gtk-4.0   → linux-config/stow/gtk-4.0/.config/gtk-4.0
```

**Home dotfiles** (file symlinks):
```
~/.bashrc           → linux-config/stow/bash/.bashrc
~/.zshrc            → linux-config/stow/zsh/.zshrc
~/.zsh_aliases      → linux-config/stow/zsh/.zsh_aliases
~/.tmux.conf        → linux-config/stow/tmux/.tmux.conf
```

### Theme-Generated Files

Theme writes color configs into stowed directories. These are gitignored:
- `stow/nvim/.config/nvim/lua/plugins/colorscheme.lua`
- `stow/foot/.config/foot/colors.ini`
- `stow/fuzzel/.config/fuzzel/colors.ini`
- `stow/sway/.config/sway/colors`
- `stow/gtk-3.0/.config/gtk-3.0/gtk.css`
- `stow/gtk-4.0/.config/gtk-4.0/gtk.css`
- `stow/yazi/.config/yazi/theme.toml`

Theme also writes to real directories (not stowed):
- `~/.config/tmux/colors.conf`
- `~/.config/mako/config`
- `~/.config/swaylock/config`

## Role Inheritance

```
server (base)
├── dev              → server + dev tools
├── desktop          → server + Sway GUI
│   ├── remote-desktop → desktop + VNC/Sunshine
│   └── attack-box     → remote-desktop + CTF/pentesting
└── net-box          → server + network analysis
```

## Theme System

Python script applying base16 colors across all apps:

```bash
theme list                    # Available schemes
theme apply gruvbox-dark-hard # Apply everywhere
theme current                 # Show current + palette
```

**Schemes:** gruvbox-dark-hard, catppuccin-mocha, kanagawa, tokyonight-moon, onehalfdark-black, summer-dusk

**Files:**
- `stow/theme/.config/theme/config.toml` - Output targets + reload hooks
- `stow/theme/.config/theme/schemes/` - Base16 YAML definitions
- `stow/theme/.config/theme/templates/` - Jinja2 per app

## Dev Containers

Isolated development environments with podman (rootless).

**Required packages** (installed by server role):
- `podman` - container runtime
- `uidmap` - user namespace mapping for rootless containers
- `passt` - network namespace tool (provides `pasta`)

```bash
forge dev create myapp python    # Create Python container
forge dev create frontend ts     # TypeScript/Node container
forge dev enter myapp            # Enter container shell
forge dev list                   # List containers
forge dev rm myapp               # Remove container
```

**Runtimes:** `python`, `c`, `ts`, `nuxt`, `bun`, `java`, `base`

**Features:**
- Debian trixie base with zsh, tmux, neovim, starship, yazi
- Full nvim config with LSPs pre-installed
- Theme system injected (inherits host theme)
- Format-on-save with conform.nvim (stylua, prettier, ruff, shfmt)

**Options:**
```bash
forge dev create myapp python -p 8000           # Port forward
forge dev create myapp python -v .:/workspace   # Mount volume
```

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
- `<leader>cf` - Format buffer
- `<leader>ca` - Code actions
- `gd` - Go to definition
- `K` - Hover docs

**Vault notes** (~/vault/*.md) have zen mode - no statusline.

## Obsidian Integration

obsidian.nvim configured for `~/vault/` and `~/linux-config/docs/`:

- `gf` - Follow [[wiki-link]] under cursor
- `<leader>of` - Follow link
- `<leader>os` - Search notes
- `<leader>oq` - Quick switch
- `<leader>ot` - Today's daily note
- `<leader>ch` - Toggle checkbox

## Incus VMs and Containers

```bash
forge lxc server              # LXC container with server role
forge vm attack-box           # VM with attack-box role
forge instances               # List all
forge delete mydev --force    # Remove
```

## Tmux Keybindings

Run `tmux-keys` for full reference. Key bindings:

- `Alt+hjkl` - Navigate panes (integrates with smart-splits.nvim)
- `Alt+HJKL` - Resize panes
- `Alt+v` - Vertical split + nvim (uses `~/.local/bin/nvim`)
- `Alt+V` - Horizontal split + nvim
- `Alt+e` - Vertical split + yazi
- `Alt+Enter` - Vertical split + shell
- `Alt+s` - Horizontal split + shell
- `Alt+q` - Close pane
- `Alt+z` - Zoom toggle
- `Ctrl+Space r` - Reload config

## Remote Desktop

Scripts in `bin/`:
- `remote-start` - Launches headless Sway + wayvnc + mako
- `remote-stop` - Stops all remote desktop services

Aliases: `rstart`, `rstop`

## Sway Keybindings

Run `keybinds` or `$mod+?` to see all bindings.

**Variables in sway config:**
- `$mod` - Mod4 (Super/Windows key)
- `$term` - foot
- `$bin` - `/home/august/linux-config/bin` (for script paths)
- `$menu` - `$bin/launcher`

Key bindings use scripts from `bin/` via `$bin` variable:
- `$mod+n` - notes-toggle
- `$mod+Shift+n` - notes-picker
- `$mod+x` - power-menu
- `$mod+p` - pass-picker
- `$mod+i` - status-panel
- `$mod+w` - wifi-menu
- `$mod+v` - nvim (uses full path `~/.local/bin/nvim`)

## Python Scripts

All scripts in `bin/` are Python 3 (stdlib only except theme which needs pyyaml + jinja2).

PATH set in `.zshrc`:
```bash
export PATH="$HOME/linux-config/bin:$HOME/.local/bin:$PATH"
```

## IMPORTANT: nvim Path Issue

nvim is installed in `~/.local/bin/nvim`. When calling nvim from:
- Sway keybindings (`bindsym ... exec`)
- `foot -e` commands
- tmux `split-window` commands

**You MUST use the full path `~/.local/bin/nvim`** because:
1. Sway doesn't inherit PATH from `.zshrc` (only interactive shells source it)
2. `foot -e` executes commands directly without a shell
3. The display manager starts sway before shell profile is loaded

This is why sway config uses `$bin` variable for scripts in `~/linux-config/bin/`.

## Common Tasks

**Add a new stow package:**
1. Create `stow/<name>/.config/<name>/` with config files
2. Add to appropriate role's `dotfiles.yml` stow command
3. Add directory removal before stow (so stow can create symlink)

**Re-stow after changes:**
```bash
cd ~/linux-config/stow
stow -R -t ~ <package>   # -R restows (unlink + link)
```

## Ansible Patterns

**Architecture support:** x86_64 and ARM64. Variables in `group_vars/all.yml`.

**User context:** Always use `target_user` and `target_home`, NOT `ansible_env.HOME`.

```yaml
path: "{{ target_home }}/.config/foo"
owner: "{{ target_user }}"
```

## Design Principles

- Everything reproducible via Ansible + Stow
- All configs in stow/, never edit ~/.config directly
- All scripts in bin/, one location
- Directory symlinks for simplicity
- Sway (Wayland) only, no X11
- Keyboard-driven, minimal UI

## Git / Backup

Update bundle:
```bash
cd ~/linux-config
git bundle create ~/linux-config.bundle --all
```

Restore:
```bash
git clone linux-config.bundle linux-config
```
