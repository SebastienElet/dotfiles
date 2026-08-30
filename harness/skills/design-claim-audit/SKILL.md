---
name: design-claim-audit
description: >
  Audit architectural and domain guarantees. Use when an ADR, design, or domain specification
  asserts authority, completeness, uniqueness, atomicity, lifecycle closure, validation, waiver, or
  legal effect. Make sure to use this skill whenever documentation claims proof or certification.
  Excludes reviews of open pull requests.
metadata:
  category: dev
---

# Design Claim Audit

## Overview

The document-changing task is the author and cannot certify its guarantee. It obtains an isolated
child audit before reading evidence.

## Usage

Apply to material claim changes. Exclude editorial work, code-only work, and reviews of open pull
requests.

## Steps

1. Record the document-returning task as `author_task`; its caller is never the author.
2. Before reading evidence or drafting, spawn `design-claim-auditor` with paths, scope, and repository
   context, but no expected finding. Record its distinct identity as `auditor_task`. If this named
   child is unavailable, stop with the audit absent.
3. Wait for the child ledger. Each changed claim gets one row with `claim`, `authority`, `mechanism`,
   `scope`, `named behavioral oracle`, `status`, and `failure mechanism`.
4. Assign status in order: `held` needs authority, exact mechanism, scope, and a named green oracle;
   `target` needs a cited approved decision; `debt` needs a cited tracker item; everything else is
   `contradicted`. Missing implementation is not a target.
5. Only now read evidence and reconcile the draft. Remove or weaken `contradicted`; keep `target` or
   `debt` within its cited authority.
6. Deliver both task identities, the complete seven-column ledger, then the reconciled document or
   diff. A shortened ledger is absent.

## Quick Reference

| Claim             | Required evidence                        |
| ----------------- | ---------------------------------------- |
| Context authority | Owner and deciding data                  |
| Complete set      | Expected-set owner and omission check    |
| Atomic operation  | Transaction covering every named write   |
| Waived state      | Every independent gate and waiver effect |
| Validated path    | Active path and contradictory-input test |
| Legal effect      | Source in force and matching scope       |

## Gotchas

| Excuse                        | Reality                                      |
| ----------------------------- | -------------------------------------------- |
| Ticket Done or pipeline green | Workflow state is not a behavioral oracle.   |
| Parent is the author          | The task returning the change is the author. |
| Receipt proves completeness   | Received items cannot reveal omissions.      |
| Missing mechanism is a target | Only an approved decision creates a target.  |
| Self-review is equivalent     | Author assumptions survive self-review.      |

## Constraints

- Auditor is the dedicated `design-claim-auditor`; its checked-in configuration requests
  `read-only`, but a live parent sandbox override can supersede that default.
- A context certifies only owned facts and decisions.
- A waiver satisfies only canonically named gates.
- `target` cites its decision; `debt` cites its tracker item.

## Red Flags

- Auditor is absent, not a child, or inherited author context.
- Author writes the ledger or calls its parent the author.
- Ledger omits a claim or required column.
- Workflow status replaces an oracle.
- `target` or `debt` lacks its required citation.

Any red flag blocks the claim.
