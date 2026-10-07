# Reference evaluation

Compared on 2026-10-06 for issue [#398](https://github.com/SebastienElet/dotfiles/issues/398).
These third-party instructions are evaluated proposals, not repository authority. ADR-029 and
ADR-040 remain accepted: maintain a local user skill and declare its three projections in Arnes.

## Generation

| Reference                                                                                                                                                 | Useful contribution                                                       | Local decision                                                                                                                                         |
| --------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [effective-html: html-diagram](https://github.com/plannotator/effective-html/blob/2ac1dfecb0f2474e75260cb6d3c9b9d6d9b5062e/skills/html-diagram/SKILL.md)  | Select notation from relationships; inspect responsive, accessible output | Primary reference for dense diagrams; preserve CLI-first selection                                                                                     |
| [effective-html: html-plan](https://github.com/plannotator/effective-html/blob/2ac1dfecb0f2474e75260cb6d3c9b9d6d9b5062e/skills/html-plan/SKILL.md)        | Preserve commitments and distinguish decisions from unknowns              | Reuse traceability principles; planning remains in its existing workflow                                                                               |
| [visual-explainer 0.12.0](https://github.com/nicobailon/visual-explainer/blob/a0ece8a01dbd1e96533ed3acc372d012dbcc552b/plugins/visual-explainer/SKILL.md) | Figures answer a specific question; explain mechanisms and differences    | Retain purpose and explicit connections; reject default HTML, numerical/table thresholds, mandatory SVG, style-library loading and its review commands |

The local skill is one workflow, with no imported skill tree, template bundle, installer or
renderer. Use local HTML/CSS/SVG without build or remote dependencies when HTML earns its cost.
The terminal remains useful for small relationships, comparisons and grounded text bars.
MIT notices from both source repositories accompany this local adaptation. No upstream executable
or dependency was copied.

## Annotation: separate decision

[Plannotator's documented surface](https://github.com/backnotprop/plannotator/blob/71c968ceee1c68af8754123d603db10523c73185/README.md)
can annotate documents, HTML and diffs and return feedback to an agent. The
[Claude integration](https://github.com/backnotprop/plannotator/blob/71c968ceee1c68af8754123d603db10523c73185/apps/hook/README.md)
uses a plugin/mod or a `PermissionRequest` hook for `ExitPlanMode`; the
[Codex integration](https://github.com/backnotprop/plannotator/blob/71c968ceee1c68af8754123d603db10523c73185/apps/codex/README.md)
uses an experimental `Stop` hook for post-turn plan review. These are upstream contracts,
not observed workstation behavior. Cursor integration is not established by these sources.

The standalone binary serves a local browser interface. Its full installer also configures
host commands, skills and hooks; upstream offers a binary-only minimal installation. Rendering
local content needs a browser. Release checks and optional AI/sharing/URL features have distinct
network behavior; local rendering must not be described as universally offline.

The harness already owns hooks through Arnes, including measurement and handoff at turn end.
A future integration must inspect ordering, continuation, timeout, hook trust and installation
ownership, rather than overwrite host settings. Hook-event overlap alone proves no conflict
or compatibility. Automatic page openings also conflict with this task's chosen delivery mode.

The user deferred annotation until visual explanations exist. Therefore this iteration does not
install Plannotator, its hooks, commands or skills, and makes no runtime success claim. Its
possible benefit—pointing to an exact part of a figure—remains unmeasured and belongs to a
separate decision after use of the generated visuals.
