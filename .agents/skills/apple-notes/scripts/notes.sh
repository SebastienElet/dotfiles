#!/usr/bin/env bash
# Create, file, and rename folders and notes in Apple Notes via AppleScript.
#
#   notes.sh folder <name> [account]
#   notes.sh note <folder> <title> [account]              # HTML body on stdin
#   notes.sh move <src/path> <title> <dst/path> [new-title] [account]
#   notes.sh attachments <folder/path> <title> <dir> [account]
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  notes.sh folder <name> [account]
  notes.sh note <folder> <title> [account]   # HTML body read from stdin
  notes.sh move <src/path> <title> <dst/path> [new-title] [account]
                                             # nested paths as "3 Resources/Recrutement";
                                             # new-title rewrites the body's <h1>, or sets the
                                             # note name directly when it has attachments
  notes.sh attachments <folder/path> <title> <dir> [account]
                                             # export every attachment to <dir>
EOF
  exit 64
}

# AppleScript string literals: escape backslash then double quote.
as_quote() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }

# Turn "A/B/C" into the AppleScript specifier `folder "C" of folder "B" of folder "A"` in $spec,
# plus the `make new folder` lines for the missing levels in $mklines. Sets globals (not stdout)
# because a command substitution would run in a subshell and lose $mklines.
folder_spec() {
  local q part mk
  spec=""
  mklines=""
  local IFS=/
  for part in $1; do
    q=$(as_quote "$part")
    if [ -z "$spec" ]; then
      mk="if not (exists folder \"$q\") then make new folder with properties {name:\"$q\"}"
      spec="folder \"$q\""
    else
      mk="if not (exists folder \"$q\" of $spec) then make new folder at $spec with properties {name:\"$q\"}"
      spec="folder \"$q\" of $spec"
    fi
    mklines+="  $mk"$'\n'
  done
}

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
move)
  [ $# -ge 3 ] || usage
  title=$(as_quote "$2")
  new_title=${4:-}
  account=$(as_quote "${5:-iCloud}")
  folder_spec "$1"; src=$spec
  folder_spec "$3"; dst=$spec; dst_mk=$mklines
  osascript <<EOF
tell application "Notes" to tell account "$account"
  if (get shared of $src) then error "refusing to move a note out of a shared folder — sharing cannot be restored by script"
$dst_mk
  move note "$title" of $src to $dst
end tell
EOF
  [ -n "$new_title" ] || exit 0
  nt=$(as_quote "$new_title")
  # A body round-trip is what destroys attachments: `get body` returns them inline as base64 on some
  # notes and drops them entirely on others, and writing the dropped version back deletes the photos
  # for good. `set name` never touches the body, so it is the only safe rename here. The body's first
  # line then still shows the old title — cosmetic, and preferable to losing the attachments.
  n_att=$(osascript -e "tell application \"Notes\" to tell account \"$account\" to get count of attachments of note \"$title\" of $dst")
  if [ "$n_att" -gt 0 ]; then
    osascript <<EOF
tell application "Notes" to tell account "$account"
  set n to note "$title" of $dst
  set name of n to "$nt"
  return (get name of n)
end tell
EOF
    echo "renamed via 'set name' ($n_att attachment(s) preserved); the body's first line still shows the old title" >&2
    exit 0
  fi
  # The title IS the body's first line — rewrite it and Notes re-derives the name. Never `set name`
  # as well: it blanks that line. Notes has by then turned the <h1> into a styled span, so the whole
  # first block is replaced rather than the tag. A body that opens on a list or a table has no title
  # block at all, so fall back to prepending one instead of silently leaving the old name in place.
  export NOTES_NEW_TITLE=$new_title   # perl reads it: a prefix assignment would only reach osascript
  osascript -e "tell application \"Notes\" to tell account \"$account\" to get body of note \"$title\" of $dst" \
    | perl -0777 -e '$_ = <>; my $t = "<div><b><span style=\"font-size: 24px\">$ENV{NOTES_NEW_TITLE}</span></b><br></div>";
                     s{\A\s*<div>.*?</div>}{$t}se or s{\A}{$t}; print' \
    | tr -d '\n' > "${TMPDIR:-/tmp}/notes-body.$$"
  raw=$(cat "${TMPDIR:-/tmp}/notes-body.$$")
  rm -f "${TMPDIR:-/tmp}/notes-body.$$"
  # A broken pipeline leaves this empty, and `set body of n to ""` wipes the note without any
  # AppleScript error. Bail out while the original body is still intact.
  if [ -z "$raw" ]; then
    echo "refusing to retitle \"$2\": rewritten body came back empty — the note is unchanged" >&2
    exit 1
  fi
  body=$(as_quote "$raw")
  osascript <<EOF
tell application "Notes" to tell account "$account"
  set n to note "$title" of $dst
  set body of n to "$body"
  return (get name of n)
end tell
EOF
  ;;
attachments)
  [ $# -ge 3 ] || usage
  title=$(as_quote "$2")
  account=$(as_quote "${4:-iCloud}")
  folder_spec "$1"; src=$spec
  mkdir -p "$3"
  dir=$(cd "$3" && pwd)
  # Attachment properties are all read-only and there is no `make new attachment`, so an export is
  # the only way to keep a copy of the photos outside Notes. Names collide (every photo from the
  # iPhone is "photo.jpg"), hence the index prefix.
  # No `tell account` here: inside it, `file (POSIX file …)` resolves against the account and fails
  # with -1728. Address the note through `of account` instead and keep the file spec at Notes level.
  osascript <<EOF
tell application "Notes"
  set n to note "$title" of $src of account "$account"
  repeat with i from 1 to (count of attachments of n)
    set a to attachment i of n
    -- an attachment Notes built from an inline base64 image has no filename at all
    set nm to (get name of a)
    if nm is missing value then set nm to "attachment"
    set f to "$(as_quote "$dir")/" & i & "-" & nm
    -- an unnamed inline image refuses to save (-10000); report it instead of aborting the export
    try
      save a in file ((POSIX file f) as string)
      log f
    on error e
      log "skipped attachment " & i & ": " & e
    end try
  end repeat
end tell
EOF
  # `save` failures are logged, not raised, so an export that wrote nothing would still exit 0 — and
  # this command exists precisely to be trusted before a risky operation.
  if [ -z "$(ls -A "$dir")" ]; then
    echo "nothing was exported from \"$2\" — do not treat its attachments as backed up" >&2
    exit 1
  fi
  ;;
*) usage ;;
esac
