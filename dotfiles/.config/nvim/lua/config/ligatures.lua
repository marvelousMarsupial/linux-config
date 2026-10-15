-- Display-only operator substitutions. Tree-sitter queries cover JS/TS/C/Python
-- with --dev-tools; Vim syntax covers Python when its parser is absent.
local supported = {
  javascript = true, javascriptreact = true, typescript = true,
  typescriptreact = true, c = true, python = true,
}
local enabled = true

local function emphasize()
  -- Terminals have one cell size; bold makes substitutions easier to spot.
  local base = vim.api.nvim_get_hl(0, { name = "Operator", link = false })
  base.bold = true
  vim.api.nvim_set_hl(0, "@operator.ligature", base)
  vim.api.nvim_set_hl(0, "OperatorLigature", base)
end

emphasize()
vim.api.nvim_create_autocmd("ColorScheme", {
  group = vim.api.nvim_create_augroup("OperatorLigatureColor", { clear = true }),
  callback = emphasize,
})

local function refresh()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    local buf = vim.api.nvim_win_get_buf(win)
    local wanted = enabled and supported[vim.bo[buf].filetype]
    local saved = vim.w[win].operator_conceal_original
    if wanted then
      if not saved then
        vim.w[win].operator_conceal_original = {
          level = vim.wo[win].conceallevel, cursor = vim.wo[win].concealcursor,
        }
      end
      vim.wo[win].conceallevel = 2
      -- Reveal the entire cursor line in every mode, so editing stays explicit.
      vim.wo[win].concealcursor = ""
    elseif saved then
      vim.wo[win].conceallevel = saved.level
      vim.wo[win].concealcursor = saved.cursor
      vim.w[win].operator_conceal_original = nil
    end
  end
end

vim.api.nvim_create_autocmd({ "FileType", "BufWinEnter", "WinEnter" }, {
  group = vim.api.nvim_create_augroup("OperatorLigatures", { clear = true }),
  callback = refresh,
})
vim.api.nvim_create_user_command("LigaturesToggle", function()
  enabled = not enabled
  refresh()
  vim.notify("Operator symbols " .. (enabled and "enabled" or "disabled"))
end, {})
vim.keymap.set("n", "<leader>ul", "<cmd>LigaturesToggle<CR>", { desc = "Toggle operator symbols" })
