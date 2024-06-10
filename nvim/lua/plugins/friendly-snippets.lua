return {
  "rafamadriz/friendly-snippets",
  config = function()
    -- TODO: this does not work anymore
    -- require("luasnip.loaders.from_vscode").lazy_load({ paths = { "./snippets" } })
    -- TODO: load from .vscode folders
    -- require("luasnip.loaders.from_vscode").lazy_load()
  end,
}
