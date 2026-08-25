create_repository divergent-branch
create_tracked_branch feature
must git -C "$repository_path" switch --quiet feature
echo local >>"$repository_path/file"
must git -C "$repository_path" commit --quiet --all --message local
must git -C "$repository_path" switch --quiet main
set -l worktree_path "$test_root/divergent-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set -l fetch_output (git fetch 2>&1)
or fail "fetch failed in divergent-branch"

command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves a branch with local commits"
not test -e "$worktree_path"
or fail "fetch removes the clean worktree of a branch with local commits"
string match --quiet -- '*keeping branch feature: local tip differs*' $fetch_output
or fail "fetch distinguishes a preserved branch from its removed worktree"

echo "ok - fetch removes its clean worktree but preserves a branch with local commits"

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
not command git config --get-regexp '^branch\.feature\.'
or fail "fetch removes configuration for a deleted branch"

echo "ok - fetch removes a clean worktree and branch without local commits"
