# Alias
alias upgrade=~/.dotfiles/zsh/bin/upgrade
alias ..='cd ..'
alias :q='exit'
[[ ! -z `command -v nvim` ]] && alias vim='nvim'
[[ ! -z `command -v nvim` ]] && alias v='nvim'
[[ ! -z `command -v nvim` ]] && alias n='nvim'
[[ ! -z `command -v tokei` ]] && alias loc='tokei'
[[ ! -z `command -v exa` ]] && alias ls='exa'
alias t='tmux'
alias tm='tmux'
[[ ! -z `command -v htop` ]] && alias top='htop'

# Oh-my-zsh git aliases
alias g='git'

function git_main_branch() {
  command git rev-parse --git-dir &>/dev/null || return
  local ref
  for ref in refs/{heads,remotes/{origin,upstream}}/{main,trunk}; do
    if command git show-ref -q --verify $ref; then
      echo ${ref:t}
      return
    fi
  done
  echo master
}

alias gpsup='git push --set-upstream origin $(git branch --show-current)'
alias grbm='git rebase $(git_main_branch)'
alias gco='git checkout'

alias gs='git status'
alias ga='git add'
alias gaa='git add --all'
alias gapa='git add --patch'
alias gau='git add --update'
alias gav='git add --verbose'
alias gap='git apply'
alias gapt='git apply --3way'

alias gb='git branch'
alias gba='git branch -a'
alias gbd='git branch -d'
alias gbda='git branch --no-color --merged | command grep -vE "^([+*]|\s*($(git_main_branch)|$(git_develop_branch))\s*$)" | command xargs git branch -d 2>/dev/null'
alias gbD='git branch -D'
alias gbl='git blame -b -w'
alias gbnm='git branch --no-merged'
alias gbr='git branch --remote'
alias gbs='git bisect'
alias gbsb='git bisect bad'
alias gbsg='git bisect good'
alias gbsr='git bisect reset'
alias gbss='git bisect start'

alias gc='git commit -v'
alias gc!='git commit -v --amend'
alias gcn!='git commit -v --no-edit --amend'
alias gca='git commit -v -a'
alias gca!='git commit -v -a --amend'
alias gcan!='git commit -v -a --no-edit --amend'
alias gcans!='git commit -v -a -s --no-edit --amend'
alias gcam='git commit -a -m'
alias gcsm='git commit -s -m'
alias gcas='git commit -a -s'
alias gcasm='git commit -a -s -m'
alias gcb='git checkout -b'
alias gcf='git config --list'

# Vim
bindkey -v

# Homebrew
[[ -d /opt/homebrew/bin ]] && export PATH=/opt/homebrew/bin:$PATH
[[ -d /opt/homebrew/sbin ]] && export PATH=/opt/homebrew/sbin:$PATH

# Starship
eval "$(starship init zsh)"

# Volta
[[ -d $HOME/.volta ]] && export PATH=$HOME/.volta/bin:$PATH

# Setup default editor
[[ ! -z `command -v nvim` ]] && export EDITOR=nvim

# Auto suggestion
[[ -f ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh ]] && source ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh
# Syntax
[[ -f ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]] && source ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
[[ -d ~/.zsh/zsh-completions ]] && fpath=(~/.zsh/zsh-completions $fpath)

# Fzf
[ -f ~/.fzf.zsh ] && source ~/.fzf.zsh

# Broot
[ -f ~/.config/broot/launcher/bash/br ] && source ~/.config/broot/launcher/bash/br
