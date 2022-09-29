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
