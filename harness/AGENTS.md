# Global AI Instructions

@SOUL.md
@USER.md

## Critical Analysis (ALWAYS)

Before implementing any request:

- Challenge unclear requirements and the assumptions behind them, not just the request
- Point out if a request might conflict with existing code or architecture
- Never record a workaround for a defect in code we own: fix it, or open a ticket and
  reference it. A memorized dance around our own bug guarantees the bug survives.
- Before writing code that parses an external value, joins, maps errors or adds a persisted
  field, put `.agents/skills/merge-verdict/references/failure-classes.md` to the design; the
  failure-path test ships in the same commit.

Raising a concern never blocks delivery: state it, then proceed as described
in `USER.md`.

## Context Management

- For open-ended exploration, research, or multi-file searches, delegate to the Explore or general-purpose agent instead of reading files directly in the main thread.
- Keep the main thread for orchestration/decisions; push bulk reading, grepping, and exploration into subagents.

## Web Fetching

Escalate only when the previous tier fails; never start above the first tier:

1. Built-in fetch, then `scrapling` `fetch` for JS-rendered pages.
2. `scrapling` `stealthy_fetch` for anti-bot protections (add `solve_cloudflare` for Turnstile).
3. CloakBrowser, when `stealthy_fetch` is still blocked. Reuse the `cloak` container instead of
   starting a new one, so a forgotten `docker stop` costs at most one container:
   ```sh
   docker start cloak 2>/dev/null ||
     docker run -d --name cloak -p 127.0.0.1:9222:9222 cloakhq/cloakbrowser:0.5.3 cloakserve --idle-timeout=300
   ```
   Then call `scrapling` `fetch` with `cdp_url=http://host.docker.internal:9222` — the Scrapling MCP
   runs inside Docker, so `localhost` would resolve to its own container. Stop the container
   (`docker stop cloak`) once done; `--idle-timeout` stops it on its own after five idle minutes.

## Verification Claims

- **Check that the barrier covers what changed.** Before saying "green", confirm a linter
  and a test actually run on the extensions you touched. If nothing covers them, that gap
  is the first thing to fix — not a reason to claim green.
- **Name the environment.** Every piece of evidence states where it was produced and is
  valid only there. Green on one platform, one shell or one image says nothing about the
  others the project supports: list the supported targets, say which you exercised.

## Code Style

- **Write no comment.** One is admissible only when it records a fact living outside the file
  — upstream defect, protocol quirk, deliberate deviation — and names that fact. Doc comments
  a project's tooling requires are out of scope.
- List in the delivery note every comment you added, and the outside fact each one records.
  An empty list is the expected outcome.
