---
name: prose-edit
description: >
  Revise existing prose while preserving the writer's voice and meaning. Use when asked to polish
  or clarify a draft. Make sure to use this skill for editorial revisions of messages, documentation,
  or PR descriptions. Excludes code review, issue scoping, skill maintenance, and drafting from scratch.
metadata:
  category: ops
---

# Prose Edit

## Overview

Improve an existing draft with the smallest useful edit. The minimal-edit approach is inspired by
[no-ai-slop](https://github.com/petergyang/no-ai-slop); preserve distinctive writing and useful detail.

## Usage

Use `/prose-edit`, `$prose-edit`, or a request such as "Polish this team message before I post it."
Apply this skill to the requested draft only. If the draft is missing, ask for it.

## Steps

1. Read the full draft and identify its meaning, voice, factual details, and uncertainties.
2. Preserve the voice and nuances. Remove generic sentences or make them specific using only
   available facts; change only what improves comprehension.
3. Compare the revision with the draft: retain useful details and uncertainty, remove unsupported
   additions, and leave already-clear passages intact. Return the revised text in the requested format.

## Gotchas

- **Treating every response as a draft** — routine answers acquire an extra editing workflow.
  Apply this skill only to an existing text the user asks to revise.
- **Polishing away personality** — blunt language, humor, or an admission becomes generic prose.
  Keep those traits when they carry the writer's voice.
- **Turning an uncertainty into advice** — a tentative possibility becomes a new recommendation.
  Preserve the writer's position and degree of certainty.

## Constraints

- Do not invent facts, figures, opinions, recommendations, or sources to make prose more concrete.
- Do not replace precise technical terms merely for variety or enforce a banned-word list.
- Keep code review, issue scoping, and skill maintenance in their owning workflows; this skill does
  not authorize changes to code, requirements, or agent instructions.
