return {
  "rafamadriz/friendly-snippets",
  config = function()
    require("luasnip.loaders.from_vscode").lazy_load({ paths = { "./snippets" } })
    -- TODO: load from .vscode folders
    -- require("luasnip.loaders.from_vscode").lazy_load()
  end,
}
