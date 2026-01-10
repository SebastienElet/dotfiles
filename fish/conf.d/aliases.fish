alias ..='cd ..'
alias :q='exit'
alias abbr='abbreviations'
alias cheque-parser-instance-reboot='~/.dotfiles/scripts/cheque-parser-instance-reboot'
alias import-instance-start='~/.dotfiles/scripts/import-instance-start'
alias import-instance-stop='~/.dotfiles/scripts/import-instance-stop'
alias oc='OCO_AI_PROVIDER="ollama" OCO_MODEL=mistral OCO_LOCAL_MODEL_LLAMA=mistral opencommit'
alias t='tmux'
alias tm='tmux'
alias upgrade='~/.dotfiles/scripts/upgrade'

if test -n (type -q nvim)
    alias vim='nvim'
    alias v='nvim'
    alias n='nvim'
end

if test -n (type -q cursor)
    alias c='cursor'
end

if test -n (type -q tokei)
    alias loc='tokei'
end

if test -n (type -q eza)
    alias ls='eza'
end
