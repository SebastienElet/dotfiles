return {
  "neovim/nvim-lspconfig",
  opts = {
    ---@type lspconfig.options
    servers = {
      tsserver = {
        init_options = {
          maxTsServerMemory = 16384,
        },
      },
      sqlls = {
        cmd = { "sql-language-server", "up", "--method", "stdio" },
      },
    },
  },
}
