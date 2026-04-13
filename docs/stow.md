# Stow

GNU Stow manages dotfiles by creating symlinks from a central location.

## How It Works

Files in `stow/<package>/` mirror the home directory structure:

```
stow/nvim/.config/nvim/
  → ~/.config/nvim (directory symlink)

stow/zsh/.zshrc
  → ~/.zshrc (file symlink)
```

Run `stow <package>` to create symlinks. Run `stow -D <package>` to remove them.

## Directory Structure

```
~/linux-config/stow/
├── bash/
│   └── .bashrc
├── zsh/
│   ├── .zshrc
│   └── .zsh_aliases
├── tmux/
│   └── .tmux.conf
├── nvim/
│   └── .config/nvim/
│       ├── init.lua
│       └── lua/...
├── yazi/
│   └── .config/yazi/
├── starship/
│   └── .config/starship.toml
├── theme/
│   └── .config/theme/
├── sway/
│   └── .config/sway/
├── foot/
│   └── .config/foot/
├── fuzzel/
│   └── .config/fuzzel/
├── gtk-3.0/
│   └── .config/gtk-3.0/
├── gtk-4.0/
│   └── .config/gtk-4.0/
└── desktop-entries/
    └── .local/share/applications/
```

## Packages by Role

### Server Role

| Package | Creates |
|---------|---------|
| bash | ~/.bashrc |
| zsh | ~/.zshrc, ~/.zsh_aliases |
| tmux | ~/.tmux.conf |
| nvim | ~/.config/nvim/ |
| yazi | ~/.config/yazi/ |
| starship | ~/.config/starship.toml |
| theme | ~/.config/theme/ |

### Desktop Role (adds)

| Package | Creates |
|---------|---------|
| sway | ~/.config/sway/ |
| foot | ~/.config/foot/ |
| fuzzel | ~/.config/fuzzel/ |
| gtk-3.0 | ~/.config/gtk-3.0/ |
| gtk-4.0 | ~/.config/gtk-4.0/ |
| desktop-entries | ~/.local/share/applications/ |

## Symlink Types

This project uses **directory symlinks** for all .config directories:

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

Home dotfiles use **file symlinks**:
```
~/.bashrc           → linux-config/stow/bash/.bashrc
~/.zshrc            → linux-config/stow/zsh/.zshrc
~/.zsh_aliases      → linux-config/stow/zsh/.zsh_aliases
~/.tmux.conf        → linux-config/stow/tmux/.tmux.conf
```

## Commands

### Basic Usage

```bash
cd ~/linux-config/stow

# Link a package
stow -t ~ nvim

# Unlink a package
stow -D -t ~ nvim

# Relink (unlink then link)
stow -R -t ~ nvim

# Simulate (dry run)
stow -n -t ~ nvim
```

### Common Options

| Option | Purpose |
|--------|---------|
| `-t ~` | Target directory (home) |
| `-R` | Restow (relink) |
| `-D` | Delete (unlink) |
| `-n` | Dry run (no changes) |
| `-v` | Verbose |

### Multiple Packages

```bash
stow -t ~ bash zsh tmux nvim yazi starship theme
```

## Theme-Generated Files

The theme system writes color configs into stowed directories. Since these are directory symlinks, theme writes directly to the repo files. These are gitignored:

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

## Adding a New Package

1. Create directory structure:
```bash
mkdir -p stow/myapp/.config/myapp
```

2. Add config files:
```bash
cp ~/.config/myapp/config.yaml stow/myapp/.config/myapp/
```

3. Remove original:
```bash
rm -rf ~/.config/myapp
```

4. Stow it:
```bash
cd ~/linux-config/stow
stow -t ~ myapp
```

5. Add to appropriate role's dotfiles.yml:
```yaml
- name: Remove existing configs
  file:
    path: "{{ target_home }}/.config/myapp"
    state: absent

- name: Stow configs
  shell: |
    cd {{ target_home }}/linux-config/stow
    stow -t {{ target_home }} myapp
```

## Important Rules

### Never Edit ~/.config Directly

Since all .config directories are symlinks to stow/, editing `~/.config/nvim/init.lua` actually edits `stow/nvim/.config/nvim/init.lua`.

This is fine, but remember the source location for git commits.

### Conflicts

Stow won't create symlinks if target exists and isn't a symlink.

**Before stowing:**
```bash
rm -rf ~/.config/nvim  # Remove existing
stow -t ~ nvim         # Now stow can link
```

Ansible handles this automatically via `dotfiles.yml`.

## Troubleshooting

### "Conflict" Error

```
WARNING! stowing nvim would cause conflicts:
  * existing target is neither a link nor a directory: .config/nvim
```

Solution: Remove the existing file/directory first.

### Broken Symlinks

```bash
# Find broken symlinks in home
find ~ -maxdepth 3 -type l -xtype l 2>/dev/null
```

Fix by restowing:
```bash
stow -R -t ~ nvim
```

### Wrong Symlink Target

If symlink points to wrong location:
```bash
stow -D -t ~ nvim  # Remove
stow -t ~ nvim     # Recreate
```

### Stow from Wrong Directory

Always run from `stow/` directory:
```bash
cd ~/linux-config/stow
stow -t ~ nvim
```

Or use full path:
```bash
stow -d ~/linux-config/stow -t ~ nvim
```

## Verifying Symlinks

```bash
# Check specific file
ls -la ~/.config/nvim

# List all symlinks in .config
ls -la ~/.config | grep ^l

# Find all symlinks pointing to stow
find ~ -type l -ls 2>/dev/null | grep linux-config/stow
```

## Files

| Path | Purpose |
|------|---------|
| `~/linux-config/stow/` | All stow packages |
| `ansible/roles/server/tasks/dotfiles.yml` | Server role stowing |
| `ansible/roles/desktop/tasks/dotfiles.yml` | Desktop role stowing |

See also: [[roles]], [[nvim]], [[theme]]
