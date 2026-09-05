# Code Simplify Validation

These are prepared scenarios, not execution results. Run them in fresh disposable contexts when
evaluating this skill; never invoke Claude automatically. Do not build an evaluation runner for
these scenarios.

## Evidence boundaries

- **Format:** `skill-manager doctor`, optional installed `skills-ref`, JSON parsing, index
  regeneration, and deployment checks establish structure and discoverability prerequisites.
- **Activation:** use `evals/trigger-queries.json` in each intended host with the normal skill
  catalog, without forcing the expected selection. Record the host, prompt, actual activation,
  and whether the loaded procedure governed the response. A declared symlink is not activation.
- **Result quality:** run the cases below with the skill available and inspect the actual patch,
  explanation, and relevant checks. Correct activation alone does not establish useful results.

For any execution, keep the prompt, fixture, host/model, observed actions, checks, and deviations
outside the repository unless an evidence document is explicitly approved. Label unavailable and
unexecuted checks separately from failures. The expected outcomes below are review criteria, not
assertions to implement as a mirror test suite.

## 1. Remove a dispensable intermediate abstraction

Fixture: a private `formatInvoiceLabel(invoice)` delegates unchanged to `invoiceLabel(invoice)`.
The complete fixture shows one direct caller, no exports or dynamic registration, identical
arguments and return value, and no logging, validation, transaction, or other boundary role.
An existing invoice output test exercises that caller.

Prompt: `Simplify the invoice label implementation in this diff; keep its behavior.`

Expected: inspect the caller and role, replace the intermediate call with `invoiceLabel`, remove
the dispensable wrapper, and run the existing output test. Explain the indirection removed without
generalizing the surrounding invoice code. If additional contract evidence contradicts the
fixture assumptions, retain the boundary and report that evidence.

## 2. Retain a useful single-use name

Fixture: private pure `isEligibleForRenewal(subscription, today)` is called once and names a
three-condition business decision inside a renewal flow. Its name separates eligibility from
payment and notification steps; existing tests cover the decision's outcomes.

Prompt: `Reduce abstractions in this renewal diff; this helper is only called once.`

Expected: retain the helper if inlining only erases the meaningful name; do not equate call count
with redundancy or combine eligibility, payment, and notification. Explain why no deletion is
useful here; no patch is acceptable.

## 3. Preserve validation at a trust boundary

Fixture: a browser validates an invoice's tenant identifier before calling a server endpoint.
The endpoint validates and authorizes it independently; callers can invoke the endpoint directly.
An existing integration test checks rejection of a direct cross-tenant request.

Prompt: `Simplify the duplicated tenant validation between the browser and invoice endpoint.`

Expected: identify distinct trust boundaries, load `enforcement-code` before any control change,
and retain server rejection. Upstream validation is insufficient deletion evidence. Exercise
the existing direct-request oracle if changing the relevant path; do not invent a gate that
merely checks whether validation text remains in the source.

## 4. Escalate a requirement change

Fixture: a report exporter offers CSV and JSON through a documented public format parameter.
Repository callers and recent usage data show only JSON, but independently deployed clients can
still request CSV. Existing public-entry tests cover both formats.

Prompt: `Simplify this exporter; CSV looks superfluous now.`

Expected: distinguish implementation simplification from removing CSV support, explain the
compatibility impact, and request an explicit decision before deletion. Search absence and usage
silence do not establish non-use. Continue independent simplifications if any are justified.

## Next real diff observation

Observe whether the skill identifies a removable concept with consumer evidence, retains useful
names and boundaries, and stops without speculative cleanup. Record actual behavior and oracle
limits; do not infer benefit from a smaller line count.
