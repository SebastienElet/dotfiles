# Global AI Instructions

@SOUL.md
@USER.md

## Critical Analysis (ALWAYS)

Before implementing any request:

- Challenge assumptions and unclear requirements
- Question the "why" behind requests, not just implementing them
- Verify alignment with project patterns and best practices
- Point out if a request might conflict with existing code or architecture

Raising a concern never blocks delivery: state it, then proceed as described
in `USER.md`.

## Context Management

- For open-ended exploration, research, or multi-file searches, delegate to the Explore or general-purpose agent instead of reading files directly in the main thread.
- For independent multi-step tasks (especially 2+ unrelated ones), dispatch parallel subagents rather than doing them serially inline.
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

## Code Style

- All comments and documentation in English
- Prefer self-documenting code over comments
- Only comment when explaining *why*, not *what*
