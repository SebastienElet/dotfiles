## Environment {{{
export PATH=/usr/local/Cellar/php56/5.6.2/bin/:$PATH
export PATH=/usr/local/bin:/usr/local/sbin:/usr/local/share/npm/bin:$PATH
## }}}

## Modules {{{
autoload -Uz compinit && compinit
autoload -Uz colors && colors
## }}}

## Completion {{{
zstyle ':completion:*:sudo:*' command-path /usr/local/sbin /usr/local/bin \
                                           /usr/sbin /usr/bin /sbin /bin \
                                           /usr/local/share/npm/bin
zstyle ':completion:*' completer _expand _complete _correct _approximate
zstyle ':completion:*' verbose yes
zstyle ':completion:*:descriptions': format '%B%d%b'
zstyle ':completion:*:messages' format '%d'
zstyle ':completion:*:warnings' format 'No matches for: %d.'
zstyle ':completion:*:corrections' format '%B%d (errors: %e)%b.'
zstyle ':completion:*' group-name ''
zstyle ':completion:*' insert-unambiguous false
zstyle ':completion:*' list-colors ''
zstyle ':completion:*' list-prompt %SAt %p: Hit TAB for more, or the character \
                                   to insert%s
zstyle ':completion:*' select-prompt %SScrolling active: current selection at \
                                   %p%s
zstyle ':completion:*' menu select=1
zstyle ':completion:*' original true
## }}}

## History {{{
export HISTSIZE=SAVEHIST=4096
export HISTFILE=~/.zsh_history
setopt hist_ignore_all_dups
setopt append_history
setopt extended_history
## }}}

## Prompt {{{
POMPT_BEFORE="
"
PROMPT_STATUS="%(?.%{$fg[green]%}✔.%{$fg[red]%}✖) "
PROMPT_HOST="%{$fg_bold[grey]%}%n@%m "
PROMPT_PWD="%{$fg[black]%}%{$fg[blue]%} %20<… <%/%<< %{$fg[blue]%}❯ "

export PROMPT=$PROMPT_BEFORE$PROMPT_STATUS$PROMPT_PWD
export RPROMPT="%{$fg_bold[grey]%}%n@%m%{$reset_color%}"
## }}}

### Misc options {{{
# Silent rm *
setopt rmstarsilent
# Vim bindings
bindkey -v
## }}}

# Load alias
source ~/.dotfiles/zsh/alias
# Load syntax hl
source ~/.dotfiles/zsh/zsh-syntax-highlighting.zsh

export FZF_DEFAULT_COMMAND='ag -l -g ""'
[ -f ~/.fzf.zsh ] && source ~/.fzf.zsh

fd() {
  local dir
  dir=$(find ${1:-*} -path '*/\.*' -prune \
                  -o -type d -print 2> /dev/null | fzf +m) &&
  cd "$dir"
}
