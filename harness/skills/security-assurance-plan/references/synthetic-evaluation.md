# Synthetic evaluation record

## Scope and method

Executed on 2026-10-06 by Codex in the main implementation session. These are manual applications
of the revised skill to the six scenarios in issue #399, followed by inspection of actual saved
synthetic artifacts. They are not isolated activation evaluations or cross-agent success rates.
No client directory, historical conversation, operational registry or private document was read.
All source roles, scopes, dates and document content below are fictional, including the 2030 dates.
The saved PAS fixtures are in French; wording and title excerpts below are English translations,
not a transcription of the French bytes identified by the hashes.

Behavioral contract: draft and revise evidence-proportionate claims, reuse applicable decisions,
and reconcile the PAS, matrix and verification manifest while preserving document structure and
keeping editorial acceptance, document checks and human approval separate.

The initial instruction audit confirmed mandatory confirmation markers for declarations and the
absence of an explicit revision/manifest contract. It did not establish a real overloaded PAS,
repeated interview or numbering incident. The scenarios exercise the requested behavior rather
than diagnosing historical use.

Environment: macOS 27.0.1, Darwin arm64, Python 3.14.8 standard-library ZIP/XML reader and writer,
native Quick Look CLI, and installed Microsoft Word 16.109.3 through AppleScript. No
dependency, validator, script or pipeline was added to the repository for this execution. Disposable
fixtures were constructed with inline commands and were not deployed or published.

## Saved artifacts and actual checks

- `baseline.docx`: synthetic template/current draft, version 2.4, cover, history table, blank
  approval/signature table, paragraph styles, explicit numbered heading, bookmark, TOC, REF and
  page-reference fields, two page breaks and a footer PAGE field.
  SHA-256: `8e3d8aa820883c2f9de814e7a6f8afb06fb8ab4389c12049dc891e8e07fb5f47`.
- `candidate.docx`: copy revised to version 2.5 with S5's retained wording and a new history row.
  SHA-256: `7c75f4f6ecd3bfb2ca6833a028ec3eee2672a12813d26794b1143906c67d4fe1`.
- `evidence-matrix.md`: S1–S5 wording, source nature, scope, fictional provenance, verification,
  reservations and retained decision; S6 document metadata.
  SHA-256: `0fa792927be6631e02a63f5573daf3a461e105deb204effe06f83093f8f961c2`.
  `verification-manifest.md` identifies this matrix and candidate, with check results and limits
  summarized below. These disposable files are execution artifacts, not committed deliverables.
- `stale-reference.docx`: negative variant with heading `3.4 Access management`, cached TOC
  `2.4 Access management — 3` and cached REF `2.4 Access management` (titles translated).
  SHA-256: `c9da3b1a3bd836701142ad36b84b2e32df2684a2fb3fbe49b393102373d71c71`.

The saved ZIP/XML was reopened and its paragraphs, tables and field codes extracted for manual
comparison. Both candidate tables, both page breaks and the bookmark remain present. Styles,
document relationships and footer parts have the same hashes as the baseline. History retains
versions 2.3 and 2.4 and adds 2.5; approval and signature cells remain empty. These checks establish
preserved file structures in this fixture, not native OOXML schema conformance or rendered fidelity.

Quick Look produced thumbnails and an HTML preview. The actual thumbnails of the candidate and
negative variant were visually inspected for wording, cover/history, blank approval/signature cells
and full numbered references. An initial synthetic candidate had cover version 2.5 but no matching
history row; the mismatch was detected, corrected in the actual history table and rechecked.
The final candidate hash above identifies the corrected artifact; results for the earlier artifact
are superseded.

Word's AppleScript open failed with `AppleEvent timed out (-1712)`. No Word save/reopen round-trip,
field refresh or PDF export completed. Quick Look's preview is unpaginated HTML: it displays cached
TOC text meaning “TOC not updated” and PAGE/page-reference values `0`. Field presence is verified; refreshed
TOC/page numbers, page destinations, complete paginated layout and style fidelity are **unverified**.
No PDF was produced, and the manifest does not call the document fully verified.

## Executed scenarios

### S1 — Declaration without independent observation

Input: a synthetic responsible role declares quarterly access review for scope A on 2030-01-01;
no review record or independent observation is supplied.

Actual PAS wording (English translation):

> According to the responsible role's declaration, access review occurs quarterly.

Matrix result: human declaration, attributed role/date/scope, `declared`, independent observation
absent. No conflict or independently verified practice was inferred, and no collection marker was
added. Human confirmation would validate the declaration's stated value/scope, not observe execution.
The document, matrix and manifest retain this distinction. Result: manual wording/provenance check
passed; paginated rendering remains unverified.

### S2 — Requirement without evidence of application

Input: a fictional policy requires quarterly review; a static parameter is quarterly; no execution
record is supplied for scope A.

Actual PAS wording (English translation):

