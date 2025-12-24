if status is-interactive
    # Commands to run in interactive sessions can go here
end

# Enable vi bindings
set -g fish_key_bindings fish_vi_key_bindings
# Disable welcome message
set fish_greeting

# Ensure Starship is installed
if test -n (type -q starship)
    # Enable Starship
    starship init fish | source
else
    echo "Starship is not installed. Please install it from https://starship.rs/"
end

# bun
set --export BUN_INSTALL "$HOME/.bun"
set --export PATH $BUN_INSTALL/bin $PATH

# Added by OrbStack: command-line tools and integration
# This won't be added again if you remove it.
if test -f ~/.orbstack/shell/init2.fish
    source ~/.orbstack/shell/init2.fish
end
