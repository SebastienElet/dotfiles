-- Cursor ACP via avante.nvim (https://cursor.com/docs/cli/acp#neovim-avantenvim)
-- Requires: Cursor CLI installed (e.g. ~/.local/bin/agent). Run `agent login` once to authenticate.
return {
  "yetone/avante.nvim",
  build = vim.fn.has("win32") ~= 0
      and "powershell -ExecutionPolicy Bypass -File Build.ps1 -BuildFromSource false"
      or "make",
  event = "VeryLazy",
  version = false,
  dependencies = {
    "nvim-lua/plenary.nvim",
    "MunifTanjim/nui.nvim",
    "echasnovski/mini.icons", -- already in config; or nvim-tree/nvim-web-devicons
    {
      "MeanderingProgrammer/render-markdown.nvim",
      opts = { file_types = { "markdown", "Avante" } },
      ft = { "markdown", "Avante" },
    },
  },
  ---@type avante.Config
  opts = {
    instructions_file = "avante.md",
    provider = "cursor",
    mode = "agentic",
    acp_providers = {
      cursor = {
        command = os.getenv("HOME") .. "/.local/bin/agent",
        args = { "acp" },
        auth_method = "cursor_login",
        env = {
          HOME = os.getenv("HOME"),
          PATH = os.getenv("PATH"),
        },
      },
    },
    behaviour = {
      acp_follow_agent_locations = true,
    },
  },
}
