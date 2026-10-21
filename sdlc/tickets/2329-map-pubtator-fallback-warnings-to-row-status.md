# 2329 — Tie PubTator3 fallback warnings to row status

Status: OPEN.

Milestone: 0.9.5

## Outcome

When variant article search falls back because PubTator3 visible metadata is unavailable, each affected result row carries an in-band status naming what is missing, and the stderr warning names the same class instead of "unavailable". A reader of the JSON or Markdown alone can tell which rows are incomplete and why.

- Proof: an outside-in replay test with PubTator3 metadata failing for some rows shows a matching in-band status on exactly those rows, a warning that names the class, and the overall incomplete flag set; a test with no failure shows no warning and no row status.

## Evidence

- On 2026-10-21, the research team's reviewed recheck of 0.9.4 (commit cdb9f4a1b) found that variant article searches still emit PubTator3 visible-metadata fallback warnings with the class "unavailable". The overall result says incomplete or truncated, but nobody has shown that every failing row maps to an in-band status. Research filed it as a concern under R3 (2269), not as a verified defect.
- 2269 fixed the original glioblastoma search, which research could not reproduce, and section quality.
- Start by reproducing and confirming the concern. If every failing row already carries a status, close this ticket with that evidence.
