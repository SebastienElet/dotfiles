function codex --description "Run Codex with TERM=dumb"
    set -lx TERM dumb
    command codex $argv
end
