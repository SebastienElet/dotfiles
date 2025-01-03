return {
  {
    "stevearc/conform.nvim",
    dependencies = { "mason.nvim" },
    opts = {
      default_format_opts = {
        timeout_ms = 5000,
      },
      formatters = {
        sqlfluff = {
          args = { "fix", "--dialect=postgres", "-" },
        },
      },
    },
  },
}
