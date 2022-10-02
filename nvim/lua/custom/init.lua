-- autocmds
local autocmd = vim.api.nvim_create_autocmd

-- Auto resize panes
autocmd("VimResized", {
  pattern = "*",
  command = "tabdo wincmd =",
})
