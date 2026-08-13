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

source "$project_root/fish/conf.d/git-wrapper.fish"
or fail "could not source fish/conf.d/git-wrapper.fish"

create_repository divergent-branch
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo local >>"$repository_path/file"
must git -C "$repository_path" commit --quiet --all --message local
must git -C "$repository_path" switch --quiet main
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
git fetch
or fail "fetch failed in divergent-branch"

command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves a branch with local commits"

echo "ok - fetch preserves a branch with local commits"

create_repository clean-worktree
create_tracked_branch feature
set -l worktree_path "$test_root/clean-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
git fetch
or fail "fetch failed in clean-worktree"

not test -e "$worktree_path"
or fail "fetch removes a clean worktree without local commits"
not command git show-ref --verify --quiet refs/heads/feature
or fail "fetch removes a branch without local commits"

echo "ok - fetch removes a clean worktree and branch without local commits"

create_repository dirty-worktree
create_tracked_branch feature
set worktree_path "$test_root/dirty-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
echo dirty >>"$worktree_path/file"
echo untracked >"$worktree_path/untracked"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set -l fetch_output (git fetch 2>&1)
or fail "fetch failed in dirty-worktree"

test -e "$worktree_path"
or fail "fetch preserves a dirty worktree"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves the branch of a dirty worktree"
string match --quiet -- '*dirty*' (command git -C "$worktree_path" diff)
or fail "fetch preserves uncommitted worktree changes"
test -e "$worktree_path/untracked"
or fail "fetch preserves untracked worktree files"
string match --quiet -- '*worktree contains local files or changes*' $fetch_output
or fail "fetch reports why a dirty worktree is preserved"

echo "ok - fetch preserves a dirty worktree and branch"

create_repository ignored-worktree
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo ignored >"$repository_path/.gitignore"
must git -C "$repository_path" add .gitignore
must git -C "$repository_path" commit --quiet --message ignore
must git -C "$repository_path" push --quiet
must git -C "$repository_path" switch --quiet main
set worktree_path "$test_root/ignored-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
echo local >"$worktree_path/ignored"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in ignored-worktree"

test -e "$worktree_path/ignored"
or fail "fetch preserves ignored worktree files"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves the branch of a worktree with ignored files"

echo "ok - fetch preserves a worktree with ignored files"

create_repository assume-unchanged-worktree
create_tracked_branch feature
set worktree_path "$test_root/assume-unchanged-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
must git -C "$worktree_path" update-index --assume-unchanged file
echo local >>"$worktree_path/file"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in assume-unchanged-worktree"

test -e "$worktree_path/file"
or fail "fetch preserves assume-unchanged worktree changes"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves the branch of an assume-unchanged worktree"

echo "ok - fetch preserves an assume-unchanged worktree"

create_repository skip-worktree
create_tracked_branch feature
set worktree_path "$test_root/skip-worktree-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
must git -C "$worktree_path" update-index --skip-worktree file
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in skip-worktree"

test -e "$worktree_path/file"
or fail "fetch preserves a skip-worktree worktree"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves the branch of a skip-worktree worktree"

echo "ok - fetch preserves a skip-worktree worktree"

create_repository multiple-worktrees
create_tracked_branch feature
set -l first_worktree_path "$test_root/first-feature"
set -l second_worktree_path "$test_root/second-feature"
must git -C "$repository_path" worktree add --quiet "$first_worktree_path" feature
must git -C "$repository_path" worktree add --quiet --force "$second_worktree_path" feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in multiple-worktrees"

test -e "$first_worktree_path/file"
or fail "fetch preserves the first of multiple worktrees"
test -e "$second_worktree_path/file"
or fail "fetch preserves the second of multiple worktrees"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves a branch checked out in multiple worktrees"

echo "ok - fetch preserves a branch checked out in multiple worktrees"

create_repository failed-fetch
create_tracked_branch feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set -l failed_fetch_output (git fetch missing 2>&1)
set -l fetch_status $status

test $fetch_status -ne 0
or fail "fetch preserves the original failure status, got $fetch_status"
string match --quiet -- '*does not appear to be a git repository*' $failed_fetch_output
or fail "failed fetch keeps its error visible"
command git show-ref --verify --quiet refs/heads/feature
or fail "failed fetch does not clean branches"

echo "ok - failed fetch preserves its status and branches"

create_repository dry-run
create_tracked_branch feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
git fetch --dry-run
or fail "dry-run fetch failed"

command git show-ref --verify --quiet refs/heads/feature
or fail "dry-run fetch does not clean branches"
command git show-ref --verify --quiet refs/remotes/origin/feature
or fail "dry-run fetch does not prune remote-tracking branches"

echo "ok - dry-run fetch leaves branches unchanged"

create_repository unavailable-upstream
create_tracked_branch feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature
must git -C "$repository_path" update-ref -d refs/remotes/origin/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in unavailable-upstream"

command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves a branch when its previous upstream tip is unavailable"
string match --quiet -- '*upstream tip unavailable*' $fetch_output
or fail "fetch reports an unavailable previous upstream tip"

echo "ok - unavailable upstream tip preserves the branch"

create_repository single-fetch
create_tracked_branch feature
must git -C "$repository_path" remote add backup "$remote_path"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature
must git -C "$repository_path" remote set-url origin "$test_root/missing.git"

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch backup 2>&1)
set fetch_status $status

test $fetch_status -eq 0
or fail "fetching another remote succeeds, got $fetch_status"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetching another remote preserves branches tracking origin"
not string match --quiet -- '*does not appear to be a git repository*' $fetch_output
or fail "fetch does not contact an unrequested remote"

echo "ok - fetch contacts only the requested remote"

create_repository pull
create_tracked_branch feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
git pull --ff-only
or fail "pull failed"

not command git show-ref --verify --quiet refs/heads/feature
or fail "pull removes a branch without local commits"

echo "ok - pull removes a branch without local commits"

create_repository global-options
create_tracked_branch feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$test_root"
or fail "could not enter $test_root"
git --no-pager -C "$repository_path" fetch
or fail "fetch with global options failed"

not command git -C "$repository_path" show-ref --verify --quiet refs/heads/feature
or fail "fetch with global options removes a branch without local commits"

echo "ok - fetch with global options removes a branch without local commits"

for primary_branch in master trunk
    create_repository "primary-$primary_branch"
    must git -C "$repository_path" branch --move "$primary_branch"
    must git -C "$repository_path" push --quiet --set-upstream origin "$primary_branch"
    must git -C "$repository_path" switch --quiet --create parking
    must git --git-dir="$remote_path" update-ref -d "refs/heads/$primary_branch"

    cd "$repository_path"
    or fail "could not enter $repository_path"
    git fetch
    or fail "fetch failed for primary branch $primary_branch"

    command git show-ref --verify --quiet "refs/heads/$primary_branch"
    or fail "fetch preserves primary branch $primary_branch"
end

echo "ok - fetch preserves dynamically detected primary branches"
