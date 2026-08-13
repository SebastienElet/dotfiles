# Fix a Skill

## Scope

`/skill-manager fix <name>` applies one justified change set to one skill. It accepts exactly two
inputs:

1. reproducible findings from a current doctor or cross-check report;
2. an explicitly requested functional evolution, including a change to a PASS skill.

Without either input, refuse automatic editing and ask what behavior should change.

## Procedure

1. Read `conventions.md` completely.
2. Identify the target slug and input type: findings or requested evolution.
3. Run doctor and record the complete baseline unless a current cross-check report is the sole
   input.
4. For an evolution, record the requested behavioral contract in one testable sentence.
5. Present the findings or contract before writing.
6. Apply only the corrections or behavior the input authorizes, in the order below.
7. Run relevant eval scenarios, then doctor on the target.
8. Compare every prior PASS check and the requested contract against the baseline.
9. Run `sync-index` and verify its second run is byte-identical.

## Correction order

### 1. Standard frontmatter

- Remove fields outside `name`, `description`, `license`, `compatibility`, `allowed-tools`, and
  `metadata`.
- Move top-level `category` to `metadata.category`.
- Keep an unknown value under `metadata` only when it is scalar metadata and the user explicitly
  approves the mapping.
- Correct field types and name constraints before changing prose.

### 2. Description and activation

- Put the distinguishing use case first.
- Keep concrete explicit and implicit triggers below the local 400-character target.
- Do not add an activation router merely because it is absent.
- For an approved router finding, preserve the repeated baseline prompts and rerun them after the
  smallest relational rule.

### 3. Required body

Add or repair `Overview`, `Usage`, `Steps` or `Workflow`, `Gotchas`, and `Constraints` in local order.
Use at least three specific gotchas and three hard constraints. Do not invent domain behavior to
fill a section; ask when the missing rule is not derivable.

### 4. Templated shell

Move executable shell containing positional placeholders from `SKILL.md` to a resource under
`scripts/`, and let the skill invoke it. When the example does not need positional arguments,
rewrite it around named environment state. Escape only literal explanatory prose.

### 5. Progressive disclosure and scoped references

- Move detailed material out of a `SKILL.md` at or above 500 lines.
- Split references into same-topic scoped siblings only when behavior genuinely differs.
- Add explicit conditional reference routing from `Steps`.
- Merge identical sibling content into one shared reference.

### 6. Evals and index

Repair present eval JSON to the schema in `conventions.md`; never create evals solely because they
are optional. Run realistic positive and negative scenarios for activation changes. Regenerate the
README only after the target passes doctor.

## Regression handling

Every baseline PASS must remain PASS. If a check regresses, restore only the edit that caused it
with `apply_patch`, investigate, and rerun doctor; never use `git reset` to hide unrelated work.

An evolution succeeds only when its recorded contract is demonstrated and unrelated behavior is
unchanged. A formatting-only diff is not evidence of functional success.

## Checklist

- Input is a current finding or explicit functional evolution.
- Baseline is recorded before writing.
- `conventions.md` was read.
- Only authorized files and behavior changed.
- Standard and local checks pass after the edit.
- Relevant positive and negative evals were run.
- No baseline PASS regressed.
- `sync-index` is byte-identical on its second run.

## Constraints

- A PASS skill may change only for an explicitly requested functional evolution. Without such a
  request, a doctor or cross-check finding is required before any automatic edit.
- Never modify more than one skill in one `fix` operation.
- Never invent a finding or domain rule to justify a write.
- Never install validation tooling.
- Never apply a cross-check correction inside the read-only cross-check operation.
- Never claim standard validation when `skills-ref` is unavailable.
