set -g project_root (path resolve (path dirname (status filename))/../..)
set -g test_root (mktemp -d)

function fail --argument-names message
    echo "not ok - $message" >&2
    exit 1
end

test -n "$test_root"; and test -d "$test_root"
or fail "mktemp did not create the test directory"

function cleanup --on-event fish_exit
    command rm -rf -- "$test_root"
    or echo "not ok - failed to remove $test_root" >&2
end

function must
    command $argv
    or begin
        set -l rendered_command (string join ' ' -- $argv)
        fail "command failed: $rendered_command"
    end
end

set -gx _ZO_DATA_DIR "$test_root/zoxide"
must mkdir "$test_root/zoxide"

function create_repository --argument-names name
    set -g remote_path "$test_root/$name.git"
    set -g repository_path "$test_root/$name"
    must git init --bare --quiet "$remote_path"
    must git init --quiet --initial-branch=main "$repository_path"
    must git -C "$repository_path" config user.email test@example.com
    must git -C "$repository_path" config user.name Test
    must git -C "$repository_path" remote add origin "$remote_path"
    echo base >"$repository_path/file"
    must git -C "$repository_path" add file
    must git -C "$repository_path" commit --quiet --message base
    must git -C "$repository_path" push --quiet --set-upstream origin main
end

function create_tracked_branch --argument-names branch
    must git -C "$repository_path" switch --quiet --create "$branch"
    echo "$branch" >"$repository_path/file"
    must git -C "$repository_path" commit --quiet --all --message "$branch"
    must git -C "$repository_path" push --quiet --set-upstream origin "$branch"
    must git -C "$repository_path" switch --quiet main
end

set -p fish_function_path "$project_root/fish/functions"

source "$project_root/fish/tests/git_cleanup/branch_test.fish"
source "$project_root/fish/tests/git_cleanup/worktree_test.fish"
source "$project_root/fish/tests/git_cleanup/recovery_test.fish"
source "$project_root/fish/tests/git_cleanup/invocation_test.fish"
