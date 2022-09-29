local M = {}
M.ui = {}
M.plugins = require("custom.plugins")

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
