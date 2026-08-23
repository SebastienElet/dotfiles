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
  grep -Fq -- "$1" "$skill" || fail "SKILL.md missing: $1"
}

require_reference() {
  grep -Fq -- "$1" "$reference" || fail "Obsidian CLI reference missing: $1"
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
  if grep -Eq -- "obsidian( [^ ]+)* $command([ :]|$)" "$skill" "$reference"; then
    fail "mutating or unrestricted Obsidian command exposed: $command"
  fi
done

python3 - "$evals" <<'PY' || fail 'trigger evaluations do not cover the acceptance cases'
import json
import re
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    document = json.load(source)

queries = document.get("queries", [])
texts = [query.get("query", "") for query in queries]
reasons = [query.get("reason", "") for query in queries]
activations = [query.get("should_activate") for query in queries]

assert document.get("skill") == "obsidian-retrieval"
assert isinstance(document.get("version"), str)
assert queries and all(isinstance(value, bool) for value in activations)
assert any(activations) and any(value is False for value in activations)
assert all(isinstance(value, str) for value in texts + reasons)
assert any(re.search(r"exact|title|identifier|tag", value, re.I) for value in texts)
assert any(re.search(r"concept|idea|theme", value, re.I) for value in texts)
assert any(re.search(r"backlink|propert|task|base", value, re.I) for value in texts)
assert any(re.search(r"web|weather", value, re.I) for value in texts)
assert any(re.search(r"write|create|edit", value, re.I) for value in texts)
assert any(re.search(r"missing|unavailable", value, re.I) for value in reasons)
assert any(re.search(r"empty|no match", value, re.I) for value in reasons)
PY

printf 'Obsidian retrieval skill contract passed\n'
