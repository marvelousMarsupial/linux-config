# Keybindings

Complete keybinding reference. Run `keybinds` or `$mod+?` in Sway to see this.

Philosophy: **hjkl is ALWAYS movement**. Modifier determines scope:
- **Alt** → tmux (panes/sessions)
- **Super ($mod)** → Sway (windows/workspaces)
- **Leader (Space)** → nvim (editor commands)

## Tmux

Prefix: `Ctrl+Space` (only used for reload: `prefix r`)

### Navigation

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+h` | Move left | Works across nvim splits too |
| `Alt+j` | Move down | smart-splits integration |
| `Alt+k` | Move up | |
| `Alt+l` | Move right | |

### Resize

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+H` | Resize left | 3 cells per press |
| `Alt+J` | Resize down | |
| `Alt+K` | Resize up | |
| `Alt+L` | Resize right | |

### Pane Management

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+Enter` | New pane (vertical split) | Side-by-side |
| `Alt+s` | New pane (horizontal split) | Stacked |
| `Alt+v` | New pane with nvim | Vertical split |
| `Alt+V` | New pane with nvim | Horizontal split |
| `Alt+e` | New pane with yazi | Vertical split |
| `Alt+E` | New pane with yazi | Horizontal split |
| `Alt+q` | Close pane | |
| `Alt+z` | Zoom toggle | Fullscreen current pane |

### Session Management

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+d` | Detach | Return to shell |
| `Alt+n` | New session | |
| `Alt+(` | Previous session | Shift+9 |
| `Alt+)` | Next session | Shift+0 |

### Copy Mode

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+[` | Enter copy mode | Vi keybindings |
| `Alt+]` | Paste buffer | |
| `v` | Begin selection | In copy mode |
| `y` | Copy selection | Uses wl-copy |

### Nested Sessions

| Key | Action | Notes |
|-----|--------|-------|
| `F12` | Toggle outer/inner | For SSH sessions |

When toggled off, outer session ignores keys (passes to inner).

## Neovim

Leader: `Space`

### File Navigation (Telescope)

| Key | Action | Notes |
|-----|--------|-------|
| `<leader>ff` | Find files | Hidden files included |
| `<leader>fg` | Live grep | Regex search in files |
| `<leader>fb` | Buffers | Open buffers |
| `<leader>fr` | Recent files | Previously opened |
| `<leader>fh` | Help tags | Vim help |
| `<leader>fk` | Keymaps | All keybindings |
| `<leader>fd` | Diagnostics | All LSP errors/warnings |
| `<leader>fs` | Document symbols | Functions, classes, etc. |
| `<leader>fw` | Grep word | Word under cursor |
| `<leader>/` | Search in buffer | Fuzzy find in current file |

Inside Telescope:
- `Ctrl+j/k` - Move up/down
- `Ctrl+q` - Send to quickfix list
- `Esc` - Close

### File Manager (Yazi)

| Key | Action | Notes |
|-----|--------|-------|
| `<leader>y` | Open yazi | Current file's directory |
| `<leader>Y` | Open yazi (cwd) | Working directory |
| `<leader>-` | Resume yazi | Last session |

### Obsidian

| Key | Action | Notes |
|-----|--------|-------|
| `<leader>on` | New note | Prompts for title |
| `<leader>oo` | Open in Obsidian | Desktop app |
| `<leader>ot` | Today's note | Daily note |
| `<leader>oy` | Yesterday's note | |
| `<leader>os` | Search notes | Full-text search |
| `<leader>oq` | Quick switch | Note picker |
| `<leader>ob` | Backlinks | Notes linking to this |
| `<leader>ol` | Links | Links in current note |
| `<leader>of` | Follow link | Under cursor |
| `<leader>ow` | Switch workspace | docs/notes |
| `<leader>ch` | Toggle checkbox | `[ ]` ↔ `[x]` |
| `gf` | Follow link | Under cursor (native) |

### LSP

| Key | Action | Notes |
|-----|--------|-------|
| `gd` | Go to definition | |
| `gD` | Go to declaration | |
| `gr` | Go to references | |
| `gi` | Go to implementation | |
| `gt` | Go to type definition | |
| `K` | Hover documentation | |
| `<leader>k` | Signature help | Function parameters |
| `<leader>rn` | Rename symbol | Project-wide |
| `<leader>ca` | Code actions | Fixes, refactors |
| `<leader>cf` | Format buffer | |
| `[d` | Previous diagnostic | |
| `]d` | Next diagnostic | |
| `<leader>e` | Show diagnostic | Float at cursor |
| `<leader>dl` | Diagnostic list | Location list |

### Completion (nvim-cmp)

| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+n` | Next item | |
| `Ctrl+p` | Previous item | |
| `Ctrl+b` | Scroll docs up | |
| `Ctrl+f` | Scroll docs down | |
| `Ctrl+Space` | Trigger completion | Manual |
| `Ctrl+e` | Cancel | |
| `Enter` | Confirm | |
| `Tab` | Confirm / Jump snippet | |
| `Shift+Tab` | Jump snippet backwards | |

