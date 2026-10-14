---
name: agent-reach
description: >
  Read web pages, YouTube transcripts and supported platforms through Agent-Reach.
  Use for web reading, public Jina fallback after built-in fetch fails,
  Twitter/X, Reddit, Facebook, Instagram, LinkedIn, Bilibili, XiaoHongShu,
  Xiaoyuzhou, Boss Zhipin, Xueqiu, V2EX and RSS, or explicit Agent-Reach requests.
  Keep sensitive reads local; prefer existing GitHub tools for code searches.
license: MIT
metadata:
  category: ops
---

# Agent Reach

## Overview

Use the packaged platform recipes. For public non-sensitive pages, follow the
Web Fetching workflow below: built-in fetch → Agent-Reach/Jina → local browser
→ local stealth browser → CloakBrowser. Scrapling remains a local fallback.

## Usage

Select the reference for the requested platform. Publishing, commenting, liking
and account changes require a separate explicit user instruction.

## Workflow

1. For web pages, follow Web Fetching below before any retrieval tool, then read
   `references/web.md` for the Jina recipe. This workflow supersedes upstream
   quick commands and platform recipes for web reads.
2. Read the matching reference below and verify actual content with the requested
   platform's read command. Do not run the global `agent-reach doctor` unless
   the user explicitly authorizes inspection of all configured browser sessions;
   its Boss backend can read Chrome cookies through CDP.
3. Install dependencies only through `moon run harness:agent-reach` in the
   dotfiles checkout, explicitly requested by the user. These instructions
   supersede setup commands in upstream references. Never install upstream skills,
   fetch unpinned setup guides, or check for updates during research.
4. Credentials are entered by the user directly into hidden tool prompts.
   Never request credentials in chat or automatically read browser cookies.
5. LinkedIn uses the named Docker service. Its explicit startup task is
   `harness:agent-reach-linkedin-start`; its interactive login task is
   `harness:agent-reach-linkedin-login`. Do not automate account login.

## Web Fetching

Check data admissibility before every tool, including hosted built-in fetch. Only public,
non-sensitive URLs may go to Agent-Reach/Jina Reader: inspect the full path and query for secrets,
personal data and access links first. Keep personal/sensitive public content, signed or unlisted
access links, private/intranet URLs, localhost and local files on authorized local tools. If unsure,
stay local. An exception requires explicit authorization naming the service, data and purpose;
secrets, cookies, tokens, profiles and sessions never go to the Reader. Never upload local content,
forward cookies or configure an authenticated proxy for it. Session access requires explicit
authorization for the account/profile; never inspect existing sessions or log in automatically.

For admissible public pages, advance only after the previous tier fails. Explicitly exclude tiers
incompatible with confidentiality or the requested capability:

1. Built-in fetch.
2. The packaged `references/web.md` Jina recipe, using anonymous `curl` to
   `https://r.jina.ai/URL`, without Scrapling or MCPorter.
3. Local browser for missing JavaScript content, interaction or navigation: `scrapling` `fetch`
   remains available. Private or authenticated reads go directly to authorized local tools.
4. Local stealth browser only for an identified anti-bot block: `scrapling` `stealthy_fetch`
   (add `solve_cloudflare` for Turnstile).
5. CloakBrowser, when the stealth browser is still blocked. Start the named container with
   `"$HOME/.dotfiles/tooling/cloakbrowser"`, then use `scrapling` `fetch` with
   `cdp_url=http://host.docker.internal:9222`: Scrapling runs in Docker, so localhost would
   address its own container. Stop `cloak` once done; its idle timeout is five minutes.

Success requires the content requested, including necessary pagination; HTTP 200, a title or partial
text is insufficient. Inspect Reader warnings and target status even when Reader returns HTTP 200.
Empty or incomplete content triggers the next admissible tier. Distinguish
challenge/403 (blocked), timeout/network error/5xx (unavailable), 429 (service limit), 404 (absent:
report it) and 401/login (authentication: use an authorized local session or report it).

Use one attempt per tier, at most 30 seconds each and 120 seconds total unless the user specifies
another budget; configure tool timeouts within the remaining budget. Never loop back to a failed
tier. Respect `Retry-After` before calling the same service again; use an admissible local fallback
or report the limit. When the budget or tools are exhausted, report failure or incompleteness.
For current reads, request `X-No-Cache: true` and date collection. `DNT: 1` is no proof of
non-retention; cache bypass does not guarantee source freshness. No paid account, API key or spend
without separate authorization. Jina is text extraction, not a general browser or proven anti-bot
bypass. Scrapling and CloakBrowser remain available; `web-reader` MCP is not configured.

{{UPSTREAM_ROUTING}}
## References

Read the packaged platform files under `references/`: search, social, career,
dev, web, video and finance. All upstream channels are retained.

## Gotchas

- Doctor availability does not establish a successful search or transcript.
- OpenCLI requires its Chrome extension and an existing user-controlled session.
- LinkedIn's container must be stopped before its interactive login.

## Constraints

- Never install tools or update this plugin as a side effect of research.
- Never export cookies, automate login or use another platform's credentials.
- Never treat challenges, empty results or partial responses as retrieved content.
- Prefer installed dedicated skills; use Jina only after built-in web fetch fails.
