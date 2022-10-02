return {
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
	["nvim-treesitter/nvim-treesitter"] = {
		override_options = {
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

				-- spell checker
				"cspell",
			},
		},
	},
}
