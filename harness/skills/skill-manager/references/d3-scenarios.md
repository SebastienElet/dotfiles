# D3 Reference Resolution Scenarios

## Execution

Use a fresh agent context to read `cross-check.md` and apply only D3 to the inputs below. Give
the inputs without the expected diagnostics. No full cross-check, repository-wide body reading,
file mutation, or live host activation measurement is required. Compare the returned scope,
availability, severity, and recommendation with the expectations afterwards.

These synthetic scenarios are contracts, not retained execution evidence. Record the source
revision, tool, environment, actual results, and unexercised hosts separately when running them.
The repository's deterministic harness checks do not execute these prose scenarios.

## Inputs

Selected collection: user, containing `workflow-automation` and `example-user`.
Other collection: project, containing `scripts` only. Both inventories are verified; no slug is
duplicated. All excerpts below are from the selected skills' Steps sections.

| Case | Skill                              | Excerpt                                                                                                                                | Current project availability                                                                        |
| ---- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| A    | `example-user`                     | See `release-helper`.                                                                                                                  | Neither collection contains `release-helper`.                                                       |
| B    | `workflow-automation`              | See `scripts`.                                                                                                                         | Dotfiles checkout; `scripts` is available.                                                          |
| C    | `example-user`                     | See `scripts`.                                                                                                                         | Another checkout; `scripts` exists only in the dotfiles project collection and is unavailable here. |
| D    | `example-user`                     | If `scripts` is available and applies in the current project, follow it; otherwise follow the current project's scripting conventions. | Same other checkout as C.                                                                           |
| E    | `example-user`                     | See `release-helper`.                                                                                                                  | Other collection inventory unavailable; selected inventory does not contain the slug.               |
| F    | `example-user`                     | See `scripts`.                                                                                                                         | Project collection contains `scripts`; availability in the current project is unknown.              |
| G    | `scripts` (select project instead) | See `workflow-automation`.                                                                                                             | User collection contains `workflow-automation`; it is available to the current agent.               |

Case G reverses the selected collection; the two source inventories otherwise remain unchanged.
Case E replaces the verified other inventory with an unavailable lookup.

## Expected diagnostics

| Case | Resolution                                                              | Dependency diagnostic                   | Recommendation                                                                                   |
| ---- | ----------------------------------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------ |
| A    | Dead Reference, CRITICAL; absent from both inventories.                 | None.                                   | Correct the slug or create a required skill in the appropriate scope.                            |
| B    | Cross-Scope Reference, INFO; project source, currently available.       | Unconditional Project Dependency, WARN. | Make availability and current-project applicability conditional; never create or copy `scripts`. |
| C    | Checkout-Only Reference, INFO; project source exists, unavailable here. | Unconditional Project Dependency, WARN. | Add the condition and project-convention fallback; never create or copy `scripts`.               |
| D    | Checkout-Only Reference, INFO.                                          | Conditional reference, no warning.      | None; preserve the condition and fallback.                                                       |
| E    | Unverified Reference, INFO.                                             | None.                                   | Verify the unavailable inventory; do not prescribe creation.                                     |
| F    | Cross-Scope Reference, INFO; current availability unknown.              | Unconditional Project Dependency, WARN. | State the availability uncertainty and make the dependency conditional.                          |
| G    | Cross-Scope Reference, INFO; user source, currently available.          | No user-to-project dependency warning.  | None; never copy the user skill into project scope.                                              |

The real `workflow-automation` constraint mentions the applicable `scripts` boundary. Resolve that
slug to `.agents/skills/scripts`, then assess its availability condition separately. Finding a
portability weakness must never become a prescription to create a missing user copy of `scripts`.
