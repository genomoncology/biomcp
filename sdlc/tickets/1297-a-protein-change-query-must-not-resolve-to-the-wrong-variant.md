# 1297 — a protein-change query must not resolve to the wrong variant

Proposed 2026-10-04 by the BioMCP 0.9 lead, from KB QA ticket 0002.

Status: OPEN.

## Folded scope from 1292's review (2026-10-06)

- Add the missing unit test for the article-search bare-VariationID refusal (src/entities/article/variant_search.rs guard): assert the InvalidArgument message for a ClinvarVariationId input.
- Decide the EGFR protein-range resolution direction: 1292 parses `EGFR E746_A750del` and refuses with candidates (dbNSFP holds no protein-range alias); this ticket owns whether refusal-with-candidates stays or alias expansion resolves it, under its no-wrong-answers rule.

## Outcome

## Outcome

A protein-change query such as `DICER1 p.Met1483Ile` resolves to the variant ClinVar means, or says it cannot resolve and lists its candidates. It never silently returns a different transcript's variant.

## Evidence

- Starts from: KB QA ticket 0002 (landed at e68111b in the KB repository) found that on both 0.9.1 and the 1290 branch, `biomcp get variant 'DICER1 p.Met1483Ile' -j` resolves to chr14:g.95562808C>A, a variant on a different transcript with no ClinVar record, while the question's ClinVar variant is NM_177438.3(DICER1) p.Met1483Ile. A protein-change query can land on the wrong variant silently. Source: KB QA message of 2026-10-04, ask 2.
- Keeps: Every query that resolves unambiguously today keeps its answer.
- Changes: Reproduce the DICER1 case on main. Then either prefer the transcript ClinVar itself names for a protein-change query, or refuse an ambiguous protein-change query with a message that lists the candidate variants and a working input form (VariationID, rsID or transcript-qualified HGVS).
- Proof: A spec table of protein-change queries, ambiguous and unambiguous, with the expected resolution or refusal, from recorded responses. The DICER1 case is one row. Validated 2026-10-05 on main: `get variant 'DICER1 p.Met1483Ile' -j` resolves to chr14:g.95562808C>T (wrong transcript), and the correct colon form `NM_177438.3:c.4449G>A` returns `not_found` — so the acceptance rows must include VariationID 577152 resolving correctly, and the protein form either resolving to 577152's variant or refusing with candidates that include it.
- Defers: Full HGVS protein notation, which moves to the shared parser in 1.0.

## Review

- Design review: pending. A fresh read-only reviewer reads this ticket before any code is written.

## Note

Numbered by hand as 1297. pm's reservation counter reads every remote branch number, and this remote carries the sibling line's branches (numbered 2003 and 2007 to 2009), so `pm ticket new` would reserve 2010. The cause and a request to scope the counter to `tickets/*` went to the pm team on 2026-10-04.
