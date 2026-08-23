if test -d /opt/homebrew/bin
    fish_add_path --global --move --path /opt/homebrew/bin
end

if test -d /opt/homebrew/sbin
    fish_add_path --global --move --path /opt/homebrew/sbin
end

if test -d /opt/homebrew/opt/gnu-sed/libexec/gnubin
    fish_add_path --global --move --path /opt/homebrew/opt/gnu-sed/libexec/gnubin
end
