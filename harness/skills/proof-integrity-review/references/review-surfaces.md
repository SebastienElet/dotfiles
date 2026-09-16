# Review Surfaces

Select the surfaces the diff actually changes. The classifier is a routing aid, not a complete
inventory of verification behavior.

| Surface                      | Relevant evidence                                                                                                                                                                                                 |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CI and task routing          | Native schema/configuration inspection, the resolved invocation paths, and an executed fault showing that the intended check is reached. Compare PR and destination-branch execution when their contracts differ. |
| File discovery               | The independently defined eligible set and files actually checked. Exercise an otherwise omitted eligible file; filenames, renames, deletions, and empty discovery can change coverage.                           |
| Test oracles                 | A representative broken behavior rejected by the actual test, then restored success. A receipt-schema test cannot stand in for a test of the product.                                                             |
| Whole-tree gates             | Evidence on the integrated candidate when destination changes affect the invariant. A green isolated branch is not integrated-tree evidence.                                                                      |
| Caches                       | Identify readers, writers, keys, restore/upload ordering, and relevant inputs. Use a cold or invalidated run when a cache hit could hide the changed behavior.                                                    |
| Tool versions                | One canonical pin where possible; native execution or rendering evidence for affected semantics. Avoid maintaining a second list solely to assert equality.                                                       |
| Concurrent runs              | Exercise simultaneous runs when resource names, ports, temporary paths, or caches can collide. Separate successful sequential runs do not establish isolation.                                                    |
| Normative document structure | Render or parse the structure when the claim depends on it. Formatting alone does not show that a row remains inside a table. Route domain authority claims through the existing domain audit workflow.           |
| Agent review policy          | Use a concrete violating scenario and a safe counterexample. Record the tested host, context, outcome, and limitations; a scenario file alone is not an executed evaluation.                                      |

For every high-impact claim retain the actual fault, expected refusal, observed refusal, restoration,
and observed success. If the experiment is unsafe or unavailable, report the gap instead of adding
a substitute assertion that cannot exercise the claimed mechanism.
