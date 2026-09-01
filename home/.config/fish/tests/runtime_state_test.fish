function fail
    echo "not ok - $argv" >&2
    exit 1
end

set -g runtime_state_test_root (mktemp -d)
test -n "$runtime_state_test_root"; and test -d "$runtime_state_test_root"
or fail "mktemp did not create the test directory"

function cleanup --on-event fish_exit
    /bin/rm -rf -- "$runtime_state_test_root"
    or echo "not ok - failed to remove $runtime_state_test_root" >&2
end

set -l config_root (path resolve (path dirname (status filename))/..)
set -l source_root (path resolve "$config_root/../../..")
set -l repository_root "$runtime_state_test_root/repository"
set -l test_home "$runtime_state_test_root/home"
set -l deployed_config "$repository_root/home/.config/fish"
set -l fish_bin_dir (path dirname (status fish-path))
set -l test_path "$fish_bin_dir:/usr/bin:/bin"

/bin/mkdir -p "$deployed_config/conf.d" "$test_home/.local/bin" "$test_home/.fzf/bin" \
    "$test_home/.bun/bin" "$test_home/.moon/bin"
or fail "could not create the fresh deployment"
/bin/cp "$config_root/config.fish" "$deployed_config/config.fish"
or fail "could not copy config.fish"
/bin/cp "$config_root/conf.d/editor.fish" "$deployed_config/conf.d/editor.fish"
or fail "could not copy editor.fish"
/bin/cp "$source_root/.gitignore" "$repository_root/.gitignore"
or fail "could not copy .gitignore"

git -C "$repository_root" init --quiet
or fail "could not initialize the fixture repository"
git -C "$repository_root" add .
or fail "could not stage the fresh deployment"
git -C "$repository_root" -c user.name=Test -c user.email=test@example.com commit --quiet -m snapshot
or fail "could not commit the fresh deployment"

env -u EDITOR HOME="$test_home" XDG_CONFIG_HOME="$repository_root/home/.config" PATH="$test_path" \
    fish -c '
        test "$EDITOR" = nvim; or exit 12
        set -q -g EDITOR; or exit 13
        not set -q -U EDITOR; or exit 14
        test -z "$fish_greeting"; or exit 15
        contains -- "$HOME/.local/bin" $PATH; or exit 16
        contains -- "$HOME/.fzf/bin" $PATH; or exit 17
        not set -q -U fish_user_paths; or exit 18
        set -U issue_68_runtime_value persisted; or exit 19
    ' >/dev/null 2>/dev/null
or fail "fresh Fish configuration contract failed with status $status"

env -u EDITOR HOME="$test_home" XDG_CONFIG_HOME="$repository_root/home/.config" PATH="$test_path" fish -c \
    'test "$issue_68_runtime_value" = persisted' >/dev/null 2>/dev/null
or fail "universal variable did not persist for the next Fish session"

test -z "$(git -C "$repository_root" status --porcelain)"
or fail "Fish runtime state changed tracked repository content"

echo "ok - Fish runtime state stays local to a fresh deployment"
