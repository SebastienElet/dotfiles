function fail
    echo "not ok - $argv" >&2
    exit 1
end

set -g path_test_root (mktemp -d)
test -n "$path_test_root"; and test -d "$path_test_root"
or fail "mktemp did not create the test directory"

function cleanup --on-event fish_exit
    /bin/rm -rf -- "$path_test_root"
    or echo "not ok - failed to remove $path_test_root" >&2
end

set -l config_root (path resolve (path dirname (status filename))/..)
set -gx HOME "$path_test_root"
/bin/mkdir -p "$HOME/.cargo/bin" "$HOME/.volta/bin" "$HOME/.bun/bin" "$HOME/.moon/bin" \
    "$HOME/homebrew/opt/postgresql@16/bin" "$HOME/homebrew/opt/ruby/bin"
or fail "could not create test directories"
set -gx PATH (path dirname (status fish-path))

function brew
    if test "$argv" = "--prefix ruby"
        echo "$HOME/homebrew/opt/ruby"
    else if test "$argv" = --prefix
        echo "$HOME/homebrew"
    else
        return 1
    end
end

function starship
end

function zoxide
end

set -l path_configs homebrew python postgresql ruby rust volta
for pass in 1 2
    for config in $path_configs
        source "$config_root/conf.d/$config.fish"
        or fail "could not source $config.fish"
    end
    source "$config_root/config.fish"
    or fail "could not source config.fish"
end

set -l seen_path
for entry in $PATH
    contains -- "$entry" $seen_path; and fail "duplicate PATH entry: $entry"
    set -a seen_path "$entry"
end

for expected in "$HOME/.cargo/bin" "$HOME/.volta/bin" "$HOME/.bun/bin" "$HOME/.moon/bin" \
    "$HOME/homebrew/opt/postgresql@16/bin" "$HOME/homebrew/opt/ruby/bin"
    contains -- "$expected" $PATH; or fail "missing PATH entry: $expected"
end

echo "ok - PATH entries remain unique after repeated configuration"
