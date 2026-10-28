return {
  {
    "mason-org/mason.nvim",
    opts = {
      registries = { "github:mason-org/mason-registry@2026-10-23-zesty-flash" },
      ensure_installed = {
        "cspell",
        "docker-compose-language-service",
        "dockerfile-language-server",
        "eslint-lsp",
        "json-lsp",
        "lua-language-server",
        "markdown-toc",
        "marksman",
        "prisma-language-server",
        "shfmt",
        "stylua",
        "tree-sitter-cli",
        "vtsls",
      },
    },
  },
}
