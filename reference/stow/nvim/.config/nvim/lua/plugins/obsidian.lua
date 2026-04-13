return {
  {
    "epwalsh/obsidian.nvim",
    version = "*",
    lazy = true,
    -- Load for files in any vault
    event = {
      "BufReadPre " .. vim.fn.expand("~") .. "/vault/**.md",
      "BufNewFile " .. vim.fn.expand("~") .. "/vault/**.md",
      "BufReadPre " .. vim.fn.expand("~") .. "/linux-config/docs/**.md",
      "BufNewFile " .. vim.fn.expand("~") .. "/linux-config/docs/**.md",
    },
    dependencies = {
      "nvim-lua/plenary.nvim",
    },
    keys = {
      { "<leader>on", "<cmd>ObsidianNew<cr>", desc = "New note" },
      { "<leader>oo", "<cmd>ObsidianOpen<cr>", desc = "Open in Obsidian" },
      { "<leader>ot", "<cmd>ObsidianToday<cr>", desc = "Today's note" },
      { "<leader>oy", "<cmd>ObsidianYesterday<cr>", desc = "Yesterday's note" },
      { "<leader>os", "<cmd>ObsidianSearch<cr>", desc = "Search notes" },
      { "<leader>oq", "<cmd>ObsidianQuickSwitch<cr>", desc = "Quick switch" },
      { "<leader>ob", "<cmd>ObsidianBacklinks<cr>", desc = "Backlinks" },
      { "<leader>ol", "<cmd>ObsidianLinks<cr>", desc = "Links in note" },
      { "<leader>of", "<cmd>ObsidianFollowLink<cr>", desc = "Follow link" },
      { "<leader>ow", "<cmd>ObsidianWorkspace<cr>", desc = "Switch workspace" },
    },
    opts = {
      workspaces = {
        {
          name = "vault",
          path = "~/vault",
        },
        {
          name = "docs",
          path = "~/linux-config/docs",
        },
      },

      daily_notes = {
        folder = "daily",
        date_format = "%Y-%m-%d",
        template = nil,
      },

      completion = {
        nvim_cmp = true,
        min_chars = 2,
      },

      mappings = {
        -- Follow link with gf
        ["gf"] = {
          action = function()
            return require("obsidian").util.gf_passthrough()
          end,
          opts = { noremap = false, expr = true, buffer = true },
        },
        -- Toggle checkbox with <leader>ch
        ["<leader>ch"] = {
          action = function()
            return require("obsidian").util.toggle_checkbox()
          end,
          opts = { buffer = true },
        },
      },

      -- Don't manage frontmatter automatically
      disable_frontmatter = true,

      -- Disable UI features (checkboxes, links rendering) to avoid conceallevel warnings
      ui = { enable = false },

      -- Wiki-style links [[note]]
      wiki_link_func = function(opts)
        return string.format("[[%s]]", opts.path)
      end,

      -- Note naming
      note_id_func = function(title)
        if title ~= nil then
          return title:gsub(" ", "-"):gsub("[^A-Za-z0-9-]", ""):lower()
        else
          return tostring(os.time())
        end
      end,
    },
  },
}
