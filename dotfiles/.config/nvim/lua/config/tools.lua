-- Editor dependencies installed explicitly by scripts/setup-editor.
return {
  servers = {
    clangd = "clangd",
    ts_ls = "typescript-language-server",
    html = "vscode-html-language-server",
    jsonls = "vscode-json-language-server",
    yamlls = "yaml-language-server",
    lua_ls = "lua-language-server",
  },
  mason = {
    "typescript-language-server", "html-lsp", "json-lsp", "yaml-language-server",
    "lua-language-server", "prettier", "stylua", "tree-sitter-cli",
  },
  parsers = {
    "c", "cpp", "typescript", "tsx", "javascript", "html", "css", "json",
    "yaml", "lua", "markdown", "markdown_inline", "bash", "vim", "vimdoc", "query",
  },
}
