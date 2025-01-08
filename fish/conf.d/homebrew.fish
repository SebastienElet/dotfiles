if test -d /opt/homebrew/bin
    set -gx PATH /opt/homebrew/bin $PATH
end

if test -d /opt/homebrew/sbin
    set -gx PATH /opt/homebrew/sbin $PATH
end

if test -d /opt/homebrew/opt/gnu-sed/libexec/gnubin
    set -gx PATH /opt/homebrew/opt/gnu-sed/libexec/gnubin $PATH
end
