all: \
	bundle/dein \
	bundle/editorconfig \
	bundle/fzf \
	bundle/neoformat \
	bundle/neomake \
	bundle/syntax-ansible \
	bundle/syntax-js \
	bundle/syntax-json \
	bundle/syntax-jsx \
	bundle/syntax-less \
	bundle/syntax-markdown \
	bundle/syntax-ts \
	help

bundle/dein:
	git clone https://github.com/Shougo/dein.vim $@

bundle/editorconfig:
	git clone https://github.com/editorconfig/editorconfig-vim.git $@

bundle/fzf:
	git clone https://github.com/junegunn/fzf.vim.git $@

bundle/neoformat:
	git clone git@github.com:sbdchd/neoformat.git $@

bundle/neomake:
	git clone https://github.com/neomake/neomake $@

bundle/syntax-ansible:
	git clone https://github.com/pearofducks/ansible-vim $@

bundle/syntax-js:
	git clone git@github.com:jelera/vim-javascript-syntax.git $@

bundle/syntax-json:
	git clone https://github.com/elzr/vim-json.git $@

bundle/syntax-jsx:
	git clone https://github.com/jelera/vim-javascript-syntax.git $@

bundle/syntax-less:
	git clone https://github.com/groenewege/vim-less.git $@

bundle/syntax-markdown:
	git clone https://github.com/tpope/vim-markdown.git $@

bundle/syntax-ts:
	git clone git@github.com:HerringtonDarkholme/yats.vim.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
