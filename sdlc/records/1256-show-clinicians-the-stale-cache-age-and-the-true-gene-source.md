---
base: a7d503be
head: 5754b910
---

Made the clinician-facing output honest about stale cache serves and
gene provenance, from the 2026-09-27 review file (Ian's order put
this before check hardening).

The stale-cache age now reaches both clinician-facing paths. A
task-local scope holds the notes (recorded where the age header is
read, keeping the log line and stripping the header as before); the
CLI card and the MCP card both append a plain "Cache note:" line —
the MCP arm drives through the same scope helper before shell.rs
redaction, the gap the first cut left — and search JSON carries the
sentence in `_meta.notes`. The wording states the fact ("older than
the provider's freshness window") and never implies revalidation
failure. Writer unit tests cover present, absent, and malformed
headers, drain-once, and no cross-command leak; dispatch tests drive
a real fixture server fresh (no note) and stale (note on both
channels), and one drives the real MCP entry proving the card note
and the JSON sentence both appear.

Genes seeded from MyDisease now carry their real source. The seed
previously destroyed provenance (an empty associations list left the
template defaulting to "Open Targets"); rows now carry "DisGeNET
(via MyDisease.info)" — the data is DisGeNET curation carried by the
MyDisease hit — and the label credits DisGeNET alone for the seed
(an exclusive match keeps the combined string from double-crediting)
while a pure MyDisease source still credits MyDisease.info. Open
Targets stays credited only for lists it produced; a wholesale Open
Targets replacement clears the seed rows. DisGeNET is now a
registered provider for the disease genes section (registry row and
architecture doc), or the provenance guard would reject the honest
label. The heading asserts on the main card, the association table
on the genes card — each where it renders.

The two wording faults: search-all's dropped-filter note now says
plainly which source failed (upstream text moves to a log line), and
a DDInter HTML download reply is a download failure, not an
unreadable bundle. Both pinned by tests; the DDInter test asserts
the read sentinel never leaks into download wording.

Deferral recorded with an issue: get-JSON bodies have no `_meta`
shape for the note (2026-09-27-get-json-bodies-have-no-notes-channel...),
an envelope change that deserves its own schema decision.

Evidence: code review REJECT once — three P0s: the label
double-match, an unearned fixture assertion, and the MCP path
bypassing the scope (the reviewer's trace was right on all three) —
fixed and re-reviewed ACCEPT (the fix commit's five files proven by
git show --stat). Yellow gates then caught what no-cargo review
cannot: a duplicated test tail unbalancing braces, two unclosed
fixture JSON objects, a self-contradictory assertion, heading and
table asserted on the wrong cards, the registry provider gap, the
CLI line cap (the MCP scope extracted into a shared helper), three
size baselines, and one clippy boolean — eight gate cycles to green
at 5754b910: lint, test, spec, stress all OK, zero failed lines.
CHANGELOG carries the user-visible bullets.
