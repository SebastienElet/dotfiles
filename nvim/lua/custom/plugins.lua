---@type NvPluginSpec[]
return {
  -- Override plugin definition options

  {
    "nvim-telescope/telescope.nvim",
    dependencies = {
      {
        "nvim-telescope/telescope-fzf-native.nvim",
        build = "make",
        config = function()
          require("telescope").setup({
            extensions = {
              fzf = {
                fuzzy = true,                   -- false will only do exact matching
                override_generic_sorter = true, -- override the generic sorter
                override_file_sorter = true,    -- override the file sorter
                case_mode = "smart_case",       -- or "ignore_case" or "respect_case"
                -- the default case_mode is "smart_case"
              },
            },
          })
          -- To get fzf loaded and working with telescope, you need to call
          -- load_extension, somewhere after setup function:
          require("telescope").load_extension("fzf")
        end,
      }
    },
    opts = {}
  },
  {
    "hrsh7th/nvim-cmp",
    dependencies = {
      {
        "zbirenbaum/copilot.lua",
        dependencies = {
          {
            "zbirenbaum/copilot-cmp",
            config = function()
              require("copilot_cmp").setup({
                method = "getCompletionsCycling",
                formatters = {
                  label = require("copilot_cmp.format").format_label_text,
                  insert_text = require("copilot_cmp.format").format_insert_text,
                  preview = require("copilot_cmp.format").deindent,
                },
              })
            end,
          }

        },
        config = function()
          require("copilot").setup({
            -- Apply the Github support recommendations
            -- From this issue https://github.com/zbirenbaum/copilot-cmp/issues/10
            -- server_opts_overrides = {
            -- 	trace = "verbose",
            -- 	settings = {
            -- 		advanced = {
            -- 			indentationMode = {
            -- 				javascript = "client",
            -- 				typescript = "client",
            -- 			},
            -- 		},
            -- 	},
            -- },
            suggestion = { enabled = false },
            panel = { enabled = false },
          })
        end,
      },

    },
    opts = {
      sources = {
        { name = "copilot" },
        { name = "luasnip" },
        { name = "nvim_lsp" },
        { name = "buffer" },
        { name = "nvim_lua" },
        { name = "path" },
      },
    },
  },

  {
    "neovim/nvim-lspconfig",
    dependencies = {
      -- format & linting
      {
        "jose-elias-alvarez/null-ls.nvim",
        config = function()
          local null_ls = require "null-ls"
          local b = null_ls.builtins
          local sources = {
            -- webdev stuff
            b.formatting.prettierd,
            b.formatting.eslint_d,
            b.diagnostics.eslint_d,
            b.code_actions.eslint_d,
            b.formatting.rustfmt.with({
              extra_args = function(params)
                local Path = require("plenary.path")
                local cargo_toml = Path:new(params.root .. "/" .. "Cargo.toml")

                if cargo_toml:exists() and cargo_toml:is_file() then
                  for _, line in ipairs(cargo_toml:readlines()) do
                    local edition = line:match([[^edition%s*=%s*%"(%d+)%"]])
                    if edition then
                      return { "--edition=" .. edition }
                    end
                  end
                end
                -- default edition when we don't find `Cargo.toml` or the `edition` in it.
                return { "--edition=2021" }
              end,
            }),

            -- Lua
            b.formatting.stylua,

            -- Shell
            b.formatting.shfmt,
            b.diagnostics.shellcheck.with({ diagnostics_format = "#{m} [#{c}]" }),
            b.code_actions.shellcheck,

            -- Codespell
            -- TODO: cspell output error code instead of information
            -- TODO: install cspell french dictionnary
            -- TODO: how to add new words
            -- b.diagnostics.cspell,
            -- b.code_actions.cspell
          }
          null_ls.setup({
            debug = true,
            sources = sources,
          })
        end,
      },
    },
    config = function()
      require "plugins.configs.lspconfig"
      -- require "custom.configs.lspconfig"
      local on_attach = require("plugins.configs.lspconfig").on_attach
      local capabilities = require("plugins.configs.lspconfig").capabilities
      local lspconfig = require("lspconfig")

      local servers = { "html", "cssls", "rust_analyzer", "graphql", "jsonls" }

      for _, lsp in ipairs(servers) do
        lspconfig[lsp].setup({
          on_attach = on_attach,
          capabilities = capabilities,
        })
      end

      lspconfig.tsserver.setup({
        settings = {
          maxTsServerMemory = 8192,
          preferences = {
            importModuleSpecifierPreference = "non-relative",
            upddateImportsOnFileMove = "always",
          },
        },
        on_attach = on_attach,
        capabilities = capabilities,
      })
    end, -- Override to setup mason-lspconfig
  },

  -- override plugin configs
  {
    "nvim-tree/nvim-tree.lua",
    opts = {
      git = {
        enable = true,
      },

      renderer = {
        highlight_git = true,
        icons = {
          show = {
            git = true,
          },
        },
      },
    }
  },
  {
    "nvim-treesitter/nvim-treesitter",
    opts = {
      ensure_installed = {
        "css",
        "graphql",
        "html",
        "javascript",
        "json",
        "lua",
        "rust",
        "typescript",
        "terraform",
        "prisma",
      },
    }
  },
  {
    "williamboman/mason.nvim",
    opts = {
      ensure_installed = {
        -- lua stuff
        "lua-language-server",
        "stylua",

        -- web dev
        "css-lsp",
        "deno",
        "emmet-ls",
        "html-lsp",
        "json-lsp",
        "eslint_d",
        "typescript-language-server",
        "prisma-language-server",
        "graphql-language-service-cli",

        -- rust
        "rust-analyzer",

        -- shell
        "shellcheck",
        "shfmt",

        -- docker
        "dockerfile-language-server",
        "docker-compose-language-service",

        -- yaml
        "yaml-language-server",
        "yamlfmt",
        "yamllint",

        -- spell checker
        "cspell",
      },
    }
  },
}
