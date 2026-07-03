# Doctor Skills

## Scope

- `/skill-manager doctor` — check all skills in `.agents/skills/`
- `/skill-manager doctor <name>` — check one specific skill

## Procedure

1. **Identify skills to check**

   ```bash
   ls .agents/skills/
   ```

   Exclude `README.md`.

2. **For each skill**, run the checklist below.

3. **Produce report** in the output format below.

4. **Propose fixes** — for each failing skill, state the exact action needed.

---

## Doctor Checklist (per skill)

### Structure — pass/fail

- [ ] Directory contains a `SKILL.md`
- [ ] Frontmatter has `name` field (kebab-case, matches directory)
- [ ] Frontmatter has `description` field
- [ ] Frontmatter has `metadata.category` field (`dev` | `support` | `product` | `ops`)
- [ ] **NO `disable-model-invocation: true`** at top-level (should be removed; this field breaks
      portability)
- [ ] **NO `category` at top-level** (must be in `metadata.category`)
- [ ] **Only standard agentskills.io fields** at top-level: `name`, `description`, `license`,
      `compatibility`, `metadata`
- [ ] Skill is listed in `.agents/skills/README.md`

### Content Quality — 0–2 pts each (max 10)

| Section       | 2 pts                                                     | 1 pt                   | 0 pts   |
| ------------- | --------------------------------------------------------- | ---------------------- | ------- |
| `Overview`    | Specific, states when to use                              | Present but generic    | Missing |
| `Usage`       | Syntax + options + examples                               | Syntax only            | Missing |
| `Steps`       | Numbered, actionable, complete                            | Present but incomplete | Missing |
| `Rules`       | 3+ hard constraints                                       | 1-2 rules              | Missing |
| `description` | Pushy format ("Use when", "Make sure"), specific keywords | Present but weak       | Generic |

### Size & Disclosure — pass/fail

- [ ] `SKILL.md` is < 500 lines
- [ ] `description` field is < 1024 characters
- [ ] If SKILL.md > 500 lines, content is progressively disclosed:
  - Lean SKILL.md with references
  - Rich content moved to `references/`
  - `## References` section linking to `references/`

### Scope Segmentation — pass/fail

- [ ] If the skill's rules **vary by package/application**, `references/` files use
      `<topic>-<scope>.md` naming (e.g. `integration-pm.md`, `integration-lm.md`) rather than a
      single monolithic file
- [ ] The `## Steps` section includes **conditional routing** ("if file is in X, read
      `references/Y.md`") when multiple scoped references exist
- [ ] Cross-scope content that is **identical** is not duplicated — it lives in a single shared
      reference or inline in SKILL.md

### Gotchas Section — pass/fail

- [ ] **`## Gotchas` section exists** (NEW, mandatory)
- [ ] Gotchas section includes 3+ project-specific warnings
- [ ] Each gotcha has cause + fix format

### Frontmatter Portability — pass/fail

- [ ] Frontmatter uses ONLY standard fields: `name`, `description`, `license`, `compatibility`,
      `metadata`
- [ ] No custom top-level fields (`disable-model-invocation`, `category`, `paths`, etc.)
- [ ] `description` is in "pushy" format (not ending with "Trigger terms: ...")

### Activation Quality

- Description uses pushy format ("Use when", "Make sure"): ✅ / ⚠️
- Description avoids ambiguity about when to use: ✅ / ⚠️
- No overlap with another skill's trigger terms: ✅ / ⚠️
- Front-loaded keywords (most important trigger terms first): ✅ / ⚠️

---

## Output Format

```text
### <slug> [PASS | WARN | FAIL]

**Structure**: ✅ all fields present | ❌ missing: <list>
**Content**: <score>/10
**Size & Disclosure**: ✅ good | ⚠️ <issue>
**Gotchas**: ✅ compliant | ❌ missing/incomplete
**Portability**: ✅ standard fields only | ❌ custom fields found: <list>
**Activation**: ✅ good | ⚠️ <issue>
**README**: ✅ listed | ❌ missing

**Action needed**: <specific fix or "none">
```

At the end, output a summary:

```text
## Summary

| Skill | Status | Content | Action |
|-------|--------|---------|--------|
| ... | PASS | 10/10 | none |
| ... | WARN | 6/10 | add Gotchas + pushy description |
| ... | FAIL | 2/10 | add Usage + Rules sections |

**Blocking issues** (must fix before Phase 2):
1. ...

**Non-blocking improvements** (nice to have):
1. ...
```

---

## Common Issues and Fixes

| Issue                                                  | Fix                                                                                                                              |
| ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| `disable-model-invocation` at top-level                | Remove it — it breaks portability. Portability is enforced by agentskills.io standard.                                           |
| `category` at top-level                                | Move to `metadata.category`. Top-level `category` is not in the standard.                                                        |
| Missing `## Gotchas`                                   | Add a new section with 3+ project-specific gotchas (cause + fix format).                                                         |
| Weak description                                       | Rewrite using pushy format: `[What]. Use when [triggers]. Make sure to use whenever [keywords + implicit], even if [edge case].` |
| Generic `Overview`                                     | Rewrite to answer: what does it do, and when should I use it? Be specific.                                                       |
| Missing `Usage`                                        | Add invocation syntax with at least one example.                                                                                 |
| Missing `Rules`                                        | Add 3+ hard constraints (must/must-not).                                                                                         |
| Not in README                                          | Run `/skill-manager sync-index` to rebuild the index.                                                                            |
| Custom fields (non-standard)                           | Remove them. Only `name`, `description`, `license`, `compatibility`, `metadata` are portable.                                    |
| `SKILL.md` too long (> 500 lines)                      | Move content to `references/` and add `## References` section with links.                                                        |
| Description > 1024 chars                               | Trim to essentials; move detailed guidance to `## Usage` or `## Steps`.                                                          |
| Monolithic `references/` file covering multiple scopes | Split into `<topic>-<scope>.md` files and add conditional routing in `## Steps`.                                                 |
| Missing conditional routing in `## Steps`              | Add "if file is in X, read `references/Y.md`" logic so the agent loads only relevant context.                                    |
