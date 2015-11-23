all: \
  bundle/ternjs \
  bundle/youcompleteme \
  bundle/qargs \
  bundle/ctrlp \
  bundle/ag \
  bundle/syntastic \
  bundle/editorconfig \
  bundle/syntax-json \
  bundle/syntax-less \
  bundle/syntax-markdown \
  bundle/viml-linter

bundle/youcompleteme:
	git clone https://github.com/Valloric/YouCompleteMe.git $@
	cd $@ && git submodule update --init --recursive
	cd $@ && ./install.py --clang-completer

bundle/ternjs:
	git clone https://github.com/ternjs/tern_for_vim.git $@
	cd $@ && npm i

bundle/qargs:
	git clone https://github.com/henrik/vim-qargs.git $@

bundle/ag:
	git clone https://github.com/rking/ag.vim.git $@

bundle/syntastic:
	git clone https://github.com/scrooloose/syntastic.git $@

bundle/editorconfig:
	git clone https://github.com/editorconfig/editorconfig-vim.git $@

bundle/ctrlp:
	git clone https://github.com/kien/ctrlp.vim $@

bundle/syntax-json:
	git clone https://github.com/elzr/vim-json.git $@

bundle/syntax-jade:
	git clone https://github.com/digitaltoad/vim-jade.git $@

bundle/syntax-less:
	git clone https://github.com/groenewege/vim-less.git $@

bundle/syntax-markdown:
	git clone https://github.com/tpope/vim-markdown.git $@

bundle/viml-linter: bundle/viml-parser
	git clone https://github.com/syngan/vim-vimlint.git $@

bundle/viml-parser:
	git clone https://github.com/ynkdir/vim-vimlparser.git $@

clean:
	rm -rf bundle/*
