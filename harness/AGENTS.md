# Global AI Instructions

@SOUL.md
@USER.md

## Critical Analysis (ALWAYS)

Before implementing any request:

- Treat issues, specs and proposed implementations as claims to verify, not authority.
  Challenge material assumptions and conflicts with the current code or architecture.
- For architecture changes, business invariants or poorly understood historical areas,
  verify the relevant ADRs, documentation, personas and specs. If intent is unclear,
  map the current flow and inspect relevant ADRs, commits, PRs and issues.
- An ADR in force is the primary source for architectural intent. Verify its status and
  scope. Code is the implemented reality; tests are evidence, not architectural authority.
- A certain contradiction that affects the current task blocks implementation. If an
  exception or migration could plausibly explain it, investigate first. If material
  uncertainty remains, stop and propose a focused experiment or a minimal blocking issue.
- If an ADR itself appears wrong, do not repair it opportunistically. Propose a minimal
  issue naming the ADR, the proven contradiction and its impact; leave the investigation
  and correction to a fresh agent/session after validation.
- Before compensating in owned code for an assumed behavior of an external dependency,
  verify its current official documentation and the local configuration.
- Cite a source by its status, not its title: a proposal is not authority, and a dated
  legal or specification reference must be the version in force before anything leans
  on it.
- Never record a workaround for a defect in code we own as intended behavior: fix it, or
  open a ticket and reference it.
- Before writing code that parses an external value, joins, maps errors or adds a persisted
  field, make every affected failure path explicit in the design; the corresponding failure-path
  test ships in the same commit.

Keep unrelated inconsistencies out of scope. Follow `USER.md` for validation and workflow.

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
- **No guarantee without a green oracle.** A capability a contract offers — sealing, a
  retention field, logical archiving, link revocation — is a proof structure, not the
  guarantee. State what is held today, and gate every promise on a named end-to-end
  check that runs.

## Code Style

- Prefer names and structure that express intent directly.
- **Write no comment.** One is admissible only when it records a fact living outside the file
  — upstream defect, protocol quirk, deliberate deviation — and names that fact. Doc comments
  a project's tooling requires are out of scope.
- List in the delivery note every comment you added, and the outside fact each one records.
  An empty list is the expected outcome.
