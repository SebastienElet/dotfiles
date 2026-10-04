---
name: grill-me
description: >
  Challenge an idea through a requested design interview. Use only when the user explicitly invokes
  `$grill-me` or `/grill-me`; never select it implicitly from an implementation request or a missing detail.
license: MIT
disable-model-invocation: true
metadata:
  category: product
---

# Grill Me

## Overview

Start a deliberate interview about a plan, design or decision. The reusable interview procedure
lives in `grilling`; this skill supplies the user entry point and keeps the workshop separate from
ordinary implementation clarification.

Adapted from [grill-me by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/productivity/grill-me).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Invoke `$grill-me <idea>` in Codex or `/grill-me <idea>` in Claude Code and Cursor.
For example: `/grill-me Challenge this invoice export proposal before we specify it.`
An invocation authorizes the interview, not implementation, issue publication or documentation writes.

## Steps

1. Identify the idea, intended outcome and scope from the request and conversation. Inspect
   relevant sources for facts rather than asking the user to rediscover them.
2. Invoke `grilling` for the interview. Pass the scope, established decisions, relevant source
   locations and material uncertainties. Keep one copy of the interview procedure.
3. Return the settled decisions and any remaining assumptions or blockers. When the user requests
   a specification next, `to-spec` synthesizes that conversation; it is a separate user invocation.

## Gotchas

- **Starting a workshop during an ordinary fix** — authorized execution stalls; reserve this entry
  point for explicit invocation and use `requirements-clarification` for material blocking questions.
- **Duplicating the interview engine here** — two procedures diverge; delegate the session to
  `grilling` and maintain its procedure there.
- **Treating agreement as permission to publish or implement** — the interview creates side effects
  beyond its scope; return the decisions and follow the user's separate authorization.

## Constraints

- Activate only on explicit `$grill-me` or `/grill-me` invocation.
- Use `grilling` rather than maintaining a second interview procedure.
- Never infer implementation, document-writing or publication authority from the interview.
