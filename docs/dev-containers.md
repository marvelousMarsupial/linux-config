# Dev Containers

Isolated development environments using [[podman]]. Managed via [[forge]].

## Requirements

Rootless podman requires these packages (installed by server role):
- `podman` - container runtime
- `uidmap` - user namespace mapping
- `passt` - network namespace tool (provides `pasta`)

## Quick Reference

```bash
forge dev create <name> <runtime>  # Create container
forge dev enter <name>             # Enter shell
forge dev list                     # List all
forge dev stop <name>              # Stop
forge dev rm <name>                # Remove
```

## Runtimes

| Runtime | Base | Packages |
|---------|------|----------|
| `base` | debian:trixie | zsh, tmux, nvim, git, starship, yazi, eza, bat, fzf, ripgrep |
| `python` | base + | python3, pip, venv, ipython |
| `c` | base + | gcc, g++, gdb, make, cmake, clang |
| `ts` | base + | nodejs (LTS via nodesource) |
| `nuxt` | base + | nodejs (LTS via nodesource) |
| `bun` | base + | bun (via curl installer) |
| `java` | base + | openjdk-17-jdk, maven, gradle |

All containers include Node.js (for Mason LSP servers).

## Create Options

```bash
# Basic
forge dev create myapp python

# With port forwarding
forge dev create myapp python -p 8000
forge dev create myapp python -p 8000 -p 3000 -p 5432

# With volume mount
forge dev create myapp python -v .:/workspace
forge dev create myapp python -v ~/projects/foo:/workspace

# Both
forge dev create myapp python -p 8000 -v .:/workspace
```

## Port Forwarding

| Format | Result |
|--------|--------|
| `-p 8000` | Host 8000 → Container 8000 |
| `-p 8080:80` | Host 8080 → Container 80 |

Multiple ports: repeat `-p` flag.

## Volume Mounts

| Format | Result |
|--------|--------|
| `-v .:/workspace` | Current dir → /workspace |
| `-v ~/foo:/workspace` | Specific dir → /workspace |
| `-v ~/foo:/app` | Specific dir → /app |

Relative paths resolved to absolute. Default container path is `/workspace`.

## Container Details

### Base Image
- `debian:trixie` (testing)
- Runs as root inside container
- Hostname = container name (without `dev-` prefix)

### Installed Tools
| Tool | Version | Notes |
|------|---------|-------|
| zsh | 5.9 | Default shell |
| tmux | 3.5a | Terminal multiplexer |
| nvim | 0.11.5 | AppImage extracted (FUSE workaround) |
| starship | latest | Prompt |
| yazi | latest | File manager (+ ya companion) |
| eza | 0.21 | ls replacement |
| bat | 0.25 | cat replacement (symlinked from batcat) |
| fzf | 0.60 | Fuzzy finder |
| ripgrep | 14.1 | Search |
| git | 2.47 | Version control |
| nodejs | LTS | For Mason LSP installs |
| file | 5.46 | MIME detection (for yazi) |
| trash-cli | 0.24 | Safe rm |

### LSPs Pre-installed
Via Mason:
- lua-language-server
- typescript-language-server
- pyright
- tailwindcss-language-server
- vue-language-server
- clangd

### Formatters Pre-installed
Via Mason (for format-on-save):
- stylua (Lua)
- prettier (JS/TS/Vue/CSS/HTML/JSON/YAML/MD)
- ruff (Python)
- shfmt (Shell)

### Dotfiles Copied
From `stow/`:
- `nvim/.config/nvim` → `/root/.config/nvim`
- `zsh/.zshrc` → `/root/.zshrc`
- `zsh/.zsh_aliases` → `/root/.zsh_aliases`
- `tmux/.tmux.conf` → `/root/.tmux.conf`
- `starship/.config/starship.toml` → `/root/.config/starship.toml`
- `yazi/.config/yazi` → `/root/.config/yazi`
- `theme/.config/theme` → `/root/.config/theme`

From `bin/`:
- `theme` → `/root/.local/bin/theme`

### Theme System

Containers inherit the host's current theme automatically during creation.

```bash
# Inside container
theme list      # See available schemes
theme apply kanagawa   # Switch theme
theme current   # Show current
```

Theme applies to: tmux, neovim, yazi

### Working Directory
Default: `/workspace`
- Created automatically
- Enter command uses `-w /workspace`
- Mount volumes here for project files

## Workflow

### New Project
```bash
mkdir ~/projects/myapp
cd ~/projects/myapp
forge dev create myapp python -p 8000 -v .:/workspace
forge dev enter myapp
# Inside container: /workspace is your project dir
```

### Existing Container
```bash
forge dev list                    # Check status
forge dev enter myapp             # Enter (starts if stopped)
```

### Cleanup
```bash
forge dev stop myapp              # Stop (keeps data)
forge dev rm myapp                # Remove (deletes container)
forge dev rm myapp -f             # Force remove (stops first)
```

## Podman Storage

Container data location:
```
~/.local/share/containers/storage/
├── overlay-images/       # Base images
├── overlay/              # Container layers
└── overlay-containers/   # Container metadata
```

Get container filesystem:
```bash
podman inspect dev-myapp --format '{{.GraphDriver.Data.MergedDir}}'
```

## Troubleshooting

### Container Won't Start
```bash
podman logs dev-myapp
podman rm -f dev-myapp
forge dev create myapp python
```

### Tool Not Found
Check if aliased in `.zsh_aliases`:
- `rm` → `trash-put`
- `ls` → `eza`
- `cat` → `bat`

Use backslash for real command: `\rm`, `\ls`, `\cat`

### Yazi "file not found" Error
```bash
\rm -rf ~/.cache/yazi
# Restart yazi
```

### Neovim VIMRUNTIME Error
Container uses wrapper script at `/usr/local/bin/nvim` that sets VIMRUNTIME.

### LSP Not Installing
Ensure npm is available:
```bash
which npm
npm --version
```

### Format-on-save Not Working
Check formatters are installed:
```bash
:MasonInstall stylua prettier ruff shfmt
```

## vs Dev Services

| Feature | Dev Containers | [[dev-services]] |
|---------|---------------|------------------|
| Purpose | Full dev environment | Single-purpose services |
| Shell access | Yes (zsh) | No (web UI only) |
| Your dotfiles | Yes | No |
| Theme system | Yes | No |
| Format-on-save | Yes | No |
| Isolation | Per-project | Per-service |
| Use case | Coding | Coder/Jupyter/DB admin |

## Files

- `bin/forge_lib/dev.py` - Implementation
- `bin/forge` - CLI entry point

See also: [[dev-containers-manual]], [[podman]], [[forge]], [[nvim]]
