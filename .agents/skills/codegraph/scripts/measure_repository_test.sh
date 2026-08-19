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
  'range(0; $files) as $index
  | {
      language: "TypeScript",
      stats: {
        name: ("fixture-" + ($index | tostring) + ".ts"),
        stats: {code: (if $index == 0 then $loc else 0 end)}
      }
    }'
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
rg -qF -- '--streaming json' "$args_log"
rg -qF -- '--exclude node_modules' "$args_log"
rg -qF -- '--exclude docs' "$args_log"
rg -qF -- '--exclude fixtures' "$args_log"
rg -qF 'TypeScript' "$args_log"
rg -qF 'Rust' "$args_log"
rg -qF 'Razor' "$args_log"
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

extension_repository="$tmp/extensions"
mkdir -p \
  "$extension_repository/razor" \
  "$extension_repository/tofu" \
  "$extension_repository/cjs" \
  "$extension_repository/hcl" \
  "$extension_repository/yaml" \
  "$extension_repository/xml"
for number in {1..500}; do
  printf '@page "/%s"\n' "$number" >"$extension_repository/razor/$number.razor"
  printf 'resource "fixture" "item_%s" {}\n' "$number" >"$extension_repository/tofu/$number.tofu"
  printf 'exports.fixture%s = true;\n' "$number" >"$extension_repository/cjs/$number.cjs"
  printf 'fixture_%s = true\n' "$number" >"$extension_repository/hcl/$number.hcl"
  printf 'fixture_%s: true\n' "$number" >"$extension_repository/yaml/$number.yaml"
  printf '<fixture id="%s" />\n' "$number" >"$extension_repository/xml/$number.xml"
done
git -C "$extension_repository" init -q
razor=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$extension_repository/razor")
tofu=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$extension_repository/tofu")
cjs=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$extension_repository/cjs")
hcl=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$extension_repository/hcl")
yaml=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$extension_repository/yaml")
xml=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$extension_repository/xml")
[ "$(printf '%s' "$razor" | jq -r .files)" = "500" ]
[ "$(printf '%s' "$razor" | jq -r .initialize)" = "true" ]
[ "$(printf '%s' "$tofu" | jq -r .files)" = "500" ]
[ "$(printf '%s' "$tofu" | jq -r .initialize)" = "true" ]
[ "$(printf '%s' "$cjs" | jq -r .files)" = "500" ]
[ "$(printf '%s' "$cjs" | jq -r .initialize)" = "true" ]
[ "$(printf '%s' "$hcl" | jq -r .files)" = "0" ]
[ "$(printf '%s' "$hcl" | jq -r .initialize)" = "false" ]
[ "$(printf '%s' "$yaml" | jq -r .files)" = "0" ]
[ "$(printf '%s' "$yaml" | jq -r .initialize)" = "false" ]
[ "$(printf '%s' "$xml" | jq -r .files)" = "0" ]
[ "$(printf '%s' "$xml" | jq -r .initialize)" = "false" ]

symlink_repository="$tmp/symlink-repository"
mkdir -p "$symlink_repository"
printf 'resource "external" "private" {}\n' >"$tmp/external.tofu"
ln -s "$tmp/external.tofu" "$symlink_repository/external.tofu"
git -C "$symlink_repository" init -q
git -C "$symlink_repository" add external.tofu
symlink=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$symlink_repository")
[ "$(printf '%s' "$symlink" | jq -r .files)" = "0" ]
[ "$(printf '%s' "$symlink" | jq -r .loc)" = "0" ]

fake_git="$tmp/git"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  'if [ "${CODEGRAPH_GIT_FAILURE:-}" = rev-parse ] && [[ " $* " == *" rev-parse "* ]]; then echo "rev-parse operational failure" >&2; exit 7; fi' \
  'if [ "${CODEGRAPH_GIT_FAILURE:-}" = ls-files ] && [[ " $* " == *" ls-files "* ]]; then printf "src/partial.tofu\\0"; echo "ls-files operational failure" >&2; exit 7; fi' \
  'exec "$CODEGRAPH_REAL_GIT" "$@"' >"$fake_git"
chmod +x "$fake_git"
mkdir -p "$tmp/git-errors/src" "$tmp/git-errors/ignored"
printf 'resource "partial" "fixture" {}\n' >"$tmp/git-errors/src/partial.tofu"
printf 'ignored/\n' >"$tmp/git-errors/.gitignore"
for number in {1..500}; do
  printf 'resource "ignored" "fixture_%s" {}\n' "$number" >"$tmp/git-errors/ignored/$number.tofu"
done
git -C "$tmp/git-errors" init -q

for failure in rev-parse ls-files; do
  if CODEGRAPH_GIT_BIN="$fake_git" \
    CODEGRAPH_GIT_FAILURE="$failure" \
    CODEGRAPH_REAL_GIT="$(command -v git)" \
    CODEGRAPH_TOKEI_BIN=tokei \
    "$measure" "$tmp/git-errors" 2>"$tmp/error"; then
    exit 1
  fi
  rg -qF "$failure operational failure" "$tmp/error"
done

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
