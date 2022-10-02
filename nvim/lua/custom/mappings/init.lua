return {
	usage = {
		n = {
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
