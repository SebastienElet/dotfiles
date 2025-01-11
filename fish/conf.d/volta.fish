# Initialize Volta
if test -d ~/.volta
    export VOLTA_HOME="$HOME/.volta"
    set -x PATH ~/.volta/bin $PATH
end
