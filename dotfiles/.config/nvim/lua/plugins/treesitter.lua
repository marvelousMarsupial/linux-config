return {
  {
    "nvim-treesitter/nvim-treesitter",
    lazy = false,
    config = function()
      require("nvim-treesitter").setup({})
      -- Parsers are installed explicitly by scripts/setup-editor.
      vim.api.nvim_create_autocmd("FileType", {
        group = vim.api.nvim_create_augroup("EditorHighlight", { clear = true }),
        callback = function(ev)
          local language = vim.treesitter.language.get_lang(vim.bo[ev.buf].filetype)
          if not language then return end
          local ok, loaded = pcall(vim.treesitter.language.add, language)
          if ok and loaded then
            vim.treesitter.start(ev.buf, language)
          end
        end,
      })
    end,
  },
}
