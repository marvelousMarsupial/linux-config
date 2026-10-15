-- Set leader key before lazy.nvim loads
vim.g.mapleader = " "
vim.g.maplocalleader = "\\"

-- Load configuration
require("config.options")
require("config.ligatures")
require("config.keymaps")
require("config.lazy")
