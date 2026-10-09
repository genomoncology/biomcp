# 2031 build record — drug name lookup never swaps the drug

Landed 2026-10-09, merge 50679cc02. Review verdicts live on
sdlc/tickets/2031-drug-name-lookup-never-swaps-the-drug.md.

Outcome: a drug lookup returns that drug's card or an honest refusal
naming what matched. Exact names win, salt forms follow, brands select
through the record's own brand fields (MyChem's drugcentral synonyms
and NDC proprietary names; TAGRISSO resolves to osimertinib on live
data), a miss names up to three drugs the text search touched, and the
discover rescue never adopts a candidate whose own names do not match.
Terfenadine, mannitol, edetate disodium, and ferric oxide return their
own cards; the keep-list resolves live. Gates: branch CI green at
a414da78; the head review ACCEPT covering the final tip; merged-tree
CI green at 50679cc02 with the measured 1_407 count.
