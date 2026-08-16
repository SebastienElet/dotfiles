return {
  "neovim/nvim-lspconfig",
  opts = {

    ---@type lspconfig.options
    servers = {
      oxlint = {},
      eslint = {
        on_attach = function(client, bufnr)
          vim.api.nvim_create_autocmd("BufWritePre", {
            buffer = bufnr,
            callback = function()
              client:request_sync("workspace/executeCommand", {
                command = "eslint.applyAllFixes",
                arguments = {
                  {
                    uri = vim.uri_from_bufnr(bufnr),
                    version = vim.lsp.util.buf_versions[bufnr],
                  },
                },
              }, nil, bufnr)
            end,
          })
        end,
      },
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
    },
  },
}
