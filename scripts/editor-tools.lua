-- Called explicitly by setup-editor; never downloads tools during ordinary editing.
-- Progress and failures go to stderr so a headless run is never silent.
local function report(text)
  io.stderr:write(text .. "\n")
end

local function setup()
  require("lazy").load({ plugins = { "mason.nvim", "nvim-lspconfig", "nvim-treesitter" } })
  local tools = require("config.tools")
  report("Installing Mason packages: " .. table.concat(tools.mason, " "))
  vim.cmd("MasonInstall " .. table.concat(tools.mason, " "))
  local registry = require("mason-registry")
  local missing = {}
  for _, name in ipairs(tools.mason) do
    if not registry.get_package(name):is_installed() then
      table.insert(missing, name)
    end
  end
  for _, executable in ipairs({ "clangd", "clang-format", "node", "npm", "cc", "make", "shellcheck", "tree-sitter" }) do
    if vim.fn.executable(executable) == 0 then
      table.insert(missing, executable)
    end
  end
  if #missing > 0 then
    error("Missing tools: " .. table.concat(missing, ", ") .. "\nFix the errors and run setup-editor again")
  end
  report("Installing Tree-sitter parsers: " .. table.concat(tools.parsers, " "))
  local task = require("nvim-treesitter").install(tools.parsers)
  if task and task.wait then
    task:wait(600000)
  end
  local failed = {}
  for _, language in ipairs(tools.parsers) do
    local loaded_ok, loaded = pcall(vim.treesitter.language.add, language)
    if not loaded_ok or not loaded then
      table.insert(failed, language)
    end
  end
  if #failed > 0 then
    error("Parser installation failed: " .. table.concat(failed, ", "))
  end
  report(("Development tools ready: %d Mason packages, %d parsers"):format(#tools.mason, #tools.parsers))
end
local ok, err = pcall(setup)
if not ok then
  report("setup-editor --dev-tools failed: " .. tostring(err))
  vim.cmd("cquit 1")
end