### Window Navigation

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+h/j/k/l` | Move between splits | Works with tmux |
| `Alt+H/J/K/L` | Resize splits | |
| `Ctrl+h/j/k/l` | Move (within nvim only) | |

### Buffer Management

| Key | Action | Notes |
|-----|--------|-------|
| `Shift+h` | Previous buffer | |
| `Shift+l` | Next buffer | |
| `<leader>bd` | Delete buffer | |

### Editing

| Key | Action | Notes |
|-----|--------|-------|
| `jj` | Exit insert mode | Faster than Esc |
| `Esc` | Clear search highlight | In normal mode |
| `J` | Move line down | Visual mode |
| `K` | Move line up | Visual mode |
| `Ctrl+d` | Scroll down | Centered |
| `Ctrl+u` | Scroll up | Centered |
| `n` | Next search | Centered |
| `N` | Previous search | Centered |
| `<leader>p` | Paste without yanking | Visual mode |

### Quick Actions

| Key | Action | Notes |
|-----|--------|-------|
| `<leader>w` | Save file | |
| `<leader>q` | Quit | |
| `<leader>Q` | Save and quit | |
| `Ctrl+s` | Save file | Works in any mode |
| `<leader>C` | Copy file contents | Entire file to clipboard |
| `<leader>D` | Delete file contents | Clear buffer |

### Terminal

| Key | Action | Notes |
|-----|--------|-------|
| `Alt+t` | Toggle terminal | FTerm floating terminal |

### Markdown

| Key | Action | Notes |
|-----|--------|-------|
| `<leader>mr` | Toggle render | In-buffer rendering |
| `<leader>mp` | Markdown preview | Browser preview |

### Dashboard

| Key | Action | Notes |
|-----|--------|-------|
| `<leader>;` | Open dashboard | Alpha-nvim |

## Sway

Mod: `Super` (Windows key)

### Window Focus

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+h` | Focus left | |
| `$mod+j` | Focus down | |
| `$mod+k` | Focus up | |
| `$mod+l` | Focus right | |
| `$mod+Arrow` | Focus (arrows) | Alternative |
| `Alt+Tab` | Cycle windows | |
| `$mod+a` | Focus parent | Container |

### Window Movement

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+Shift+h` | Move left | |
| `$mod+Shift+j` | Move down | |
| `$mod+Shift+k` | Move up | |
| `$mod+Shift+l` | Move right | |
| `$mod+Shift+Arrow` | Move (arrows) | Alternative |

### Window Resize

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+Ctrl+h` | Shrink width | 50px |
| `$mod+Ctrl+l` | Grow width | |
| `$mod+Ctrl+k` | Shrink height | |
| `$mod+Ctrl+j` | Grow height | |
| `$mod+Ctrl+Arrow` | Resize (arrows) | Alternative |

