all: \
	bundle/coc \
	bundle/copilot \
	bundle/dein \
	bundle/fugitive \
	bundle/polyglot \
	bundle/prisma \
	bundle/telescope \
	bundle/theme-oceanic \
	help

bundle/coc:
	git clone --depth 1 https://github.com/neoclide/coc.nvim.git $@

bundle/copilot:
	git clone --depth 1 https://github.com/github/copilot.vim.git $@

bundle/dein:
	git clone --depth 1 https://github.com/Shougo/dein.vim $@

bundle/fugitive:
	git clone --depth 1 https://github.com/tpope/vim-fugitive.git $@

bundle/polyglot:
	git clone --depth 1 https://github.com/sheerun/vim-polyglot.git $@

bundle/prisma:
	git clone --depth 1 https://github.com/pantharshit00/vim-prisma.git $@

bundle/theme-oceanic:
	git clone --depth 1 https://github.com/mhartington/oceanic-next.git $@

bundle/popup:
	git clone --depth 1 https://github.com/nvim-lua/popup.nvim.git $@

bundle/plenary:
	git clone --depth 1 https://github.com/nvim-lua/plenary.nvim.git $@

bundle/telescope/: bundle/popup bundle/plenary
	git clone --depth 1 https://github.com/nvim-telescope/telescope.nvim.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
