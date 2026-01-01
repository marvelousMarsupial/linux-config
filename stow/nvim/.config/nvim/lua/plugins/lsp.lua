return {
  {
    "williamboman/mason.nvim",
    opts = {
      ensure_installed = {
        "typescript-language-server",
        "vue-language-server",
        "pyright",
        "bash-language-server",
        "yaml-language-server",
        "lua-language-server",
        "prettier",
        "eslint-lsp",
      },
    },
  },
  {
    "neovim/nvim-lspconfig",
    opts = {
      servers = {
        ts_ls = {},
        volar = {
          filetypes = { "vue", "typescript", "javascript" },
        },
        pyright = {},
        bashls = {},
        yamlls = {},
        lua_ls = {},
      },
    },
  },
  {
    "nvim-treesitter/nvim-treesitter",
    opts = {
      ensure_installed = {
        "vue",
        "typescript",
        "javascript",
        "python",
        "bash",
        "yaml",
        "lua",
        "html",
        "css",
        "json",
        "markdown",
      },
    },
  },
}
