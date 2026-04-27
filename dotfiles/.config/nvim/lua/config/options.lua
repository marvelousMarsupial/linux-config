local opt = vim.opt

-- Suppress noisy LSP/Mason startup warnings
local original_notify = vim.notify
vim.notify = function(msg, level, opts)
  -- Filter out common startup noise
  if type(msg) == "string" then
    if msg:match("mason") or msg:match("lspconfig") or msg:match("not found") or msg:match("spawn") then
      return
    end
  end
  original_notify(msg, level, opts)
end

-- Line numbers
opt.number = true
opt.relativenumber = true

-- Tabs & indentation
opt.tabstop = 4
opt.shiftwidth = 4
opt.expandtab = true
opt.autoindent = true
opt.smartindent = true

-- Search
opt.ignorecase = true
opt.smartcase = true
opt.hlsearch = true
opt.incsearch = true

-- Appearance
opt.termguicolors = true
opt.signcolumn = "yes"
opt.scrolloff = 8
opt.sidescrolloff = 8
opt.cursorline = false

-- Behavior
opt.splitright = true
opt.splitbelow = true
opt.wrap = false
opt.mouse = "a"

-- Files
opt.undofile = true
opt.swapfile = false
opt.backup = false

-- Clipboard
opt.clipboard = "unnamedplus"

-- Faster updates
opt.updatetime = 250
opt.timeoutlen = 300

-- Listen on a socket for remote commands (used by theme switcher)
-- Each instance gets a unique socket based on PID
local xdg_runtime = vim.fn.expand("$XDG_RUNTIME_DIR")
if xdg_runtime ~= "" and vim.fn.isdirectory(xdg_runtime) == 1 then
  local socket_path = xdg_runtime .. "/nvim." .. vim.fn.getpid() .. ".sock"
  vim.fn.serverstart(socket_path)
end

-- Disable default mode indicator (lualine shows it)
opt.showmode = false

-- Hide command line when not in use
opt.cmdheight = 0

-- Hide tabline
opt.showtabline = 0

-- Soft-wrap note buffers.
--
-- The global default is `wrap = false` (set above) because we want
-- code/config files to scroll horizontally — wrapping a Lua function
-- or a long shell pipeline mid-line is harder to read than scrolling.
-- Notes are different: prose paragraphs in markdown should reflow to
-- the window width so the user isn't forced to either insert hard
-- newlines (which `wtype` dictation already wants to avoid — see the
-- `dictate` script) or scroll horizontally through every sentence.
--
-- Scope: the same paths obsidian.nvim picks up (~/vault/*.md and
-- ~/linux-config/docs/*.md). That matches "note mode" exactly and
-- intentionally leaves arbitrary markdown elsewhere (e.g. a project
-- README opened from another repo) on the global no-wrap default.
--
-- Three settings, all load-bearing together:
--   * `wrap = true`         — turn soft wrapping on for this buffer.
--   * `linebreak = true`    — break at `breakat` characters (spaces,
--                             punctuation) instead of mid-word, so a
--                             wrapped paragraph reads naturally.
--   * `breakindent = true`  — continuation lines inherit the leading
--                             indent of the wrapped line, which keeps
--                             bullet lists and quoted blocks visually
--                             aligned across the wrap.
vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = {
    vim.fn.expand("~") .. "/vault/*.md",
    vim.fn.expand("~") .. "/linux-config/docs/*.md",
  },
  callback = function()
    vim.opt_local.wrap = true
    vim.opt_local.linebreak = true
    vim.opt_local.breakindent = true
  end,
})

-- Auto-rename vault notes based on # Title (like Obsidian)
vim.api.nvim_create_autocmd("BufWritePost", {
  pattern = vim.fn.expand("~") .. "/vault/*.md",
  callback = function()
    local filepath = vim.fn.expand("%:p")
    local filename = vim.fn.expand("%:t:r")

    -- Only rename timestamp-named files (YYYYMMDD-HHMMSS)
    if not filename:match("^%d%d%d%d%d%d%d%d%-%d%d%d%d%d%d$") then
      return
    end

    -- Get title from first line
    local first_line = vim.fn.getline(1)
    local title = first_line:match("^#%s*(.+)$")

    if not title or title == "" then
      return
    end

    -- Sanitize title for filename
    local new_name = title:gsub("[^%w%s%-_]", ""):gsub("%s+", "-"):lower()
    if new_name == "" then
      return
    end

    local dir = vim.fn.expand("%:p:h")
    local new_path = dir .. "/" .. new_name .. ".md"

    -- Don't rename if file already exists
    if vim.fn.filereadable(new_path) == 1 then
      return
    end

    -- Rename file and reopen
    os.rename(filepath, new_path)
    vim.cmd("edit " .. new_path)
    vim.cmd("bdelete #")
  end,
})

-- Auto-delete empty vault notes on exit
vim.api.nvim_create_autocmd("VimLeavePre", {
  callback = function()
    local vault = vim.fn.expand("~") .. "/vault"
    local files = vim.fn.glob(vault .. "/*.md", false, true)

    for _, filepath in ipairs(files) do
      local filename = vim.fn.fnamemodify(filepath, ":t:r")

      -- Only cleanup timestamp-named files
      if filename:match("^%d%d%d%d%d%d%d%d%-%d%d%d%d%d%d$") then
        local lines = vim.fn.readfile(filepath)
        local content = table.concat(lines, "")
        content = content:gsub("%s+", "")

        if content == "" or content == "#" then
          os.remove(filepath)
        end
      end
    end
  end,
})
