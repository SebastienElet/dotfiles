local on_attach = require("plugins.configs.lspconfig").on_attach
local capabilities = require("plugins.configs.lspconfig").capabilities
local lspconfig = require("lspconfig")

local servers = { "html", "cssls", "rust_analyzer", "graphql", "jsonls" }

for _, lsp in ipairs(servers) do
	lspconfig[lsp].setup({
		on_attach = on_attach,
		capabilities = capabilities,
	})
end

lspconfig.tsserver.setup({
  settings = {
    maxTsServerMemory = 8192,
  },
  on_attach = on_attach,
  capabilities = capabilities,
})
