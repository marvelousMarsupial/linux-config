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
      { "<leader>y", "<cmd>Yazi<cr>", desc = "Browse file folder" },
      { "<leader>Y", "<cmd>Yazi cwd<cr>", desc = "Browse working folder" },
      { "<leader>-", "<cmd>Yazi toggle<cr>", desc = "Resume file browser" },
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
