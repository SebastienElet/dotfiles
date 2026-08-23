# Apple Silicon
if test -d /opt/homebrew/opt/python/libexec/bin
    fish_add_path --global --move --path /opt/homebrew/opt/python/libexec/bin
    # Intel Mac
else if test -d /usr/local/opt/python/libexec/bin
    fish_add_path --global --move --path /usr/local/opt/python/libexec/bin
end
