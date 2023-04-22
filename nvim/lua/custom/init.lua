-- autocmds
local autocmd = vim.api.nvim_create_autocmd

-- Auto resize panes
autocmd("VimResized", {
	pattern = "*",
	command = "tabdo wincmd =",
})

-- Fold
vim.opt.foldmethod = "expr"
vim.opt.foldexpr = "nvim_treesitter#foldexpr()"
vim.opt.foldlevel = 2
