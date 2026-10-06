# Evidence, registry and output contract

## Sources, scope and verification

Choose evidence for the claim, not the newest document. Current applied configuration and relevant
executed checks support deployed behavior. Current code supports implemented behavior in the
inspected revision. Accepted/amended ADRs establish architectural intent, not effective enforcement.
An approved, dated policy establishes a requirement, not its execution. A responsible owner's
declaration reports what that role states; human confirmation establishes acceptance of that
declaration's value and scope, not independent observation of the practice. Existing PAS documents
and template text are comparison sources until applicability is established. A newer PAS cannot
silently override an older policy, a contract or observed production behavior.

Classify each claim on three separate axes: source nature, claim scope (object, entity, environment
and period), and verification actually obtained. One status must not collapse those axes.

| Source nature           | What it supports                                                               | What it does not establish                                     |
| ----------------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------- |
| Documentary requirement | What a policy or contract requires in its applicable version and scope         | Execution, compliance or breach without evidence of practice   |
| Human declaration       | What the attributed role declared, with provenance and date                    | Independently observed practice, even after human confirmation |
| Static configuration    | The inspected configured state for the stated revision and environment         | Deployment, enforcement or execution in another environment    |
| Operational observation | What the dated check, record or exercise actually observed within its coverage | Universal or continuing effectiveness beyond that coverage     |
| Objective               | An explicitly stated target and its scope                                      | A guarantee or achieved result without demonstration           |

Missing evidence proves only that evidence was not found; it establishes neither compliance nor
breach. Call sources contradictory only when their claims are incompatible for the same object,
scope and period. Retain both sources and the unresolved reservation; different environments or
periods require an applicability check, not automatic conflict classification.

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
with a source and validation date. It denotes human validation of that fact, not independently
observed execution, wording acceptance or approval of the whole PAS. Preserve the source type and
verification limits in the matrix. Dates must not be invented to satisfy the record.
`not_applicable` requires a reason and source. An elapsed review date makes a fact `expired`;
if no review date is established, flag freshness for confirmation rather than assuming permanence.
Preserve competing values and source references in a `conflict` row. Never resolve it by choosing
the nicer claim. Validate the YAML with an available parser after changes; do not install tooling.

Capture only operational facts that need human or external evidence. Keep code-derived findings
in the run's matrix with commit and environment, so the next run rediscovers them. Do not copy
entire PSSI, pentest reports or PAS documents into the registry. Preserve supplied original files.

## Required outputs

In collect mode, create `evidence-matrix.md` and `questions.md` locally. In draft mode, also create
`PAS-<project>-draft.docx` and `verification-manifest.md`. Use names reflecting a different confirmed
scope when applicable. The manifest records draft verification; its presence does not imply approval.

The matrix covers every heading, boilerplate claim and document metadata item:

| Template heading path / claim | French wording / document version | Claim scope | Source nature / evidence and locator | Verification obtained / fact status | Environment / revision / date | Material reservations | Missing evidence / owner | Retained decision / applicability / revision reason |
| ----------------------------- | --------------------------------- | ----------- | ------------------------------------ | ----------------------------------- | ----------------------------- | --------------------- | ------------------------ | --------------------------------------------------- |

Keep internal collection detail in the matrix while making the PAS readable without it. An
attributed declaration or explicit objective needs no mechanical collection marker. Use
`À confirmer — <specific missing fact>` when unresolved text would otherwise assert an unsupported
fact. Keep any material weakness, expired evidence or contradiction necessary to understand the
claim visible in the PAS; moving evidence requests into the matrix must not conceal it. Illustrative
wording below is in English; write the corresponding PAS text in French:

- `According to the responsible role's declaration, access review occurs quarterly.`
  The matrix records the dated declaration and absence of independent observation; confirmation,
  if obtained, remains distinct from observation.
- `The policy requires quarterly review. The inspected parameter is configured quarterly; the available evidence does not establish actual execution.`
  Neither a compliance conclusion nor a demonstrated violation follows.
- `The restoration target is below four hours; no available exercise demonstrates this duration.`
  The matrix records the missing restoration exercise and its required scope.

Include checks and limitations so the draft can be assessed without chat history. For every modified
claim, compare its wording, scope, document version, reservations and retained decision across the
PAS, matrix and manifest. Signal and correct any divergence before declaring that check complete.

Prioritize questions affecting contractual commitments: legal entity and approver, certification,
production hosting and transfers, backups/restoration/RTO/RPO, then access/logging and remaining
group policies. Ask for a document/date/scope or responsible-owner confirmation rather than a
generic yes/no. Preserve unanswered questions for the next run.

