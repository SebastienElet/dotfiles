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
" Setup python
" brew install python
" python -m pip install --user --upgrade pynvim

let g:coc_global_extensions = [
  \ 'coc-sh',
  \ 'coc-css',
  \ 'coc-git',
  \ 'coc-sql',
  \ 'coc-json',
  \ 'coc-html',
  \ 'coc-vimlsp',
  \ 'coc-lists',
  \ 'coc-snippets',
  \ 'coc-tsserver',
  \ 'coc-prettier',
\ ]
nnoremap <silent><C-p> :CocList files<CR>
command! -nargs=0 Prettier :call CocAction('runCommand', 'prettier.formatFile')

" Use <c-space> to trigger completion.
if has('nvim')
  inoremap <silent><expr> <c-space> coc#refresh()
else
  inoremap <silent><expr> <c-@> coc#refresh()
endif

" GoTo code navigation.
nmap <silent> gd <Plug>(coc-definition)
nmap <silent> gy <Plug>(coc-type-definition)
nmap <silent> gi <Plug>(coc-implementation)
nmap <silent> gr <Plug>(coc-references)

" Use K to show documentation in preview window.
nnoremap <silent> K :call <SID>show_documentation()<CR>

function! s:show_documentation()
  if (index(['vim','help'], &filetype) >= 0)
    execute 'h '.expand('<cword>')
  else
    call CocActionAsync('doHover')
  endif
endfunction

" Highlight the symbol and its references when holding the cursor.
autocmd CursorHold * silent call CocActionAsync('highlight')

" Symbol renaming.
nmap <leader>rn <Plug>(coc-rename)

" Formatting selected code.
xmap <leader>f  <Plug>(coc-format-selected)
nmap <leader>f  <Plug>(coc-format-selected)

" Use <C-l> for trigger snippet expand.
imap <C-l> <Plug>(coc-snippets-expand)

" Use <C-j> for select text for visual placeholder of snippet.
vmap <C-j> <Plug>(coc-snippets-select)

" Use <C-j> for jump to next placeholder, it's default of coc.nvim
let g:coc_snippet_next = '<c-j>'

" Use <C-k> for jump to previous placeholder, it's default of coc.nvim
let g:coc_snippet_prev = '<c-k>'

" Use <C-j> for both expand and jump (make expand higher priority.)
imap <C-j> <Plug>(coc-snippets-expand-jump)

" Use <leader>x for convert visual selected code to snippet
xmap <leader>x  <Plug>(coc-convert-snippet)
" }}}
" Colors {{{
set background=dark
colorscheme OceanicNext
" }}}
