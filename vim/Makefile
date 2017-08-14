all: \
	bundle/dein \
	bundle/fzf \
	bundle/neoformat \
	bundle/neomake \
	bundle/syntax-js \
	help

bundle/dein:
	git clone https://github.com/Shougo/dein.vim $@

bundle/fzf:
	git clone https://github.com/junegunn/fzf.vim.git $@

bundle/neoformat:
	git clone git@github.com:sbdchd/neoformat.git $@

bundle/neomake:
	git clone https://github.com/neomake/neomake $@

bundle/syntax-js:
	git clone git@github.com:jelera/vim-javascript-syntax.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
