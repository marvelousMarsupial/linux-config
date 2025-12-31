local map = vim.keymap.set

-- jj to escape insert mode
map("i", "jj", "<ESC>", { desc = "Exit insert mode" })

-- Quick actions
map("n", "<leader><leader>c", ":%y+<CR>", { desc = "Copy file" })
map("n", "<leader><leader>w", "<cmd>w<CR>", { desc = "Save" })
map("n", "<leader><leader>d", ":%d<CR>", { desc = "Delete content" })
map("n", "<leader><leader>q", "<cmd>x<CR>", { desc = "Save and quit" })
