return {
  "mrjones2014/smart-splits.nvim",
  lazy = false,
  config = function()
    local ss = require("smart-splits")

    ss.setup({
      -- Ignore these filetypes/buftypes when determining if at edge
      ignored_filetypes = { "nofile", "quickfix", "prompt" },
      ignored_buftypes = { "NvimTree" },
      -- Resize amount
      default_amount = 3,
      -- Stop at the outermost editor split
      at_edge = "stop",
      -- Multiplexer integration
      multiplexer_integration = false,
    })

    -- Navigation: Alt + hjkl
    vim.keymap.set("n", "<M-h>", ss.move_cursor_left, { desc = "Move to left split" })
    vim.keymap.set("n", "<M-j>", ss.move_cursor_down, { desc = "Move to below split" })
    vim.keymap.set("n", "<M-k>", ss.move_cursor_up, { desc = "Move to above split" })
    vim.keymap.set("n", "<M-l>", ss.move_cursor_right, { desc = "Move to right split" })

    -- Resize: Alt + Shift + hjkl
    vim.keymap.set("n", "<M-H>", ss.resize_left, { desc = "Resize left" })
    vim.keymap.set("n", "<M-J>", ss.resize_down, { desc = "Resize down" })
    vim.keymap.set("n", "<M-K>", ss.resize_up, { desc = "Resize up" })
    vim.keymap.set("n", "<M-L>", ss.resize_right, { desc = "Resize right" })
  end,
}
