function codex --description "Run Codex with TERM=dumb and writable Git metadata"
    set -lx TERM dumb
    set -l prefix
    set -l arguments $argv
    set -l index 1
    while test $index -le (count $argv)
        switch "$argv[$index]"
            case --
                break
            case -c --config -m --model -p --profile -s --sandbox -a --ask-for-approval -C --cd -i --image --local-provider --enable --disable --remote --remote-auth-token-env --add-dir
                set index (math $index + 2)
            case '-*'
                set index (math $index + 1)
            case exec e resume fork
                set prefix $argv[1..$index]
                set arguments $argv[(math $index + 1)..-1]
                break
            case agents review login logout mcp plugin app-server remote-control app completion update doctor sandbox debug apply a queue archive delete migrate-rollouts unarchive cloud exec-server features help
                command codex $argv
                return $status
            case '*'
                break
        end
    end

    argparse --ignore-unknown 'c/config=' 'C/cd=' -- $argv 2>/dev/null
    or begin
        command codex $prefix $arguments
        return $status
    end
    set -l working_directory "$PWD"
    if set -q _flag_cd
        set working_directory "$_flag_cd"
    end

    set -l git_directories (command git -C "$working_directory" rev-parse --path-format=absolute --git-common-dir --absolute-git-dir 2>/dev/null)
    set -l git_arguments
    if test $status -eq 0; and test (count $git_directories) -eq 2; and test -n "$working_directory"
        set -a git_arguments --add-dir "$git_directories[1]"
        if test "$git_directories[1]" != "$git_directories[2]"
            set -a git_arguments --add-dir "$git_directories[2]"
        end
    end
    command codex $prefix $git_arguments $arguments
end
