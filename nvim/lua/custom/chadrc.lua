local M = {}
M.ui = {}
M.plugins = {
	user = {
		["neovim/nvim-lspconfig"] = {
			config = function()
				require("plugins.configs.lspconfig")
				require("custom.lspconfig")
			end,
		},

		["jose-elias-alvarez/null-ls.nvim"] = {
			after = "nvim-lspconfig",
			config = function()
				-- TODO: move in a dedicated configuration file
				local present, null_ls = pcall(require, "null-ls")

				if not present then
					return
				end

				local b = null_ls.builtins

				local sources = {

					-- webdev stuff
					b.formatting.deno_fmt,
					b.formatting.prettierd,
					b.formatting.eslint,
					b.formatting.rustfmt,

					-- Lua
					b.formatting.stylua,

					-- Shell
					b.formatting.shfmt,
					b.diagnostics.shellcheck.with({ diagnostics_format = "#{m} [#{c}]" }),
				}

				null_ls.setup({
					debug = true,
					sources = sources,
				})
			end,
		},
	},
	override = {
		["nvim-treesitter/nvim-treesitter"] = {
			ensure_installed = {
				"html",
				"css",
				"typescript",
				"javascript",
				"json",
				"rust",
			},
		},
		["williamboman/mason.nvim"] = {
			ensure_installed = {
				-- lua stuff
				"lua-language-server",
				"stylua",

				-- web dev
				"css-lsp",
				"html-lsp",
				"typescript-language-server",
				"deno",
				"emmet-ls",
				"json-lsp",
				"rust-analyzer",

				-- shell
				-- "shfmt",
				"shellcheck",
			},
		},
	},
}

M.mappings = {
	usage = {
		n = {
			["<CR>"] = { ":nohlsearch<CR>:w<CR>", "Save file", {} },
			["<Tab>"] = { "%", "Find the next item", {} },
		},
		v = {
			["<Tab>"] = { ">gv", "Indent the selected block", {} },
			["<S-Tab>"] = { "<gv", "Unindent the selected block", {} },
			[">"] = { ">gv", "Indent the selected block", {} },
			["<"] = { "<gv", "Unindent the selected block", {} },
		},
	},
}

return M
