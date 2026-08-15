function __git_cleanup_candidates --argument-names command_name main_branch
    set -l git_context $argv[3..]
    set -l branch_refs (command git $git_context for-each-ref --format='%(refname)' refs/heads)
    or return 1

    for branch_ref in $branch_refs
        set -l branch (string replace refs/heads/ '' "$branch_ref")
        if contains -- "$branch" "$main_branch" development staging
            continue
        end

        set -l upstream_ref (command git $git_context for-each-ref --format='%(upstream)' "$branch_ref")
        if test $status -ne 0
            echo "git: keeping $branch: upstream lookup failed before $command_name" >&2
            continue
        end
        if test -z "$upstream_ref"
            continue
        end

        set -l upstream_oid (command git $git_context rev-parse --verify --quiet "$upstream_ref^{commit}")
        if test $status -eq 0
            command git $git_context config "branch.$branch.cleanup-base" "$upstream_oid"
            or echo "git: cleanup retry disabled for $branch: could not record its upstream tip" >&2
            string join \t -- tracked "$branch" "$upstream_oid"
            continue
        end

        set upstream_oid (command git $git_context config --get "branch.$branch.cleanup-base")
        set -l config_status $status
        if test $config_status -eq 1
            string join \t -- unavailable "$branch"
        else if test $config_status -ne 0; or test -z "$upstream_oid"
            echo "git: keeping $branch: cleanup base lookup failed before $command_name" >&2
        else
            string join \t -- tracked "$branch" "$upstream_oid"
        end
    end
end
