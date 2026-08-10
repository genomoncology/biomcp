---
flow: build
priority: 6
---
# Return a typed error for a non-retrievable article asset

Ticket 0679 already detects the NCBI proof-of-work interstitial, refuses its
bytes, and records the coverage outcome. Do not reimplement that work.

## Lookup contract

The asset command prefers the opaque asset_key returned by the manifest.
For backward compatibility it may also resolve an exact displayed filename.

- A key or filename matching a coverage record marked non-retrievable returns
  article_asset_not_retrievable.
- An unknown key/name returns article_asset_not_found.
- The two outcomes are never collapsed.

## Done when

For PMID 30311380's spreadsheet coverage record, JSON and human errors are
nonzero and include:

- stable code article_asset_not_retrievable;
- reason ncbi_interstitial;
- asset key when one exists and the exact filename;
- the safe PMC article page a browser can open;
- no interstitial bytes and no working-looking download handle.

The resolver retains the typed non-retrievable record from manifest discovery
through direct lookup. It does not guess from filename extension at the final
error boundary.

## Proof required

Local tests cover opaque key, filename fallback, non-retrievable record,
unknown record, a valid retrievable asset, JSON envelope, human error, and
exit code. No routine test calls NCBI.

## Authorized test changes

Design commits may restate article asset CLI resolver tests, 0679 coverage
fixtures, typed error contracts, and schemas/examples that currently expect a
generic failure. Interstitial detection assertions stay intact.

The src line ceiling may rise by at most 100 lines.
