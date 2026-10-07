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
  assert(not vim.o.modeline and not vim.o.exrc, "File/project command loading must stay off")
  local hostile = root .. "/modeline.txt"
  local tabstop = vim.bo.tabstop
  vim.fn.writefile({ "text", "vim: set ts=13:" }, hostile)
  vim.cmd.edit(vim.fn.fnameescape(hostile))
  assert(vim.bo.tabstop == tabstop, "Untrusted modeline changed an editor option")
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
    assert(vim.fn.readfile(file)[1] == sample[4], sample[1] .. ": save unexpectedly rewrote content")
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
  -- Run Conform against a deterministic external formatter without installing tools.
  local conform = require("conform")
  conform.formatters.fixture_json = {
    command = "/usr/bin/python3",
    args = { "-c", "import json,sys; print(json.dumps(json.load(sys.stdin), indent=2))" },
    stdin = true,
  }
  conform.formatters_by_ft.json = { "fixture_json" }
  vim.cmd.edit(vim.fn.fnameescape(root .. "/sample.json"))
  conform.format({ async = false, timeout_ms = 2000, lsp_format = "never" })
  assert(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), "\n") == '{\n  "value": 1\n}',
    "External formatter did not update the buffer")
  vim.cmd.write()
  -- Exercise the configured LspAttach callback with a real stdio client/server.
  local file = root .. "/protocol.c"
  vim.fn.writefile({ "int value;" }, file)
  vim.cmd.edit(vim.fn.fnameescape(file))
  local client_id = assert(vim.lsp.start({
    name = "config-test-fixture",
    cmd = { "/usr/bin/python3", assert(vim.env.LINUX_CONFIG_LSP_FIXTURE) },
    root_dir = root,
  }))
  assert(vim.wait(5000, function()
    return #vim.diagnostic.get(0) > 0
  end, 20), "LSP fixture failed to publish diagnostics")
  assert(vim.diagnostic.get(0)[1].message == "fixture diagnostic")
  for _, key in ipairs({ "gd", "gD", "gr", "gi", "gt", "K", "<leader>rn", "<leader>ca" }) do
    assert(vim.fn.maparg(key, "n", false, true).buffer == 1, key .. ": LSP mapping missing")
  end
  local client = assert(vim.lsp.get_client_by_id(client_id))
  local params = vim.lsp.util.make_position_params(0, client.offset_encoding)
  local hover = assert(client:request_sync("textDocument/hover", params, 2000, 0))
  assert(not hover.err and hover.result.contents.value == "fixture documentation")
  local definition = assert(client:request_sync("textDocument/definition", params, 2000, 0))
  assert(not definition.err and definition.result.uri == vim.uri_from_fname(file))
  client:stop()
  assert(vim.wait(2000, function() return client:is_stopped() end, 20), "LSP fixture did not stop")
  assert(#captured == 0, table.concat(captured, "\n"))
  print("Neovim: eight filetypes, prose wrapping, saves, plugins, LSP configs, and formatter mappings passed")
end
local ok, err = pcall(check)
if not ok then
  vim.api.nvim_err_writeln(tostring(err))
  vim.cmd("cquit 1")
end
