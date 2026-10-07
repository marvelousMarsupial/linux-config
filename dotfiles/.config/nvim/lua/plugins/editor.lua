return {
  -- Auto-pair brackets and quotes
  {
    "windwp/nvim-autopairs",
    event = "InsertEnter",
    opts = {
      check_ts = true, -- use treesitter to check for pairs
    },
  },

  -- Yazi file manager integration
  {
    "mikavilpas/yazi.nvim",
    event = "VeryLazy",
    keys = {
      { "<leader>y", "<cmd>Yazi<cr>", desc = "Open yazi (current file)" },
      { "<leader>Y", "<cmd>Yazi cwd<cr>", desc = "Open yazi (cwd)" },
      { "<leader>-", "<cmd>Yazi toggle<cr>", desc = "Resume yazi session" },
    },
    opts = {
      open_for_directories = true,
      keymaps = {
        show_help = "<f1>",
      },
    },
  },

  -- Which-key: shows available keybindings
  {
    "folke/which-key.nvim",
    event = "VeryLazy",
    opts = {
      delay = 300,
      icons = {
        mappings = false,
      },
      spec = {
        { "<leader>f", group = "find" },
        { "<leader>c", group = "code" },
        { "<leader>b", group = "buffer" },
        { "<leader>d", group = "diagnostics" },
      },
    },
  },
}
