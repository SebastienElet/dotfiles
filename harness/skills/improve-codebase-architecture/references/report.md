# Visual Architecture Report

Adapted from the upstream `improve-codebase-architecture/HTML-REPORT.md`. The version and license
are recorded in this skill. Keep the report a temporary, self-contained artifact rather than an
application or a new repository reporting tool.

## Structure

Include the repository, inspected scope and date, then one card per candidate:

- Title in canonical domain language and source locations.
- Recommendation: strong, worth exploring or speculative, with a concrete reason.
- Observed problem and evidence; mark inferred behavior or unavailable evidence explicitly.
- Before/after visual showing the proposed change, labelled as a proposal.
- Expected benefit, invariant to preserve and the validation needed before implementation.
- Relevant ADR constraint or unresolved conflict, when present.

Finish with the strongest candidate and its reason. Use a short caption or legend so each diagram
can be understood without a paragraph of architectural vocabulary.

## Rendering

Use one HTML file with embedded CSS and inline SVG or ordinary HTML diagrams. Do not require CDN
scripts or install rendering dependencies. Draw call relationships, responsibilities and existing
boundaries rather than invented module-size ratios. Preserve the domain's actual names.

Resolve the operating-system temporary directory with the available filesystem tools, use a unique
filename such as `architecture-review-<unique-id>.html`, and return its absolute path. Preview the
actual saved artifact when the host offers a browser or file preview; inspect labels, diagram flow,
clipping and readability. A valid HTML source alone is not evidence of a checked rendering.

The report may present a proposal conflicting with an ADR only when observed friction justifies
investigation. Name the decision and consequence. Keep the implementation blocked by that conflict;
the report does not amend an ADR or establish a new domain guarantee.
