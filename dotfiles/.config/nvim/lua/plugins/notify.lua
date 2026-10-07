return {
  "rcarriga/nvim-notify",
  event = "VeryLazy",
  config = function()
    local notify = require("notify")
    notify.setup({
      render = "compact",
      stages = "static",
      timeout = 3000,
      background_colour = function()
        local normal = vim.api.nvim_get_hl(0, { name = "Normal", link = false })
        return normal.bg and string.format("#%06x", normal.bg) or "#222436"
      end,
    })
    vim.notify = notify
  end,
}
