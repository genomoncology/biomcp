---
flow: build
priority: 10
---
# Canceled without implementation: do not add domain defaults

This ticket was archived and must not be rerun. It proposed deriving or
implementing `Default` on coordinate-carrying biomedical domain structures so
tests could use struct-update syntax. That is the wrong product design: an
unknown assembly, coordinate, gene, or variant fact is not a meaningful
default, and manufacturing one would let tests and runtime code create
misleading biomedical values.

No `Default` conversion from this ticket was approved or implemented. Queue
reconciliation may retain this file as a done archive tombstone for dependency
history; that status means retired, not successfully landed behavior.

Ticket 0950 is the replacement. It authorizes the design commit to update the
specific exhaustive test constructors required by the new typed build field,
while production constructors remain explicit and no domain `Default` is
introduced. Tickets 0899 and 0900 consume that construction-only approach.

Do not restore this request, do not infer its old checklist as current intent,
and do not rerun 0898.
