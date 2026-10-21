function fail
    echo "not ok - $argv" >&2
    exit 1
end

set -g codex_test_root (mktemp -d)
test -n "$codex_test_root"; and test -d "$codex_test_root"
or fail "could not create the test directory"

function cleanup --on-event fish_exit
    command rm -rf -- "$codex_test_root"
end

set -l config_root (path resolve (path dirname (status filename))/..)
set -gx HOME "$codex_test_root/home"
mkdir -p "$HOME" "$codex_test_root/bin"
or fail "could not create fixture directories"
printf '%s\n' '#!/usr/bin/env fish' 'printf "%s\n" "TERM=$TERM" $argv' 'exit 17' >"$codex_test_root/bin/codex"
chmod +x "$codex_test_root/bin/codex"
set -gx PATH "$codex_test_root/bin" $PATH
source "$config_root/conf.d/codex.fish"

cd "$codex_test_root"
set -l output (codex exec 'keep this prompt')
test $status -eq 17; or fail "outside Git: exit status changed"
test (count $output) -eq 3; or fail "outside Git: argument boundaries changed"
test "$output" = 'TERM=dumb exec keep this prompt'; or fail "outside Git: arguments changed"

set -l repository "$codex_test_root/main repository"
git init --quiet --initial-branch=main "$repository"; or fail "git init failed"
git -C "$repository" -c core.hooksPath=/dev/null -c user.name=Test -c user.email=test@example.com commit --quiet --allow-empty -m base
or fail "initial commit failed"
set -l worktree "$codex_test_root/linked worktree"
git -C "$repository" -c core.hooksPath=/dev/null worktree add --quiet -b linked "$worktree"
or fail "git worktree add failed"

cd "$repository"
set -l common_directory (path resolve "$repository/.git")
set output (codex)
test $status -eq 17; or fail "main checkout: exit status changed"
test (count $output) -eq 3; or fail "main checkout: argument boundaries changed"
test "$output" = "TERM=dumb --add-dir $common_directory"; or fail "main checkout: Git directory was missing or duplicated"

cd "$worktree"
mkdir nested
cd nested
set -l administrative_directory "$common_directory/worktrees/linked-worktree"
test -d "$administrative_directory"; or fail "unexpected worktree fixture layout"
set -l roots --add-dir "$common_directory" --add-dir "$administrative_directory"

set output (codex -m example 'prompt with spaces')
test (count $output) -eq 8; or fail "TUI prompt: argument boundaries changed"
test "$output" = "TERM=dumb $roots -m example prompt with spaces"; or fail "TUI prompt: arguments changed"

for subcommand in exec e resume fork
    set output (codex $subcommand 'prompt with spaces')
    test $status -eq 17; or fail "$subcommand: exit status changed"
    test (count $output) -eq 7; or fail "$subcommand: argument boundaries changed"
    test "$output" = "TERM=dumb $subcommand $roots prompt with spaces"; or fail "$subcommand: Git directories or arguments changed"
end

set output (codex -c model=example --no-daemon exec --ephemeral 'prompt with spaces')
test "$output" = "TERM=dumb -c model=example --no-daemon exec $roots --ephemeral prompt with spaces"
or fail "options before exec: Git directories or arguments changed"

for subcommand in resume fork review
    set output (codex exec $subcommand --help)
    test "$output" = "TERM=dumb exec $roots $subcommand --help"; or fail "exec $subcommand: directories were not passed to the parent"
end

for subcommand in agents review login logout mcp plugin app-server remote-control app completion update doctor sandbox debug apply a queue archive delete migrate-rollouts unarchive cloud exec-server features help
    set output (codex $subcommand --help)
    test "$output" = "TERM=dumb $subcommand --help"; or fail "$subcommand: unsupported options were injected"
end

set output (codex -c exec doctor --help)
test "$output" = 'TERM=dumb -c exec doctor --help'; or fail "option value was mistaken for a subcommand"

set output (codex -- 'exec')
test "$output" = "TERM=dumb $roots -- exec"; or fail "explicit prompt separator was not preserved"

for shell_directory in "$repository" "$codex_test_root"
    cd "$shell_directory"
    for option in -C --cd
        set output (codex $option "$worktree" exec prompt)
        test $status -eq 17; or fail "$option before exec: exit status changed"
        test (count $output) -eq 9; or fail "$option before exec: argument boundaries changed"
        test "$output" = "TERM=dumb $option $worktree exec $roots prompt"
        or fail "$option before exec: target Git directories were not used"

        set output (codex exec $option "$worktree" prompt)
        test $status -eq 17; or fail "$option after exec: exit status changed"
        test (count $output) -eq 9; or fail "$option after exec: argument boundaries changed"
        test "$output" = "TERM=dumb exec $roots $option $worktree prompt"
        or fail "$option after exec: target Git directories were not used"
    end

    set output (codex exec --cd="$worktree" prompt)
    test "$output" = "TERM=dumb exec $roots --cd=$worktree prompt"
    or fail "--cd=: target Git directories were not used"
end

set output (codex exec -C 'linked worktree' prompt)
test "$output" = "TERM=dumb exec $roots -C linked worktree prompt"
or fail "relative -C: target Git directories were not used"

cd "$repository"
for destination in "$codex_test_root" "$codex_test_root/missing" ''
    set output (codex exec --cd "$destination" prompt)
    test $status -eq 17; or fail "non-Git destination: exit status changed"
    test (count $output) -eq 5; or fail "non-Git destination: argument boundaries changed"
    test "$output" = "TERM=dumb exec --cd $destination prompt"
    or fail "non-Git destination: shell Git directories were injected"
end

set output (codex exec -C)
test $status -eq 17; or fail "missing -C value: exit status changed"
test "$output" = 'TERM=dumb exec -C'; or fail "missing -C value: arguments changed"

set output (codex -- --cd "$worktree")
test "$output" = "TERM=dumb --add-dir $common_directory -- --cd $worktree"
or fail "prompt separator: --cd was interpreted as an option"

mkdir "$codex_test_root/broken"
printf 'gitdir: missing\n' >"$codex_test_root/broken/.git"
cd "$codex_test_root/broken"
set output (codex exec prompt)
test $status -eq 17; or fail "broken Git metadata: exit status changed"
test "$output" = 'TERM=dumb exec prompt'; or fail "broken Git metadata: arguments changed"

echo "ok - Codex receives Git metadata directories only for session commands"
