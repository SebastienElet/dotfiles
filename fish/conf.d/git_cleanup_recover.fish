function git_cleanup_recover --argument-names command_name main_branch branch
    set -l git_context $argv[4..]
    set -l local_oid (command git $git_context rev-parse --verify --quiet "refs/heads/$branch^{commit}")
    if test $status -ne 0
        echo "local tip unavailable after $command_name"
        return 1
    end

    set -l unique_merge (command git --no-replace-objects $git_context rev-list --merges --max-count=1 "$main_branch..$branch")
    if test $status -ne 0
        echo "merge lookup failed after $command_name"
        return 1
    end
    if test -n "$unique_merge"
        echo "merge commits are not integrated into $main_branch"
        return 1
    end

    # git cherry compares patch equivalence, not merge history; replacement refs cannot authorize deletion.
    set -l cherry_output (command git --no-replace-objects $git_context cherry "$main_branch" "$branch")
    if test $status -ne 0
        echo "patch lookup failed after $command_name"
        return 1
    end
    if string match -rq -- '^\+ ' $cherry_output
        echo "commits are not integrated into $main_branch"
        return 1
    end

    command git $git_context config "branch.$branch.cleanup-base" "$local_oid"
    or echo "git: cleanup retry disabled for $branch: could not record its integrated tip" >&2
    string join \t -- tracked "$branch" "$local_oid"
end
