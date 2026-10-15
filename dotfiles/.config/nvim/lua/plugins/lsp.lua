return {
  {
    "mason-org/mason.nvim",
    name = "mason.nvim",
    cmd = { "Mason", "MasonInstall", "MasonUpdate" },
    opts = {},
  },
  {
    "neovim/nvim-lspconfig",
    event = { "BufReadPre", "BufNewFile" },
    dependencies = { "mason.nvim", "hrsh7th/cmp-nvim-lsp" },
    config = function()
      local capabilities = require("cmp_nvim_lsp").default_capabilities()
      for server in pairs(require("config.tools").servers) do
        vim.lsp.config(server, { capabilities = capabilities })
      end
      vim.lsp.config("clangd", {
        cmd = { "clangd", "--background-index", "--clang-tidy", "--completion-style=detailed" },
      })
      vim.lsp.config("lua_ls", {
        settings = {
          Lua = {
            runtime = { version = "LuaJIT" },
            workspace = { checkThirdParty = false, library = { vim.env.VIMRUNTIME } },
            diagnostics = { globals = { "vim" } },
            telemetry = { enable = false },
          },
        },
      })
      vim.lsp.config("jsonls", { settings = { json = { validate = { enable = true } } } })
      -- pyright owns hover and completion; ruff lints and formats.
      vim.lsp.config("ruff", { on_attach = function(client) client.server_capabilities.hoverProvider = false end })
      vim.lsp.config("yamlls", {
        settings = { yaml = { validate = true, keyOrdering = false, format = { enable = false } } },
      })
      -- Missing tools remain visible in :EditorTools; no failing processes on startup.
      for server, command in pairs(require("config.tools").servers) do
        if vim.fn.executable(command) == 1 then
          vim.lsp.enable(server)
        end
      end
      vim.api.nvim_create_autocmd("LspAttach", {
        group = vim.api.nvim_create_augroup("EditorLsp", { clear = true }),
        callback = function(ev)
          local function map(keys, action, description)
            vim.keymap.set("n", keys, action, { buffer = ev.buf, desc = description })
          end
          map("gd", vim.lsp.buf.definition, "Go to definition")
          map("gD", vim.lsp.buf.declaration, "Go to declaration")
          map("gr", vim.lsp.buf.references, "References")
          map("gi", vim.lsp.buf.implementation, "Implementation")
          map("gt", vim.lsp.buf.type_definition, "Type definition")
          map("K", function() vim.lsp.buf.hover({ border = "rounded" }) end, "Documentation")
          map("<leader>k", vim.lsp.buf.signature_help, "Signature help")
          map("<leader>rn", vim.lsp.buf.rename, "Rename symbol")
          map("<leader>ca", vim.lsp.buf.code_action, "Code action")
        end,
      })
      vim.diagnostic.config({
        virtual_text = { spacing = 2, prefix = "●", severity = { min = vim.diagnostic.severity.WARN } },
        signs = true,
        underline = true,
        update_in_insert = false,
        severity_sort = true,
        float = { border = "rounded", source = true },
      })
    end,
  },
}
