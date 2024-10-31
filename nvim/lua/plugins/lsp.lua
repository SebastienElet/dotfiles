return {
  "neovim/nvim-lspconfig",
  opts = {

    ---@type lspconfig.options
    servers = {
      vtsls = {
        settings = {
          typescript = {
            tsserver = {
              maxTsServerMemory = 16384,
            },
            inlayHints = {
              enumMemberValues = { enabled = false },
              functionLikeReturnTypes = { enabled = false },
              parameterNames = { enabled = "literals" },
              parameterTypes = { enabled = false },
              propertyDeclarationTypes = { enabled = false },
              variableTypes = { enabled = false },
            },
          },
        },
      },
      ts_ls = {
        enabled = false,
        init_options = {
          maxTsServerMemory = 16384,
        },
      },
    },
  },
}
