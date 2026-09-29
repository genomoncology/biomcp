# Typed CSpec schema leaves mode constraints to runtime

`gene_cspec` advertises unbounded `gene`, and its `version_iri`, `capture_id`,
and `files` mode constraints are not represented in its typed MCP schema.
This was found while surveying typed tools for ticket 1031 and is outside that
ERepo schema change.

2026-08-23: left as a watch item per Ian's triage ruling — file it only if it recurs.

## Decision (ticket 1238, 2026-09-26)

Dormant watch item; the typed-schema work that 1240/1251 landed
kept the cspec constraints in-body by design, and tightening the
typed schema would revisit that decision — a 1.0 conversation, not a
0.9.1 one.
