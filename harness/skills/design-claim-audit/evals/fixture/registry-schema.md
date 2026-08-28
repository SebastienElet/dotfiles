# Registry model

`DistributionKey` stores `id`, `tenantId`, `locationNodeId`, `label`, `validFrom`, and `validTo`.
It stores no condominium or co-owners-association reference.

The public contract exposes `createDistributionKey` and `replaceDistributionKeyShares` as separate commands.
Registry receives only the commands submitted by callers; it does not own the caller's required-key inventory.
