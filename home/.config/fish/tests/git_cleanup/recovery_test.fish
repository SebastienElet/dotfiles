create_repository unavailable-upstream
create_tracked_branch feature
must git -C "$repository_path" replace (command git -C "$repository_path" rev-parse feature) (command git -C "$repository_path" rev-parse main)
set -l worktree_path "$test_root/unavailable-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature
must git -C "$repository_path" update-ref -d refs/remotes/origin/feature

cd "$repository_path"
or fail "could not enter $repository_path"
set -l fetch_output (git fetch 2>&1)
or fail "fetch failed in unavailable-upstream"

command git show-ref --verify --quiet refs/heads/feature
or fail "fetch preserves a branch when its previous upstream tip is unavailable"
not test -e "$worktree_path"
or fail "fetch removes its clean worktree when the previous upstream tip is unavailable"
string match --quiet -- '*commits are not integrated into main*' $fetch_output
or fail "fetch reports unique work when the previous upstream tip is unavailable"

echo "ok - unavailable upstream tip preserves the branch"

create_repository unavailable-integrated-upstream
create_tracked_branch feature
must git -C "$repository_path" merge --quiet --squash feature
must git -C "$repository_path" commit --quiet --message squash
must git -C "$repository_path" push --quiet
set worktree_path "$test_root/unavailable-integrated-feature"
must git -C "$repository_path" worktree add --quiet "$worktree_path" feature
must git --git-dir="$remote_path" update-ref -d refs/heads/feature
must git -C "$repository_path" update-ref -d refs/remotes/origin/feature

cd "$repository_path"
or fail "could not enter $repository_path"
git fetch
or fail "fetch failed in unavailable-integrated-upstream"

not test -e "$worktree_path"
or fail "fetch removes a clean worktree whose patches are integrated"
not command git show-ref --verify --quiet refs/heads/feature
or fail "fetch removes a branch whose patches are integrated"

echo "ok - integrated branch is cleaned when its upstream tip was already pruned"
