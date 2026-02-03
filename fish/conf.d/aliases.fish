alias ..='cd ..'
alias :q='exit'
alias cheque-parser-instance-reboot='~/.dotfiles/scripts/cheque-parser-instance-reboot'
# Git aliases have been moved to fish/conf.d/git-abbreviations.fish
# Git abbreviations are preferred over aliases in Fish shell
# Special git aliases that need to stay as aliases:
alias gp='~/.dotfiles/scripts/git_hook_push; and git push'
alias gpsup='git push --set-upstream origin (git branch --show-current)'
alias import-instance-start='~/.dotfiles/scripts/import-instance-start'
alias import-instance-stop='~/.dotfiles/scripts/import-instance-stop'
alias oc='OCO_AI_PROVIDER="ollama" OCO_MODEL=mistral OCO_LOCAL_MODEL_LLAMA=mistral opencommit'
alias t='tmux'
alias tm='tmux'
alias upgrade='~/.dotfiles/scripts/upgrade'
alias mcp_edit='~/.dotfiles/scripts/mcp_edit'

if type -q nvim
    alias vim='nvim'
    alias v='nvim'
    alias n='nvim'
end

if type -q cursor
    alias c='cursor'
end

if type -q tokei
    alias loc='tokei'
end

if type -q procs
    alias ps='procs'
end

if type -q eza
    alias ls='eza'
end
