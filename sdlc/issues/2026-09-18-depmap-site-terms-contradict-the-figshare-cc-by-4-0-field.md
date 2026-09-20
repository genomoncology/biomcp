# DepMap's own terms contradict the Figshare CC BY 4.0 field, and they restrict rehosting

Observed 2026-09-18 while starting ticket 1206. The ticket records DepMap's licence as CC BY 4.0. That string comes from the `license` field on the Figshare mirror article, not from DepMap. DepMap's own terms say something materially different and more restrictive, and they were revised on 2026-04-02, after the ticket's source survey.

The live terms are readable without the portal bot check at `https://depmap.org/portal/terms_text`, which answered HTTP 200 with `application/json` on 2026-09-18. `https://depmap.org/portal/terms/` renders the same text behind a Cloudflare Turnstile page. The relevant sentences, verbatim:

> The data made available on this website were generated for research purposes and are not intended for clinical or commercial uses, including direct sale, incorporation into a product, or the use of the data to train, develop, or enhance machine learning or AI models other than for internal research use (each a "Commercial Use"). ... Commercial Use of the Data is not permitted under these terms and may require a separate license agreement from Broad or its contributors.

> Any rehosting of the Data on Your website requires that You both adhere to, and require any third party users to adhere to, these terms and conditions. You agree to repost these terms and conditions in full if you elect to rehost the Data.

The page header reads `Last Revised: April 2nd, 2026`. The 2018 revision, still visible in archived captures, carried neither the Commercial Use definition nor the Continuity of Terms of Use clause.

Meanwhile `GET https://api.figshare.com/v2/articles/27993248` returns `"license": {"value": 1, "name": "CC BY 4.0", "url": "https://creativecommons.org/licenses/by/4.0/"}` for "DepMap 24Q4 Public" in group 36075. CC BY 4.0 permits commercial use and imposes no downstream terms beyond attribution. The two statements cannot both describe the same rights.

Two consequences for BioMCP.

Recording "CC BY 4.0" in `docs/reference/source-licensing.md` and `docs/reference/sources.json`, and printing it in every DepMap output as ticket 1206 designs, would tell users they may use DepMap data commercially and may incorporate it into a product. DepMap's own page says they may not. This is the same error class as the wrong HPA licence found in ticket 1213 and the paper-licence-for-database-licence error found in ticket 1205, one layer further out: a mirror's metadata field read as the source's terms.

Committing recorded fixture bytes cut from `Model.csv` and `CRISPRGeneEffect.csv` into this repository is rehosting the Data. The Continuity clause then asks BioMCP to repost DepMap's terms in full and to bind every third-party user of the repository to them. BioMCP is public and MIT licensed, and MIT grants exactly the commercial rights DepMap withholds. Ticket 1206 needs recorded fixtures for all fifteen of its acceptance items, so this is not a corner of the ticket.

Ian ruled on 2026-09-16 that DepMap terms do not block the work because BioMCP is open-source and non-commercial. That ruling answers the Commercial Use clause for BioMCP's own use. It does not reach the Continuity clause, which governs what the repository may carry and what it obliges its users to accept, and it was made against the Figshare CC BY 4.0 reading rather than the April 2026 site terms.

A reading that would resolve it: the Broad's own DepMap group deposited article 27993248 under CC BY 4.0 on 2024-12-10, a CC grant is irrevocable, and `depmap sync` fetches from Figshare rather than from depmap.org, so the site terms arguably never attach to those bytes. That reading may well be right. It is a legal judgement about which of two contradictory statements by the same publisher governs, it is not settled by anything either page says, and it decides whether this repository may carry DepMap bytes at all. It belongs to Ian, not to an implementing agent.

Separately, the mirror is now further behind than the ticket records. The DepMap community forum lists releases 25Q2 (2025-06-05), 25Q3 (2025-09-30), and 26Q1 (2026-04-01). Figshare group 36075 still holds only 24Q4, 24Q2, and 23Q4, confirmed by article search on 2026-09-18. The newest DepMap release is 26Q1; the newest release BioMCP could install is three releases and sixteen months older than that, and twenty-one months old in absolute terms.

Ticket 1206 is parked on both points.

## Decision 2026-09-19

Ian deferred ticket 1206. The contradiction is unresolved and no BioMCP code
reads DepMap. No DepMap bytes were recorded into this repository, and no licence
row was added to `docs/reference/source-licensing.md` or
`docs/reference/sources.json`, so nothing here asserts terms the publisher did
not publish.

Reopening needs two things: a ruling on whether the Figshare deposit's CC BY 4.0
grant governs the mirrored 24Q4 bytes despite the site terms, and a reworded
attribution line for acceptance item 11. The mirror's twenty-one-month staleness
is the weaker of the two reasons but the easier one to watch: if Figshare group
36075 gains 25Q2 or later, the data-age objection goes away on its own.
