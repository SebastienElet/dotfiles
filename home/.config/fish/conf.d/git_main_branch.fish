function git_main_branch
    if contains -- --bitbucket-cloud $argv
        echo "git-main-branch: unsupported option --bitbucket-cloud" >&2
        return 2
    end

    set -l strict 0
    while test "$argv[1]" = --strict
        set strict 1
        set -e argv[1]
    end

    if test $strict -eq 1
        command git $argv rev-parse --git-dir >/dev/null
        or return 1
    else if not command git $argv rev-parse --git-dir >/dev/null 2>&1
        echo main
        return
    end

    for branch in main master trunk
        command git $argv show-ref -q --verify refs/heads/$branch
        set -l probe_status $status
        if test $probe_status -eq 0
            echo $branch
            return
        end
        if test $strict -eq 1; and test $probe_status -ne 1
            return $probe_status
        end
    end

    echo main
end
