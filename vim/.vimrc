" vim:fdm=marker

" Editor {{{
set nocompatible                "Non compatiblity with vi
set encoding=utf-8              "Default encoding
set ttyfast                     "More move while redraw
set lazyredraw                  "No redraw while doing macro
set hidden                      "Buffer are hide when abandoned
set visualbell                  "No sound
set title                       "Change term title
set autoread                    "Reload files changed outside vim
set noswapfile
set clipboard=unnamed           "Use alt to paste in osx
set backspace=indent,eol,start  "Delete w/ insert
let &titleold=getcwd()          "Reset term title when exit vim
set wildmenu                    "Autocomplete filenames
set wildmode=longest:full,list:full
set cursorline                  "Hl the line of the cursor
set showcmd                     "Display cmd
set scrolloff=7                 "Keep 7 lines when scroll (top|bottom)
set timeout timeoutlen=1000 ttimeoutlen=100
" 80 chars limit
if exists("&colorcolumn")
  set colorcolumn=80
endif
" }}}
" Mouse {{{
set mouse=
" }}}
" Statusline {{{
set statusline=[%n]\ %<         "Buffer Number
set statusline+=%<%w%f\ %=%y[%{&ff}] "FileName
set statusline+=[%6c]           "Filetype
set statusline+=[%{printf('%'.strlen(line('$')).'s',line('.'))}/%L]
set statusline+=[%3p%%]
set statusline+=%{'['.(&readonly?'RO':'\ \ ').']'}
set statusline+=%{'['.(&modified?'+':'-').']'}
set laststatus=2                "Always show status bar
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

" Indent with tabs
vmap <Tab> >gv
vmap <S-Tab> <gv
" Keep selection after indent
vnoremap < <gv
vnoremap > >gv
" }}}
" Folds {{{
set foldmethod=syntax
set foldlevelstart=0

" Space to toggle folds.
nnoremap <Space> za
vnoremap <Space> za

" Make zO recursively open whatever fold we're in, even if it's partially open.
nnoremap zO zczO

let g:html_indent_tags = ['p', 'li']
augroup ft_html
    au!
    au FileType html setlocal foldmethod=manual
augroup END

augroup ft_javascript
    au!
    au FileType javascript setlocal foldmethod=marker
    au FileType javascript setlocal foldmarker={,}
augroup END
" }}}
" Files {{{ 
autocmd BufNewFile,BufRead *.vue.php set ft=html
filetype on
filetype plugin on
" }}}
" Windows {{{
" Smart moving between windows r
map <C-j> <C-W>j
map <C-k> <C-W>k
map <C-h> <C-W>h
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

let mapleader = "-"
nnoremap <Tab> %
nnoremap H ^
nnoremap L g_
nnoremap Q <nop>
cnoremap <C-a> <home>
cnoremap <C-e> <end>
nnoremap <leader>h :left<CR>
nnoremap <leader>c :center<CR>
nnoremap <leader>l :right<CR>

" iabbrev </ </<C-x><C-o>
call MakeSpacelessIabbrev('</', '</<C-x><C-o>')
inoremap jk <Esc>
inoremap {<CR>  {<CR>}<Esc>O<Tab>
" }}}

" Plugin:Pathogen {{{
call pathogen#runtime_append_all_bundles()
call pathogen#infect()
" }}}
" Plugin:Syntastic {{{
let g:syntastic_error_symbol='✘'
let g:syntastic_warning_symbol='⚠'
let g:syntastic_style_error_symbol="✗"
let g:syntastic_style_warning_symbol="⚑"
let g:syntastic_php_checkers = ['php', 'phpcs', 'phpmd']
let g:syntastic_javascript_checkers = ['eslint', 'jscs']
let g:syntastic_css_checkers = ['recess']
let g:syntastic_less_checkers = ['recess']

let local_eslint = finddir('node_modules', '.;') . '/.bin/eslint'
if matchstr(local_eslint, "^\/\\w") == ''
    let local_eslint = getcwd() . "/" . local_eslint
endif
if executable(local_eslint)
    let g:syntastic_javascript_eslint_exec = local_eslint
endif

" }}}
" Plugin:Php-cs-fixer {{{
let g:php_cs_fixer_path = "/usr/local/bin/php-cs-fixer"
let g:php_cs_fixer_level = "all"
let g:php_cs_fixer_config = "default"
nnoremap <silent><leader>pcd :call PhpCsFixerFixDirectory()<CR>
nnoremap <silent><leader>pcf :call PhpCsFixerFixFile()<CR>

autocmd BufWritePre *.php %s/\s\+$//ge
autocmd BufWritePre *.yml %s/\s\+$//ge
autocmd BufWritePre *.php %s/if ( /if (/ge
autocmd BufWritePre *.php %s/if(/if (/ge
autocmd BufWritePre *.php %s/,\$/, \$/ge
autocmd BufWritePre *.php %s/foreach(/foreach (/ge
" }}}
" Plugin:matchit {{{
runtime macros/matchit.vim      " Enable jump betwen tags
" }}}
" Plugin:ctrlp {{{
let g:ctrlp_user_command = 'ag --nogroup --nobreak --noheading --nocolor -g "" %s '
" }}}

" Colors {{{
set background=dark
syntax on
let &t_Co=256
color default
" }}}
