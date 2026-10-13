# 2067 — Keep variant follow-ups on the headline build

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every follow-up command a variant card prints uses the same genome build and coordinate as its headline, and following it returns the evidence the card counted. ClinVar rows show their submitter count.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel.

- `biomcp get variant "BRAF V600E" all` headlines "Genomic coordinate (GRCh37, provider default): chr7:g.140453136A>T" but prints "Therapeutic evidence: 13 CIViC predictive item(s) — see get variant \"chr7:g.140753336A>T\" civic", and every other see-also command uses the GRCh38 coordinate. Following it returns "Evidence Items: 0" and a card without its consequence and COSMIC lines. Any request that includes `population` does this. 0.9.1 did the same, but its `all` failed outright on the retired GWAS endpoint, so 0.9.2 shows this to far more users.
- `get variant "BRAF V600E" clinvar` rows read "evaluated 2014-09-04;  submitter(s)." with no number, while the JSON carries `submission_count: 1`. 0.9.1 does the same.

- Starts from: the variant card's see-also builder and the ClinVar section renderer.
- Keeps: the headline's build label.
- Changes: build follow-ups from the headline's coordinate; print the submitter count.
- Proof: a recorded-reply test that follows the printed civic command and gets the counted items; a renderer test for the count; both fail on `4049666f5`.
- Defers: nothing.
