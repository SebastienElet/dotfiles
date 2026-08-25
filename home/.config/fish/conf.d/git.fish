function git
    set -l command_name
    set -l command_arguments
    set -l git_context
    set -l git_prefix
    git_parse_invocation $argv

    if not contains -- "$command_name" fetch pull; or contains -- --dry-run $command_arguments
        command git $argv
        return $status
    end

    set -l main_branch_script (path resolve (path dirname (status filename))/../../../../tooling/git-main-branch)
    if not test -x "$main_branch_script"
        echo "git: branch cleanup disabled: $main_branch_script is unavailable" >&2
        command git $argv
        return $status
    end

    set -l main_branch (command "$main_branch_script" --strict $git_context)
    if test $status -ne 0; or test -z "$main_branch"
        echo "git: branch cleanup disabled: primary branch lookup failed" >&2
        command git $argv
        return $status
    end

    set -l candidates (git_cleanup_candidates "$command_name" "$main_branch" $git_context)
    if test $status -ne 0
        echo "git: branch cleanup disabled: candidate lookup failed" >&2
        command git $argv
        return $status
    end

    command git $git_prefix $command_name --prune $command_arguments
    set -l command_status $status
    if test $command_status -ne 0
        return $command_status
    end

    for candidate in $candidates
        set -l fields (string split \t -- "$candidate")
        if test "$fields[1]" = unavailable
            set -l branch "$fields[2]"
            git_cleanup_worktree "$command_name" "$branch" $git_context
            set -l worktree_cleanup_status $status
            if test $worktree_cleanup_status -eq 2
                continue
            end
            if test $worktree_cleanup_status -ne 0
                continue
            end
            set -l recovered (git_cleanup_recover "$command_name" "$main_branch" "$branch" $git_context)
            if test $status -ne 0
                echo "git: keeping branch $branch: $recovered" >&2
                continue
            end
            set fields (string split \t -- "$recovered")
        end
        git_cleanup_branch "$command_name" "$fields[2]" "$fields[3]" $git_context
    end

    return $command_status
end
