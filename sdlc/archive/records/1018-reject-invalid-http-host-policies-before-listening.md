---
base: 903ac20768892b05c50deab27c562fae80655f76
head: 3de63db794a4d943934283c931dcae44752b4252
---

# Reject invalid HTTP host policies before listening

Invalid `--allowed-hosts` entries let the HTTP server bind and report success,
then caused every route to reject requests.

The repair validates, normalizes, and deduplicates explicit policies before
binding. The transport and router share that policy, and tests prove invalid
entries leave the requested port unbound.
