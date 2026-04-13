# Theme

Base16 color scheme system applying consistent colors across all apps.

## Quick Reference

```bash
theme list              # Available schemes
theme apply <scheme>    # Apply everywhere
theme current           # Show current + palette
```

Or via forge:
```bash
forge theme list
forge theme apply gruvbox-dark-hard
```

## How It Works

1. Scheme file defines 16 colors (Base16 format)
2. Python script reads scheme + Jinja2 templates
3. Generates config files for each app
4. Reloads running apps via hooks

### Flow

```
~/.config/theme/schemes/gruvbox-dark-hard.yaml
         ↓
   theme apply gruvbox-dark-hard
         ↓
~/.config/theme/templates/*.j2
         ↓
┌────────────────────────────────────────┐
│ ~/.config/foot/colors.ini              │
│ ~/.config/tmux/colors.conf             │
│ ~/.config/sway/colors                  │
│ ~/.config/fuzzel/colors.ini            │
│ ~/.config/yazi/theme.toml              │
│ ~/.config/nvim/lua/plugins/colorscheme.lua │
│ ~/.config/gtk-3.0/gtk.css              │
│ ~/.config/gtk-4.0/gtk.css              │
│ ~/.config/mako/config                  │
│ ~/.config/swaylock/config              │
└────────────────────────────────────────┘
         ↓
   Reload hooks (swaymsg, nvim socket, etc.)
```

## Available Schemes

Built-in schemes in `~/.config/theme/schemes/`:

| Scheme | Description |
|--------|-------------|
| gruvbox-dark-hard | Warm retro (default) |
| catppuccin-mocha | Purple-ish pastel |
| kanagawa | Japanese ink painting |
| tokyonight-moon | Cool blue night |
| summer-dusk | Warm sunset |
| onehalfdark-black | Clean minimal |

## Base16 Colors

Each scheme defines 16 colors:

| Base | Role | Typical Use |
|------|------|-------------|
| base00 | Background | Default background |
| base01 | Lighter bg | Status bars, selection |
| base02 | Selection | Highlighted text bg |
| base03 | Comments | Subtle text |
| base04 | Dark fg | Secondary text |
| base05 | Foreground | Default text |
| base06 | Light fg | Bright text |
| base07 | Lightest fg | Cursor, highlights |
| base08 | Red | Errors, deletions |
| base09 | Orange | Integers, booleans |
| base0A | Yellow | Classes, search |
| base0B | Green | Strings, success |
| base0C | Cyan | Regex, escape chars |
| base0D | Blue | Functions, links |
| base0E | Purple | Keywords |
| base0F | Brown | Deprecated, embedded |

## Adding Schemes

Create `~/.config/theme/schemes/<name>.yaml`:

```yaml
scheme: "My Theme"
author: "You"
base00: "1d2021"  # Background
base01: "3c3836"  # Lighter bg
base02: "504945"  # Selection
base03: "665c54"  # Comments
base04: "bdae93"  # Dark fg
base05: "d5c4a1"  # Foreground
base06: "ebdbb2"  # Light fg
base07: "fbf1c7"  # Lightest fg
base08: "fb4934"  # Red
base09: "fe8019"  # Orange
base0A: "fabd2f"  # Yellow
base0B: "b8bb26"  # Green
base0C: "8ec07c"  # Cyan
base0D: "83a598"  # Blue
base0E: "d3869b"  # Purple
base0F: "d65d0e"  # Brown
```

Find schemes at: https://github.com/tinted-theming/base16-schemes

## Apps Themed

| App | Config Generated | Reload Method |
|-----|-----------------|---------------|
| foot | `~/.config/foot/colors.ini` | SIGUSR1 |
| tmux | `~/.config/tmux/colors.conf` | `tmux source` |
| nvim | `lua/plugins/colorscheme.lua` | Socket command |
| yazi | `~/.config/yazi/theme.toml` | Restart yazi |
| sway | `~/.config/sway/colors` | `swaymsg reload` |
| fuzzel | `~/.config/fuzzel/colors.ini` | Auto (next launch) |
| mako | `~/.config/mako/config` | `makoctl reload` |
| swaylock | `~/.config/swaylock/config` | Auto |
| GTK 3 | `~/.config/gtk-3.0/gtk.css` | App restart |
| GTK 4 | `~/.config/gtk-4.0/gtk.css` | App restart |

## Configuration

### config.toml

`~/.config/theme/config.toml` defines outputs:

```toml
[[outputs]]
template = "foot.ini.j2"
target = "~/.config/foot/colors.ini"

[[outputs]]
template = "tmux.conf.j2"
target = "~/.config/tmux/colors.conf"
hook = "tmux source-file ~/.tmux.conf"

[[outputs]]
template = "nvim.lua.j2"
target = "~/.config/nvim/lua/plugins/colorscheme.lua"
# nvim reload via socket in script
```

### Templates

Jinja2 templates in `~/.config/theme/templates/`:

```jinja2
{# foot.ini.j2 #}
[colors]
foreground={{ base05 }}
background={{ base00 }}
regular0={{ base00 }}
regular1={{ base08 }}
{# ... #}
```

## Nvim Theme Integration

### Native Plugins

Some schemes have native nvim plugins with extra features:

| Scheme | Plugin |
|--------|--------|
| tokyonight-* | tokyonight.nvim |
| onedark-* | onedark.nvim |

Theme system detects these and uses native plugin instead of mini.base16.

### Base16 Fallback

For other schemes, uses mini.base16 with:
- Full Base16 palette
- Treesitter highlight overrides (for vibrant syntax)
- Transparent backgrounds

### Live Reload

Theme script finds all nvim sockets and sends reload:
```bash
nvim --server $sock --remote-send ':source ~/.config/nvim/lua/plugins/colorscheme.lua<CR>'
```

## Transparency

All themes use transparent backgrounds to show foot terminal background. Solid elements:
- NormalFloat (slightly visible)
- StatusLine (subtle)
- Borders

## Dev Container Integration

Dev containers inherit the host's current theme during creation. Inside a container:

```bash
theme list      # See available schemes
theme apply kanagawa   # Switch theme
theme current   # Show current
```

Theme applies to: tmux, neovim, yazi (foot is not in containers).

## Troubleshooting

### Theme Not Applying

1. Check template exists: `ls ~/.config/theme/templates/`
2. Check scheme exists: `ls ~/.config/theme/schemes/`
3. Run with verbose: `theme apply <scheme> -v`

### Nvim Colors Wrong

1. Check socket: `ls $XDG_RUNTIME_DIR/nvim.*.sock`
2. Manual reload: `:source ~/.config/nvim/lua/plugins/colorscheme.lua`
3. Check file was updated: `head ~/.config/nvim/lua/plugins/colorscheme.lua`

### Foot Colors Not Updating

Foot auto-reloads on config change. If not:
```bash
pkill -USR1 foot
```

### Sway Not Reloading

Check swaymsg works:
```bash
swaymsg reload
```

## Files

| Path | Purpose |
|------|---------|
| `bin/theme` | Main script |
| `stow/theme/.config/theme/config.toml` | Output configuration |
| `stow/theme/.config/theme/templates/` | Jinja2 templates |
| `stow/theme/.config/theme/schemes/` | Base16 YAML schemes |
| `~/.cache/theme/current` | Current scheme name |

See also: [[nvim]], [[forge]], [[stow]]
