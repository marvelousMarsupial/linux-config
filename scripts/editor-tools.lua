-- Called explicitly by setup-editor; never downloads tools during ordinary editing.
local function setup()
  require("lazy").load({ plugins = { "mason.nvim", "nvim-lspconfig", "nvim-treesitter" } })
  local tools = require("config.tools")
  vim.cmd("MasonInstall " .. table.concat(tools.mason, " "))
  local registry = require("mason-registry")
  local missing = {}
  for _, name in ipairs(tools.mason) do
    if not registry.get_package(name):is_installed() then
      table.insert(missing, name)
    end
  end
  for _, executable in ipairs({ "clangd", "clang-format", "node", "npm", "cc", "make" }) do
    if vim.fn.executable(executable) == 0 then
      table.insert(missing, executable)
    end
  end
  if #missing > 0 then
    error("Missing tools: " .. table.concat(missing, ", ") .. ". Fix the installation errors and rerun setup-editor.")
  end
  require("nvim-treesitter").install(tools.parsers):wait(300000)
  for _, language in ipairs(tools.parsers) do
    local loaded_ok, loaded = pcall(vim.treesitter.language.add, language)
    if not loaded_ok or not loaded then
      error("Parser installation failed: " .. language)
    end
  end
end
local ok, err = pcall(setup)
if not ok then
  vim.api.nvim_err_writeln(tostring(err))
  vim.cmd("cquit 1")
end
