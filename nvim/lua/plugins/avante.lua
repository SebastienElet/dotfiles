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
    "ibhagwan/fzf-lua", -- file_selector provider (already in config)
    {
      "MeanderingProgrammer/render-markdown.nvim",
      opts = { file_types = { "markdown", "Avante" } },
      ft = { "markdown", "Avante" },
    },
  },
  ---@type avante.Config
  opts = {
    instructions_file = "avante.md",
    provider = "claude",
    --- ACP (Agent Client Protocol) for Cursor-like agent capabilities in Neovim.
    acp_providers = {
      ["claude-code"] = {
        command = "npx",
        args = { "@zed-industries/claude-code-acp" },
        env = {
          NODE_NO_WARNINGS = "1",
          ANTHROPIC_API_KEY = os.getenv("ANTHROPIC_API_KEY") or os.getenv("AVANTE_ANTHROPIC_API_KEY"),
        },
      },
      ["codex"] = {
        command = "npx",
        args = { "@zed-industries/codex-acp" },
        env = {
          NODE_NO_WARNINGS = "1",
          OPENAI_API_KEY = os.getenv("OPENAI_API_KEY") or os.getenv("AVANTE_OPENAI_API_KEY"),
        },
      },
    },
    behaviour = {
      acp_follow_agent_locations = true,
    },
  },
}
