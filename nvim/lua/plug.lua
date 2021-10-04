local paq_install_path = vim.fn.stdpath("data") .. "/site/pack/paqs/start/paq-nvim"
if vim.fn.empty(vim.fn.glob(paq_install_path)) > 0 then
  vim.fn.system({"git", "clone", "--depth=1", "https://github.com/savq/paq-nvim.git", paq_install_path})
end

require "paq" {
  "savq/paq-nvim", -- Let Paq manage itself
  "nvim-lua/popup.nvim",
  "nvim-lua/plenary.nvim",
  "nvim-telescope/telescope.nvim",
  "neovim/nvim-lspconfig",
  "glepnir/lspsaga.nvim",
  "nvim-lua/completion-nvim",
}

require "telescope-config"
require "completion-config"
require "lsp-config"
require "lspsaga-config"
