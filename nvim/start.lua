-- vim:fdm=marker

-- Editor {{{
vim.o.expandtab = true
vim.o.shiftwidth = 2
vim.o.tabstop = 2
vim.o.shortmess = 'I'
-- TODO: vim.o.clipboard = 'unnamed'
-- }}}

-- Tabs & Indent {{{
-- Indent with tabs
vim.api.nvim_set_keymap('v', '<Tab>', 'gv', {})
vim.api.nvim_set_keymap('v', '<S-Tab>', '<gv', {})
-- Keep selection after indent
vim.api.nvim_set_keymap('v', '<', '<gv', { noremap = true })
vim.api.nvim_set_keymap('v', '>', '>gv', { noremap = true })
-- }}}

-- Bindings {{{
-- Save on <CR>
vim.api.nvim_set_keymap('n', '<CR>', ':nohlsearch<CR>:w<CR>', { noremap = true, silent = true })
-- Go to the next bracket with <Tab>
vim.api.nvim_set_keymap('n', '<Tab>', '%', { noremap = true })
-- Navigate between buffers
vim.api.nvim_set_keymap('n', '<C-j>', '<C-W>j', {})
vim.api.nvim_set_keymap('n', '<C-k>', '<C-W>k', {})
vim.api.nvim_set_keymap('n', '<C-h>', '<C-W>h', {})
vim.api.nvim_set_keymap('n', '<C-l>', '<C-W>l', {})
-- }}}

-- Windows {{{
vim.api.nvim_command('autocmd VimResized * tabdo wincmd =')
-- }}}

require "plug"
