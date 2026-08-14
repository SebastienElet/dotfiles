#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
measure="$here/measure_repository.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fake_tokei="$tmp/tokei"
args_log="$tmp/args"

cat >"$fake_tokei" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >"$CODEGRAPH_TOKEI_ARGS"
jq -n \
  --argjson loc "$CODEGRAPH_TEST_LOC" \
  --argjson files "$CODEGRAPH_TEST_FILES" \
  '{TypeScript: {code: $loc, reports: [range(0; $files) | {name: tostring}]}, Total: {code: $loc}}'
SCRIPT
chmod +x "$fake_tokei"

measure_case() {
  CODEGRAPH_TOKEI_BIN="$fake_tokei" \
    CODEGRAPH_TOKEI_ARGS="$args_log" \
    CODEGRAPH_TEST_LOC="$1" \
    CODEGRAPH_TEST_FILES="$2" \
    "$measure" "$tmp/repository"
}

mkdir -p "$tmp/repository"

low=$(measure_case 49999 499)
[ "$(printf '%s' "$low" | jq -r .initialize)" = "false" ]
[ "$(printf '%s' "$low" | jq -r .loc)" = "49999" ]
[ "$(printf '%s' "$low" | jq -r .files)" = "499" ]

loc_threshold=$(measure_case 50000 1)
[ "$(printf '%s' "$loc_threshold" | jq -r .initialize)" = "true" ]

file_threshold=$(measure_case 1 500)
[ "$(printf '%s' "$file_threshold" | jq -r .initialize)" = "true" ]

rg -qF -- '--hidden' "$args_log"
rg -qF -- '--output json' "$args_log"
rg -qF -- '--exclude node_modules' "$args_log"
rg -qF -- '--exclude docs' "$args_log"
rg -qF -- '--exclude fixtures' "$args_log"
rg -qF 'TypeScript' "$args_log"
rg -qF 'Rust' "$args_log"
rg -qF 'Vue' "$args_log"

real_repository="$tmp/real-repository"
mkdir -p "$real_repository/src" "$real_repository/docs" "$real_repository/ignored"
printf '%s\n' 'export const kept = 1' >"$real_repository/src/kept.ts"
printf '%s\n' 'export const documentation = 2' >"$real_repository/docs/documentation.ts"
printf '%s\n' 'export const ignored = 3' >"$real_repository/ignored/ignored.ts"
printf '%s\n' 'ignored/' >"$real_repository/.gitignore"
git -C "$real_repository" init -q
real=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$real_repository")
[ "$(printf '%s' "$real" | jq -r .loc)" = "1" ]
[ "$(printf '%s' "$real" | jq -r .files)" = "1" ]

if CODEGRAPH_TOKEI_BIN="$tmp/missing" "$measure" "$tmp/repository" 2>"$tmp/error"; then
  exit 1
fi
rg -qF 'tokei is required' "$tmp/error"

if CODEGRAPH_TOKEI_BIN="$fake_tokei" CODEGRAPH_JQ_BIN="$tmp/missing" \
  "$measure" "$tmp/repository" 2>"$tmp/error"; then
  exit 1
fi
rg -qF 'jq is required' "$tmp/error"

echo ok
