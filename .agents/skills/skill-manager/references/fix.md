# Fix Skills

## Scope

- `/skill-manager fix <name>` — apply all fixes from doctor to one specific skill

The `fix` command is the automated remediation step that follows `doctor`. Always run `doctor` first
to generate a report, then `fix` to apply the corrections systematically.

---

## Procedure

### 1. Run doctor first

If no doctor was run in the current session for the target skill, run the doctor procedure
(`references/doctor.md`) before proceeding. The fix procedure relies on the doctor report's "Action
needed" field.

Record the full doctor result as a **baseline** (all check statuses: PASS / WARN / FAIL). You will
compare against this baseline after applying fixes to detect any regression.

### 2. Present doctor findings to the user

Show the doctor report — which checks are FAIL, which are WARN, and the overall score.

### 3. Read `references/conventions.md`

All fixes must conform to the agentskills.io standard documented there. Read it before touching any
SKILL.md.

### 4. Apply fixes in this order

Work through each failing check in this priority order to avoid rework:

#### A. Remove non-portable frontmatter fields (blocking)

| Field found                            | Fix                                  |
| -------------------------------------- | ------------------------------------ |
| `disable-model-invocation: true`       | Delete the line entirely             |
| `category: <value>` at top-level       | Move to `metadata.category: <value>` |
| Any other custom field (`paths`, etc.) | Remove or move into `metadata.*`     |

#### B. Fix the `description` to pushy format (blocking)

Rewrite using the pattern from `conventions.md § description Format`:

```text
<What the skill does>. Use when <triggers>.
Make sure to use this skill whenever <keywords + implicit cases>,
even if <edge case where user might not think to name the domain>.
```

**Before/after example:**

```yaml
# ❌ Before — deprecated "Trigger terms" format
description: >
  Write and run tests. Trigger terms: test, jest, unit, integration, spec.

# ✅ After — pushy format
description: >
  Write and run tests using Jest. Use when adding unit or integration tests, debugging test
  failures, or setting up test infrastructure. Make sure to use this skill whenever writing a
  .test.ts file, running test:unit or test:integration, or asking about coverage — even if the user
  just says "check this works".
```

#### C. Add or fix `## Gotchas` section (blocking)

Every skill must have a `## Gotchas` section with **3+ entries**. Each entry must follow cause + fix
format:

```markdown
## Gotchas

- **<Cause / bad pattern>** — <what happens> — <how to fix it>.
```

Add project-specific warnings: common misuses, wrong patterns for this codebase, configuration
traps.

#### D. Add missing required sections (blocking)

Required sections in this order: `Overview`, `Usage`, `Steps`, `Gotchas`, `Constraints`,
`References` (if any).

| Section missing  | Action                                                                 |
| ---------------- | ---------------------------------------------------------------------- |
| `## Overview`    | Write 2–3 sentences: what the skill does and when to use it            |
| `## Usage`       | Add invocation syntax + at least one example                           |
| `## Steps`       | Add numbered, actionable steps (reference files where content is rich) |
| `## Constraints` | Add 3+ hard must/must-not rules                                        |

#### E. Apply scope segmentation if needed (non-blocking)

If the skill's behaviour varies by package/application:

1. Split the monolithic `references/` file into `<topic>-<scope>.md` files (e.g.
   `integration-pm.md`, `integration-lm.md`).
2. Add conditional routing in `## Steps`: "If the file is in `packages/prisma-lm`, read
   `references/<topic>-lm.md`."
3. Verify identical cross-scope content lives only once (in a shared reference or inline in
   SKILL.md).

#### F. Apply progressive disclosure if needed (non-blocking)

If `SKILL.md` exceeds 500 lines:

1. Identify self-contained content blocks (detailed procedures, long examples, reference tables).
2. Move each block to `references/<topic>.md`.
3. Replace the block in SKILL.md with a one-line reference:
   `See [references/<topic>.md](references/<topic>.md)`.
4. Add or update the `## References` section in SKILL.md listing all moved files.

### 5. Re-run doctor to validate

After applying all fixes, run the doctor procedure again on the same skill. Compare the new result
against the **baseline** recorded in Step 1:

- Every previously FAIL check must now be PASS.
- No previously PASS check may have regressed to WARN or FAIL. If a regression is detected, undo the
  specific edit that caused it (restore the previous content of the affected section in SKILL.md —
  do not use `git reset`) and investigate before proceeding.

### 6. Sync the index

After all fixes are validated, run the sync-index procedure (`references/sync-index.md`) to keep
README up to date.

---

## Constraints

- **Never modify a PASS skill without justification** — a failing doctor check or a cross-check
  finding (D1–D6) both constitute valid justification. If neither is present, refuse and ask the
  user to clarify.
- **Cross-check findings are also valid input** — if the user runs `cross-check` first and then asks
  to fix a D1/D2/D3/D4/D5/D6 finding, you do not need to re-run `doctor`. Skip step 1 and proceed
  from step 2 using the cross-check report as your baseline.
- **Always run doctor first when no report exists** — fixing without any report (doctor or
  cross-check) is guesswork; the report drives the fix order.
- **Always read `references/conventions.md`** before touching any SKILL.md — it is the SSOT for
  format and portability.
- **Record the baseline** before any write — the pre-fix report result is required to detect
  regressions after Step 6.
- **Never apply fixes during `cross-check`** — cross-check is read-only. If a user asks you to fix a
  finding inline, refuse: run `/skill-manager fix <name>` instead.

---

## Fix Checklist (per skill)

Use this checklist to confirm a skill is fully fixed before marking it done:

- [ ] Doctor was run and baseline recorded before any write
- [ ] Doctor findings were presented to the user before modification
- [ ] `references/conventions.md` was read before any `SKILL.md` was modified
- [ ] No non-portable frontmatter fields (`disable-model-invocation`, top-level `category`, `paths`)
- [ ] `description` uses pushy format ("Use when", "Make sure to use whenever", "even if")
- [ ] `description` is < 1024 characters
- [ ] `## Gotchas` section exists with 3+ cause+fix entries
- [ ] All required sections present: `Overview`, `Usage`, `Steps`, `Constraints`
- [ ] `SKILL.md` is < 500 lines (or content moved to `references/`)
- [ ] Scope-segmented references use `<topic>-<scope>.md` naming (if applicable)
- [ ] `## Steps` has conditional routing (if multiple scoped references)
- [ ] Skill is listed in `README.md` (run sync-index if not)
- [ ] `doctor` re-run confirms PASS with no regressions vs. baseline
