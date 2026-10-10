# Failure classes

Fifteen questions to put to the diff in phase 3. Each is a question, not a checklist item to tick: the
answer is a sentence about _this_ diff. Record one of four outcomes per class.

- **not applicable** — the diff does not touch that concern; say why in one clause.
- **holds because `<evidence>`** — name the constraint, the transaction, the schema, the test.
- **unproven** — the concern applies, but sufficient relevant evidence is missing; name the gap.
- **broken by `<mechanism>`** — an ordered sequence of steps that ends with a violated invariant.

`broken by` becomes a blocking defect only when its mechanism is written out.
"This looks racy" is not a mechanism; "request A snapshots at T1, request B writes at T2, A commits
at T3 and B's row is absent from the successor" is.

`unproven` becomes an evidence blocker only when the missing proof is essential to approval under
phase 5: explain why it is essential and what would lift the gap. Otherwise retain a bounded
reservation with its lift condition. Missing proof never establishes a violated invariant; do not
invent a failure mechanism or record `holds` to fit the verdict.

The same questions apply to a design document: the mechanism under review is what the document
authorizes someone to build. A document that leaves a class open produces the defect during
implementation instead of preventing it during review.

## 1. Atomicity and ordering

**Ask:** is the read that grounds the decision inside the same transaction as the write it
authorizes?

**Broken when:** a snapshot, a count, or a set of validation checks runs before the transaction
opens. Anything committed in that window is invisible to the decision but visible to everyone
afterwards — the write is authorized by a state that no longer exists.

**Lift:** the read moves inside the transaction, at an isolation level that actually prevents the
interleaving, or the invariant is enforced by a constraint the concurrent write cannot satisfy.

## 2. Retry idempotence

**Ask:** after a conflict, does the retry re-read the winning state, or does it replay the decision
it computed before the conflict?

**Broken when:** the retry loop wraps only the write. The second attempt re-applies a stale
decision and produces a duplicate — a second successor, a second invoice, a second charge — with no
error anywhere.

**Lift:** the retry re-enters the whole read-decide-write unit, or the write is keyed so that the
duplicate is rejected by the database.

## 3. Invariant without a constraint

**Ask:** which index or database constraint makes this invariant unfalsifiable?

**Broken when:** the answer is "the service checks it". Application-level uniqueness or ordering does
not exist under concurrency: two processes both read "absent" and both insert. A unique index, an
exclusion constraint or a serializable transaction is the only thing that survives.

**Lift:** the constraint exists in a migration, and a test proves the second writer receives the
violation rather than succeeding.

## 4. Authorization as a side effect

**Ask:** is the access check named in a dedicated call, or obtained incidentally through a function
called for another reason?

**Broken when:** authorization happens because some loader happens to filter by the caller's scope. A
refactor that swaps that loader for a cheaper query removes the check, every test stays green, and
nothing in the diff looks like a security change.

**Lift:** an explicit authorization call whose removal fails a test written against it.

## 5. Tenant scope

**Ask:** can a client-supplied identifier override the scope established by authentication?

**Broken when:** a tenant, organization or account id arrives in the request body or path and is used
without being reconciled against the authenticated scope. The endpoint then reads or writes across
tenants for any caller who edits an id.

**Lift:** the authenticated scope is the only source of the identifier, or the supplied value is
verified against it and the mismatch is rejected — with a test for the mismatch.

## 6. Error contract

**Ask:** are the returned status codes documented, and do the documented ones match the real
behavior?

**Broken when:** the code returns 409 where the documentation promises 412, or a new precondition
failure is added and nothing says so. Clients build retry logic on these codes; an undocumented
change is a silent behavioral break in every consumer.

**Lift:** documentation and implementation agree, and the mapping is asserted somewhere that runs in
CI. This class alone is usually a reservation, not a block — unless a consumer's retry path depends
on the code that changed.

## 7. Deferred functionality

**Ask:** is the absent regulatory or business control declared as an explicit contract — what is
missing, why it is acceptable now, and the condition that lifts it — or is it silently omitted?

**Broken when:** a control the domain requires is simply not there, and the PR says nothing. The gap
then becomes invisible: it survives review, ships, and gets discovered by the party the control
protected.

**Lift:** the contract is written down in the code and in the PR, with its lift condition. Silence is
the defect; a stated deferral with a named condition is a reservation.

## 8. Parsing versus assertion

**Ask:** is the external value validated by a schema, or degraded by a ternary that turns anything
unexpected into a plausible default?

**Broken when:** `input.mode === "strict" ? "strict" : "lenient"` — a typo, a renamed enum member or
a null silently selects the permissive branch. The system keeps running and produces wrong results,
which is strictly worse than rejecting the input.

