return {
  -- Mason: Package manager for LSP servers, formatters, linters
  {
    "williamboman/mason.nvim",
    cmd = "Mason",
    opts = {},
  },

  -- Bridge between mason and lspconfig
  {
    "williamboman/mason-lspconfig.nvim",
    dependencies = { "williamboman/mason.nvim" },
    opts = {
      -- LSPs are pre-installed during container creation
      -- Use :Mason to manually install additional servers
      automatic_installation = false,
    },
  },

  -- LSP configuration (nvim 0.11+ API)
  {
    "neovim/nvim-lspconfig",
    event = { "BufReadPre", "BufNewFile" },
    dependencies = {
      "williamboman/mason.nvim",
      "williamboman/mason-lspconfig.nvim",
      "hrsh7th/cmp-nvim-lsp",
    },
    config = function()
      local cmp_nvim_lsp = require("cmp_nvim_lsp")

      -- Capabilities from nvim-cmp for better completion
      local capabilities = cmp_nvim_lsp.default_capabilities()

      -- LSP keybindings on attach
      vim.api.nvim_create_autocmd("LspAttach", {
        group = vim.api.nvim_create_augroup("UserLspConfig", { clear = true }),
        callback = function(ev)
          local bufnr = ev.buf
          local map = function(keys, func, desc)
            vim.keymap.set("n", keys, func, { buffer = bufnr, desc = "LSP: " .. desc })
          end

          -- Navigation
          map("gd", vim.lsp.buf.definition, "Go to definition")
          map("gD", vim.lsp.buf.declaration, "Go to declaration")
          map("gr", vim.lsp.buf.references, "Go to references")
          map("gi", vim.lsp.buf.implementation, "Go to implementation")
          map("gt", vim.lsp.buf.type_definition, "Go to type definition")

          -- Information
          map("K", vim.lsp.buf.hover, "Hover documentation")
          map("<leader>k", vim.lsp.buf.signature_help, "Signature help")

          -- Actions
          map("<leader>rn", vim.lsp.buf.rename, "Rename symbol")
          map("<leader>ca", vim.lsp.buf.code_action, "Code actions")
          -- Format handled by conform.nvim (format.lua)

          -- Diagnostics
          map("[d", vim.diagnostic.goto_prev, "Previous diagnostic")
          map("]d", vim.diagnostic.goto_next, "Next diagnostic")
          map("<leader>e", vim.diagnostic.open_float, "Show diagnostic")
          map("<leader>dl", vim.diagnostic.setloclist, "Diagnostic list")
        end,
      })

      -- Server configurations using vim.lsp.config
      vim.lsp.config("clangd", {
        capabilities = capabilities,
      })

      vim.lsp.config("pyright", {
        capabilities = capabilities,
      })

      vim.lsp.config("ts_ls", {
        capabilities = capabilities,
      })

      vim.lsp.config("lua_ls", {
        capabilities = capabilities,
        settings = {
          Lua = {
            runtime = { version = "LuaJIT" },
            workspace = {
              checkThirdParty = false,
              library = { vim.env.VIMRUNTIME },
            },
            completion = { callSnippet = "Replace" },
            diagnostics = {
              globals = { "vim" },
            },
          },
        },
      })

      vim.lsp.config("vue_ls", {
        capabilities = capabilities,
      })

      vim.lsp.config("tailwindcss", {
        capabilities = capabilities,
      })

      -- Enable all configured servers
      vim.lsp.enable({
        "clangd",
        "pyright",
        "ts_ls",
        "lua_ls",
        "vue_ls",
        "tailwindcss",
      })

      -- Diagnostic display configuration
      vim.diagnostic.config({
        virtual_text = { spacing = 4, prefix = "●" },
        signs = true,
        underline = true,
        update_in_insert = false,
        severity_sort = true,
        float = {
          border = "rounded",
          source = true,
        },
      })
    end,
  },
}
