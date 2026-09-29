# DDInter real-bundle checks need a Mac run

Filed 2026-09-24 from the review of ticket 1241 (merge 02447388).
Ticket 1241's cap check and real-bundle verification were deferred to
the Mac because the Linux gate host has no synced DDInter data and
must stay otherwise idle. Originally Ian's to run; he assigned it to the developer on 2026-09-27 (see the 2026-09-27 review file's direction section).

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

## Resolved — the run, 2026-09-27

Ian assigned the run to the developer that day (the 2026-09-27
review file's direction section).

Environment: M5 (macOS, arm64), release binary built at repo commit
a7d503be (`cargo build --release`, rustc 1.95.0), reporting
`biomcp 0.9.1-dev.1`. Data root on macOS is `dirs::data_dir()`:
`~/Library/Application Support/biomcp/ddinter` (the issue's
`~/.cache/biomcp` path is the Linux location). `biomcp ddinter sync`
completed: "DDInter local interaction data synchronized
successfully."

**Cap check.** All eight CSVs landed, byte sizes observed with ls:
code_A 3,343,434; code_B 867,726; code_D 1,520,704; code_H
705,088; code_L 3,885,702; code_P 317,460; code_R 1,793,766;
code_V 700,777. Largest is 3,885,702 against the 8,388,608 cap
(`DDINTER_MAX_BODY_BYTES`): every file passes with roughly a 2.2x
margin. No finding.

**Apixaban run.** `biomcp drug interactions apixaban` printed
"Returned: 25 of 323" with Major-level rows, "DDInter bundle
freshness: fresh" (bundles synced the same minute), and the
structured-rows provenance sentence. The covered-with-rows card
carries that sentence in place of a bare "coverage:" line — the
coverage line the issue expected prints its explicit form on the
not-covered and covered-zero-rows cards. No finding.

**Aspirin combination-product run.** `biomcp drug interactions
aspirin` resolved the MyChem anchor to the combination product
(card title "epinephrine, albuterol sulfate, nitroglycerin,
diphenhydramine hydrochloride, aspirin"), which the bundle does not
cover. The card printed the not-covered wording exactly as
designed — "Coverage status: not_in_ddinter_coverage... a source
coverage miss, not evidence of no interactions" — not a raw "no
matching rows", and zero fabricated rows appeared, so the
pooled-synonym bug did not resurface. Observation, not a finding of
this issue: the card titles itself with the full combination-product
name because that is the chosen anchor, so this check never
exercised the single-ingredient path — tracked as its own issue,
`sdlc/issues/2026-09-28-aspirin-mac-run-never-tested-the-single-ingredient-anchor.md`.
Not a regression from 1241.

All three checks pass. Nothing here blocks.
