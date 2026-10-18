---
name: agent-reach
description: >
  Read web pages, YouTube transcripts and supported platforms through Agent-Reach.
  Use for web reading through Jina Reader,
  Twitter/X, Reddit, Facebook, Instagram, LinkedIn, Bilibili, XiaoHongShu,
  Xiaoyuzhou, Boss Zhipin, Xueqiu, V2EX and RSS, or explicit Agent-Reach requests.
  Keep sensitive reads local; prefer existing GitHub tools for code searches.
license: BSD-2-Clause
metadata:
  category: ops
---

# Agent Reach

## Overview

Use the packaged platform recipes. Agent-Reach's Jina recipe is the generic
reader for public non-sensitive pages; follow Web Fetching below.

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
3. Install the prerequisites declared by the adapter package only when explicitly
   requested by the user. Acquisition and preparation do not activate agents or
   install Python, its tools or Docker. These instructions
   supersede setup commands in upstream references. Never install upstream skills,
   fetch unpinned setup guides, or check for updates during research.
4. Credentials are entered by the user directly into hidden tool prompts.
   Never request credentials in chat or automatically read browser cookies.
5. LinkedIn uses a separately configured Docker service with an image pinned
   by digest. Startup and interactive login are explicit user actions through
   native Docker commands. Preserve its existing profile volume. Do not automate account login.

## Web Fetching

Check data admissibility before every tool. Only public,
non-sensitive URLs may go to Agent-Reach/Jina Reader: inspect the full path and query for secrets,
personal data and access links first. Do not transmit personal/sensitive public content, signed or
unlisted access links, private/intranet URLs, localhost or local files. If unsure, report the limit.
An exception requires explicit authorization naming the service, data and purpose;
secrets, cookies, tokens, profiles and sessions never go to the Reader. Never upload local content,
forward cookies or configure an authenticated proxy for it. Never inspect existing sessions or
log in automatically. Generic authenticated/private web reading is unavailable in this workflow.
Dedicated platform recipes retain their separate authorization requirements.

Read admissible public pages with the packaged `references/web.md` Jina recipe, using anonymous
`curl` to `https://r.jina.ai/URL`. No MCP connection is needed. There is no generic browser,
stealth or CDP fallback. Browser interaction, complete DOM access and session continuity are not
provided by this reader; report those needs rather than substituting another web reader.

Success requires the content requested, including necessary pagination; HTTP 200, a title or partial
text is insufficient. Inspect Reader warnings and target status even when Reader returns HTTP 200.
Report empty or incomplete content without claiming success. Distinguish
challenge/403 (blocked), timeout/network error/5xx (unavailable), 429 (service limit), 404 (absent:
report it) and 401/login (authentication unsupported by this reader: report it).

Use one attempt per URL, at most 30 seconds each and 120 seconds for the complete read unless the
user specifies another budget; configure timeouts within the remaining budget. Do not retry a failed
read automatically. Respect `Retry-After` before any separately requested retry; if it exceeds the
remaining budget, report the service limit. Report failure or incompleteness when retrieval fails.
For current reads, request `X-No-Cache: true` and date collection. `DNT: 1` is no proof of
non-retention; cache bypass does not guarantee source freshness. No paid account, API key or spend
without separate authorization. Jina is text extraction, not a general browser or proven anti-bot
bypass. Do not add a duplicate web-reader MCP or a fallback browser during research.

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
- Use this workflow for generic web reading; keep dedicated platform capabilities separate.
