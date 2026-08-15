alias ..='cd ..'
alias :q='exit'
# Git aliases have been moved to home/.config/fish/conf.d/git-abbreviations.fish
# Git abbreviations are preferred over aliases in Fish shell
alias gpsup='git push --set-upstream origin (git branch --show-current)'
alias oc='OCO_AI_PROVIDER="ollama" OCO_MODEL=mistral OCO_LOCAL_MODEL_LLAMA=mistral opencommit'
alias t='tmux'
alias tm='tmux'
alias upgrade='~/.dotfiles/tooling/upgrade'
alias mcp_edit='~/.dotfiles/tooling/mcp-edit'

if type -q nvim
    alias vim='nvim'
    alias v='nvim'
    alias n='nvim'
end

if type -q claude
    alias c='claude'
end

if type -q codex
    alias co='codex'
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
