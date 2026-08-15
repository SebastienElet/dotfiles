return {
  {
    "nvim-treesitter/nvim-treesitter",
    opts = function(_, opts)
      vim.list_extend(opts.ensure_installed, {
        "bash",
        "css",
        "fish",
        "graphql",
        "html",
        "javascript",
        "json",
        "lua",
        "markdown",
        "markdown_inline",
        "prisma",
        "query",
        "regex",
        "rust",
        "sql",
        "terraform",
        "tsx",
        "typescript",
        "vim",
        "yaml",
      })
    end,
  },
}
