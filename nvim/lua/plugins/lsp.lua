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
              enumMemberValues = { enabled = true },
              functionLikeReturnTypes = { enabled = false },
              parameterNames = { enabled = "literals" },
              parameterTypes = { enabled = true },
              propertyDeclarationTypes = { enabled = true },
              variableTypes = { enabled = false },
            },
          },
        },
      },
      tsserver = {
        init_options = {
          maxTsServerMemory = 16384,
        },
      },
    },
  },
}
