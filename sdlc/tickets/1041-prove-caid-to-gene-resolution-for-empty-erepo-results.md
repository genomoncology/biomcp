---
flow: build
priority: 14
---
# Prove CAid-to-gene resolution for an empty ERepo result

When `variant erepo` is given a CAid and the ClinGen Evidence
Repository returns nothing, there is no gene in the answer to count
assertions against. Ticket 1032 wants to report the gene's assertion
count in that case, and its design review refused twice for the same
missing piece: nothing says how the gene is obtained when the empty
response carries none.

This ticket answers that one question, so 1032 can be about the
message rather than about the lookup.

The behavior to settle and prove: given a CAid whose ERepo result is
empty, where does the gene come from, and what happens when it cannot
be had. The cases that must be answered rather than assumed — the
mapping exists; the mapping is missing; the source that provides it
is unavailable or slow. An answer that only works when everything
succeeds is not an answer, because the empty case is already the
common one.

Done, observably: an empty CAid-only ERepo response resolves to a
gene when a gene can be resolved, and says so plainly when it cannot,
without changing what a successful empty result reports today. The
existing empty message stays exactly as it is when no gene is
available, so this ticket adds a capability and removes no behavior.

Filed 2026-08-21 from the drafted successor in ticket 1032's
design-review output, which refused for want of this proof.
