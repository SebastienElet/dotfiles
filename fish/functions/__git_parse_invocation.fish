function __git_parse_invocation --no-scope-shadowing
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
end
