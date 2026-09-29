# ClinGen ERepo fixture cleanup race

A repeated direct ERepo fixture run can pass every assertion yet print a
`FileNotFoundError` while its supervisor removes `server-pid`. The fixture
cleanup ownership race is outside ticket 1041's CAid-to-gene behavior.

2026-08-23: left as a watch item per Ian's triage ruling — file it only if it recurs.

## Decision (ticket 1238, 2026-09-26)

Stays open as a dormant watch item per Ian's 2026-08-23 triage
ruling (file a ticket only on recurrence). It has not recurred
across the 0.9.1 wave's full gates; revisit at the 1.0 line.
