local map = vim.keymap.set
local del = vim.keymap.del

-- Remove LazyVim defaults (we use vim-tmux-navigator)
pcall(del, "n", "<C-h>")
pcall(del, "n", "<C-j>")
pcall(del, "n", "<C-k>")
pcall(del, "n", "<C-l>")

-- jj to escape insert mode
map("i", "jj", "<ESC>", { desc = "Exit insert mode" })

-- Quick actions
map("n", "<leader><leader>c", ":%y+<CR>", { desc = "Copy file" })
map("n", "<leader><leader>w", "<cmd>w<CR>", { desc = "Save" })
map("n", "<leader><leader>d", ":%d<CR>", { desc = "Delete content" })
map("n", "<leader><leader>q", "<cmd>x<CR>", { desc = "Save and quit" })
