return {
  {
    "williamboman/mason.nvim",
    opts = {
      ensure_installed = {
        "cspell",
        "shellcheck",
        "shfmt",
        "sqlfluff",
        "sqlls",
        "stylua",
        "xmlformatter",
      },
    },
  },
}
