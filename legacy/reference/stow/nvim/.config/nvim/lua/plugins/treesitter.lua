return {
  {
    "nvim-treesitter/nvim-treesitter",
    build = ":TSUpdate",
    event = { "BufReadPost", "BufNewFile" },
    config = function()
      -- Nvim 0.11+ uses vim.treesitter directly
      -- nvim-treesitter just provides :TSInstall and parser management

      local ok, ts = pcall(require, "nvim-treesitter")
      if not ok then return end

      -- Install parsers
      ts.setup({
        ensure_installed = {
          "c", "python", "typescript", "javascript", "java", "lua",
          "markdown", "markdown_inline", "json", "yaml", "bash",
          "vim", "vimdoc", "query", "vue", "css", "scss", "html", "tsx",
        },
        auto_install = true,
      })

      -- Enable treesitter highlighting (built-in to nvim 0.11)
      vim.api.nvim_create_autocmd("FileType", {
        callback = function(args)
          pcall(vim.treesitter.start, args.buf)
        end,
      })
    end,
  },
}
