# D3 Reference Resolution Scenarios

## Execution

Use a fresh agent context to read `cross-check.md` and apply structured cross-reference extraction
(Procedure step 2), then D3 to the inputs below. Give the inputs without the expected diagnostics.
No full cross-check, repository-wide body reading,
file mutation, or live host activation measurement is required. Compare the returned scope,
availability, severity, and recommendation with the expectations afterwards.

These synthetic scenarios are contracts, not retained execution evidence. Record the source
revision, tool, environment, actual results, and unexercised hosts separately when running them.
The repository's deterministic harness checks do not execute these prose scenarios.

## Inputs

Selected collection: user, containing `workflow-automation` and `example-user`.
Other collection: project, containing `scripts` only. Both inventories are verified; no slug is
duplicated. Unless a case names another section, excerpts below are from the selected skills'
Steps sections.

| Case | Skill                                 | Excerpt                                                                                                                                | Current project availability                                                                                            |
| ---- | ------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| A    | `example-user`                        | See `release-helper`.                                                                                                                  | Neither collection contains `release-helper`.                                                                           |
| B    | `workflow-automation`                 | Follow `scripts` for scripting work.                                                                                                   | Dotfiles checkout; `scripts` is available.                                                                              |
| C    | `example-user`                        | Follow `scripts` for scripting work.                                                                                                   | Another checkout; `scripts` exists only in the dotfiles project collection and is unavailable here.                     |
| D    | `example-user`                        | If `scripts` is available and applies in the current project, follow it; otherwise follow the current project's scripting conventions. | Same other checkout as C.                                                                                               |
| E    | `example-user`                        | See `release-helper`.                                                                                                                  | Other collection inventory unavailable; selected inventory does not contain the slug.                                   |
| F    | `example-user`                        | Follow `scripts` for scripting work.                                                                                                   | Project collection contains `scripts`; availability in the current project is unknown.                                  |
| G    | `scripts` (select project instead)    | See `workflow-automation`.                                                                                                             | User collection contains `workflow-automation`; it is available to the current agent.                                   |
| H    | `example-user`                        | See `workflow-automation`.                                                                                                             | Selected user inventory contains `workflow-automation`, available to the agent; other collection inventory unavailable. |
| I    | `example-user`                        | Unlike `scripts`, this skill only discusses deployment ownership and delegates no scripting work.                                      | Dotfiles checkout; `scripts` is available.                                                                              |
| J    | `workflow-automation` (`Constraints`) | Follow `scripts` for scripting work.                                                                                                   | Dotfiles checkout; `scripts` is available.                                                                              |
| K    | `scripts` (select project instead)    | Follow `workflow-automation` for automation work.                                                                                      | User source exists; discovery explicitly establishes the skill is unavailable to the current agent.                     |
| L    | `scripts` (select project instead)    | If `workflow-automation` is unavailable, stop and report the unmet prerequisite; otherwise follow it for automation work.              | Same availability as K.                                                                                                 |
| M    | `scripts` (select project instead)    | Follow `workflow-automation` for automation work.                                                                                      | User source exists; availability to the current agent is unknown.                                                       |
| N    | `example-user`                        | Follow `workflow-automation` for automation work.                                                                                      | Selected user source exists; discovery establishes the skill is unavailable to the current agent.                       |

Case G reverses the selected collection; the two source inventories otherwise remain unchanged.
Case E replaces the verified other inventory with an unavailable lookup.
Case H retains a known selected-scope match while the other inventory is unavailable.
Cases K–M reverse the selected collection like G. Source existence or a manifest's declared
installation targets alone do not establish actual host availability; K supplies that fact explicitly.

## Expected diagnostics

| Case | Resolution                                                                          | Dependency diagnostic                                                              | Recommendation                                                                                                      |
| ---- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| A    | Dead Reference, CRITICAL; absent from both inventories.                             | None.                                                                              | Correct the slug or create a required skill in the appropriate scope.                                               |
| B    | Cross-Scope Reference, INFO; project source, currently available.                   | Unconditional Project Dependency, WARN.                                            | Make availability and current-project applicability conditional; never create or copy `scripts`.                    |
| C    | Checkout-Only Reference, INFO; project source exists, unavailable here.             | Unavailable Skill Dependency + Unconditional Project Dependency, one WARN finding. | Add the condition and project-convention fallback; never create or copy `scripts`.                                  |
| D    | Checkout-Only Reference, INFO.                                                      | Conditional reference, no warning.                                                 | None; preserve the condition and fallback.                                                                          |
| E    | Unverified Reference, INFO.                                                         | None.                                                                              | Verify the unavailable inventory; do not prescribe creation.                                                        |
| F    | Cross-Scope Reference, INFO; current availability unknown.                          | Unconditional Project Dependency, WARN.                                            | State the availability uncertainty and make the dependency conditional.                                             |
| G    | Cross-Scope Reference, INFO; user source, currently available.                      | No user-to-project dependency warning.                                             | None; never copy the user skill into project scope.                                                                 |
| H    | Resolved in selected user scope; known and currently available.                     | No dependency warning; unavailable other inventory is an audit limitation.         | None; do not turn a known match into an unverified reference.                                                       |
| I    | Cross-Scope Reference, INFO; project source, currently available.                   | No dependency, no warning.                                                         | None; a comparison is not actual skill use and needs no availability guard.                                         |
| J    | Extracted from `Constraints`; Cross-Scope Reference, INFO.                          | Unconditional Project Dependency, WARN.                                            | Make availability and project applicability conditional; do not omit the slug during extraction.                    |
| K    | Cross-Scope Reference, INFO; user source exists, known unavailable.                 | Unavailable Skill Dependency, WARN.                                                | Satisfy the user prerequisite in its proper scope; never copy it into project scope or bypass mandatory validation. |
| L    | Cross-Scope Reference, INFO; user source exists, known unavailable.                 | Unavailability explicitly handled by stopping; no warning.                         | None; preserve the mandatory prerequisite and stop.                                                                 |
| M    | Cross-Scope Reference, INFO; user source exists, current availability unknown.      | No unavailable-dependency warning on this evidence.                                | Record the uncertainty; do not infer unavailability from source location.                                           |
| N    | Resolved in selected user scope; source exists, known unavailable to current agent. | Unavailable Skill Dependency, WARN.                                                | Satisfy the user prerequisite in its proper scope; stop while unavailable, without copying or bypassing validation. |

The real `workflow-automation` constraint mentions the applicable `scripts` boundary. Resolve that
slug to `.agents/skills/scripts`, then assess its availability condition separately. Finding a
portability weakness must never become a prescription to create a missing user copy of `scripts`.
