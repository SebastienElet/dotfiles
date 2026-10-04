# Design Heuristics

Adapted from the upstream `code-review` Fowler smell baseline; the version and license are recorded
in this skill. These are prompts for investigation, not automatic defects. Repository standards,
ADRs and domain invariants take precedence. Do not duplicate findings already enforced by tooling.

| Possible smell         | Review question                                                               |
| ---------------------- | ----------------------------------------------------------------------------- |
| Mysterious name        | Can a domain name explain the intent more precisely?                          |
| Duplicated code        | Are the data, invariants and expected evolution actually shared?              |
| Feature envy           | Does this operation belong to the module owning the data and decision?        |
| Data clump             | Do the values form a real domain concept with one reason to change?           |
| Primitive obsession    | Does a domain type prevent a meaningful invalid state?                        |
| Repeated switch        | Is one canonical decision being repeated across callers?                      |
| Shotgun surgery        | Does one logical change force edits across unrelated locations?               |
| Divergent change       | Does the file mix responsibilities that evolve independently?                 |
| Speculative generality | Does a consumer or requirement justify the abstraction today?                 |
| Message chain          | Does navigation leak details the caller should not know?                      |
| Middle man             | Does the adapter enforce a boundary, translate values or own a real decision? |
| Refused inheritance    | Would composition express the actual contract more faithfully?                |

For a concrete defect, state the failure mechanism and the affected contract. For a preference,
name the heuristic, the suggested improvement and its trade-off without making it blocking.
