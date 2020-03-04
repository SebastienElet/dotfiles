all: \
	bundle/coc \
	bundle/dein \
	bundle/polyglot \
	bundle/theme-oceanic \
	help

bundle/coc:
	git clone https://github.com/neoclide/coc.nvim.git $@

bundle/dein:
	git clone https://github.com/Shougo/dein.vim $@

bundle/polyglot:
	git clone https://github.com/sheerun/vim-polyglot.git $@

bundle/theme-oceanic:
	git clone https://github.com/mhartington/oceanic-next.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
