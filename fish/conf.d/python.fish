# Python via Homebrew (évite l'appel lent à `brew --prefix`)
# Apple Silicon
if test -d /opt/homebrew/opt/python/libexec/bin
    set -gx PATH /opt/homebrew/opt/python/libexec/bin $PATH
# Intel Mac
else if test -d /usr/local/opt/python/libexec/bin
    set -gx PATH /usr/local/opt/python/libexec/bin $PATH
end
