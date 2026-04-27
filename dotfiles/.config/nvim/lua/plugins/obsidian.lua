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
    -- `opts` is a function (not a table) so we can filter workspace
    -- specs by on-disk existence at plugin-load time. obsidian.nvim
    -- resolves every declared workspace path with `strict = true`
    -- (workspace.lua: `Path.new(path):resolve { strict = true }`),
    -- which throws a noisy error popup as soon as setup runs if a
    -- declared path doesn't exist. We hit this when a workspace
    -- (e.g. ~/linux-config/docs) hadn't been created yet — every
    -- single note open showed a stack trace from new_from_spec →
    -- get_workspace_for_cwd → setup. Filtering here means a missing
    -- directory is just absent from the workspace list (silent
    -- no-op); creating it later transparently re-enables the
    -- workspace on the next nvim start without further config edits.
    opts = function()
      -- Declare workspace specs in one place. Adding a new vault
      -- means appending an entry here — the existence filter below
      -- handles the "directory not yet created" case for free.
      local workspace_specs = {
        { name = "vault", path = "~/vault" },
        { name = "docs", path = "~/linux-config/docs" },
      }

      -- Drop any spec whose expanded path isn't a real directory.
      -- `vim.fn.isdirectory` returns 1 for a directory, 0 for a
      -- non-existent path OR a non-directory (e.g. a regular file
      -- at that path), which is exactly the set we want to exclude
      -- — obsidian needs a directory to scan, not a file.
      local workspaces = {}
      for _, spec in ipairs(workspace_specs) do
        if vim.fn.isdirectory(vim.fn.expand(spec.path)) == 1 then
          table.insert(workspaces, spec)
        end
      end

      return {
      workspaces = workspaces,

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
      }
    end,
  },
}
