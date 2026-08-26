# Doctor Skills

## Scope

- `/skill-manager doctor` audits every directory under the selected `<skills-root>`.
- `/skill-manager doctor <name>` audits one skill.
- `<skills-root>/README.md` is not a skill directory.

Doctor is read-only. It reports findings for a later `fix` operation.

## Status model

- **FAIL**: a standard rule or mandatory local convention fails.
- **WARN**: only a qualitative, non-blocking weakness exists.
- **PASS**: no FAIL or WARN exists.
- Missing optional tooling is an environment limitation, not a status downgrade.

## Procedure

1. Read `conventions.md` completely.
2. Enumerate the requested skill directories.
3. Detect `skills-ref` once for the whole run.
4. Ensure no tool is mutating the skill tree, then run
   `bun --no-install <skill-manager-root>/scripts/check-resource-files.ts <skills-root>/<slug>` on
   each canonical skill directory, never on a deployed symlink, and preserve every finding or
   execution failure.
5. Apply every standard and remaining local check below to each skill.
6. Produce one report per skill and a global summary.
7. Propose exact fixes, but modify nothing.

## Standard checks

When `skills-ref` exists, run `skills-ref validate ./<skills-root>/<slug>` and preserve its full
failure. When absent, report `Standard validation: unavailable (skills-ref not installed)` and apply
these checks manually:

- `name` exists, is a string, has 1–64 characters, matches `[a-z0-9]+(-[a-z0-9]+)*`, and equals the
  directory slug;
- `description` exists, is a non-empty string, and is at most 1024 characters;
- `license`, if present, is a string;
- `compatibility`, if present, is a non-empty string of at most 500 characters;
- `allowed-tools`, if present, is a space-separated string and is reported as experimental;
- `metadata`, if present, is a map whose keys and values are strings;
- no top-level field exists outside `name`, `description`, `license`, `compatibility`,
  `allowed-tools`, and `metadata`.

## Local checks

### Frontmatter and index

- `metadata.category` exists and is `dev`, `support`, `product`, or `ops`;
- the skill appears exactly once under the matching README section;
- description starts with the distinguishing case, contains concrete `Use when` conditions, and
  stays below the local 400-character target;
- a weak but valid description is WARN, not FAIL.

### Body structure

- one H1 follows frontmatter;
- `Overview`, `Usage`, `Steps` or `Workflow`, `Gotchas`, and `Constraints` exist in that order;
- `Gotchas` contains at least three cause/consequence/correction entries;
- `Constraints` contains at least three hard rules;
- `SKILL.md` stays below 500 lines, or detailed material is progressively disclosed and linked.

### Resources and routing

- the bundled checker accepts every physical entry against the canonical
  `assets/resource-file-policy.json` allowlist and reports each unexpected path as FAIL;
- symbolic links, FIFOs, sockets, and other non-regular entries are FAIL;
- a symbolic link passed as the canonical skill root is FAIL;
- a checker execution failure is FAIL, never a skipped resource check;
- every entry matched by a Git ignore rule is FAIL, including below an open resource directory;
- runtime artifacts never belong inside a skill;
- `agents/openai.yaml`, when present, is valid YAML and its interface metadata still matches the
  skill;
- same-topic scoped sibling references have explicit conditional routing in `SKILL.md`;
- identical cross-scope content is not duplicated;
- the repository adapters still resolve to `../.agents/skills`;
- the slug does not exist in the other canonical collection;
- a user skill's declared installations resolve back to `harness/skills/<slug>`;
- absence of an activation router is never a finding;
- a router rule is WARN unless its repeated behavioral evidence is identified.

### Templated shell safety

Search the `SKILL.md` body for unescaped `$0` through `$9`, including braced forms such as `${1}`,
plus `$@` and `$ARGUMENTS`. Executable shell containing a match is FAIL and moves to `scripts/`.
Literal prose must escape the dollar sign once. Do not scan `references/` or `scripts/` as
templated bodies.

### Optional evals

When `evals/trigger-queries.json` exists, require:

- valid JSON;
- string `skill` equal to the directory slug;
- string `version`;
- non-empty `queries` array;
- string `query`, boolean `should_activate`, and string `reason` in every entry;
- at least one `true` and one `false` `should_activate` value.

An absent eval file is not a finding unless an approved requirement explicitly demands one.

## Report format

```text
### <slug> [PASS | WARN | FAIL]

Standard validation: PASS | FAIL: <finding> | unavailable (skills-ref not installed)
Local conventions: PASS | WARN: <finding> | FAIL: <finding>
Frontmatter: PASS | <exact finding>
Body: PASS | <exact finding>
Resources: PASS | <exact finding>
Templated shell: PASS | <exact finding>
Evals: PASS | absent (optional) | <exact finding>
README: PASS | <exact finding>
Action needed: none | <one exact corrective action per finding>
```

Global summary:

```text
| Skill | Status | Standard | Local | Action |
| --- | --- | --- | --- | --- |
| <slug> | PASS | unavailable | PASS | none |
```

List environment limitations after the table once, rather than repeating them as failures.

## Common findings

| Finding                               | Exact correction                                                                             |
| ------------------------------------- | -------------------------------------------------------------------------------------------- |
| Non-standard field                    | Remove it; preserve approved scalar metadata under `metadata` only when semantically correct |
| Invalid `allowed-tools` type          | Replace it with a space-separated string or remove it                                        |
| Description above local target        | Keep the specific first clause and concrete triggers; move process detail to the body        |
| Missing `Gotchas`                     | Add three repository-specific cause/consequence/correction entries                           |
| Missing `Constraints`                 | Add three hard must or must-not rules                                                        |
| Positional shell placeholder          | Move executable shell to `scripts/`; escape literal prose once                               |
| Unexpected resource file              | Remove it or change the canonical policy in a separately approved convention update          |
| Missing conditional reference routing | Route each same-topic scoped sibling from `Steps`                                            |
| Missing README entry                  | Run deterministic `sync-index` after the skill itself passes                                 |

## Constraints

- Never modify a file during doctor.
- Never install `skills-ref` or another validator.
- Never skip or manually reinterpret a failed resource-file check.
- Never report unavailable tooling as PASS.
- Never fail a skill solely because it lacks an activation router or eval file.
- Never convert a qualitative judgment into a mandatory finding without a local rule.
- Always give a reproducible finding and exact correction for every FAIL or WARN.
