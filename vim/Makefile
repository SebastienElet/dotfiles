all: \
	bundle/dein \
	bundle/editorconfig \
	bundle/fugitive \
	bundle/fzf \
	bundle/neoformat \
	bundle/neomake \
	bundle/syntax-ansible \
	bundle/syntax-js \
	bundle/syntax-json \
	bundle/syntax-jsx \
	bundle/syntax-less \
	bundle/syntax-markdown \
	bundle/syntax-rust \
	bundle/syntax-terraform \
	bundle/syntax-ts \
	bundle/tsuquyomi \
	help

bundle/dein:
	git clone https://github.com/Shougo/dein.vim $@

bundle/editorconfig:
	git clone https://github.com/editorconfig/editorconfig-vim.git $@

bundle/fugitive:
	git clone https://github.com/tpope/vim-fugitive.git $@

bundle/fzf:
	git clone https://github.com/junegunn/fzf.vim.git $@

bundle/neoformat:
	git clone https://github.com/sbdchd/neoformat.git $@

bundle/neomake:
	git clone https://github.com/neomake/neomake $@

bundle/syntax-ansible:
	git clone https://github.com/pearofducks/ansible-vim $@

bundle/syntax-js:
	git clone https://github.com/jelera/vim-javascript-syntax.git $@

bundle/syntax-json:
	git clone https://github.com/elzr/vim-json.git $@

bundle/syntax-jsx:
	git clone https://github.com/jelera/vim-javascript-syntax.git $@

bundle/syntax-less:
	git clone https://github.com/groenewege/vim-less.git $@

bundle/syntax-markdown:
	git clone https://github.com/tpope/vim-markdown.git $@

bundle/syntax-rust:
	git clone https://github.com/rust-lang/rust.vim $@

bundle/syntax-terraform:
	git clone https://github.com/hashivim/vim-terraform.git $@

bundle/syntax-ts:
	git clone https://github.com/leafgarland/typescript-vim.git $@

bundle/tsuquyomi:
	git clone https://github.com/Quramy/tsuquyomi.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
