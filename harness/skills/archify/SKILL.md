---
name: archify
description: >
  Generate interactive HTML diagrams with Archify. Use when the user asks for Archify or requests
  an interactive HTML diagram or one-page explainer. Keep ordinary visual explanations under
  visual-explanation; never generate pages proactively.
metadata:
  category: dev
---

# Archify

## Overview

Use the official Archify distribution through a local companion, alongside `visual-explanation`.
Bun acquires [Archify 3.2.0](https://github.com/tt-a1i/archify/releases/tag/v3.2.0) from upstream
commit `208acba8d412335185b2056fedf54b3405df43fb`; `package.json` selects the archive and
`bun.lock` records its integrity. The upstream files are consumed without modification.

## Usage

Invoke `$archify <subject>` or `/archify <subject>`, or ask for an interactive HTML diagram.
For example: `Use Archify to diagram the harness deployment to Claude, Codex and Cursor.`
This companion changes upstream's storage and runtime defaults to match the local harness.

## Steps

1. Resolve this skill's deployed symlink to its canonical directory. From that directory,
   `../../../node_modules/archify-upstream/archify/` is the upstream root. Read its
   `SKILL.md` and the authoring references
   for the requested diagram type. Resolve upstream examples and resources from that root.
   If the distribution is missing, run `moon run repository:dependencies` in the canonical
   dotfiles checkout; do not install a second copy or fetch floating sources.
2. Follow `visual-explanation` for source evidence, temporary artifacts and browser inspection.
   Allocate a fresh private temporary directory, create the candidate exclusively there, and
   use that directory as the CLI working directory. Keep HTML, candidates and receipts there
   instead of upstream's default `.archify/` repository directory. Identify the repository,
   subject and producing agent in the artifact; disclose omissions and unknowns.
3. Run upstream CLI commands with `ARCHIFY_UPDATE_CHECK_DISABLED=1 bun --config=/dev/null
--no-env-file <absolute-upstream-root>/bin/archify.mjs ...`, replacing upstream's `node`
   invocation. Use the documented `finalize` command with the actual repository root for
   repository-backed evidence. Keep update networking and reminder-state writes disabled.
4. Inspect the generated HTML in a background browser, including narrow and wide views and
   useful interactions. Report installation, session activation, artifact checks and observed
   browser rendering separately. Deliver the absolute local HTML link; never open it in the
   foreground without an explicit request.

## Gotchas

- **Using the deployed directory as the package root** — the companion has no renderer. Resolve
  its canonical directory and use the upstream root above.
- **Following upstream storage or mode offers** — repository output or proactive pages conflict
  with local preferences. Keep artifacts temporary and generate only on request; omit mode offers.
- **Calling a link or receipt activation** — neither proves an agent loaded the skill or a human
  inspected the page. Name only the installation, activation and rendering actually observed.

## Constraints

- Use Bun only, the pinned distribution and the existing Moon deployment; add no global installer.
- Preserve the upstream `LICENSE` and `THIRD_PARTY_NOTICES.md` in that root,
  and bundled font and vendor licenses. Brand marks retain their own terms.
- Never modify global instructions, enable proactive modes, publish artifacts or overwrite
  divergent destinations as part of diagram generation.
- Do not claim agent activation or browser rendering from manifest selection or file existence.
