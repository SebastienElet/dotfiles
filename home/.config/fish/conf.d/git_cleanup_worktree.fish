function git_cleanup_worktree --argument-names command_name branch
    set -l git_context $argv[3..]
    set -l worktree_list (command git $git_context worktree list --porcelain)
    if test $status -ne 0
        echo "git: keeping worktree for $branch: worktree list failed after $command_name" >&2
        return 1
    end
    if test (count (string match -- "branch refs/heads/$branch" $worktree_list)) -gt 1
        echo "git: keeping worktrees for $branch: branch is checked out in multiple worktrees" >&2
        return 1
    end

    set -l worktree (command git $git_context for-each-ref --format='%(worktreepath)' "refs/heads/$branch" | string collect)
    if test $pipestatus[1] -ne 0
        echo "git: keeping worktree for $branch: worktree lookup failed after $command_name" >&2
        return 1
    end
    if test -z "$worktree"
        return 0
    end

    set -l inspect_worktree env -u GIT_DIR -u GIT_COMMON_DIR -u GIT_INDEX_FILE \
        -u GIT_LITERAL_PATHSPECS -u GIT_GLOB_PATHSPECS -u GIT_NOGLOB_PATHSPECS -u GIT_ICASE_PATHSPECS \
        git -C "$worktree" --work-tree="$worktree"

    set -l index_state (command $inspect_worktree ls-files -v)
    if test $status -ne 0
        echo "git: keeping worktree for $branch: worktree index lookup failed after $command_name" >&2
        return 1
    end
    if string match -rq -- '^[a-zS] ' $index_state
        echo "git: keeping worktree for $branch: worktree index hides file changes" >&2
        return 1
    end

    set -l worktree_state (command $inspect_worktree status --porcelain --untracked-files=all --ignore-submodules=none)
    if test $status -ne 0
        echo "git: keeping worktree for $branch: worktree status failed after $command_name" >&2
        return 1
    end
    if test (count $worktree_state) -gt 0
        echo "git: keeping worktree for $branch: worktree contains local files or changes" >&2
        return 1
    end

    set -l invocation_worktree (command git $git_context rev-parse --show-toplevel | string collect)
    if test $pipestatus[1] -ne 0; or test -z "$invocation_worktree"
        echo "git: keeping worktree for $branch: invoking worktree lookup failed after $command_name" >&2
        return 1
    end
    if test (path resolve "$worktree") = (path resolve "$invocation_worktree")
        echo "git: keeping worktree for $branch: refusing to remove the invoking worktree" >&2
        return 1
    end

    set -l protected_ignored (command $inspect_worktree ls-files --others --ignored --exclude-standard -- \
        ':(top,exclude,glob)node_modules/**' ':(top,exclude,glob)target/**' ':(top,exclude,glob).moon/cache/**' 2>&1)
    if test $status -ne 0; or set -q protected_ignored[1]
        echo "git: keeping worktree for $branch: worktree may contain protected ignored files" >&2
        return 1
    end

    command git $git_context worktree remove "$worktree"
    or return 1
    command git $git_context config --remove-section "branch.$branch"
    or begin
        echo "git: removed worktree for $branch but could not detach its stale upstream" >&2
        return 1
    end
    echo "git: removed worktree for $branch; keeping its detached local branch" >&2
    return 2
end
