# 1290 build record — variant headline states the current ClinVar classification

Landed 2026-10-04, merge 8b188c995. Built on its lane branch; QA-era
ticket (the QA pass on 1290 is recorded on the ticket). Review
verdicts and the change detail live on the ticket,
sdlc/tickets/1290-make-the-variant-headline-match-current-clinvar.md.

Outcome: `get variant` states the NCBI record-level germline
classification when the clinvar section answers from NCBI; the default
view keeps the MyVariant cached value, names it, and prints the
clinvar command. Gates at landing: branch CI green, yellow gates rc=0.
This record backfilled 2026-10-07 under ticket 2020 (finding: seven
landed tickets had no record).
