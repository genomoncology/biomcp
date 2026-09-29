# CHANGELOG Unreleased misses user-visible changes

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Blocks 0.9.1.

## Symptom

`CHANGELOG.md` Unreleased has the 1219 container entry and the 1202 base cell-line entry. It lacks:

- the wheel stack overflow fix (1225, GitHub #282)
- `BIOMCP_CA_BUNDLE` and the `SSL_CERT_FILE` fallback (1221, GitHub #250)
- `get cell-line <acc> drug_response` and `get drug <name> cell_lines` from PharmacoDB (bccd2871)
- `gene cell-lines <symbol> --group` from HPA (6d8fd435)
- `get cell-line <acc> chembl` (fd6a100c)
- the MCP `search` and `get` tool schemas declaring a top-level `"type":"object"` (1223, e559cae2)

## Fix

Add the entries. Consider a release check that fails when Unreleased names no ticket merged since the previous tag.

## Resolved

Ticket 1233, branch `tickets/1233-release-readiness`. The Unreleased section
names every user-visible change since 0.9.0, and
`scripts/check-changelog-coverage.py` in the `version-check` job fails a
release when a ticket merged since the previous release is missing from it.
See the record in `sdlc/records/`.
