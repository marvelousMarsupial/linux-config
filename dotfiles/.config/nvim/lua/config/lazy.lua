-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.uv.fs_stat(lazypath) then
  local lockfile = vim.fn.stdpath("config") .. "/lazy-lock.json"
  local lock = vim.json.decode(table.concat(vim.fn.readfile(lockfile), "\n"))
  local commit = lock["lazy.nvim"] and lock["lazy.nvim"].commit
  if not commit or not commit:match("^[0-9a-f]+$") or #commit ~= 40 then
    error("Set a lazy.nvim commit in lazy-lock.json")
  end
  vim.fn.system({
    "git",
    "clone",
    "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--no-checkout",
    lazypath,
  })
  if vim.v.shell_error ~= 0 then
    error("Could not download lazy.nvim\nCheck the network and run setup-editor again")
  end
  vim.fn.system({ "git", "-C", lazypath, "checkout", "--detach", commit })
  if vim.v.shell_error ~= 0 then
    error("Could not load the pinned lazy.nvim commit\nRemove the incomplete clone and try again")
  end
end
vim.opt.rtp:prepend(lazypath)

-- Setup lazy.nvim
require("lazy").setup({
  spec = {
    { import = "plugins" },
  },
  defaults = {
    lazy = false,
  },
  install = {
    colorscheme = { "habamax" },
  },
  checker = {
    enabled = false, -- Update explicitly with :Lazy update
  },
  change_detection = {
    enabled = true,
    notify = false,
  },
  performance = {
    rtp = {
      disabled_plugins = {
        "gzip",
        "tarPlugin",
        "tohtml",
        "tutor",
        "zipPlugin",
      },
    },
  },
})
