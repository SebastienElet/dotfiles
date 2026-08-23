-- ponytail: no built-in conform check for "does this project use X" beyond
-- command availability, so detect via config files / package.json deps
local function pkg_has_dep(bufnr, dep)
  local pkg = vim.fs.find("package.json", { path = vim.api.nvim_buf_get_name(bufnr), upward = true })[1]
  if not pkg then
    return false
  end
  local ok, data = pcall(vim.json.decode, table.concat(vim.fn.readfile(pkg), "\n"))
  if not ok then
    return false
  end
  return (data.dependencies or {})[dep] ~= nil or (data.devDependencies or {})[dep] ~= nil
end

local function uses_oxfmt(bufnr)
  local found = vim.fs.find(
    { ".oxfmtrc.json", ".oxfmtrc.jsonc", "oxfmt.config.ts" },
    { path = vim.api.nvim_buf_get_name(bufnr), upward = true }
  )[1]
  return found ~= nil or pkg_has_dep(bufnr, "oxfmt")
end

local function js_formatter(bufnr)
  return uses_oxfmt(bufnr) and { "oxfmt" } or { "prettier" }
end

return {
  {
    "stevearc/conform.nvim",
    dependencies = { "mason.nvim" },
    opts = {
      default_format_opts = {
        timeout_ms = 5000,
      },
      formatters_by_ft = {
        sql = { "prettier" },
        javascript = js_formatter,
        javascriptreact = js_formatter,
        typescript = js_formatter,
        typescriptreact = js_formatter,
      },
    },
  },
}
