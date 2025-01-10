if status is-interactive
    # Commands to run in interactive sessions can go here
end

# Enable vi bindings
set -g fish_key_bindings fish_vi_key_bindings

# Ensure Starship is installed
if test -n (type -q starship)
    # Enable Starship
    starship init fish | source
else
    echo "Starship is not installed. Please install it from https://starship.rs/"
end
