return {
  {
    "projekt0n/github-nvim-theme",
    name = "github-theme",
    lazy = false,
    priority = 1000,
    config = function()
      require("github-theme").setup({
        options = {
          transparent = false,
          dim_inactive = false,
          styles = {
            comments = "italic",
          },
        },
      })
    end,
  },

  {
    "f-person/auto-dark-mode.nvim",
    opts = {
      update_interval = 1000, -- Check every second
      set_dark_mode = function()
        vim.o.background = "dark"
        vim.cmd("colorscheme github_dark")
      end,
      set_light_mode = function()
        vim.o.background = "light"
        vim.cmd("colorscheme github_light")
      end,
    },
  },

  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = function() end, -- Handled by auto-dark-mode
    },
  },
}
