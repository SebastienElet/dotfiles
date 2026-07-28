#!/usr/bin/env bash
# Create folders and notes in Apple Notes via AppleScript.
#
#   notes.sh folder <name> [account]
#   notes.sh note <folder> <title> [account]   # HTML body on stdin
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  notes.sh folder <name> [account]
  notes.sh note <folder> <title> [account]   # HTML body read from stdin
EOF
  exit 64
}

# AppleScript string literals: escape backslash then double quote.
as_quote() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }

cmd=${1:-}; shift || usage

case "$cmd" in
folder)
  [ $# -ge 1 ] || usage
  name=$(as_quote "$1")
  account=$(as_quote "${2:-iCloud}")
  osascript <<EOF
tell application "Notes" to tell account "$account"
  if not (exists folder "$name") then make new folder with properties {name:"$name"}
  return name of folder "$name"
end tell
EOF
  ;;
note)
  [ $# -ge 2 ] || usage
  folder=$(as_quote "$1")
  title=$(as_quote "$2")
  account=$(as_quote "${3:-iCloud}")
  # Newlines are illegal inside an AppleScript string literal; HTML ignores them anyway.
  body=$(as_quote "$(tr -d '\n')")
  # ponytail: title is emitted as the body's first line — Notes derives the title from it.
  # Passing `name:` as well would duplicate the title inside the note.
  osascript <<EOF
tell application "Notes" to tell account "$account"
  if not (exists folder "$folder") then make new folder with properties {name:"$folder"}
  set n to make new note at folder "$folder" with properties {body:"<div><h1>$title</h1></div>" & "$body"}
  return name of n
end tell
EOF
  ;;
*) usage ;;
esac
