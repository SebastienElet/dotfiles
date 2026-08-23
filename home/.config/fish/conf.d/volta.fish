# Initialize Volta
if test -d ~/.volta
    export VOLTA_HOME="$HOME/.volta"
    fish_add_path --global --move --path ~/.volta/bin
end
