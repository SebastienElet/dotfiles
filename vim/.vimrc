" vim:fdm=marker

" Editor {{{
set nocompatible                "Non compatiblity with vi
set clipboard=unnamed           "Use alt to paste in osx
set backspace=indent,eol,start  "Delete w/ insert
set noswapfile                  "No swap file !
" }}}
" Mouse {{{
set mouse=
" }}}
" Statusline {{{
" }}}
" Search {{{
set hlsearch                    "Hihlight matches
set incsearch                   "Incremental searching
set ignorecase                  "Searches are case insisensitive
set smartcase                   " ... unless they contain one capital letter

" Center screen on occurence
nnoremap n nzz
nnoremap N Nzz
" clear the search buffer when hitting return
nnoremap <silent> <CR> :nohlsearch<CR>:w<CR>
" }}}
" Tabs & Indent {{{
set autoindent
set expandtab                   "Convert tab -> spacei
set tabstop=2                   "A tab = 2 spaces
set shiftwidth=2
set softtabstop=2
set copyindent
set smarttab
set list listchars=tab:→\ ,trail:·,eol:↩,extends:»,precedes:«,nbsp:×
set showbreak=…

" Indent with tabs
vmap <Tab> >gv
vmap <S-Tab> <gv
" Keep selection after indent
vnoremap < <gv
vnoremap > >gv
" }}}
" Folds {{{
set foldmethod=syntax
set foldlevelstart=1
" }}}
" Space to toggle folds.
nnoremap <Space> za
vnoremap <Space> za
" Make zO recursively open whatever fold we're in, even if it's partially open.
nnoremap zO zczO
" }}}
" Files {{{ 
filetype on
filetype plugin on
" }}}
" Windows {{{
" Smart moving between windows r
map <C-j> <C-W>j
map <C-k> <C-W>k
map <C-h> <C-W>h
if has('nvim')
  nmap <BS> <C-W>h
endif
map <C-l> <C-W>l
autocmd VimResized * tabdo wincmd =
" }}}
" Diff {{{
if &diff
  set diffopt+=iwhite
endif
" }}}
" Hard mode {{{
noremap <Up> <nop>
noremap <Down> <nop>
noremap <Left> <nop>
noremap <Right> <nop>
inoremap <up> <nop>
inoremap <down> <nop>
inoremap <left> <nop>
inoremap <right> <nop>
" }}}
" Binds {{{
function! EatChar(pat)
  let c = nr2char(getchar(0))
  return (c =~ a:pat) ? '' : c
endfunction
function! MakeSpacelessIabbrev(from, to)
  execute "iabbrev <silent> ".a:from." ".a:to."<C-R>=EatChar('\\s')<CR>"
endfunction

let g:mapleader = "-"
nnoremap <Tab> %
nnoremap H ^
nnoremap L g_
" }}}

" Plugin:dein {{{
set runtimepath+=~/.dotfiles/vim/bundle/dein/
if dein#load_state('~/.vim/bundle/dein/')
  call dein#begin('~/.vim/bundle/')

  call dein#add('~/.vim/bundle/editorconfig')
  call dein#add('~/.vim/bundle/fugitive')
  call dein#add('~/.vim/bundle/fzf')
  call dein#add('~/.vim/bundle/neoformat')
  call dein#add('~/.vim/bundle/neomake')
  call dein#add('~/.vim/bundle/syntax-ansible')
  call dein#add('~/.vim/bundle/syntax-js')
  call dein#add('~/.vim/bundle/syntax-json')
  call dein#add('~/.vim/bundle/syntax-jsx')
  call dein#add('~/.vim/bundle/syntax-less')
  call dein#add('~/.vim/bundle/syntax-markdown')
  call dein#add('~/.vim/bundle/syntax-rust')
  call dein#add('~/.vim/bundle/syntax-terraform')
  call dein#add('~/.vim/bundle/syntax-ts')

  call dein#end()
  call dein#save_state()
endif
" }}}
" Plugin:matchit {{{
runtime macros/matchit.vim      " Enable jump betwen tags
" }}}
" Plugin:neoformat {{{
autocmd FileType javascript setlocal formatprg=npx\ prettier
  \\ --stdin
  \\ --parser\ flow
  \\ --single-quote
  \\ --trailing-comma\ es5
  \\ --print-width\ 80
let g:neoformat_try_formatprg = 1
" }}}
" Public:neomake {{{
let g:local_eslint = finddir('node_modules', '.;') . '/.bin/eslint'
if matchstr(local_eslint, "^\/\\w") == ''
  let g:local_eslint = getcwd() . "/" . local_eslint
endif
if executable(local_eslint)
  let g:neomake_javascript_eslint_exe = local_eslint
endif
autocmd! BufWritePost * Neomake
" }}}
" Plugin:neoformat {{{
" }}}
" Plugin:fzf {{{
set rtp+=~/.fzf
noremap <silent><C-p> :FZF<CR>
cabbrev ls <c-r>=(getcmdtype()==':' && getcmdpos()==1 ? 'Buffers' : 'ls')<CR>
" }}}
" Autocmd {{{
autocmd BufRead *.mjs set filetype=javascript
" autocmd BufWritePre *.js Neoformat
" }}}
" Colors {{{
set background=dark
syntax on
let &t_Co=256
color default
colorscheme desert
set t_ZH=[3m
set t_ZR=[23m
highlight Comment cterm=italic
" }}}
