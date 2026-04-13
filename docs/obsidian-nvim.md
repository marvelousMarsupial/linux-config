# Obsidian in Nvim

Using obsidian.nvim for markdown note-taking with wiki-style links.

## Overview

obsidian.nvim provides:
- Wiki-style `[[links]]` between notes
- Daily notes with template support
- Full-text search across vaults
- Backlink tracking
- Checkbox toggling
- Completion for note links

## Vaults

Two configured vaults:

| Name | Path | Purpose | Git Tracked |
|------|------|---------|-------------|
| docs | `~/linux-config/docs` | System documentation | Yes |
| vault | `~/vault` | Personal notes | No |

Switch between vaults: `<leader>ow` (ObsidianWorkspace)

### Vault Structure

```
vault/
├── index.md              # Entry point
├── daily/                # Daily notes (YYYY-MM-DD.md)
├── forge.md              # CLI tool docs
├── roles.md              # Ansible roles
├── keybindings.md        # All keybindings
├── nvim.md               # Neovim config
├── obsidian-nvim.md      # This file
├── theme.md              # Color scheme system
├── dev-containers.md     # Podman dev environments
└── ...
```

## Keybindings

All under `<leader>o` prefix:

| Key | Command | Action |
|-----|---------|--------|
| `<leader>on` | ObsidianNew | Create new note |
| `<leader>oo` | ObsidianOpen | Open in Obsidian app |
| `<leader>ot` | ObsidianToday | Today's daily note |
| `<leader>oy` | ObsidianYesterday | Yesterday's note |
| `<leader>os` | ObsidianSearch | Full-text search |
| `<leader>oq` | ObsidianQuickSwitch | Note picker |
| `<leader>ob` | ObsidianBacklinks | Notes linking here |
| `<leader>ol` | ObsidianLinks | Links in this note |
| `<leader>of` | ObsidianFollowLink | Follow link at cursor |
| `<leader>ow` | ObsidianWorkspace | Switch vault |

### Buffer-Local Mappings

Only active in markdown files within vaults:

| Key | Action |
|-----|--------|
| `gf` | Follow wiki link under cursor |
| `<leader>ch` | Toggle checkbox `[ ]` ↔ `[x]` |

## Wiki Links

Link between notes using `[[note-name]]` syntax.

### Link Formats

| Syntax | Result |
|--------|--------|
| `[[note]]` | Link to note.md |
| `[[folder/note]]` | Link to nested note |
| `[[note#heading]]` | Link to specific heading |
| `[[note\|display text]]` | Custom display text |

### Following Links

- `gf` - Native vim binding, works on wiki links
- `<leader>of` - ObsidianFollowLink command
- Click with mouse (if enabled)

If the linked note doesn't exist, it will be created.

### Creating Links

In insert mode, type `[[` and start typing. Completion will suggest existing notes.

Alternatively:
1. Select text in visual mode
2. Type `[[` to wrap selection in link

## Daily Notes

### Quick Access

| Key | Action |
|-----|--------|
| `<leader>ot` | Open today's note |
| `<leader>oy` | Open yesterday's note |

### Format

Daily notes are stored in `daily/` folder with format `YYYY-MM-DD.md`:

```
vault/daily/
├── 2024-01-01.md
├── 2024-01-02.md
└── ...
```

### Configuration

```lua
daily_notes = {
  folder = "daily",
  date_format = "%Y-%m-%d",
  template = nil,  -- No template, just empty file
},
```

## Search

### Full-Text Search

`<leader>os` (ObsidianSearch) - Uses Telescope to search all note content.

Tips:
- Regex supported
- Results show context around matches
- Enter to open, Esc to cancel

### Quick Switch

`<leader>oq` (ObsidianQuickSwitch) - Fuzzy find by note title.

Faster than full-text when you know the note name.

## Backlinks

`<leader>ob` (ObsidianBacklinks) - Shows all notes that link to the current note.

Useful for:
- Understanding note relationships
- Finding related content
- Discovering orphaned notes

