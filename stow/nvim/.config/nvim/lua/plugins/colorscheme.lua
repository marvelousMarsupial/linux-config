return {
  {
    "tinted-theming/tinted-vim",
    lazy = false,
    priority = 1000,
    config = function()
      vim.cmd("colorscheme base16-catppuccin-mocha")
    end,
  },
}
