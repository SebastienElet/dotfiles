all: \
	bundle/coc \
	bundle/dein \
	bundle/neoformat \
	bundle/theme-oceanic \
	help

bundle/theme-oceanic:
	git clone https://github.com/mhartington/oceanic-next.git $@

bundle/coc:
	git clone https://github.com/neoclide/coc.nvim.git $@

bundle/dein:
	git clone https://github.com/Shougo/dein.vim $@

bundle/neoformat:
	git clone https://github.com/sbdchd/neoformat.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
