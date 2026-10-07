-- Headless behavioral checks using installed plugins, without downloading tools.
local captured = {}
local original_notify = vim.notify
vim.notify = function(message, level, opts)
  if level == vim.log.levels.ERROR then
    table.insert(captured, tostring(message))
  end
  original_notify(message, level, opts)
end
local function check()
  local root = assert(vim.env.LINUX_CONFIG_TEST_DIR)
  local samples = {
    { "sample.txt", "text", true, "plain text" },
    { "sample.md", "markdown", true, "# A heading" },
    { "sample.c", "c", false, "int main(void) { return 0; }" },
    { "sample.ts", "typescript", false, "const value: number = 1;" },
    { "sample.html", "html", false, "<html><body>test</body></html>" },
    { "sample.json", "json", false, '{"value": 1}' },
    { "sample.yaml", "yaml", false, "value: 1" },
    { "sample.lua", "lua", false, "local value = 1" },
  }
  for _, sample in ipairs(samples) do
    local file = root .. "/" .. sample[1]
    vim.fn.writefile({ sample[4] }, file)
    vim.cmd.edit(vim.fn.fnameescape(file))
    assert(vim.bo.filetype == sample[2], sample[1] .. ": wrong filetype")
    assert(vim.wo.wrap == sample[3], sample[1] .. ": wrong wrapping")
    if sample[3] then
      assert(vim.wo.linebreak and vim.wo.breakindent, sample[1] .. ": incomplete prose wrapping")
    end
    vim.cmd.write()
  end
  require("lazy").load({ plugins = { "nvim-lspconfig", "conform.nvim", "telescope.nvim", "nvim-cmp" } })
  for _, server in ipairs({ "clangd", "ts_ls", "html", "jsonls", "yamlls", "lua_ls" }) do
    assert(vim.lsp.config[server], server .. ": missing config")
  end
  local formatters = require("conform").formatters_by_ft
  for _, filetype in ipairs({ "c", "typescript", "html", "json", "yaml", "lua" }) do
    assert(formatters[filetype], filetype .. ": missing formatter mapping")
  end
  assert(not formatters.markdown and not formatters.text, "Prose should not be reformatted")
  assert(vim.fn.exists(":EditorTools") == 2, "Missing EditorTools")
  assert(vim.o.cmdheight == 1, "Command line must remain visible")
  assert(vim.fn.maparg("j", "n", false, true).expr == 1, "Screen-line movement missing")
  assert(not require("lazy.core.config").plugins["obsidian.nvim"], "Obsidian must be retired")
  for _, event in ipairs({ "BufWritePost", "VimLeavePre" }) do
    for _, autocmd in ipairs(vim.api.nvim_get_autocmds({ event = event })) do
      assert(not (autocmd.pattern or ""):find("vault"), "Unexpected note file automation")
    end
  end
  assert(#captured == 0, table.concat(captured, "\n"))
  print("Neovim: eight filetypes, prose wrapping, saves, plugins, LSP configs, and formatter mappings passed")
end
local ok, err = pcall(check)
if not ok then
  vim.api.nvim_err_writeln(tostring(err))
  vim.cmd("cquit 1")
end
