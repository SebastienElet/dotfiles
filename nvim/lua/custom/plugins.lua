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
						-- TODO: load the default telescope setting from nvchad
						vimgrep_arguments = {
							"rg",
							"-L",
							"--color=never",
							"--no-heading",
							"--with-filename",
							"--line-number",
							"--column",
							"--smart-case",
						},
						prompt_prefix = "   ",
						selection_caret = "  ",
						entry_prefix = "  ",
						initial_mode = "insert",
						selection_strategy = "reset",
						sorting_strategy = "ascending",
						layout_strategy = "horizontal",
						layout_config = {
							horizontal = {
								prompt_position = "top",
								preview_width = 0.55,
								results_width = 0.8,
							},
							vertical = {
								mirror = false,
							},
							width = 0.87,
							height = 0.80,
							preview_cutoff = 120,
						},
						file_sorter = require("telescope.sorters").get_fuzzy_file,
						file_ignore_patterns = { "node_modules" },
						generic_sorter = require("telescope.sorters").get_generic_fuzzy_sorter,
						path_display = { "truncate" },
						winblend = 0,
						border = {},
						borderchars = { "─", "│", "─", "│", "╭", "╮", "╯", "╰" },
						color_devicons = true,
						set_env = { ["COLORTERM"] = "truecolor" }, -- default = nil,
						file_previewer = require("telescope.previewers").vim_buffer_cat.new,
						grep_previewer = require("telescope.previewers").vim_buffer_vimgrep.new,
						qflist_previewer = require("telescope.previewers").vim_buffer_qflist.new,
						-- Developer configurations: Not meant for general override
						buffer_previewer_maker = require("telescope.previewers").buffer_previewer_maker,
						mappings = {
							n = { ["q"] = require("telescope.actions").close },
						},
						--
						extensions = {
							fzf = {
								fuzzy = true, -- false will only do exact matching
								override_generic_sorter = true, -- override the generic sorter
								override_file_sorter = true, -- override the file sorter
								case_mode = "smart_case", -- or "ignore_case" or "respect_case"
								-- the default case_mode is "smart_case"
							},
						},
					})
					-- To get fzf loaded and working with telescope, you need to call
					-- load_extension, somewhere after setup function:
					require("telescope").load_extension("fzf")
				end,
			},
		},
		opts = {},
	},
	{
		"hrsh7th/nvim-cmp",
		dependencies = {
			{
				"jcdickinson/codeium.nvim",
				dependencies = {
					{
						"nvim-lua/plenary.nvim",
					},
				},
				config = function()
					require("codeium").setup({})
				end,
			},
		},
		opts = {
			sources = {
				{ name = "codeium" },
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
					local null_ls = require("null-ls")
					local builtins = null_ls.builtins
					local sources = {
						-- webdev stuff
						builtins.formatting.prettierd,
						builtins.formatting.eslint_d,
						builtins.diagnostics.eslint_d,
						builtins.code_actions.eslint_d,
						builtins.formatting.rustfmt.with({
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
						builtins.formatting.stylua,

						-- Shell
						builtins.formatting.shfmt,
						builtins.diagnostics.shellcheck.with({ diagnostics_format = "#{m} [#{c}]" }),
						builtins.code_actions.shellcheck,

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
			require("plugins.configs.lspconfig")
			-- require "custom.configs.lspconfig"
			-- local on_attach = require("plugins.configs.lspconfig").on_attach
			local utils = require("core.utils")
			local on_attach = function(client, bufnr)
				client.server_capabilities.documentFormattingProvider = false
				client.server_capabilities.documentRangeFormattingProvider = false

				utils.load_mappings("lspconfig", { buffer = bufnr })

				if client.server_capabilities.signatureHelpProvider then
					require("nvchad_ui.signature").setup(client)
				end

				if not utils.load_config().ui.lsp_semantic_tokens then
					client.server_capabilities.semanticTokensProvider = nil
				end
			end
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
		},
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
		},
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
		},
	},
}
