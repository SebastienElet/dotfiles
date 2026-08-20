# Failure classes

Ten questions to put to the diff in phase 3. Each is a question, not a checklist item to tick: the
answer is a sentence about _this_ diff. Record one of three outcomes per class.

- **not applicable** — the diff does not touch that concern; say why in one clause.
- **holds because `<evidence>`** — name the constraint, the transaction, the schema, the test.
- **broken by `<mechanism>`** — an ordered sequence of steps that ends with a violated invariant.

Only the third outcome can become a blocking finding, and only when the mechanism is written out.
"This looks racy" is not a mechanism; "request A snapshots at T1, request B writes at T2, A commits
at T3 and B's row is absent from the successor" is.

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

## Reporting

Classes 1, 2, 3, 4, 5, 7 and 9 name mechanisms that lose data, corrupt state or cross a security
boundary: when broken, they block. Classes 6, 8 and 10 are usually reservations — promote class 6
when a concrete consumer's retry path depends on the changed code, class 8 when the degraded value
reaches a person or a legal act, and class 10 when the claim is legal, evidentiary or contractual.

Findings outside these ten classes are legitimate but non-blocking by default: report at most three
of them, one line each, labelled non-blocking, or drop them. The cap is what stops the sweep from
turning into a second review that competes with the verdict — rank them by whether they would change
a reviewer's decision and keep the top three. If the verdict runs past about thirty lines, that is
the symptom that preferences have crept into the blocking section.
