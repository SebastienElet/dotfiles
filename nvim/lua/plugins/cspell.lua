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

      local sources = {
        -- cspell
        cspell.diagnostics.with({
          -- Set the severity to HINT for unknown words
          diagnostics_postprocess = function(diagnostic)
            diagnostic.severity = vim.diagnostic.severity["HINT"]
          end,
        }),
        cspell.code_actions,
      }
      -- Define the debounce value
      local debounce_value = 500
      return {
        sources = sources,
        debounce = debounce_value,
        debug = true,
      }
    end,
  },
}
