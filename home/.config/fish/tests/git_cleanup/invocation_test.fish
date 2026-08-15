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

create_repository single-fetch
create_tracked_branch feature
must git -C "$repository_path" remote add backup "$remote_path"
must git --git-dir="$remote_path" update-ref -d refs/heads/feature
must git -C "$repository_path" remote set-url origin "$test_root/missing.git"

cd "$repository_path"
or fail "could not enter $repository_path"
set -l fetch_output (git fetch backup 2>&1)
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
