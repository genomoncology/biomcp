# 2066 — Finish the 0.9.2 fixes the install test found incomplete

Status: OPEN.

Milestone: 0.9.3

## Outcome

Four 0.9.2 changelog promises hold on live data: `get disease MM` leads with multiple myeloma, article-entities disease links never open a benign-tumour card for a cancer heading, Europe PMC pages neither overlap nor skip rows, and a name several drug products share refuses by naming those products.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel.

- `biomcp get disease MM` lists one candidate, "Miyoshi muscular dystrophy (MONDO:0009685)", and names multiple myeloma only on a later "Clinical reading" line. `search trial -c MM --source nci` prints the same refusal, telling the user to retry `biomcp get disease` rather than how to fix the trial search.
- `biomcp article entities 22663011` prints `biomcp get disease MONDO:0002531` for the row "skin neoplasms" (MeSH D012878), and that card is "skin benign neoplasm". 2047 fixed the same shape for Lung Cancers only.
- `--json search article pembrolizumab --source europepmc --limit 5` and the same with `--offset 5` both return PMID 42805115 fifth, and PMID 42786043, fifth in a single `--limit 10` search, appears on neither page. Page 2 matches 0.9.1's.
- `get drug "Pain Relief"` refuses, but exits as not found and lists "Pain (HP:0012531)" and "Pain Relief (Symptom, NCIT:C112216)". The changelog promises it names what matched among drugs.

- Starts from: 2032 and 2040 (ambiguous abbreviations), 2047 (disease links), 1298 (Europe PMC paging), 2031 and 2039 (shared drug names).
- Keeps: every recorded refusal and resolution those tickets pinned.
- Changes: put the oncology reading first among the candidates; make a trial-search refusal suggest a trial-search fix; apply the crosswalk rule to every MeSH cancer heading, not just one; page Europe PMC so offset windows tile the single-search order; make the shared-name refusal list drug products.
- Proof: recorded-reply tests for each of the four that fail on `4049666f5`, plus a recorded live check of each command.
- Defers: nothing.
