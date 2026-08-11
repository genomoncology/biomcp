---
base: 0a054c15
head: 0327f22a
---

Drug regulatory behavior now runs in the routine offline specification lane.
A supervised loopback fixture serves dated, receipted MyChem and openFDA
responses plus isolated EMA and WHO datasets, records production requests, and
returns 404 for unknown routes. ChEMBL target and DDInter assertions moved
temporarily to `drug-live.md` for ticket 0905.

The routine page proves multi-region search, Keytruda canonicalization,
structured indication empties, WHO detail, U.S. Drugs@FDA decoding, Markdown
rendering, and observed MyChem/openFDA request parameters. Existing source and
entity tests retain request-plan, decoder, failure, and not-configured coverage.
The new fixture lifecycle test proves receipted bytes, local readiness,
environment isolation, request logging, fail-closed routing, and cleanup.

Verification passed: 17 capture/fixture tests, 78 runner/lifecycle tests, all
seven drug specification blocks, and the complete routine `make spec` gate.
No source lines were added against the 150-line ceiling.
