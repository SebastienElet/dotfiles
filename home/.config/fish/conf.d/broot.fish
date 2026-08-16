function br --wraps=broot
    set -l cmd_file (mktemp)
    if broot --outcmd $cmd_file $argv
        source $cmd_file
        rm -f $cmd_file
    else
        set -l code $status
        rm -f $cmd_file
        return $code
    end
end