## Completion

In insert mode, obsidian.nvim adds completion source for note links:

1. Type `[[`
2. Start typing note name
3. nvim-cmp shows matching notes
4. Select to insert link

Configuration:
```lua
completion = {
  nvim_cmp = true,
  min_chars = 2,
},
```

## Checkboxes

Toggle checkboxes with `<leader>ch`:

- `- [ ] Todo item` → `- [x] Done item`
- `- [x] Done item` → `- [ ] Todo item`

Works on the current line or visual selection.

## Configuration Details

### Frontmatter

Frontmatter is **disabled** (`disable_frontmatter = true`).

Notes are plain markdown without YAML headers. This keeps files clean and compatible with other tools.

### Note ID Function

When creating a new note, the filename is derived from the title:

```lua
note_id_func = function(title)
  if title ~= nil then
    -- "My Note Title" → "my-note-title"
    return title:gsub(" ", "-"):gsub("[^A-Za-z0-9-]", ""):lower()
  else
    return tostring(os.time())  -- Fallback to timestamp
  end
end,
```

### Wiki Link Function

Links use simple wiki format:

```lua
wiki_link_func = function(opts)
  return string.format("[[%s]]", opts.path)
end,
```

## Lazy Loading

obsidian.nvim only loads when opening markdown files in vault paths:

```lua
event = {
  "BufReadPre ~/vault/**.md",
  "BufNewFile ~/vault/**.md",
  "BufReadPre ~/linux-config/docs/**.md",
  "BufNewFile ~/linux-config/docs/**.md",
},
```

This keeps nvim startup fast when not working with notes.

## Integration with Obsidian App

`<leader>oo` (ObsidianOpen) opens the current note in the Obsidian desktop app.

Useful for:
- Graph view
- Canvas
- Mobile sync
- Plugins not available in nvim

The desktop app and nvim can edit the same files - they're just markdown.

## Markdown Rendering

Two additional plugins enhance markdown viewing:

### render-markdown.nvim

In-buffer rendering: `<leader>mr`

- Renders headings with icons
- Pretty bullets and checkboxes
- Code block styling
- Quote blocks
- Tables

### markdown-preview.nvim

Browser preview: `<leader>mp`

- Live updating preview
- Full markdown rendering
- Stays open until manually closed

## Tips

### Quick Note Workflow

1. `<leader>ot` - Start with today's note
2. Jot down thoughts, todos, links to other notes
3. `<leader>os` to find related content
4. `<leader>ob` to see what links here

### Documentation Workflow

1. `<leader>oq` to quickly switch between docs
2. Use `[[links]]` to connect related topics
3. `<leader>ff` to find by filename
4. `<leader>fg` to grep across all files

### Vault Organization

Keep it flat:
- Fewer folders = easier to find things
- Use links instead of hierarchy
- Daily notes in their own folder is fine

Use prefixes for grouping:
- `dev-containers.md`, `dev-services.md`
- `nvim.md`, `nvim-plugins.md`

## Troubleshooting

### Links Not Working

1. Check you're in a vault path
2. Verify obsidian.nvim loaded: `:Lazy check obsidian.nvim`
3. Try `:ObsidianFollowLink` directly

### Completion Not Showing

1. Ensure nvim-cmp is working
2. Check completion source: `:lua print(vim.inspect(require('cmp').get_config().sources))`
3. Type at least 2 characters after `[[`

### Daily Notes Wrong Folder

Check workspace is correct: `:ObsidianWorkspace`

Daily notes go to `<vault>/daily/` for whichever vault is active.

### Search Not Finding Notes

1. Verify vault path in config
2. Check file has `.md` extension
3. Try `:ObsidianSearch` with simple term

## Files

| Path | Purpose |
|------|---------|
| `stow/nvim/.config/nvim/lua/plugins/obsidian.lua` | Plugin config |
| `~/linux-config/docs/` | Documentation vault |
| `~/vault/` | Personal notes vault |

See also: [[nvim]], [[keybindings]], [[index]]
