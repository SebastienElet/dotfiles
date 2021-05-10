all: \
	bundle/dein \
	bundle/editorconfig \
	bundle/fugitive \
	bundle/fzf \
	bundle/neoformat \
	bundle/ale \
	bundle/syntax-ansible \
	bundle/syntax-js \
	bundle/syntax-json \
	bundle/syntax-jsx \
	bundle/syntax-less \
	bundle/syntax-graphql \
	bundle/syntax-markdown \
	bundle/syntax-rust \
	bundle/syntax-terraform \
	bundle/syntax-ts \
	bundle/tsuquyomi \
	help

bundle/dein:
	git clone --depth 1 https://github.com/Shougo/dein.vim $@

bundle/editorconfig:
	git clone --depth 1 https://github.com/editorconfig/editorconfig-vim.git $@

bundle/fugitive:
	git clone --depth 1 https://github.com/tpope/vim-fugitive.git $@

bundle/fzf:
	git clone --depth 1 https://github.com/junegunn/fzf.vim.git $@

bundle/neoformat:
	git clone --depth 1 https://github.com/sbdchd/neoformat.git $@

bundle/neomake:
	git clone --depth 1 https://github.com/neomake/neomake $@

bundle/ale:
	git clone --depth 1 https://github.com/w0rp/ale $@

bundle/syntax-ansible:
	git clone --depth 1 https://github.com/pearofducks/ansible-vim $@

bundle/syntax-js:
	git clone --depth 1 https://github.com/jelera/vim-javascript-syntax.git $@

bundle/syntax-json:
	git clone --depth 1 https://github.com/elzr/vim-json.git $@

bundle/syntax-jsx:
	git clone --depth 1 https://github.com/jelera/vim-javascript-syntax.git $@

bundle/syntax-less:
	git clone --depth 1 https://github.com/groenewege/vim-less.git $@

bundle/syntax-graphql:
	git clone --depth 1 git@github.com:jparise/vim-graphql.git $@

bundle/syntax-markdown:
	git clone --depth 1 https://github.com/tpope/vim-markdown.git $@

bundle/syntax-rust:
	git clone --depth 1 https://github.com/rust-lang/rust.vim $@

bundle/syntax-terraform:
	git clone --depth 1 https://github.com/hashivim/vim-terraform.git $@

bundle/syntax-ts:
	git clone --depth 1 https://github.com/leafgarland/typescript-vim.git $@

bundle/tsuquyomi:
	git clone --depth 1 https://github.com/Quramy/tsuquyomi.git $@

help:
	@vim -c "call pathogen#helptags()" -c "qall"

clean:
	rm -rf bundle/*

.PHONY: help
