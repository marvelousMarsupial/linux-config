return {
  "numToStr/FTerm.nvim",
  config = function()
    local fterm = require("FTerm")
    fterm.setup({
      dimensions = {
        height = 0.33,
        width = 1,
        x = 0,
        y = 1,
      },
      border = "none",
    })
    vim.keymap.set({"n", "t"}, "<M-t>", fterm.toggle, { desc = "Toggle terminal" })
    vim.keymap.set({"n", "t"}, "<C-\\>", fterm.toggle, { desc = "Toggle terminal" })
  end,
}
