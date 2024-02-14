return {
  {
    "stevearc/conform.nvim",
    dependencies = { "mason.nvim" },
    opts = {
      formatters_by_ft = {
        sh = { "shfmt" },
        sql = { "sqlfluff" },
        xml = { "xmlformatter" },
      },
      formatters = {
        sqlfluff = {
          args = { "fix", "--force", "-" },
        },
        shfmt = {
          prepend_args = { "-i", "2", "-ci" },
        },
        xmlformatter = {
          command = "xmlformat",
          args = { "-" },
        },
      },
    },
  },
}
