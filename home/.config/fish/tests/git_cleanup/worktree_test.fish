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
command git config --get branch.feature.cleanup-base >/dev/null
or fail "fetch records the upstream tip for a cleanup retry"

echo "ok - fetch preserves a dirty worktree and branch"

must git -C "$worktree_path" restore file
must rm "$worktree_path/untracked"
git fetch
or fail "second fetch failed in dirty-worktree"

not test -e "$worktree_path"
or fail "a later fetch removes the cleaned worktree"
not command git show-ref --verify --quiet refs/heads/feature
or fail "a later fetch removes the branch using its recorded upstream tip"

echo "ok - fetch retries cleanup after a worktree becomes clean"

create_repository ignored-worktree
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo ignored >"$repository_path/.gitignore"
must git -C "$repository_path" add .gitignore
must git -C "$repository_path" commit --quiet --message ignore
must git -C "$repository_path" push --quiet
must git -C "$repository_path" switch --quiet main
must mkdir "$repository_path/.worktrees"
set worktree_path "$repository_path/.worktrees/ignored-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
echo local >"$worktree_path/ignored"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in ignored-worktree"

not test -e "$worktree_path"
or fail "fetch removes a worktree containing only ignored files"
not command git show-ref --verify --quiet refs/heads/feature
or fail "fetch removes the branch of a worktree containing only ignored files"

echo "ok - fetch removes a worktree containing only ignored files"

create_repository external-ignored-worktree
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo ignored >"$repository_path/.gitignore"
must git -C "$repository_path" add .gitignore
must git -C "$repository_path" commit --quiet --message ignore
must git -C "$repository_path" push --quiet
must git -C "$repository_path" switch --quiet main
set worktree_path "$test_root/external-ignored-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
echo local >"$worktree_path/ignored"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
git fetch
or fail "fetch failed in external-ignored-worktree"

not test -e "$worktree_path"
or fail "fetch removes an external worktree containing only ignored files"
not command git show-ref --verify --quiet refs/heads/feature
or fail "fetch removes the branch of an external worktree containing only ignored files"

echo "ok - fetch removes an external worktree containing only ignored files"

create_repository active-ignored-worktree
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo ignored >"$repository_path/.gitignore"
must git -C "$repository_path" add .gitignore
must git -C "$repository_path" commit --quiet --message ignore
must git -C "$repository_path" push --quiet
must git -C "$repository_path" switch --quiet main
set worktree_path "$test_root/active-ignored-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
echo local >"$worktree_path/ignored"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$test_root"
or fail "could not leave active-ignored-worktree"
set fetch_output (git -C "$worktree_path" fetch 2>&1)
or fail "fetch failed in active-ignored-worktree"

test -e "$worktree_path/ignored"
or fail "fetch preserves ignored files in its invoking worktree"
command git -C "$worktree_path" show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves the branch of its invoking worktree"
string match --quiet -- '*invoking worktree*' $fetch_output
or fail "fetch reports why its invoking worktree is preserved"

echo "ok - fetch preserves its invoking linked worktree"

create_repository locked-ignored-worktree
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo ignored >"$repository_path/.gitignore"
must git -C "$repository_path" add .gitignore
must git -C "$repository_path" commit --quiet --message ignore
must git -C "$repository_path" push --quiet
must git -C "$repository_path" switch --quiet main
set worktree_path "$test_root/locked-ignored-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
echo local >"$worktree_path/ignored"
must git -C "$repository_path" worktree lock "$worktree_path"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter locked-ignored-worktree"
set fetch_output (git fetch 2>&1)
or fail "fetch failed in locked-ignored-worktree"

test -e "$worktree_path/ignored"
or fail "fetch preserves ignored files in a locked worktree"
command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves the branch of a locked worktree"
string match --quiet -- '*locked*' $fetch_output
or fail "fetch reports why a locked worktree is preserved"

echo "ok - fetch preserves a locked linked worktree"

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
