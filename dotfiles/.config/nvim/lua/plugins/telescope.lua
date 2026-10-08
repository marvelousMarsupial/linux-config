return {
  {
    "nvim-telescope/telescope.nvim",
    dependencies = {
      "nvim-lua/plenary.nvim",
    },
    cmd = "Telescope",
    keys = {
      { "<leader>ff", "<cmd>Telescope find_files<cr>", desc = "Find files" },
      { "<leader>fg", "<cmd>Telescope live_grep<cr>", desc = "Search text" },
      { "<leader>fb", "<cmd>Telescope buffers<cr>", desc = "Buffers" },
      { "<leader>fr", "<cmd>Telescope oldfiles<cr>", desc = "Recent files" },
      { "<leader>fh", "<cmd>Telescope help_tags<cr>", desc = "Help" },
      { "<leader>fk", "<cmd>Telescope keymaps<cr>", desc = "Shortcuts" },
      { "<leader>fd", "<cmd>Telescope diagnostics<cr>", desc = "Issues" },
      { "<leader>fs", "<cmd>Telescope lsp_document_symbols<cr>", desc = "File symbols" },
      { "<leader>fw", "<cmd>Telescope grep_string<cr>", desc = "Search current word" },
      { "<leader>/", "<cmd>Telescope current_buffer_fuzzy_find<cr>", desc = "Search this file" },
    },
    config = function()
      local telescope = require("telescope")
      local actions = require("telescope.actions")

      telescope.setup({
        defaults = {
          mappings = {
            i = {
              ["<C-j>"] = actions.move_selection_next,
              ["<C-k>"] = actions.move_selection_previous,
              ["<C-q>"] = actions.send_selected_to_qflist + actions.open_qflist,
              ["<Esc>"] = actions.close,
            },
          },
          file_ignore_patterns = {
            "node_modules",
            ".git/",
            "%.o",
            "%.a",
            "%.out",
            "%.class",
            "%.pdf",
            "%.mkv",
            "%.mp4",
            "%.zip",
          },
          layout_config = {
            horizontal = {
              preview_width = 0.55,
            },
          },
        },
        pickers = {
          find_files = {
            hidden = true,
            prompt_title = "Files",
          },
          live_grep = { prompt_title = "Search text" },
          oldfiles = { prompt_title = "Recent files" },
          help_tags = { prompt_title = "Help" },
          keymaps = { prompt_title = "Shortcuts" },
          diagnostics = { prompt_title = "Issues" },
          lsp_document_symbols = { prompt_title = "File symbols" },
          grep_string = { prompt_title = "Matches" },
          current_buffer_fuzzy_find = { prompt_title = "Search this file" },
          buffers = {
            prompt_title = "Open files",
            sort_mru = true,
            ignore_current_buffer = true,
          },
        },
      })

    end,
  },
}
