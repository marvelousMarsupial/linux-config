-- Editor dependencies installed explicitly by scripts/setup-editor.
return {
  servers = {
    clangd = "clangd",
    ts_ls = "typescript-language-server",
    html = "vscode-html-language-server",
    jsonls = "vscode-json-language-server",
    yamlls = "yaml-language-server",
    lua_ls = "lua-language-server",
    pyright = "pyright-langserver",
    ruff = "ruff",
    bashls = "bash-language-server",
    rust_analyzer = "rust-analyzer", -- from rustup, not Mason
  },
  mason = {
    "typescript-language-server", "html-lsp", "json-lsp", "yaml-language-server",
    "lua-language-server", "prettier", "stylua", "tree-sitter-cli",
    "pyright", "ruff", "bash-language-server", "shfmt",
  },
  parsers = {
    "c", "cpp", "typescript", "tsx", "javascript", "html", "css", "json",
    "yaml", "lua", "python", "markdown", "markdown_inline", "bash", "vim", "vimdoc", "query", "rust", "toml",
  },
}
