function __git_cleanup_branch --argument-names command_name branch upstream_oid
    set -l git_context $argv[4..]
    set -l upstream_state (command git $git_context for-each-ref --format='%(upstream:track)' "refs/heads/$branch")
    if test "$upstream_state" != '[gone]'
        return 0
    end

    set -l local_oid (command git $git_context rev-parse --verify --quiet "refs/heads/$branch^{commit}")
    if test $status -ne 0
        echo "git: keeping $branch: local tip unavailable after $command_name" >&2
        return 1
    end
    if test "$local_oid" != "$upstream_oid"
        echo "git: keeping $branch: local tip differs from its last upstream" >&2
        return 1
    end

    __git_cleanup_worktree "$command_name" "$branch" $git_context
    or return 1

    if command git $git_context update-ref -d "refs/heads/$branch" "$upstream_oid"
        command git $git_context config --remove-section "branch.$branch"
        or echo "git: removed $branch but failed to remove its configuration" >&2
    else
        echo "git: keeping $branch: local tip changed during cleanup" >&2
        return 1
    end
end