> The policy requires quarterly review. The inspected parameter is configured quarterly; the available evidence does not establish actual execution.

Matrix result: documentary requirement and static configuration remain separate from execution.
The missing scoped review record remains an evidence request. Neither compliance nor demonstrated
breach was concluded. Result: manual wording/evidence distinction check passed.

### S3 — Explicit contradiction and distinct-scope variant

Input: fictional declaration of 30-day retention and dated observation of 7 days for the same
object, scope A and period.

Actual PAS wording (English translation):

> The declaration states 30-day retention; the synthetic observation dated 2030-01-01 establishes 7 days for the same object, scope and period. This contradiction remains unresolved.

Matrix result: both sources retained, `conflict`, responsible-role resolution pending; the PAS
visibly reserves the claim rather than treating 30 days as established. The manually executed
variant changes the observation to scope B: both scopes were retained and the result was recorded
as no established contradiction, with scope A observation still needed. Result: both manual scope
and contradiction checks passed; no scope A practice was inferred from scope B.

### S4 — Objective without demonstration and divergent matrix

Input: fictional restoration target below four hours, no exercise.

Actual PAS wording (English translation):

> The restoration target is below four hours; no available exercise demonstrates this duration.

Matrix result: objective only, scoped restoration exercise missing; no guaranteed duration or test
result. A deliberately divergent matrix changed four hours to two hours. Reading that row against
the saved DOCX and identified manifest exposed the mismatch. The variant was rejected and the
four-hour matrix retained. Result: manual target/guarantee and cross-artifact divergence checks passed.

### S5 — Previously accepted correction and targeted reopening

Input: the supplied synthetic decision accepts attributed wording for unchanged scope A, valid
sources and period; the current draft contains an old formulation. Only applying the wording is
requested, with no PAS approval supplied.

Actual replacement in the saved candidate (English translation):

> According to the declaration collected, access review occurs quarterly.

The retained wording was applied without a new user question or complete collection. The matrix
records the editorial decision and unchanged applicability; the manifest records S5 and checks of
the affected document location and companion artifacts. Approval/signature cells stayed blank.

Two manual variants were executed and their outcomes recorded: scope changes to B reopen only S5
for changed applicability; a same-scope/period monthly observation contradicting the quarterly
declaration reopens S5 with both sources and a visible reservation. Independent S1–S4 claims remain
unchanged. Result: manual correction/reopening checks passed. This demonstrates this session's
handling, not a guarantee that every agent will reuse prior decisions.

### S6 — Version and heading-number collision

Input: version `2.4` and heading `2.4 Access management` (translated); only the document version becomes `2.5`.

Actual final candidate: cover `Draft — version 2.5` (translated), retained historical versions plus new 2.5
row, heading and REF still `2.4 Access management` (translated). Styles/relationships/footer were preserved in
the saved package. The candidate was reopened before checking.

Negative variant: heading `3.4 Access management`, TOC and REF still `2.4 Access management` (translated). The
different full numbered titles were detected in both extracted text and the actual Quick Look
thumbnail despite identical title words. The negative variant remains intentionally divergent.
Result: targeted version editing and stale numbered-reference detection passed manually; native
field refresh and pagination checks are unavailable, so the complete rendered-document criterion
is **not demonstrated**.

## Limits and validation boundaries

Editorial acceptance is simulated input, not a real person's approval. No approver, signature,
approval date or certification was supplied or invented. The synthetic manifest explicitly records
draft status and failed/unperformed checks. All claims in this record concern this manual execution.

The initial local doctor audit passed frontmatter, section order, gotchas/constraints, resource
routing, shell-placeholder safety, canonical-scope uniqueness and README membership; optional
activation evals were absent. The description is unchanged (335 characters). Standard validation
is unavailable: `skills-ref` is not installed. These procedural checks are not behavioral evidence.

The final local doctor audit preserves all baseline PASS checks; both references resolve and all
three skill files remain below 500 lines. The existing Prettier and CSpell entry points
(`tooling/check-prettier.ts` and `tooling/check-cspell.ts`) passed on indexed skill Markdown,
including this record, using the repository's available dependencies and fixture-local CSpell
configuration. CSpell checked 97 skill Markdown files with no issues. English translations replaced
the initially rejected French quotations; no spelling configuration or dictionary was changed.
The user README was regenerated twice with byte-identical results, no membership/description changes
or invalid entries; only optional-folder ordering changed to match the canonical template.
Independent standards and contract reviews found no concrete defects; they did not rerun the fixtures.

The supported Claude/Cursor/Codex user links resolve to canonical skill sources, and project
adapters still target `../.agents/skills`; this change does not redeploy the host installations.
Linux, Claude, Cursor, an isolated Codex activation run, Word on Windows, complex template fields,
text boxes, automatic heading numbering and full paginated PDF rendering were not exercised.
No result here establishes the earlier incident hypotheses or universal document compatibility.
