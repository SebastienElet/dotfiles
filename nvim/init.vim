" vim:fdm=marker

" Editor {{{
scriptencoding utf-8
set clipboard=unnamed           "Use alt to paste in osx
set cmdheight=1
" }}}
" Mouse {{{
set mouse=
" }}}
" Search {{{
set hlsearch                    "Hihlight matches
set incsearch                   "Incremental searching
set ignorecase                  "Searches are case insisensitive
set smartcase                   " ... unless they contain one capital letter
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
nnoremap <Tab> %
nnoremap <silent> <CR> :nohlsearch<CR>:w<CR>
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
" Plugin:dein {{{
set runtimepath+=~/.config/nvim/bundle/dein/
if dein#load_state('~/.config/nvim/bundle/dein/')
  call dein#begin('~/.config/nvim/bundle/')
  call dein#add('~/.config/nvim/bundle/coc')
  call dein#add('~/.config/nvim/bundle/editorconfig')
  call dein#add('~/.config/nvim/bundle/neoformat')
  call dein#add('~/.config/nvim/bundle/polyglot')
  call dein#add('~/.config/nvim/bundle/theme-oceanic')
  call dein#end()
  call dein#save_state()
endif
" }}}
" Plugin:Coc {{{
nnoremap <silent><C-p> :CocList files<CR>
command! -nargs=0 Prettier :call CocAction('runCommand', 'prettier.formatFile')
" }}}
" Colors {{{
set background=dark
colorscheme OceanicNext
" }}}
