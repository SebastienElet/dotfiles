# Skill Quality Reference

## Source Standards

- Agent Skills specification: `SKILL.md` is required; `scripts/`, `references/`, and `assets/` are optional.
- Frontmatter must include `name` and `description`.
- `name` must be lowercase kebab-case, cannot start or end with a hyphen, cannot contain consecutive hyphens, and must match the parent directory.
- `description` should describe both the capability and the situations that should trigger the skill.
- Progressive disclosure matters: startup metadata is tiny, `SKILL.md` loads only after activation, and bundled files load only when needed.

## Design Heuristics

- Start from use cases, not implementation details.
- Prefer outcome-oriented trigger examples.
- Optimize for composability across agents.
- Avoid README-style documentation inside the skill folder unless a client explicitly requires it.
- Scripts should be self-contained, deterministic, and have clear errors.
- References should be focused and loaded on demand.
- Assets should be files used in final output, not extra documentation.

## Common Fixes

- Broad description: add explicit trigger phrases and file/task types.
- Overloaded `SKILL.md`: move examples, specs, or long checklists into `references/`.
- Placeholder resources: delete empty or unused directories/files.
- Product-specific assumptions: move them into compatibility notes or agent-specific metadata.
- Unvalidated scripts: run a representative test before finalizing.
