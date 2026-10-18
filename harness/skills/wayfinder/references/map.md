# Decision Map

Adapted from the upstream `wayfinder` map and ticket structures. The version and license are
recorded in this skill. Use the established project format when it already serves the same purpose.

## Canonical map

```text
Title
Destination: the outcome whose route this effort must clarify
Notes: domain, relevant source pointers and established constraints
Decisions so far: linked question titles with short answer summaries
Open questions: question titles, canonical locations, prerequisites and current state
Not yet specified: in-scope uncertainty that is not yet a precise question
Out of scope: exclusions and their rationale
```

On a tracker with native child issues and dependency queries, open questions live in those child
issues and the map links the query rather than copying their state into another inventory. With a
local file or a tracker lacking those capabilities, keep an explicit question index and dependency
links. Name that representation; do not imply native blocking where only text links exist.

## Decision ticket

```text
Title: the decision in domain language
Type: research, prototype, grilling or task (when no established type label exists)
Question: the precise choice or investigation
Prerequisites: named decisions or unavailable evidence
Resolution criterion: what makes this question answered
Answer and evidence: populated when resolved, with proposals distinguished from facts
```

Use the established tracker convention for types. Where `wayfinder:` labels are already supported,
use `wayfinder:map` for the map and `wayfinder:<type>` for each decision; read that label when
resuming. Otherwise record the type in the ticket field above. Do not add implementation-triage
labels such as `ready-for-agent` to decision work or remove unrelated existing tracker metadata.
Applying labels still requires the write authority described in the skill.

Choose `research` for a discoverable fact, `grilling` for a user choice, or a separately authorized
`prototype` when a concrete artifact is needed to make a decision. A prerequisite `task` is justified
only by the decision it unblocks; it does not silently extend planning into delivery.

Display links by their map or ticket title and target the verified real tracker ID; never publish
placeholder issue numbers, which can link to unrelated work. Keep the detailed answer in one
canonical ticket or section and index it from the map. An unresolved question never becomes closed
merely because the session ends or a recommendation was offered.
