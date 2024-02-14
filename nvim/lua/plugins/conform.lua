return {
  {
    "stevearc/conform.nvim",
    dependencies = { "mason.nvim" },
    opts = {
      formatters_by_ft = {
        sql = { "sqlfluff" },
        sh = { "shfmt" },
      },
      formatters = {
        sqlfluff = {
          args = { "fix", "--force", "-" },
        },
        shfmt = {
          prepend_args = { "-i", "2", "-ci" },
        },
      },
    },
  },
}
