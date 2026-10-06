# 1294 — return whole abstracts and cleaner full text

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: OPEN.
Milestone: 0.9.2

## Build status

- Built on branch `tickets/1294-whole-abstracts`, sha `d00f515c`, 2026-10-05. Two review cycles.
- Code review (cycle 1): REJECT 2026-10-05, two findings: the ticket's named spec page did not exist (P1), and one receipt recorded a non-production request URL while claiming the production shape (P2).
- Code re-review (cycle 2): ACCEPT 2026-10-05, both findings resolved with evidence; the two proof substitutions (PubTator3 export for PMID 30738221 because no PMC record exists; authored fixture for the ragged banner because no recorded capture contains one) ruled acceptable.
- Spec page `spec/entity/article-text-fidelity.md` pins six behaviors from recorded bytes, names the 240-byte snippet cap, and is registered in the routine lane and the registry test. 51+ targeted tests, lint green.

## Outcome

Article JSON returns whole abstracts with no cut-off, and full text reads cleanly: spaced JATS inline text, labeled reference identifiers, kept PMC titles, and no viewer links or lookup URLs.

## Evidence

- Starts from: The owner ran `biomcp get article 30738221 -j` on 0.9.1. `abstract_text` holds 1,534 characters and ends with `(truncated, 2369 chars total)`. Experiment 437 saw `batch article --mode detail` cut the same way. No flag returns the whole abstract. Agents in experiments 421 and 432 read abstracts through this path. Experiment 434 (`~/workspace/experiments/434-meaning-search-in-a-paper/REPORT.md`) fetched eight cancer treatment papers with `get article PMID fulltext`. In JATS papers, tables print as `*[Complex table: N×M]*` followed by `Row N:` dumps rather than Markdown tables, and some words lose their spaces (`JemalA`, `recurrencescore`). In PMC HTML papers, the title is missing (the file starts with `* * *`), and the text carries `[Open in a new tab]` lines, image viewer links and long `scholar_lookup` URLs in every reference.
- Keeps: Section names, the full-text source ladder and `full_text_coverage` stay the same. Compact search rows stay compact, with their 240-byte snippet.
- Changes: Code pointers from the ticket review: `truncate_abstract` caps abstracts at 1,500 bytes at transform time (`src/transform/article/anchors.rs:76-89`, applied in `src/transform/article/federation.rs:66-67, 256-260, 312-317`). JATS inline elements join without whitespace (`src/transform/article/jats.rs:505-541`). PMC HTML keeps the readability body but drops its title and byline (`src/transform/article/html.rs:261-264`) and keeps every link (`html.rs:246-249`). Tables without merged cells already render as Markdown (`src/transform/article/jats/tables.rs:8-44`), so every dumped table in experiment 434 had merged cells. The numbered change follows in Change detail.
- Proof: Recorded fixtures with expected output in a spec page: PMID 30738221's abstract; one JATS paper with inline markup, references and a ragged table; one PMC HTML paper. Each fixture names the defect it pins. Bound, measured 2026-10-05: real abstracts run 1,467 to 5,057 characters, median about 3,550 — whole abstracts roughly triple today's abstract cost, so `get article` and `batch --mode detail` carry whole abstracts, search rows keep their snippet, and `search article --full` keeps a documented per-row abstract cap with a way to ask for the whole one. The proof names that cap.
- Defers: Rendering tables with merged cells as Markdown. JATS author bylines.

## Change detail
## Change detail
## Review amendments (2026-10-05, folded from code review)

- Change 2 wording: element-adjacent splits (adjacent elements whose text touches with alphanumeric characters on both sides, like a surname element joined to a given-names element) gain one space. Defects that live in the source text itself, with no element boundary in the rendition, stay as-is; BioMCP does not rewrite source prose. On the recorded NCBI EFetch rendition of the TAILORx paper, `JemalA` is an element join and gains the space; `recurrencescore` is plain source text and stays.
- Reference-link scope for Change 4: drop scholar_lookup URLs, tileshop viewer links (including linked viewer images), and whole-line open-in-new-tab entries. Keep short canonical links: DOI, PubMed, PMC free article.
- Proof, 30738221 case: PMID 30738221 has no PMC record (esearch db=pmc returns count 0), so no EFetch capture can exist. The abstract's production source is the PubTator3 export; the capture `pubtator/export_30738221.json` records that production request shape. The whole abstract is pinned byte-for-byte against the recorded bytes. The EFetch-shaped whole-abstract pin is delivered on the TAILORx capture instead. The seam test over the synthetic abstract remains as the edge-case floor.
- Ragged-table banner: no recorded capture contains a ragged table (TAILORx and RxPONDER tables are all merged-cell), so the banner is pinned on a fixture-authored JATS document with uneven row widths.


1. JSON output carries the whole abstract. If Markdown output keeps a short form, it says how to get the rest. Search rows keep their 240-byte snippet.
2. JATS conversion inserts whitespace between inline elements where the source has it, fixing `JemalA` and `recurrencescore`. Defects in the source text itself, such as `Score of≤1O`, stay as they are.
3. JATS references print `PMID` and `PMCID` as separate labeled values rather than glued to the citation text.
4. PMC HTML conversion keeps the title and byline. It drops viewer links, `[Open in a new tab]` lines and reference lookup URLs.
5. JATS tables with ragged rows, which fall through both table converters today and print nothing (`tables.rs:24, 32-33, 72`), print in the raw-rows form.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 9962598b (fresh SWE-2 researcher, read-only). The first pass returned FIX; the revision addressed every finding.
