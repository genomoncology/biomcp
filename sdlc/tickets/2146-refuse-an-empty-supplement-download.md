---
flow: build
priority: 2
deps: []
---

# 2146: Refuse an empty supplement download

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision bb6b990, accept

## Outcome

`get article ... asset KEY --output FILE` exits nonzero and names the empty response when a source returns no bytes, and the assets list marks or drops an alternative known to return nothing, so a caller never takes an empty copy for the supplement.

## Evidence

Reported 2026-10-16 by the research team on the released 0.9.2.

- PMID 30209399 lists `NIHMS1501643-supplement-2.xlsx` under a Europe PMC key and a PMC OA key.
- `biomcp --no-cache get article 30209399 asset europe-pmc-a88699323445e1fab52547c22f2b5903b5d7429286dd006f08e8882cb458e6f9--NIHMS1501643-supplement-2.xlsx --output empty.xlsx` exits 0 and writes a zero-byte file.
- The same command with `pmc-oa-b1b0e6561493678f5003f1338cfb33b170a777cf6a55198c32cf0fe2ee00d1d9--NIHMS1501643-supplement-2.xlsx` writes 2,306,341 bytes, MD5 `c17f1a475560db5b98d83f9e14e8465f`.

## Work

- Reproduce on main first, and find why the Europe PMC route returns no bytes: a wrong URL, a redirect not followed, or a real empty reply.
- Fix the Europe PMC route if the file exists there. Either way, treat a zero-byte body as a failed download with its own reason, and write no output file.
- When the same filename has several routes, prefer the one that has returned bytes, and mark an alternative that returned nothing.

## Acceptance

1. A recorded-reply test of an empty asset reply exits nonzero with the empty-response reason and leaves no file; it fails on current main.
2. A recorded-reply test of the two-route supplement downloads the full copy.
3. The changelog carries a bullet for this ticket.
