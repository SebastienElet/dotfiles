# Dependencies and Module Depth

Adapted from the upstream `codebase-design/DEEPENING.md`; the version and license are recorded in
this skill.

Depth is useful only within the responsibilities the module owns. A small public interface may
coordinate several cohesive internal parts; it does not require one large file or the removal of
domain and transaction boundaries.

| Dependency                      | Design and verification                                                                                                         |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Pure computation or local state | Keep transformations explicit and test the interface owning the invariant.                                                      |
| Database or filesystem          | Preserve the I/O adapter; use the real component when its semantics are under test.                                             |
| Remote owned service            | Respect module ownership and the deployed contract; test owned decisions with an in-memory dependency and transport separately. |
| Third-party service             | Inject the boundary; distinguish application evidence from evidence of the external protocol.                                   |

An existing architectural or security boundary can have one implementation and still be necessary.
Conversely, having two implementations does not automatically justify an abstraction. Require a
concrete responsibility, invariant or variation rather than counting adapters.

Before consolidating modules, identify which decisions would otherwise spread across callers.
Keep parsing, orchestration, policy and mutation separately understandable unless their combination
is trivial. Prefer specific APIs and domain values over general operation selectors.

When moving a test surface, inventory its successful and failing behaviors before replacing tests.
Retire an old check only when its required coverage is retained or an authorized contract change
explicitly removes the requirement. Never infer equivalence from the new suite passing alone.