**Lift:** the value is parsed into the type it claims to be, and the unparseable case is an error
with a message naming what was received.

## 9. Upstream control as a trust boundary

**Ask:** does this diff keep validation or confinement at the trust boundary distinct from every
upstream allowlist, index, internal view or owned schema?

**Broken when:** an upstream control is treated as proof that its output is trustworthy, so the
boundary guard does not run. The upstream mechanism controls selection or representation, not the
validity or confinement of every value it emits, and its drift crosses the boundary unchecked.

**Lift:** the boundary guard still validates the value or confines it against the canonical boundary
when it is consumed, independently of the upstream control, and each mechanism remains explicit.

## 10. Claim stronger than mechanism

**Ask:** does every guarantee claimed by this diff match what its mechanism actually establishes?

**Broken when:** the wording promotes a weaker property into a stronger outcome — intent into
effect, integrity into legal proof, or proposal into authority. A reader then builds or relies on
an assurance that no mechanism delivers.

**Lift:** the claim uses the weaker term the mechanism supports when no external obligation requires
more; otherwise the missing mechanism, evidence or authority makes the stronger guarantee true.

## 11. Hostile input in normalization and redaction

**Ask:** does every normalization or redaction of an untrusted URL or path use a real parser or
explicit grammar-aware analysis of its authority and boundaries, with a table of hostile inputs?

**Broken when:** an ad hoc regex decides identity or removes credentials without respecting those
boundaries. For example, `[^/@]*@` stops at the first `@`, leaving credentials after a second one,
and turns `https://evil.invalid?@github.com/...` into an apparently trusted GitHub URL. A query or
fragment becomes authority, or a secret survives into displayed output. A repository picker rejects
equivalent Git remotes because it requires an SCP user, compares DNS hosts literally, or preserves
the default SSH port as a distinct identity.

**Lift:** a real parser or explicit authority/boundary analysis replaces the ad hoc regex. An
executed hostile-input table covers multiple `@`, `@` in query or fragment, backslashes, control
characters, newlines, bidirectional controls, encoded delimiters, and repeated or adjacent
occurrences. Assert the expected identity or rejection and safe displayed output for each relevant
case, including malformed inputs; normalization must not turn a rejected authority into a trusted one.

For Git remotes, require protocol-aware parsing, including the SCP grammar `[user@]host:path`.
Extend the hostile-input table with these identity and rejection cases; provider rules must come
from the supported provider's contract, not a blanket lowercasing of paths.

| Input family                                                                              | Required observation                                                                                                                             |
| ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `github.com:Owner/Repo.git`, `git@github.com:Owner/Repo.git`                              | Optional SCP user does not change repository identity.                                                                                           |
| `ssh://git@github.com/Owner/Repo.git`, `ssh://git@github.com:22/Owner/Repo.git`           | Omitted and explicit default SSH port 22 identify the same repository.                                                                           |
| `git@GitHub.COM:Owner/Repo.git`, `ssh://git@GITHUB.COM/Owner/Repo.git`                    | DNS host casing does not change identity in either syntax.                                                                                       |
| GitHub `Owner/Repo` versus `owner/repo`; GitLab `Team/Project` versus `team/project`      | GitHub case aliases match. For other providers, follow the supported identity contract; exact GitLab path matching may be a declared local rule. |
| `Repo`, `Repo.git`, `Repo/`, `Repo.git/` in supported remote forms                        | Final `.git` and trailing slash normalize according to the repository identity contract; interior path segments remain intact.                   |
| `ssh://git@github.com:2222/Owner/Repo.git`, a foreign host, malformed SCP and local paths | Preserve distinct endpoints or reject with a diagnostic; never alias them to a trusted default endpoint.                                         |

Missing executed cases are `unproven`; a traced rejection or trust-boundary violation is
`broken by` its mechanism. Class 12 separately checks every display output.

## 12. Complete display-output coverage

**Ask:** are all display outputs inventoried and covered by the presentation cleanup, including
stdout, stderr and every textual report field such as `version`, `steps.*` and `deferrals`?

**Broken when:** only the main error message or one output stream is cleaned. Untrusted host
metadata or a rejected path reaches another textual report field, which is serialized or rendered
unchanged; credentials or terminal controls still leak through that output despite the cleaned error.

**Lift:** trace every display output and textual report field to the presentation boundary, and
exercise hostile values through each reachable output in success and failure paths, including
stdout and stderr. Keep raw values available for internal decisions and execution; displayed
cleanup must not change trust or error classification. Name excluded internal-only fields and why
they cannot reach display instead of treating a cleaned sample report as complete coverage.

## 13. Unchanged public contract without exported comparison

**Ask:** does a claim that a public schema, MCP or CLI contract is unchanged have a byte-for-byte
comparison of the projection actually exported on the base and head?