### Workspaces

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+1-9` | Switch workspace | |
| `$mod+Shift+1-9` | Move to workspace | |
| `$mod+Tab` | Next workspace | |
| `$mod+Shift+Tab` | Previous workspace | |

### Multi-Monitor

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+o` | Focus next output | |
| `$mod+Shift+o` | Move workspace to output | |

### Layout

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+\` | Split horizontal | Side-by-side |
| `$mod+-` | Split vertical | Stacked |
| `$mod+f` | Fullscreen toggle | |
| `$mod+Shift+Space` | Floating toggle | |
| `$mod+t` | Toggle split/tabbed | |
| `$mod+Shift+s` | Sticky toggle | All workspaces |
| `$mod+`` ` | Scratchpad show | |
| `$mod+Shift+`` ` | Move to scratchpad | |

### Apps

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+Return` | Terminal | foot + fresh tmux |
| `$mod+Space` | Launcher | fuzzel |
| `$mod+b` | Browser | Zen |
| `$mod+n` | Notes | Obsidian |
| `$mod+e` | File manager | yazi in foot |
| `$mod+p` | Password manager | pass (fuzzel picker) |
| `$mod+v` | Editor | nvim in foot |
| `$mod+c` | Chat | nheko (Matrix) |
| `$mod+x` | Power menu | lock/suspend/logout/etc |

### System

| Key | Action | Notes |
|-----|--------|-------|
| `$mod+q` | Close window | |
| `$mod+Shift+c` | Reload config | |
| `$mod+Shift+e` | Exit Sway | |
| `$mod+Escape` | Lock screen | swaylock |
| `$mod+?` | Keybindings help | This reference |
| `$mod+i` | Status panel | Time, battery, network |
| `$mod+Shift+i` | System monitor | btop |
| `$mod+w` | WiFi menu | |
| `$mod+Shift+b` | Bluetooth manager | |
| `$mod+u` | Eject USB | All devices |
| `$mod+Shift+v` | Clipboard history | |
| `$mod+Ctrl+p` | Passthrough mode | For VNC |

### Screenshots

| Key | Action | Notes |
|-----|--------|-------|
| `Print` | Selection to clipboard | Default |
| `$mod+Print` | Full screen to file | ~/Pictures/ |
| `$mod+Shift+Print` | Selection to file | ~/Pictures/ |

### Media Keys

| Key | Action | Notes |
|-----|--------|-------|
| `XF86MonBrightnessUp` | Brightness +10% | |
| `XF86MonBrightnessDown` | Brightness -10% | |
| `XF86AudioRaiseVolume` | Volume +5% | |
| `XF86AudioLowerVolume` | Volume -5% | |
| `XF86AudioMute` | Mute toggle | |
| `XF86AudioMicMute` | Mic mute toggle | |
| `XF86AudioPlay` | Play/pause | |
| `XF86AudioNext` | Next track | |
| `XF86AudioPrev` | Previous track | |

## Design Notes

### Why Alt for tmux?

- `Alt` is the most accessible modifier
- tmux is the primary window manager (more pane switching than Sway window switching)
- Prefix-less bindings are faster

### Why Super for Sway?

- Standard for window managers
- Less frequent than tmux operations
- Clearly separates "desktop" from "terminal" operations

### Why Space as leader in nvim?

- Easy to reach
- Standard in modern configs (LazyVim, etc.)
- Groups related commands logically

### smart-splits Integration

The same `Alt+hjkl` keys work seamlessly across:
1. nvim splits (within a single nvim instance)
2. tmux panes (between nvim and shell, or between shells)

This is achieved by:
- smart-splits.nvim detecting when at nvim edge
- tmux config checking `@pane-is-vim` user option
- Both passing keys to the other when at boundaries

## Files

| File | Keybindings For |
|------|-----------------|
| `stow/tmux/.tmux.conf` | tmux |
| `stow/sway/.config/sway/config` | Sway |
| `stow/nvim/.config/nvim/lua/config/keymaps.lua` | nvim general |
| `stow/nvim/.config/nvim/lua/plugins/*.lua` | nvim plugin-specific |

See also: [[tmux]], [[sway]], [[nvim]]
