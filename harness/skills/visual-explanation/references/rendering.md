# Rendering and inspection

## Terminal

Text diagrams, aligned before/after blocks and small tables are the portable default. They need
no image protocol, browser or rendering package. Aim to keep an unknown or narrow pane around
60 columns, but use the actual available width when known. Do not make a fixed node or row count
the decision rule: label length, branching and wrapping determine whether the visual helps.

For example, a proposed format decision can stay inside the conversation:

```text
Explanation
  |
  +-- small relationships --> CLI diagram
  +-- short comparison ----> compact table
  +-- dense / interactive -> temporary HTML link
```

Avoid color-only semantics, emoji-dependent alignment and wide box art. A simple ASCII fallback
must preserve the same relationships. Quantities need a labeled axis or scale and exact values;
text bars may show a comparison, but never imply more precision than the evidence supports.

Mermaid and terminal-image support belong to the current host, version, terminal and launch
configuration. A terminal supporting images does not prove the agent displays them in its reply.
Verify the assembled path through a real interactive session before choosing it as the default.
`--print` and `exec` samples inspect agent output; they do not establish interactive TUI rendering.
The repository's Fish wrapper sets `TERM=dumb` for Codex: include that launch path in any TUI probe.

## Temporary HTML

Use the operating system's temporary-directory allocator, with a prefix identifying the task,
then create the HTML file exclusively inside it. This protects simultaneous agent sessions and
an existing destination. On a collision or write failure, preserve existing bytes, allocate a
fresh destination if possible, otherwise deliver the CLI fallback and the failed operation.
Do not use a shared archive such as `~/.agent/diagrams/` or add output to the user's repository.
Temporary storage can outlive a turn; no cleanup hook is installed. The user requested no durable
retrieval, not premature deletion of a link they have yet to read.

Prefer semantic HTML and CSS for comparisons and document structure, inline SVG for precise
relationships or modest charts. Use JavaScript only for useful interaction. Leave meaning
available without animation, hover or color alone. Do not add decorative motion, remote fonts,
analytics, remote scripts or a framework. Sources may be ordinary clickable links; essential
rendering must work without fetching them. Follow the project's design language when relevant.

Include a visible subject, producing agent, repository and evidence date. For charts, label units,
the population and the period or snapshot date, and link the counted or measured source.
Explain material exclusions rather than implying a complete graph or causal relationship.

## Composition

Choose a deliberate visual register from the subject: a technical map, a quiet comparison or
an editorial explanation. Establish a clear type hierarchy, consistent spacing and a restrained
palette before building the page. Use the first viewport to show the answer and its main figure,
not a large title followed by a stack of generic cards.

Draw the actual relationships, changes and emphasis. A flow needs visible connections; an
after-state needs the changed element to stand out. Use surfaces and borders for real grouping,
not around every paragraph or label. Keep supporting numbers and sources secondary unless they
answer the reader's main question. Avoid exhaustive inventories when a summary and useful
disclosure preserve the meaning. Quality is visible in the rendered page, not in CSS complexity.

## Background inspection

Use existing browser tools only. Headless Chromium or an available hidden browser is suitable;
an `open` command that raises the user's browser is not a background inspection.

1. Open the artifact and capture the wide and narrow views. Inspect actual screenshots for label
   collisions, clipping, ambiguous connections, contrast and reading order. Inspect whether the
   visual hierarchy and composition help explain the subject; a technically valid box stack is
   not enough. Revise a cluttered or visually flat result before delivery.
2. Exercise every useful interactive state and keyboard operation. Verify source and navigation
   link targets, overflow behavior and that the first viewport explains the main result.
3. Check available console/page errors and failed resource loads. Offline rendering should retain
   the essential figure. A successful write or browser launch is insufficient evidence.
4. If any check cannot run, distinguish inspected properties from unverified ones. Correct a broken
   figure before delivering it as verified. If correction is unavailable, retain the CLI fallback.

Return the absolute Markdown link and, when the CLI cannot open local links, the absolute path
with a shell-quoted manual opening command. Do not execute it without the user's opening request.
