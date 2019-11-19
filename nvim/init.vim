" vim:fdm=marker

" Editor {{{
scriptencoding utf-8
set clipboard=unnamed           "Use alt to paste in osx
set cmdheight=1
" }}}
" Mouse {{{
set mouse=
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
  call dein#add('~/.config/nvim/bundle/denite')
  call dein#add('~/.config/nvim/bundle/editorconfig')
  call dein#add('~/.config/nvim/bundle/neoformat')
  call dein#add('~/.config/nvim/bundle/theme-oceanic')
  call dein#end()
  call dein#save_state()
endif
" }}}
" Plugin:denite {{{
autocmd FileType denite call s:denite_settings()
function! s:denite_settings() abort
  nnoremap <silent><buffer><expr> <CR>
        \ denite#do_map('do_action')
  nnoremap <silent><buffer><expr> <C-v>
        \ denite#do_map('do_action', 'vsplit')
  nnoremap <silent><buffer><expr> d
        \ denite#do_map('do_action', 'delete')
  nnoremap <silent><buffer><expr> p
        \ denite#do_map('do_action', 'preview')
  nnoremap <silent><buffer><expr> <Esc>
        \ denite#do_map('quit')
  nnoremap <silent><buffer><expr> q
        \ denite#do_map('quit')
  nnoremap <silent><buffer><expr> i
        \ denite#do_map('open_filter_buffer')
endfunction
function! Setup_denite_mappings()
  inoremap <silent><buffer><expr> <CR> denite#do_map('do_action')
endfunction
autocmd FileType denite-filter call Setup_denite_mappings()
nnoremap <silent><C-p> :Denite file/rec -start-filter<CR>
call denite#custom#var('file/rec', 'command',
  \ ['rg', '--files'])
" call denite#custom#var('grep', 'command', ['rg', '--threads', '1'])
" call denite#custom#var('grep', 'recursive_opts', [])
" call denite#custom#var('grep', 'final_opts', [])
" call denite#custom#var('grep', 'separator', ['--'])
" call denite#custom#var('grep', 'default_opts',
"  \ ['--vimgrep', '--no-heading'])
" }}}
" Colors {{{
set background=dark
colorscheme OceanicNext
" }}}
