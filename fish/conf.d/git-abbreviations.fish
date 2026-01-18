# Git abbreviations using Fish best practices
# Abbreviations expand only when typed, showing full command in history
# This makes debugging easier and follows Fish shell conventions

# Basic git commands
abbr -a g git
abbr -a ga 'git add'
abbr -a gaa 'git add --all'
abbr -a gap 'git apply'
abbr -a gapa 'git add --patch'
abbr -a gapt 'git apply --3way'
abbr -a gau 'git add --update'
abbr -a gav 'git add --verbose'

# Branch operations
abbr -a gb 'git branch'
abbr -a gbD 'git branch -D'
abbr -a gba 'git branch -a'
abbr -a gbd 'git branch -d'
abbr -a gbl 'git blame -b -w'
abbr -a gbnm 'git branch --no-merged'
abbr -a gbr 'git branch --remote'

# Bisect operations
abbr -a gbs 'git bisect'
abbr -a gbsb 'git bisect bad'
abbr -a gbsg 'git bisect good'
abbr -a gbsr 'git bisect reset'
abbr -a gbss 'git bisect start'

# Commit operations
abbr -a gc 'git commit -v'
abbr -a gc! 'git commit -v --amend'
abbr -a gca 'git commit -v -a'
abbr -a gca! 'git commit -v -a --amend'
abbr -a gcam 'git commit -a -m'
abbr -a gcan! 'git commit -v -a --no-edit --amend'
abbr -a gcans! 'git commit -v -a -s --no-edit --amend'
abbr -a gcas 'git commit -a -s'
abbr -a gcasm 'git commit -a -s -m'
abbr -a gcb 'git checkout -b'
abbr -a gcf 'git config --list'
abbr -a gcn! 'git commit -v --no-edit --amend'
abbr -a gco 'git checkout'
abbr -a gcsm 'git commit -s -m'

# Pull and push operations
abbr -a gl 'git pull'
abbr -a gs 'git status'

# Rebase operations
abbr -a grbc 'git rebase --continue'
abbr -a grbm 'git rebase (~/.dotfiles/scripts/git_main_branch)'
