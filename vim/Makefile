all: \
  bundle/ag \
  bundle/beautify-js \
  bundle/editorconfig \
  bundle/fugitive \
  bundle/fzf \
  bundle/qargs \
  bundle/syntastic \
  bundle/syntax-ansible \
  bundle/syntax-jade \
  bundle/syntax-js-libs \
  bundle/syntax-json \
  bundle/syntax-jsx \
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

bundle/airline:
	git clone https://github.com/bling/vim-airline.git $@

bundle/ctrlp:
	git clone https://github.com/kien/ctrlp.vim $@

bundle/fugitive:
	git clone https://github.com/tpope/vim-fugitive $@

bundle/fzf:
	git clone https://github.com/junegunn/fzf.vim.git $@

bundle/surround:
	git clone https://github.com/tpope/vim-surround.git $@

bundle/beautify-js:
	git clone https://github.com/maksimr/vim-jsbeautify.git $@
	cd $@ && git submodule update --init --recursive

bundle/syntax-json:
	git clone https://github.com/elzr/vim-json.git $@

bundle/syntax-jsx:
	git clone https://github.com/mxw/vim-jsx.git $@

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

bundle/syntax-js:
	git clone https://github.com/othree/yajs.git $@

bundle/syntax-js-libs:
	git clone https://github.com/othree/javascript-libraries-syntax.vim.git $@

bundle/syntax-ansible:
	git clone https://github.com/pearofducks/ansible-vim $@

clean:
	rm -rf bundle/*
