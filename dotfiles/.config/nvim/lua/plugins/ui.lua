return {
  -- Statusline
  {
    "nvim-lualine/lualine.nvim",
    dependencies = { "nvim-tree/nvim-web-devicons" },
    opts = {
      options = {
        theme = "auto",
        component_separators = { left = "", right = "" },
        section_separators = { left = "", right = "" },
        globalstatus = true,
      },
      sections = {
        lualine_a = { "mode" },
        lualine_b = { "branch" },
        lualine_c = { { "filename", path = 1 } },
        lualine_x = { "diagnostics" },
        lualine_y = { "filetype" },
        lualine_z = { "location" },
      },
    },
    config = function(_, opts)
      require("lualine").setup(opts)

      -- Zen mode for vault notes - hide statusline
      vim.api.nvim_create_autocmd("BufEnter", {
        pattern = "*/vault/*.md",
        callback = function()
          vim.opt.laststatus = 0
        end,
      })
      vim.api.nvim_create_autocmd("BufLeave", {
        pattern = "*/vault/*.md",
        callback = function()
          vim.opt.laststatus = 3
        end,
      })
    end,
  },

  -- Dashboard
  {
    "goolord/alpha-nvim",
    cmd = "Alpha",
    event = "VimEnter",
    keys = {
      { "<leader>;", "<cmd>Alpha<CR>", desc = "Dashboard" },
    },
    dependencies = { "nvim-tree/nvim-web-devicons" },
    config = function()
      local alpha = require("alpha")
      local dashboard = require("alpha.themes.dashboard")

      -- Buttons
      dashboard.section.buttons.val = {
        dashboard.button("f", "  Find file", "<cmd>Telescope find_files<cr>"),
        dashboard.button("r", "  Recent files", "<cmd>Telescope oldfiles<cr>"),
        dashboard.button("g", "  Grep text", "<cmd>Telescope live_grep<cr>"),
        dashboard.button("n", "  New file", "<cmd>ene<cr>"),
        dashboard.button("c", "  Config", "<cmd>e ~/.config/nvim/init.lua<cr>"),
        dashboard.button("q", "  Quit", "<cmd>qa<cr>"),
      }

      -- Layout
      dashboard.section.buttons.opts.hl = "AlphaButtons"

      dashboard.config.layout = {
        { type = "padding", val = 8 },
        dashboard.section.buttons,
      }

      alpha.setup(dashboard.config)

      -- Disable statusline on dashboard
      vim.api.nvim_create_autocmd("User", {
        pattern = "AlphaReady",
        callback = function()
          vim.opt.laststatus = 0
          vim.opt.showtabline = 0
        end,
      })

      vim.api.nvim_create_autocmd("BufUnload", {
        buffer = 0,
        callback = function()
          vim.opt.laststatus = 3
          vim.opt.showtabline = 0
        end,
      })
    end,
  },
}
