# CSpec fixture lacks a selectable BRAF document

The routine CSpec fixture lists BRAF resource IRIs, but selecting its listed
short version `1.0.0` fails with the generic ClinGen CSpec API error. The
fixture needs a receipt-backed BRAF document or must not advertise a selectable
manifest entry. This is outside ticket 1033's short-version selection change.

2026-08-23: left as a watch item per Ian's triage ruling — file it only if it recurs.

## Decision (ticket 1238, 2026-09-26)

Same ruling and same state as the ERepo cleanup race: dormant watch
item, no recurrence across the 0.9.1 gates, revisit at 1.0.
