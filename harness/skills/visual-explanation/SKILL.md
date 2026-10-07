---
name: visual-explanation
description: >
  Explain relationships, diagnostics and changes visually. Use when explicitly requested or when
  a flow, comparison or quantitative explanation benefits from a visual. Make sure to use this
  skill whenever a dense explanation needs a local HTML artifact, even if unnamed. Excludes simple
  answers, routine reviews without a visual need, websites and decorative images.
license: MIT
metadata:
  category: dev
---

# Visual Explanation

## Overview

Make the mechanism, difference or decision visible with the smallest useful representation.
The shared presentation preference chooses the default; this skill builds and checks the visual.
`output-discipline` owns concise reporting. Existing diagnosis, planning and review workflows
continue to own their investigation and verdicts.

Locally adapted from ideas in `effective-html` and `visual-explainer`; the comparison records
the selected revisions. Their notices are preserved in [LICENSE.effective-html](LICENSE.effective-html)
and [LICENSE.visual-explainer](LICENSE.visual-explainer).

## Usage

Invoke `$visual-explanation <subject>` in Codex or `/visual-explanation <subject>` in Claude
Code and Cursor. For example: `Explain how this deployment reaches the three agents visually.`
Implicit use requires an explanatory need, not merely a review command or a mention of architecture.
For a trivial request, answer briefly even after explicit invocation; say why no figure helps.

## Steps

1. Establish the reader's question and inspect its sources. Use the existing investigative workflow
   where needed. Identify observed facts, assumptions and proposals before drawing; mark unknown
   relations as unknown. Keep conclusions linked to the evidence, including relevant file locations.
   Source code describes a path, not proof that it ran. For agent skills, call manifest selection
   a declaration and file installation deployment. Reserve activation for loading and applying
   the skill during a session; never label a manifest entry or an installed link as activation.
2. Choose the smallest representation that answers the question:
   - CLI text diagram for a small flow, sequence, state transition or dependency;
   - compact table or aligned before/after for a short comparison;
   - labeled bars or a chart for quantities, with source, units and period;
   - local HTML for density, exploration or interaction that genuinely helps;
   - short prose when a figure adds no information.
     Read [references/rendering.md](references/rendering.md) for terminal and HTML constraints.
3. Make one claim per figure. Label connections with their meaning and direction; use consistent
   terms. Preserve meaningful errors, branches and uncertainty. Simplify only after stating what
   was omitted. Never invent a count, time, percentage or numerical example.
4. Keep small visuals inside the reply, in a fenced `text` block when alignment matters. Fit the
   available pane; prefer vertical layouts and ASCII when glyphs or width are uncertain. Use
   Mermaid only after verifying rendering in this exact host/interface. An unrendered fence is
   source code, not the delivered visual. Check that wrapping does not destroy relationships.
5. For HTML, use a fresh private temporary directory and an exclusively created descriptive file.
   Include the repository, subject and producing agent in its identification. Keep essential CSS
   and JavaScript inline and require no build, CDN or external service. Use semantic HTML, CSS
   and SVG as needed; do not install a renderer or add a framework just to draw. Read the HTML
   inspection procedure in [references/rendering.md](references/rendering.md).
6. Inspect HTML in an available background/headless browser at wide and narrow widths; exercise
   useful controls, links and keyboard navigation. Do not open a foreground window spontaneously.
   If no suitable browser is available, or generation/rendering fails, provide a readable CLI
   alternative and name the limitation. A file existing does not prove its render works.
7. Deliver the visual or an identified absolute artifact link with a short conclusion and sources.
   Report inspection accurately. Temporary artifacts are not archived or committed; links may
   expire. Never remove a still-needed artifact during the turn or promise automatic session-end
   cleanup. Record representative observations only when the task asks for evaluation.

## Gotchas

- **Assuming CLI and app render alike** — a Mermaid fence becomes unreadable source. Verify the
  current interface or deliver the text diagram instead.
- **Opening several agents' pages automatically** — the reader loses the originating task. Keep
  openings under user control and label links by repository, subject and agent.
- **Sharing a predictable temporary filename** — concurrent agents can overwrite each other.
  Allocate a private directory and create the output exclusively; preserve an existing path.
- **Importing visual-explainer's workflow** — visual generation starts replacing investigation or
  forcing HTML. Keep local routing and format choices; consult the reference comparison only for
  provenance and design rationale.
- **Calling a screenshot a full check** — hidden states or broken links remain untested. Exercise
  the actual interactions and report precisely what was inspected.

## Constraints

- Keep established facts, assumptions and proposals distinguishable; preserve source traceability.
- Never deliver invented data, an unrendered diagram as an image, or an artifact without inspection as verified.
- Never overwrite an existing artifact or install tools, plugins, hooks or third-party skills implicitly.
- Keep artifacts temporary, local and self-contained; do not publish or archive without a separate request.
- Respect the established workflow's scope and authority; visualization grants no approval or implementation authority.

## References

- [references/rendering.md](references/rendering.md) — rendering choices, inspection and fallback
- [references/comparison.md](references/comparison.md) — evaluated upstream references and annotation boundary
