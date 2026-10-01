# Evidence, registry and output contract

## Evidence hierarchy

Choose evidence for the claim, not the newest document. Current applied configuration and relevant
executed checks support deployed behavior. Current code supports implemented behavior in the
inspected revision. Accepted/amended ADRs establish architectural intent, not effective enforcement.
An approved, dated policy or a responsible owner's validated declaration supports organizational
practice only within its stated scope. Existing PAS documents and template text are comparison
sources until their claims are independently validated. A newer PAS cannot silently override an
older policy, a contract or observed production behavior.

For legal, regulatory, standard or vendor certification claims, verify current authoritative
sources and the version in force; cite the supporting source and retrieval date. Do not browse to
infer private company practice. For claims of completeness or certification, apply
`design-claim-audit` when available; do not turn the PAS into a certificate.

Keep these distinctions explicit: code / configured environment / observed production / group
policy / user declaration / historical reference. A 7-day development backup setting cannot
confirm or refute a claimed 35-day production retention. A conflict on the same production resource
does require resolution before either number appears as an established fact.

## External fact registry

`security-context.yaml` (local data directory, never committed) is the canonical editable record of facts absent from the repository.
It is evidence metadata, not an architectural decision or an independent copy of code constants.
The initial topic rows are an interview backlog: null values are deliberately unknown.
Paths for historical PAS files refer to the user's local originals; do not assume another machine
has those files. Request replacement paths if needed. Resolve `pas-template.docx` relative to the
registry directory. Keep updated source hashes when a user supplies a new version.

Each atomic fact uses the existing row shape:

| Field                                        | Meaning                                                                                             |
| -------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `id`, `sections`, `subject`, `scope`         | Stable ID, exact template heading paths, claim and entity/environment boundary                      |
| `value`, `status`                            | The statement and its evidence status                                                               |
| `sources`                                    | Source ID/reference, page or section, date/version and evidence type; never just an unsupported URL |
| `owner`                                      | Responsible role or authorized contact who can verify the fact                                      |
| `validated_by`, `validated_on`, `review_due` | Human validation and review dates, ISO dates; null until known                                      |
| `question`                                   | Specific missing evidence or decision; null after resolution                                        |

Statuses are `unknown`, `declared`, `validated`, `conflict`, `expired`, `not_applicable`.
User answers start as `declared`; retain their date and role as provenance. Promotion to
`validated` requires explicit confirmation of the value and scope by the responsible human,
with a source and validation date. Dates must not be invented to satisfy the record.
`not_applicable` requires a reason and source. An elapsed review date makes a fact `expired`;
if no review date is established, flag freshness for confirmation rather than assuming permanence.
Preserve competing values and source references in a `conflict` row. Never resolve it by choosing
the nicer claim. Validate the YAML with an available parser after changes; do not install tooling.

Capture only operational facts that need human or external evidence. Keep code-derived findings
in the run's matrix with commit and environment, so the next run rediscovers them. Do not copy
entire PSSI, pentest reports or PAS documents into the registry. Preserve supplied original files.

## Required outputs

In collect mode, create `evidence-matrix.md` and `questions.md` locally. In draft mode, also create
`PAS-<project>-draft.docx`. Use names reflecting a different confirmed scope when applicable.

The matrix covers every heading, boilerplate claim and document metadata item:

| Template heading path / claim | Proposed French wording | Status | Evidence and locator | Environment / revision / date | Missing evidence / owner |
| ----------------------------- | ----------------------- | ------ | -------------------- | ----------------------------- | ------------------------ |

Separate verified repository evidence from validated external declarations. Unknown, conflicting
and expired claims remain marked in the DOCX; a declared claim is attributed and marked for
confirmation, never presented as an established control. Include checks and limitations in the
matrix, so a reader can assess the draft without needing the chat history.

Prioritize questions affecting contractual commitments: legal entity and approver, certification,
production hosting and transfers, backups/restoration/RTO/RPO, then access/logging and remaining
group policies. Ask for a document/date/scope or responsible-owner confirmation rather than a
generic yes/no. Preserve unanswered questions for the next run.

## Word handling and verification

Discover available document runtimes first. In Codex desktop use
`mcp__codex_app__load_workspace_dependencies`; use an existing DOCX-capable library such as
`python-docx` without installing dependencies. Extract PDF comparison sources with an available
PDF reader, retaining page numbers. Use OCR only if necessary and report uncertain extraction.

Start from a copy of the original template. Preserve paragraph styles, numbering, section breaks,
table cells and relationships; inspect text boxes and drawing XML for cover placeholders that
paragraph APIs miss. Never recreate the entire document from plain extracted text.
Text in headers, footers, tables or multiple runs also needs replacement. Word's table of contents
and page-number fields require refresh in compatible tooling: request a field update on open and
do not claim page references are refreshed unless rendered/checked.

Reopen the output with the DOCX reader, check heading-path coverage, required tables, unresolved
markers and leftover example names from the template (`XXX`, `xxxx`, placeholder company or product names).
Inspect claims containing quantities or absolutes against their cited evidence, especially
35 days, 3-2-1, 24/7, annual tests, never-deleted logs and “all data in one region”.
If a renderer is available, inspect the actual pages. Otherwise deliver the editable draft with
the explicit limitation that layout, field refresh and pagination are unverified.
