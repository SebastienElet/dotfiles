#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
skill="$here/../SKILL.md"
reference="$here/../references/obsidian-cli.md"
evals="$here/../evals/trigger-queries.json"

fail() {
  printf 'Obsidian retrieval contract failed: %s\n' "$1" >&2
  exit 1
}

require_skill() {
  rg -qF -- "$1" "$skill" || fail "SKILL.md missing: $1"
}

require_reference() {
  rg -qF -- "$1" "$reference" || fail "Obsidian CLI reference missing: $1"
}

[[ -f $skill ]] || fail 'SKILL.md is absent'
[[ -f $reference ]] || fail 'Obsidian CLI reference is absent'
[[ -f $evals ]] || fail 'trigger evaluations are absent'

require_skill 'explicit corpus input'
require_skill 'nearest ancestor containing `.obsidian/`'
require_skill 'single current workspace root'
require_skill 'Never search parent directories, `$HOME`, or Obsidian'
require_skill 'Exact anchors'
require_skill 'Conceptual questions'
require_skill 'Obsidian semantics'
require_skill 'Read every source note used for the answer'
require_skill 'A search hit is a'
require_skill 'candidate, not evidence'
require_skill 'not prove that the information is absent'
require_skill 'Report unavailable tools, inaccessible roots, and incomplete indexes'
require_skill 'Do not install QMD'
require_skill 'Never write, create, append, prepend, move, rename, or delete vault content'
require_skill 'references/obsidian-cli.md'

require_reference 'https://help.obsidian.md/cli'
require_reference 'Confirm that the Obsidian desktop application is already running'
require_reference 'before the first CLI call.'
require_reference '`backlinks`'
require_reference '`links`'
require_reference '`properties` and `property:read`'
require_reference '`tasks`'
require_reference '`bases`, `base:views`, and `base:query`'

for command in create append prepend move rename delete property:set property:remove eval command; do
  if rg -q -- "obsidian( [^ ]+)* $command([ :]|$)" "$skill" "$reference"; then
    fail "mutating or unrestricted Obsidian command exposed: $command"
  fi
done

jq -e '
  .skill == "obsidian-retrieval" and
  (.version | type == "string") and
  ([.queries[].should_activate] | any) and
  ([.queries[].should_activate] | any(. == false)) and
  ([.queries[].query] | any(test("exact|title|identifier|tag"; "i"))) and
  ([.queries[].query] | any(test("concept|idea|theme"; "i"))) and
  ([.queries[].query] | any(test("backlink|propert|task|base"; "i"))) and
  ([.queries[].query] | any(test("web|weather"; "i"))) and
  ([.queries[].query] | any(test("write|create|edit"; "i"))) and
  ([.queries[].reason] | any(test("missing|unavailable"; "i"))) and
  ([.queries[].reason] | any(test("empty|no match"; "i")))
' "$evals" >/dev/null || fail 'trigger evaluations do not cover the acceptance cases'

printf 'Obsidian retrieval skill contract passed\n'