**Broken when:** source types, shared schemas, passing validation fixtures or adapter reuse stand
in for exported equality. For example, `mcpSchema(HandoffCapsuleSchema)` can preserve accepted values
while dropping the published `version` description. Without the exported comparison, the
unchanged-contract claim is `unproven`; source similarity does not establish compatibility.

**Lift:** capture the actual public export on the named base and head with the public export command,
each revision's pinned dependencies and a recorded environment, then compare its bytes, including
descriptions and other metadata. Do not strip metadata or normalize away differences. If bytes differ, restore equality
or declare and assess the contract change; if the comparison cannot run, retain `unproven` and
state the precise evidence gap.

## 14. Selection without complete accounting

**Ask:** can every input occurrence be traced to a retained element or a diagnosed rejection?
Do eligibility and destination follow an independently established contract?

**Broken when:** a candidate disappears before classification, an unavailable dependency is silent,
or a missing expected project becomes a successful empty selection. Filtering before counting hides
loss; equal counts still hide wrong eligibility or routing. For example, a selector keeps a parent
with an unfinished child despite a rule excluding it, or sends an update to a creation-only handler.

**Lift:** observe `input = retained + diagnosed rejections` by identity and occurrence, with no loss
or double-counting and all required diagnostic reasons. Keep ambiguity visible until resolution or
rejection; an unclassifiable input fails explicitly. Validate expected projects and targets before
filtering. Derive eligibility, required prerequisites and authorized destinations from the independent
contract, then exercise valid, empty, ambiguous, unavailable and wrongly routed cases. A successful
empty result accounts for every exclusion. Native inspection, task graphs and existing diagnostics
suffice when they expose this proof; do not require a new mirror gate or validator.

Missing accounting is `unproven`; observed loss, wrong eligibility or routing is `broken by` its
traced mechanism. The Reporting section determines whether its consequence blocks.

## 15. Performance or rewritten instructions without comparison

**Ask:** before installation or merge, is the claimed performance or rewritten instruction's behavior
compared on base and head against a criterion fixed before measuring?

**Broken when:** shorter wording, source similarity, projection checks or green correctness tests
substitute for that comparison. For example, a rewritten instruction is installed without observing
its decisions, or an inconclusive benchmark is presented as satisfying a required performance budget.
Missing or inconclusive measurements leave the claim `unproven`; they demonstrate neither a
regression nor a gain.

**Lift:** bind observations to named base/head revisions, exact text or executable inputs, pinned
dependencies, recorded host/model/runtime and environment, identical scenarios or workload, metric
and a criterion declared before measuring. For instructions, execute violating and safe cases in
fresh contexts with each wording. For performance, preserve equivalent facts and workload, account
for noise and order effects, and keep inconclusive results `unproven`. Reuse traceable relevant
measurements under phase 4; a single favorable sample proves no uplift.

Compare before adopting a rewritten instruction. Before merge, remove or bound an unsupported
optional performance claim; a required budget or adoption criterion still blocks until qualified.
Class 13 separately requires byte-for-byte comparison of public exports.

## Reporting

Classes 1, 2, 3, 4, 5, 7, 9, 11 and 12 name mechanisms that lose data, corrupt state or cross a security
boundary: when broken, they block. Class 14 blocks when its demonstrated loss, wrong eligibility or
routing removes required checks or prevents a promised workflow; other diagnostic omissions are
bounded reservations unless their evidence is essential under phase 5.
Classes 6, 8, 10 and 13 are usually reservations — promote class 6
when a concrete consumer's retry path depends on the changed code, class 8 when the degraded value
reaches a person or a legal act, and class 10 when the claim is legal, evidentiary or contractual.
For class 13, label the unchanged-contract claim `unproven` until the exported bytes are compared;
block when the comparison exposes a promised compatibility break or when compatibility proof is
essential to approval. Missing hostile-input or output coverage in classes 11 and 12 is an evidence
gap, not an invented leak: apply phase 5's essential-evidence rule and name the missing proof.
For class 15, retain `unproven` until a relevant base/head comparison satisfies the declared criterion;
block installation or merge when rule adoption or a required performance budget depends on it.
An optional unsupported claim can instead be removed or bounded; do not invent a measured defect.

The default below does not downgrade phase-2 defects: a demonstrated guard that prevents a promised
user action blocks, even outside these fifteen classes.

Other findings outside these fifteen classes are legitimate but non-blocking by default: report at most three
of them, one line each, labelled non-blocking, or drop them. The cap is what stops the sweep from
turning into a second review that competes with the verdict — rank them by whether they would change
a reviewer's decision and keep the top three. If the verdict runs past about thirty lines, that is
the symptom that preferences have crept into the blocking section.
