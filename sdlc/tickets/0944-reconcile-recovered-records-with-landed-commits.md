---
flow: quickfix
priority: 2
---
# Reconcile recovered records with landed commits

Recovered completion records 0158, 0160, and 0161 name branch-only head commits
instead of the commits through which their work landed on `main`. The known
recorded-to-landed pairs are:

- 0158: `7529a718bd48bcc8ef93f6c13ad90f02c71b85ee` to
  `fb56bd624c0a984ba7c76839048859556e4e5190`;
- 0160: `f6e634c6b480942ba71b81215ebf1843a3d5384f` to
  `f68a2589043cd3b97cf825b60f524548751d21b7`; and
- 0161: `8b6f9304bb461d612cfd2a46b711d6b318ddcf6e` to
  `7bca6b8163716d23b70937f4947c8f5f1e6a2033`.

## Done when

For each record, derive the correct main-reachable base/head range and prove
the ticket-owned patch is equivalent to the recovered branch work after
accounting for intervening commits. Update the record frontmatter and recovery
note to name that landed range and the evidence used. Do not merely substitute
a hash because a subject looks similar, and do not rewrite product history.

Every repaired base/head object exists, base is an ancestor of head, head is an
ancestor of current `main`, every artifact path still exists, and the record
validator passes. If any pair is not patch-equivalent, leave that record
unchanged and file a specific issue rather than claiming provenance that was
not proved.

## Authorized changes

This ticket may change only the three named records and record-validation tests
needed to pin main reachability and patch-equivalence evidence. No product,
ticket, queue, or planning artifact changes belong here.

The src line ceiling may not rise.
