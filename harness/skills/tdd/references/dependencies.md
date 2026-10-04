# Test Dependencies

Adapted from the upstream `tdd/mocking.md`; the version and license are recorded in this skill.

Substitute dependencies only where the test's contract permits it. Prefer the real owned component
or a minimal in-memory implementation over mocks of internal collaborators.

| Invariant owner                                 | Relevant dependency strategy                                                          |
| ----------------------------------------------- | ------------------------------------------------------------------------------------- |
| Pure domain function                            | Call the actual function with explicit values.                                        |
| Application orchestration                       | Inject a minimal in-memory application dependency.                                    |
| Database constraint, transaction or ORM mapping | Exercise the real database and repository.                                            |
| External protocol integration                   | Exercise the protocol against a supported test endpoint or real local implementation. |
| Time or randomness                              | Inject a deterministic source at the existing boundary.                               |

Accept dependencies instead of constructing them inside the tested operation. Keep application
interfaces specific to their use case; a generic fetcher or conditional operation selector pushes
policy into test doubles and makes unrelated calls hard to distinguish.

A substitute for a third-party service can test the owned application's decisions, but it cannot
prove that the service accepts its requests or behaves as assumed. Name that limitation and use
existing integration or contract checks when the change affects that external boundary.

Avoid global module patches and test-only flags in production APIs. Add only the state and behavior
the in-memory implementation needs under test pressure; do not build a second production system.
