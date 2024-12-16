# Homebrew
[[ -d /opt/homebrew/bin ]] && export PATH=/opt/homebrew/bin:$PATH
[[ -d /opt/homebrew/sbin ]] && export PATH=/opt/homebrew/sbin:$PATH
[[ -d /opt/homebrew/opt/gnu-sed/libexec/gnubin ]] && export PATH="/opt/homebrew/opt/gnu-sed/libexec/gnubin:$PATH"

# Alias
alias upgrade=~/.dotfiles/zsh/bin/upgrade
alias ..='cd ..'
alias :q='exit'
[[ ! -z `command -v nvim` ]] && alias vim='nvim'
[[ ! -z `command -v nvim` ]] && alias v='nvim'
[[ ! -z `command -v nvim` ]] && alias n='nvim'
[[ ! -z `command -v tokei` ]] && alias loc='tokei'
[[ ! -z `command -v eza` ]] && alias ls='eza'
alias t='tmux'
alias tm='tmux'
[[ ! -z `command -v htop` ]] && alias top='htop'

# Oh-my-zsh git aliases
alias g='git'

source ~/.dotfiles/zsh/bin/git_main_branch
source ~/.dotfiles/zsh/bin/git_hook_push

alias '??'='ghcs'
alias ga='git add'
alias gaa='git add --all'
alias gap='git apply'
alias gapa='git add --patch'
alias gapt='git apply --3way'
alias gau='git add --update'
alias gav='git add --verbose'
alias gb='git branch'
alias gbD='git branch -D'
alias gba='git branch -a'
alias gbd='git branch -d'
alias gbda='git branch --no-color --merged | command grep -vE "^([+*]|\s*($(git_main_branch)|$(git_develop_branch))\s*$)" | command xargs git branch -d 2>/dev/null'
alias gbl='git blame -b -w'
alias gbnm='git branch --no-merged'
alias gbr='git branch --remote'
alias gbs='git bisect'
alias gbsb='git bisect bad'
alias gbsg='git bisect good'
alias gbsr='git bisect reset'
alias gbss='git bisect start'
alias gc!='git commit -v --amend'
alias gc='git commit -v'
alias gca!='git commit -v -a --amend'
alias gca='git commit -v -a'
alias gcam='git commit -a -m'
alias gcan!='git commit -v -a --no-edit --amend'
alias gcans!='git commit -v -a -s --no-edit --amend'
alias gcas='git commit -a -s'
alias gcasm='git commit -a -s -m'
alias gcb='git checkout -b'
alias gcf='git config --list'
alias gcn!='git commit -v --no-edit --amend'
alias gco='git checkout'
alias gcsm='git commit -s -m'
alias gp='git_hook_push && git push'
alias gpsup='git push --set-upstream origin $(git branch --show-current)'
alias grbc='git rebase --continue'
alias grbm='git rebase $(git_main_branch)'
alias gs='git status'
alias oc='OCO_AI_PROVIDER="ollama" OCO_MODEL=mistral OCO_LOCAL_MODEL_LLAMA=mistral opencommit'

# Vim
bindkey -v

# History
export HISTSIZE=10000     # Set history size
export SAVEHIST=10000     # Save history after logo
setopt INC_APPEND_HISTORY # Append into history file
setopt HIST_IGNORE_DUPS   # Save only one command if 2 common are same and consistent
setopt EXTENDED_HISTORY   # Add timestamp for each entry

# Starship
eval "$(starship init zsh)"

# Volta
[[ -d $HOME/.volta ]] && export VOLTA_HOME="$HOME/.volta" && export PATH="$VOLTA_HOME/bin:$PATH"

# Krew
[[ -d $HOME/.krew ]] && export PATH="$HOME/.krew/bin:$PATH"

# Setup default editor
[[ ! -z `command -v nvim` ]] && export EDITOR=nvim

# Setup python
[[ ! -z `brew --prefix python` ]] && export PATH="$(brew --prefix python)/libexec/bin:$PATH"

# Setup ruby
[[ ! -z `brew --prefix ruby` ]] && export PATH="$(brew --prefix ruby)/bin:$PATH"

# Auto suggestion
[[ -f ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh ]] && source ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh
# Syntax
[[ -f ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]] && source ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
[[ -d ~/.zsh/zsh-completions ]] && fpath=(~/.zsh/zsh-completions $fpath)

# Fzf
[ -f ~/.fzf.zsh ] && source ~/.fzf.zsh

# Copilot
[[ ! -z `command -v gh copilot` ]] && eval "$(gh copilot alias -- zsh)"

# Broot
[ -f ~/.config/broot/launcher/bash/br ] && source ~/.config/broot/launcher/bash/br
