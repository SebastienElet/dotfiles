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
require_skill "nearest ancestor containing \`.obsidian/\`"
require_skill 'single current workspace root'
require_skill "Never search parent directories, \`\$HOME\`, or Obsidian"
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
require_reference 'Only these commands are allowed:'

python3 - "$skill" "$reference" "$evals" <<'PY' | grep -Fxq 'structured-contract-pass'
import json
import re
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    skill = source.read()

with open(sys.argv[2], encoding="utf-8") as source:
    reference = source.read()

with open(sys.argv[3], encoding="utf-8") as source:
    document = json.load(source)

expected_commands = {
    "aliases", "backlinks", "base:query", "base:views", "bases", "file", "files", "links",
    "properties", "property:read", "read", "search", "search:context", "tag", "tags", "tasks",
    "vault",
}
unsafe_commands = set("""
append base:create bookmark command create daily:append daily:prepend delete dev:cdp dev:console
dev:css dev:debug dev:dom dev:errors dev:mobile dev:screenshot devtools eval history:open
history:restore hotkey move open plugin plugin:disable plugin:enable plugin:install plugin:reload
plugin:uninstall plugins:restrict prepend property:remove property:set publish:add publish:open
publish:remove random reload rename restart search:open snippet:disable snippet:enable sync:open
sync:restore tab:open task template:insert theme theme:install theme:set theme:uninstall unique
vault:open vaults web workspace:delete workspace:load workspace:save
""".split())
allowed_reference_tokens = expected_commands | {".base", "obsidian vault info=path", "obsidian version"}
unsafe_pattern = "|".join(sorted((re.escape(command) for command in unsafe_commands), key=len, reverse=True))

def validate_reference(reference_text, skill_text):
    if reference_text.count("## Read-only allowlist") != 1:
        raise ValueError("read-only allowlist section count is not one")
    section_match = re.search(r"^## Read-only allowlist\n(.*?)(?=^## )", reference_text, re.M | re.S)
    if section_match is None:
        raise ValueError("read-only allowlist section is absent")
    section = section_match.group(1)
    actual_commands = re.findall(r"`([^`]+)`", section)
    if len(actual_commands) != len(expected_commands) or set(actual_commands) != expected_commands:
        raise ValueError(f"read-only allowlist mismatch: {actual_commands}")
    if re.search(rf"(?<![\w:])(?:{unsafe_pattern})(?![\w:])", section):
        raise ValueError("unsafe command appears in the read-only allowlist")
    reference_tokens = set(re.findall(r"`([^`]+)`", reference_text))
    if not reference_tokens <= allowed_reference_tokens:
        raise ValueError(f"unexpected command-like reference token: {sorted(reference_tokens)}")
    permission_pattern = rf"(?i)\b(?:allow(?:ed)?|permit(?:ted)?|run|invoke|expose)\b[^\n]{{0,100}}(?<![\w:])(?:{unsafe_pattern})(?![\w:])"
    permission_text = "\n".join(
        line for line in (reference_text + "\n" + skill_text).splitlines()
        if not line.startswith("- Never ") and "Do not " not in line
    )
    if re.search(permission_pattern, permission_text):
        raise ValueError("unsafe command is described as permitted")
    invocations = re.findall(r"\bobsidian\s+([a-z][a-z:-]*)", reference_text + "\n" + skill_text)
    if not set(invocations) <= {"version", "vault"}:
        raise ValueError(f"unsafe Obsidian invocation: {invocations}")
    positive_skill_lines = (
        line for line in skill_text.splitlines()
        if not line.startswith("- Never ") and "Do not " not in line
    )
    for line in positive_skill_lines:
        tokens = set(re.findall(r"`([^`]+)`", line))
        if tokens & unsafe_commands:
            raise ValueError(f"unsafe command exposed in SKILL.md: {sorted(tokens & unsafe_commands)}")

try:
    validate_reference(reference, skill)
    adversarial_references = (
        reference.replace("## Degraded behavior", "Also allowed: create\n\n## Degraded behavior"),
        reference.replace("This closed set", "<code>base:create</code>\n\nThis closed set"),
        reference + "\n## Read-only allowlist\n\n`vaults`\n",
        reference + "\nAllowed command: `eval`\n",
        reference.replace("`read`,", "`read`, `read`,"),
    )
    for adversarial_reference in adversarial_references:
        try:
            validate_reference(adversarial_reference, skill)
        except ValueError:
            continue
        raise ValueError("adversarial reference bypassed the validator")
    try:
        validate_reference(reference, skill + "\nAllowed command: obsidian property:set\n")
    except ValueError:
        pass
    else:
        raise ValueError("adversarial skill instruction bypassed the validator")
    try:
        validate_reference(reference, skill + "\nUse `create` to add a note.\n")
    except ValueError:
        pass
    else:
        raise ValueError("backticked SKILL.md mutator bypassed the validator")
except ValueError as error:
    raise SystemExit(str(error)) from error

queries = document.get("queries", [])
texts = [query.get("query", "") for query in queries]
reasons = [query.get("reason", "") for query in queries]
activations = [query.get("should_activate") for query in queries]

checks = (
    document.get("skill") == "obsidian-retrieval",
    isinstance(document.get("version"), str),
    bool(queries) and all(isinstance(value, bool) for value in activations),
    any(activations) and any(value is False for value in activations),
    all(isinstance(value, str) for value in texts + reasons),
    any(re.search(r"exact|title|identifier|tag", value, re.I) for value in texts),
    any(re.search(r"concept|idea|theme", value, re.I) for value in texts),
    any(re.search(r"backlink|propert|task|base", value, re.I) for value in texts),
    any(re.search(r"web|weather", value, re.I) for value in texts),
    any(re.search(r"write|create|edit", value, re.I) for value in texts),
    any(re.search(r"missing|unavailable", value, re.I) for value in reasons),
    any(re.search(r"empty|no match", value, re.I) for value in reasons),
)
if not all(checks):
    raise SystemExit("trigger evaluations do not cover the acceptance cases")
print("structured-contract-pass")
PY

printf 'Obsidian retrieval skill contract passed\n'
