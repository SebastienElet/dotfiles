set -l git_main_branch_script (path resolve (path dirname (status filename))/../../scripts/git_main_branch)

function git --inherit-variable git_main_branch_script
    set -l command_name
    set -l command_arguments
    set -l git_context
    set -l git_prefix
    set -l argument_index 1
    while test $argument_index -le (count $argv)
        set -l argument $argv[$argument_index]
        switch "$argument"
            case -C -c --git-dir --work-tree --namespace --config-env
                if test $argument_index -eq (count $argv)
                    break
                end
                set -l option_value $argv[(math $argument_index + 1)]
                set -a git_context "$argument" "$option_value"
                set -a git_prefix "$argument" "$option_value"
                set argument_index (math $argument_index + 2)
            case -p --paginate -P --no-pager --no-replace-objects --no-lazy-fetch --no-optional-locks --no-advice --bare
                set -a git_prefix "$argument"
                if contains -- "$argument" --no-replace-objects --no-lazy-fetch --no-optional-locks --bare
                    set -a git_context "$argument"
                end
                set argument_index (math $argument_index + 1)
            case '--git-dir=*' '--work-tree=*' '--namespace=*' '--config-env=*'
                set -a git_context "$argument"
                set -a git_prefix "$argument"
                set argument_index (math $argument_index + 1)
            case '-*'
                break
            case '*'
                set command_name "$argument"
                if test $argument_index -lt (count $argv)
                    set command_arguments $argv[(math $argument_index + 1)..-1]
                end
                break
        end
    end

    set -l tracked_tips
    set -l sync_requested
    if contains -- "$command_name" fetch pull; and not contains -- --dry-run $command_arguments
        if not test -x "$git_main_branch_script"
            echo "git: branch cleanup disabled: $git_main_branch_script is unavailable" >&2
            command git $argv
            return $status
        end

        set -l primary_branch (command "$git_main_branch_script" --strict $git_context)
        if test $status -ne 0; or test -z "$primary_branch"
            echo "git: branch cleanup disabled: primary branch lookup failed" >&2
            command git $argv
            return $status
        end

        set sync_requested 1
        for branch_ref in (command git $git_context for-each-ref --format='%(refname)' refs/heads)
            set -l branch (string replace refs/heads/ '' "$branch_ref")
            if contains -- "$branch" "$primary_branch" development staging
                continue
            end

            set -l upstream_ref (command git $git_context for-each-ref --format='%(upstream)' "$branch_ref")
            if test -z "$upstream_ref"
                continue
            end

            set -l upstream_oid (command git $git_context rev-parse --verify --quiet "$upstream_ref^{commit}")
            if test $status -ne 0
                echo "git: keeping $branch: upstream tip unavailable before $command_name" >&2
                continue
            end

            set -a tracked_tips (string join \t -- "$branch" "$upstream_oid")
        end
    end

    if test -n "$sync_requested"
        command git $git_prefix $command_name --prune $command_arguments
    else
        command git $argv
    end
    set -l command_status $status

    if test -n "$sync_requested"; and test $command_status -eq 0
        for tracked_tip in $tracked_tips
            set -l fields (string split \t -- "$tracked_tip")
            set -l branch $fields[1]
            set -l upstream_oid $fields[2]
            set -l upstream_state (command git $git_context for-each-ref --format='%(upstream:track)' "refs/heads/$branch")
            if test "$upstream_state" != '[gone]'
                continue
            end

            set -l local_oid (command git $git_context rev-parse --verify --quiet "refs/heads/$branch^{commit}")
            if test $status -ne 0
                echo "git: keeping $branch: local tip unavailable after $command_name" >&2
                continue
            end
            if test "$local_oid" != "$upstream_oid"
                echo "git: keeping $branch: local tip differs from its last upstream" >&2
                continue
            end

            set -l worktree_list (command git $git_context worktree list --porcelain)
            if test $status -ne 0
                echo "git: keeping $branch: worktree list failed after $command_name" >&2
                continue
            end
            set -l branch_worktrees (string match -- "branch refs/heads/$branch" $worktree_list)
            if test (count $branch_worktrees) -gt 1
                echo "git: keeping $branch: branch is checked out in multiple worktrees" >&2
                continue
            end

            set -l wt (command git $git_context for-each-ref --format='%(worktreepath)' "refs/heads/$branch" | string collect)
            set -l worktree_lookup_status $pipestatus[1]
            if test $worktree_lookup_status -ne 0
                echo "git: keeping $branch: worktree lookup failed after $command_name" >&2
                continue
            end
            if test -n "$wt"
                set -l index_state (command git -C "$wt" ls-files -v)
                if test $status -ne 0
                    echo "git: keeping $branch: worktree index lookup failed after $command_name" >&2
                    continue
                end
                if string match -rq -- '^[a-zS] ' $index_state
                    echo "git: keeping $branch: worktree index hides file changes" >&2
                    continue
                end
                set -l worktree_state (command git -C "$wt" status --porcelain --untracked-files=all --ignored --ignore-submodules=none)
                if test $status -ne 0
                    echo "git: keeping $branch: worktree status failed after $command_name" >&2
                    continue
                end
                if test (count $worktree_state) -gt 0
                    echo "git: keeping $branch: worktree contains local files or changes" >&2
                    continue
                end
                command git $git_context worktree remove "$wt"
                or continue
            end
            command git $git_context update-ref -d "refs/heads/$branch" "$upstream_oid"
            or echo "git: keeping $branch: local tip changed during cleanup" >&2
        end
    end

    return $command_status
end
