# Tmux

Terminal multiplexer - the real window manager.

Philosophy: **tmux handles windows/panes, Sway just holds terminals**.

## Quick Reference

| Key | Action |
|-----|--------|
| `Alt+Enter` | New pane (side by side) |
| `Alt+s` | New pane (stacked) |
| `Alt+h/j/k/l` | Navigate panes |
| `Alt+H/J/K/L` | Resize panes |
| `Alt+q` | Close pane |
| `Alt+z` | Zoom toggle |
| `Alt+d` | Detach |

## Configuration

| File | Purpose |
|------|---------|
| `stow/tmux/.tmux.conf` | Main config |
| `~/.config/tmux/colors.conf` | Theme colors (generated) |

## Key Features

### No Prefix (Mostly)

Almost all bindings use `Alt+key` directly. No `Ctrl+b` prefix needed.

Only exception: `Ctrl+Space, r` to reload config.

### smart-splits Integration

Navigation works seamlessly between tmux panes AND nvim splits:

```
┌─────────────┬─────────────┐
│   nvim      │   shell     │
│  ┌───┬───┐  │             │
│  │   │   │  │             │
│  └───┴───┘  │             │
└─────────────┴─────────────┘
```

`Alt+l` from left nvim split → right shell pane (crosses application boundary)

### No Status Bar

Status bar is disabled for minimal UI. To check session:
- `Alt+d` (detach shows session name)
- `tmux ls` from shell

### Nested Sessions (SSH)

For SSH into another tmux session:

| Key | Action |
|-----|--------|
| `F12` | Toggle outer/inner focus |

When inner is active:
- Outer status bar dims
- All keys pass to inner session

Remote sessions automatically show status bar at top (detected via `$SSH_CLIENT`).

## Session Management

### Starting

```bash
tmux                    # New unnamed session
tmux new -s work        # Named session
```

### Navigating

| Key | Action |
|-----|--------|
| `Alt+d` | Detach |
| `Alt+n` | New session |
| `Alt+(` | Previous session |
| `Alt+)` | Next session |

### From Shell

```bash
tmux ls                 # List sessions
tmux attach -t work     # Attach to named
tmux kill-session -t work # Kill session
```

## Pane Management

### Creating Panes

| Key | Opens | Split |
|-----|-------|-------|
| `Alt+Enter` | Shell | Vertical (side by side) |
| `Alt+s` | Shell | Horizontal (stacked) |
| `Alt+v` | Nvim | Vertical |
| `Alt+V` | Nvim | Horizontal |
| `Alt+e` | Yazi | Vertical |
| `Alt+E` | Yazi | Horizontal |

### Navigation

| Key | Direction |
|-----|-----------|
| `Alt+h` | Left |
| `Alt+j` | Down |
| `Alt+k` | Up |
| `Alt+l` | Right |

### Resize

| Key | Direction |
|-----|-----------|
| `Alt+H` | Left (shrink) |
| `Alt+J` | Down (grow) |
| `Alt+K` | Up (shrink) |
| `Alt+L` | Right (grow) |

Resize amount: 3 cells per keypress.

### Other

| Key | Action |
|-----|--------|
| `Alt+q` | Close pane |
| `Alt+z` | Zoom/unzoom (fullscreen pane) |

## Copy Mode

Vi-style copy mode:

| Key | Action |
|-----|--------|
| `Alt+[` | Enter copy mode |
| `v` | Begin selection |
| `y` | Copy to clipboard |
| `Alt+]` | Paste |

Navigation in copy mode uses vim keys (`h/j/k/l`, `Ctrl+d/u`, `/` search, etc.).

Copy goes to system clipboard via `wl-copy`.

## Settings

| Setting | Value | Reason |
|---------|-------|--------|
| escape-time | 0 | No delay for Esc (vim) |
| base-index | 1 | Windows start at 1 |
| history-limit | 50000 | Big scrollback |
| mouse | on | Mouse support |
| mode-keys | vi | Vi keybindings |

### Terminal Features

```bash
set -g default-terminal "tmux-256color"
set -ag terminal-overrides ",xterm-256color:RGB"
```

Enables true color in nvim and other apps.

## Theme Integration

Colors loaded from [[theme]]-generated file:

```bash
source-file -q ~/.config/tmux/colors.conf
```

Theme changes apply after `theme apply <scheme>` (auto-reloads).

## Workflow

### Typical Session

1. `$mod+Return` in Sway opens foot terminal
2. foot automatically starts tmux (via .zshrc)
3. `Alt+Enter` to split for another shell
4. `Alt+v` to open nvim in split
5. `Alt+h/l` to navigate between them
6. `Alt+z` to zoom nvim when focused

### Long-Running Tasks

1. Start task in pane
2. `Alt+d` to detach
3. Close terminal
4. Later: `tmux attach`
5. Task still running

### Multiple Projects

```bash
tmux new -s project1
# work on project1
Alt+d  # detach

tmux new -s project2
# work on project2
Alt+d  # detach

tmux ls
# project1: 1 windows
# project2: 1 windows

tmux attach -t project1
```

## Troubleshooting

### Keys Not Working

Check tmux is running:
```bash
echo $TMUX  # Should show socket path
```

### Colors Wrong

Reload config:
```bash
tmux source-file ~/.tmux.conf
```

Or check terminal:
```bash
echo $TERM  # Should be tmux-256color
```

### Clipboard Not Working

Check wl-copy:
```bash
echo "test" | wl-copy
wl-paste
```

### Nested Session Keys Stuck

Press `F12` to toggle back to outer session.

## Files

| Path | Purpose |
|------|---------|
| `stow/tmux/.tmux.conf` | Main config |
| `~/.config/tmux/colors.conf` | Generated theme colors |
| `/tmp/tmux-$UID/default` | Socket (sessions) |

See also: [[keybindings]], [[nvim]], [[theme]]
