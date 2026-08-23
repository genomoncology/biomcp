# ClinGen ERepo fixture cleanup race

A repeated direct ERepo fixture run can pass every assertion yet print a
`FileNotFoundError` while its supervisor removes `server-pid`. The fixture
cleanup ownership race is outside ticket 1041's CAid-to-gene behavior.

2026-08-23: left as a watch item per Ian's triage ruling — file it only if it recurs.
