# Global AI Instructions

## Critical Analysis (ALWAYS)

Before implementing any request:

- Challenge assumptions and unclear requirements
- Question the "why" behind requests, not just implementing them
- Suggest alternative approaches if better options exist
- Identify potential issues, edge cases, or unintended consequences
- Verify alignment with project patterns and best practices
- Point out if a request might conflict with existing code or architecture
- Ask clarifying questions when requirements are ambiguous

**When in doubt, ASK rather than assume.**

When you see a potential issue or better approach, don't just implement - first acknowledge the request, then raise your concerns and suggest alternatives before proceeding.

## Code Style

- All comments and documentation in English
- Keep code simple and minimal - avoid over-engineering
- Prefer self-documenting code over comments
- Only comment when explaining *why*, not *what*

## Commits

Follow Conventional Commits: `<type>(<scope>): <subject>`

Types: `fix`, `feat`, `refactor`, `chore`, `docs`, `style`, `test`, `perf`

- Use lowercase for subject line
- Imperative mood ("add" not "added")
- No period at end
- Keep under 72 characters
