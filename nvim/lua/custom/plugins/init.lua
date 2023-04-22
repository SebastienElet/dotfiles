return {
	-- Disable autopairs to make copilot working
	["windwp/nvim-autopairs"] = false,
	["neovim/nvim-lspconfig"] = {
		config = function()
			require("plugins.configs.lspconfig")
			require("custom.plugins.lspconfig")
		end,
	},
	["jose-elias-alvarez/null-ls.nvim"] = {
		after = "nvim-lspconfig",
		config = function()
			require("custom.plugins.null-ls")
		end,
	},
	["zbirenbaum/copilot.lua"] = {
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
	["zbirenbaum/copilot-cmp"] = {
		after = "copilot.lua",
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
	},
	["hrsh7th/nvim-cmp"] = {
		after = "copilot.lua",
		override_options = {
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
	["nvim-treesitter/nvim-treesitter"] = {
		override_options = {
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
	["nvim-telescope/telescope-fzf-native.nvim"] = {
		after = "telescope.nvim",
		run = "make",
		config = function()
			require("telescope").setup({
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
	["williamboman/mason.nvim"] = {
		override_options = {
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
