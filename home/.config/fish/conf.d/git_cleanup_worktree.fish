function git_cleanup_worktree --argument-names command_name branch
    set -l git_context $argv[3..]
    set -l worktree_list (command git $git_context worktree list --porcelain)
    if test $status -ne 0
        echo "git: keeping $branch: worktree list failed after $command_name" >&2
        return 1
    end
    if test (count (string match -- "branch refs/heads/$branch" $worktree_list)) -gt 1
        echo "git: keeping $branch: branch is checked out in multiple worktrees" >&2
        return 1
    end

    set -l worktree (command git $git_context for-each-ref --format='%(worktreepath)' "refs/heads/$branch" | string collect)
    if test $pipestatus[1] -ne 0
        echo "git: keeping $branch: worktree lookup failed after $command_name" >&2
        return 1
    end
    if test -z "$worktree"
        return 0
    end

    set -l index_state (command git -C "$worktree" ls-files -v)
    if test $status -ne 0
        echo "git: keeping $branch: worktree index lookup failed after $command_name" >&2
        return 1
    end
    if string match -rq -- '^[a-zS] ' $index_state
        echo "git: keeping $branch: worktree index hides file changes" >&2
        return 1
    end

    set -l worktree_state (command git -C "$worktree" status --porcelain --untracked-files=all --ignored --ignore-submodules=none)
    if test $status -ne 0
        echo "git: keeping $branch: worktree status failed after $command_name" >&2
        return 1
    end
    if test (count $worktree_state) -gt 0
        echo "git: keeping $branch: worktree contains local files or changes" >&2
        return 1
    end

    command git $git_context worktree remove "$worktree"
end
