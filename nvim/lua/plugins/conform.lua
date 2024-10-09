return {
  {
    "stevearc/conform.nvim",
    dependencies = { "mason.nvim" },
    opts = {
      formatters = {
        sqlfluff = {
          args = { "fix", "--dialect=postgres", "-" },
        },
      },
    },
  },
}
