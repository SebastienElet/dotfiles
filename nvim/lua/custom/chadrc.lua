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
					-- b.formatting.deno_fmt,
					b.formatting.prettierd,
					b.formatting.eslint_d,
					b.diagnostics.eslint_d,
					b.code_actions.eslint_d,
					-- b.formatting.eslint_d,
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
					-- b.formatting.shfmt,
					-- b.diagnostics.shellcheck.with({ diagnostics_format = "#{m} [#{c}]" }),

					-- Codespell
					-- TODO: cspell output error code instead of information
					-- TODO: install cspell french dictionnary
					-- TODO: how to add new words
					-- b.diagnostics.cspell,
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
				"css",
				"html",
				"javascript",
				"json",
				"rust",
				"typescript",
				"graphql",
				"prisma",
			},
		},
		["williamboman/mason.nvim"] = {
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
				-- "shfmt",
				"shellcheck",

				-- spell checker
				"cspell",
			},
		},
	},
}

M.mappings = {
	usage = {
		n = {
			-- Bad idea, this is conflicting with quickfix
			-- ["<CR>"] = { ":nohlsearch<CR>:w<CR>", "Save file", {} },
			["<Tab>"] = { "%", "Find the next item", {} },
			-- cargo install typos-cli
			["<Leader>t"] = { ':execute "!typos" @%<CR>', "Run typos cli on the current file", {} },
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
