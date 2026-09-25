# DDInter real-bundle checks need a Mac run

Filed 2026-09-24 from the review of ticket 1241 (merge 02447388).
Ticket 1241's cap check and real-bundle verification were deferred to
the Mac because the Linux gate host has no synced DDInter data and
must stay otherwise idle. Ian runs these; nothing here blocks coding.

## The 8 MB cap check against real sizes

`DDINTER_MAX_BODY_BYTES` is 8 * 1024 * 1024 (`src/sources/ddinter.rs:67`).
On the Mac, with DDInter already synced:

    find ~/.cache/biomcp/ddinter -name '*.csv' -exec ls -l {} \;

(adjust the root if `BIOMCP_CACHE_DIR` is set). Record each file
size. If any bundle file exceeds the cap, the sync path rejects it on
principle — that is a finding, not a pass; file it and we will raise
or split the cap deliberately.

## The real-bundle interaction run

    biomcp drug interactions apixaban

With the bundles synced, the card must show interaction rows, the
`DDInter coverage:` line naming covered, and a freshness label
matching the bundle mtimes (Fresh under 72 h). A covered drug with
zero rows must print the covered-zero-rows wording, not "no matching
rows".

## The aspirin combination-product check

    biomcp drug interactions aspirin

Aspirin is the known combination-product case. After ticket 1241,
synonyms come from the chosen MyChem anchor hit only (not pooled), so
if a combination product's synonym list ever names a real interaction
partner, the row must still appear; if it disappears, that is the
pooled-synonym bug resurfacing and needs a new ticket with the exact
anchor hit JSON.

Record the outputs of all three in a comment on this file, then close
it.
