local repo = vim.fn.fnamemodify(debug.getinfo(1, 'S').source:sub(2), ':p:h:h')
vim.opt.runtimepath:prepend(repo .. '/dotfiles/.config/nvim')
vim.opt.runtimepath:append(repo .. '/dotfiles/.config/nvim/after')

local function query_checks(lang, query_lang, sample, wanted)
  local source = table.concat(vim.fn.readfile(repo .. '/dotfiles/.config/nvim/after/queries/' .. query_lang .. '/highlights.scm'), '\n')
  local tree = vim.treesitter.get_string_parser(sample, lang):parse()[1]
  local query = vim.treesitter.query.parse(lang, source)
  local counts = {}
  for _, node, metadata in query:iter_captures(tree:root(), sample) do
    local token = vim.treesitter.get_node_text(node, sample)
    assert(node:start() == 0, lang .. ': concealed a string/comment')
    assert(metadata.conceal == wanted[token], lang .. ': wrong symbol for ' .. token)
    counts[token] = (counts[token] or 0) + 1
  end
  for token in pairs(wanted) do
    assert(counts[token] == 1, lang .. ': missing/duplicate ' .. token)
  end
  local merged = assert(vim.treesitter.query.get(lang, 'highlights'))
  local found = false
  for _, node, metadata in merged:iter_captures(tree:root(), sample) do
    if vim.treesitter.get_node_text(node, sample) == '<=' and metadata.conceal == '≤' then found = true end
  end
  assert(found, lang .. ': custom query not merged')
end

local ecma = 'const f = (a,b) => a === b || a !== b || a != b || a <= b || a >= b;\nconst s="=> === !== != <= >="; // => === !== != <= >=\n'
for _, lang in ipairs({ 'javascript', 'typescript', 'tsx' }) do
  query_checks(lang, 'ecma', ecma,
    { ['=>'] = '⇒', ['==='] = '≡', ['!=='] = '≢', ['!='] = '≠', ['<='] = '≤', ['>='] = '≥' })
end
local c = 'struct S { int x; }; int f(struct S *p,int a,int b) { return p->x <= a && a >= b && a != b; }\nchar *s="-> <= >= !="; // -> <= >= !=\n'
query_checks('c', 'c', c, { ['->'] = '→', ['!='] = '≠', ['<='] = '≤', ['>='] = '≥' })

require('config.ligatures')
assert(vim.api.nvim_get_hl(0, { name = 'OperatorLigature' }).bold, 'symbols should be bold')
vim.bo.filetype = 'javascript'
assert(vim.wo.conceallevel == 2)
vim.cmd.LigaturesToggle()
assert(vim.wo.conceallevel == 0)
vim.cmd.LigaturesToggle()
assert(vim.wo.conceallevel == 2)
vim.bo.filetype = 'text'
assert(vim.wo.conceallevel == 0)

vim.cmd.syntax('on')
vim.bo.filetype = 'python'
vim.api.nvim_buf_set_lines(0, 0, -1, false, {
  'def f(a, b) -> bool: return a <= b and a >= b and a != b',
  'text = "-> <= >= !="  # -> <= >= !=',
})
vim.cmd('runtime! syntax/python.vim')
local code = vim.api.nvim_buf_get_lines(0, 0, 1, false)[1]
for token, group in pairs({ ['->'] = 'pythonOperatorLigatureArrow', ['<='] = 'pythonOperatorLigatureLessEqual',
  ['>='] = 'pythonOperatorLigatureGreaterEqual', ['!='] = 'pythonOperatorLigatureNotEqual' }) do
  local column = assert(code:find(token, 1, true))
  assert(vim.fn.synIDattr(vim.fn.synID(1, column, true), 'name') == group, 'Python operator: ' .. token)
  local quoted = assert(vim.api.nvim_buf_get_lines(0, 1, 2, false)[1]:find(token, 1, true))
  local actual = vim.fn.synIDattr(vim.fn.synID(2, quoted, true), 'name')
  assert(actual ~= group, 'Python string concealed: ' .. token)
end
assert(vim.wo.conceallevel == 2)
vim.bo.filetype = 'text'
assert(vim.wo.conceallevel == 0)
print('JS/TS/C queries, Python syntax, exclusions, bold style and toggle passed')
