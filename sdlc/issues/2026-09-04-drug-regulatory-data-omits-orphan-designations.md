# Drug regulatory data omits FDA orphan designations

BioMCP's `get drug eflornithine regulatory` section returned Drugs@FDA approval history, EMA medicine records, and WHO prequalification data on `biomcp 0.9.0-dev.6` on 2026-09-04. It did not return FDA orphan designations.

The FDA Orphan Drug Product database contains an eflornithine hydrochloride designation for treatment of Bachmann-Bupp syndrome. FDA designated it on 2024-03-11. The record distinguishes “designated” from “approved,” and it states that the orphan indication has not received FDA approval: <https://www.accessdata.fda.gov/scripts/opdlisting/oopd/detailedIndex.cfm?cfgridkey=992323>.

The FDA search interface supports product, designation, date, approval-state filters, detailed results, and Excel output: <https://www.accessdata.fda.gov/scripts/opdlisting/oopd/index.cfm/>. OpenFDA does not list an orphan-designation endpoint. The underlying public service supports ingestion, but it requires a maintained download or legacy search adapter rather than reuse of BioMCP's current OpenFDA client.

## Recommended design

Add FDA orphan designations to the existing drug `regulatory` section. Match on normalized ingredient and product aliases. Return the designation text, designation date, sponsor, status, approval date when present, exclusivity facts when present, and the FDA record link. Keep orphan designation and marketing approval as separate fields and labels.

This adds one legacy FDA source with a less convenient acquisition path. The distinction has high value because agents otherwise confuse designation with approval during rare-disease research.

## Done, observably

- `get drug eflornithine regulatory` includes the Bachmann-Bupp orphan designation.
- The row says that the orphan indication is designated and not approved.
- A product with an approved orphan indication shows both dates without collapsing them.
- Source health and section outcomes distinguish unavailable acquisition from no matching designation.
- Human-readable and JSON outputs link to the exact FDA record.
