if status is-interactive
    # Commands to run in interactive sessions can go here
end

# Enable vi bindings
set -g fish_key_bindings fish_vi_key_bindings
# Disable welcome message
set fish_greeting

# Ensure Starship is installed
if type -q starship
    # Enable Starship
    starship init fish | source
else
    echo "Starship is not installed. Please install it from https://starship.rs/"
end

# bun
set --export BUN_INSTALL "$HOME/.bun"
set --export PATH $BUN_INSTALL/bin $PATH

# zoxide - smarter cd command
if type -q zoxide
    zoxide init fish | source
end

# Added by OrbStack: command-line tools and integration
# This won't be added again if you remove it.
if test -f ~/.orbstack/shell/init2.fish
    source ~/.orbstack/shell/init2.fish
end

# pnpm
set -gx PNPM_HOME $HOME/Library/pnpm
if not string match -q -- $PNPM_HOME $PATH
    set -gx PATH "$PNPM_HOME" $PATH
end
# pnpm end

# purge local branches whose upstream was deleted ([gone]) after every fetch/pull
# only branches that HAD an upstream now gone are pruned — never-pushed work is left untouched
function git
    command git $argv
    if test (count $argv) -gt 0; and contains -- $argv[1] fetch pull
        # --prune marks deleted upstreams as [gone] in `git branch -vv`
        command git fetch --prune 2>/dev/null
        for branch in (command git branch -vv | grep ': gone]' | string replace -r '^[ *+]+(\S+).*' '$1' | grep -vE '^(main|development|staging)$')
            set wt (command git worktree list 2>/dev/null | grep "\[$branch\]" | awk '{print $1}')
            if test -n "$wt"
                command git worktree remove --force $wt 2>/dev/null
            end
            command git branch -D $branch 2>/dev/null
        end
    end
end
