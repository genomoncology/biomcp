---
flow: build
priority: 10
---
# Superseded: label the genome build on variant detail JSON

This ticket was archived without implementation on 2026-08-10. Its legacy
queue row had seven attempts and no content identity, so revising it could not
reliably create a fresh claim. Runnable ticket 0950 contains the same corrected
scope, and every active dependency was rewired to 0950. Do not rerun 0689.

The original intent was to label every genomic coordinate emitted by
`get variant` with the actual answering assembly while preserving every
coordinate and biomedical value. Ticket 0950 is authoritative.