## Revision and validation boundaries

1. Start from the supplied current draft, matrix, manifest and retained decisions. Reuse accepted
   wording and human validations while their scope, sources and validity remain applicable. Check
   applicability rather than restarting collection; a changed file hash is a prompt to inspect the
   relevant changes, not proof that every decision expired.
2. For an editorial correction with unchanged meaning and valid sources, apply the retained wording
   without asking for the same agreement again. Record the affected claim and correction; check the
   corresponding document locations, matrix, manifest and any layout or references they affect.
3. Reopen only affected claims and consequences when new evidence contradicts them, evidence expires
   or scope changes. Record the reason and preserve prior decisions as history, rather than silently
   overwriting them. Continue independent claims. Refresh code-derived evidence only as relevant to
   its applicability; do not treat last run's matrix as current deployed-state proof.
4. Keep three decisions distinct: **wording acceptance** approves a formulation within its stated
   scope; **document verification** records checks on an identified artifact; **human approval** is
   the named approver's explicit decision on the identified PAS version. Neither of the first two
   fills a signature or approval field or establishes certification. Without explicit human approval,
   leave those fields blank and retain draft status. Preserve supplied approval history as history;
   never carry approval forward to a revised version without a decision applicable to that version.

## Verification manifest

Record the following in `verification-manifest.md`, without introducing a new registry schema:

- Exact saved DOCX identity: filename, document version, cryptographic hash, scope and the companion
  matrix identity. Identify any rendered PDF and its hash and tie it to that saved DOCX. Recheck or
  invalidate affected results after any artifact changes; a render of an earlier file is insufficient.
- Modified claim/heading references with the applied wording, scope, version, reservations and
  retained decision, or exact locators into the identified matrix for those details. State the result
  of comparing them with the delivered PAS. Record editorial acceptance and targeted reopenings.
- Each check actually executed, tool/version, environment, result, and exact coverage. Separate
  checks of XML/field presence, actual field update, full numbered TOC/cross-reference targets and
  visually inspected pagination. Log detected divergences and their correction/recheck outcomes.
- Unperformed, unavailable or failed checks and their consequences. Never label the document fully
  verified when rendered layout, field refresh or pagination was not checked.
- Draft/approval state, distinguishing wording decisions and document checks from human approval.
  Record approver, date and exact approved version only when explicitly supplied; never invent them.

## Word handling and verification

Discover available document runtimes first. In Codex desktop use
`mcp__codex_app__load_workspace_dependencies`; use an existing DOCX-capable library such as
`python-docx` without installing dependencies. Extract PDF comparison sources with an available
PDF reader, retaining page numbers. Use OCR only if necessary and report uncertain extraction.

Start a new draft from a copy of the original template; revise a copy of the current draft when
applying retained corrections, comparing required structure to the template. Preserve paragraph
styles, numbering, section breaks, table cells, relationships, cover, version history and approval
fields; inspect text boxes and drawing XML for placeholders that paragraph APIs miss. Never recreate
the entire document from plain extracted text. Preserve unrelated history and metadata.

Identify version-bearing fields on the cover, headers/footers and history before editing. A version
change from `2.4` to `2.5` must leave a heading `2.4 Access management` and its references unchanged.
If that heading is deliberately renumbered to `3.4 Access management`, a reference still reading
`2.4 Access management` is stale even though the title words match. Compare every TOC entry and
cross-reference with its complete numbered target heading, including rendered automatic numbering;
do not compare title words alone. Preserve field codes, bookmarks and links where used.

Text in headers, footers, tables or multiple runs also needs targeted replacement. Inventory TOC,
cross-reference and page-number fields separately from their cached results. Requesting an update
on open or finding a field code is not evidence that it refreshed. Use compatible tooling to update
fields, save, reopen the saved DOCX and render that artifact; then inspect actual TOC entries,
cross-reference labels and page destinations. Record which updates and page checks actually ran.

Reopen the output with the DOCX reader, compare heading paths, required styles, tables, cover,
history, approval fields, numbering, TOC and references with the baseline and intended changes.
Check unresolved markers and leftover example names (`XXX`, `xxxx`, placeholder company or product names).
Inspect claims containing quantities or absolutes against their cited evidence, especially
35 days, 3-2-1, 24/7, annual tests, never-deleted logs and “all data in one region”.
If a renderer is available, inspect the actual saved artifact's pages, including cover/history,
approval fields, section breaks, tables and full numbered references. Otherwise deliver the editable
draft with the manifest explicitly recording layout, field refresh and pagination as unverified.
