# PR repair record template

Fill this compact skeleton in the pull request's language and pass it through a body file. Repeat
the correction line, not the surrounding structure.

## Skeleton

```text
<!-- pr-fix:<pr>:<final-head-sha-12> -->
## Repair record

Review completed on `<final-head-sha>`: <what held under review, with the evidence that established it>.

Corrections pushed:
- `<correction>` — mechanism: <failure sequence and broken invariant>; proof: <failing observation, then passing check and result>.

Not repaired:
- `<finding>` — reason: <why it was deliberately excluded>.
<Write "None" when every finding was repaired.>

Barrier: <tier, commands, and numeric results>. Limits: <what those results do not cover>.
```

## Self-check before publishing

- Marker first; same `<pr>:<sha>` updated, different SHA published once.
- What held comes first; every correction, including a correction to an earlier correction, has a
  mechanism and proof.
- Every deliberate omission has a reason, or the record says `None`.
- Barrier numbers are immediately followed by their limits.
- Body passed through a file; no verdict or merge decision.
