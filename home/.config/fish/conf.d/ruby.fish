if test -n (brew --prefix ruby)
    fish_add_path --global --move --path (brew --prefix ruby)/bin
end
