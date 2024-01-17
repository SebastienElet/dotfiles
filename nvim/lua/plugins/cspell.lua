return {
  {
    "nvimtools/none-ls.nvim",
    dependencies = { "mason.nvim", "davidmh/cspell.nvim" },
    event = { "BufReadPre", "BufNewFile" },
    opts = function()
      local cspell = require("cspell")
      local ok = pcall(require, "null-ls")
      if not ok then
        return
      end

      local null_ls = require("null-ls")

      -- local b = none_ls.builtins

      local sources = {
        -- spell check
        -- b.diagnostics.codespell,
        -- b.diagnostics.misspell,
        -- cspell
        cspell.diagnostics.with({
          -- Set the severity to HINT for unknown words
          diagnostics_postprocess = function(diagnostic)
            diagnostic.severity = vim.diagnostic.severity["HINT"]
          end,
        }),
        cspell.code_actions,
        null_ls.builtins.diagnostics.sqlfluff.with({
          extra_args = { "--dialect", "postgres" }, -- change to your dialect
        }),
      }
      -- Define the debounce value
      local debounce_value = 300
      return {
        sources = sources,
        debounce = debounce_value,
        debug = true,
      }
    end,
  },
}
