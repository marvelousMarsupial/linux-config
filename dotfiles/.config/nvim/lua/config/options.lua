local opt = vim.opt

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
opt.winborder = "rounded"
opt.pumheight = 10
opt.pumblend = 0
opt.winblend = 0
opt.fillchars:append({ eob = " " })

-- Behavior
opt.splitright = true
opt.splitbelow = true
opt.wrap = false
opt.mouse = "a"

-- Files
-- Opening a file must not load project-local editor commands or modelines.
opt.exrc = false
opt.modeline = false
opt.undofile = true
opt.swapfile = false
opt.backup = false

-- Clipboard
opt.clipboard = "unnamedplus"

-- Faster updates
opt.updatetime = 250
opt.timeoutlen = 300

-- Disable default mode indicator (lualine shows it)
opt.showmode = false

-- Hide command line when not in use
opt.cmdheight = 1

-- Hide tabline
opt.showtabline = 0

-- Prose wraps at word boundaries; code keeps horizontal scrolling.
vim.api.nvim_create_autocmd("FileType", {
  group = vim.api.nvim_create_augroup("ProseWrap", { clear = true }),
  pattern = { "markdown", "text" },
  callback = function()
    vim.opt_local.wrap = true
    vim.opt_local.linebreak = true
    vim.opt_local.breakindent = true
  end,
})

vim.api.nvim_create_user_command("EditorTools", function()
  local lines = {}
  for server, executable in pairs(require("config.tools").servers) do
    table.insert(lines, server .. ": " .. (vim.fn.exepath(executable) ~= "" and vim.fn.exepath(executable) or "Not installed"))
  end
  for _, executable in ipairs({ "clang-format", "prettier", "stylua", "tree-sitter" }) do
    table.insert(lines, executable .. ": " .. (vim.fn.exepath(executable) ~= "" and vim.fn.exepath(executable) or "Not installed"))
  end
  table.sort(lines)
  vim.notify(table.concat(lines, "\n"), vim.log.levels.INFO, { title = "Editor tools" })
end, {})
