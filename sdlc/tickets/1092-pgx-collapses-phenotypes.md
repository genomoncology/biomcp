---
flow: build
priority: 7
---

# `get pgx` collapses multi-gene CPIC phenotypes onto one label and hides two Level A drugs

`biomcp get pgx TPMT recommendations` renders contradictory dosing advice against a single phenotype label, and it omits two of the three drugs its own interactions table flags as CPIC Level A. Both faults live in `src/entities/pgx.rs::map_recommendations`.

Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01.

## Evidence: contradictory advice under one phenotype

```
$ biomcp get pgx TPMT recommendations
# TPMT - recommendations
## Recommendations (CPIC)

| Drug | Phenotype | Activity Score | Recommendation | Classification |
|---|---|---|---|---|
| azathioprine | Normal Metabolizer | - | Initiate therapy with standard starting dose (e.g., 2 mg/kg/day for autoimmune diseases). ... | Strong |
| azathioprine | Normal Metabolizer | - | Initiate therapy with reduced starting doses (30-80% of standard starting dose) ... | Strong |
| azathioprine | Normal Metabolizer | - | Initiate therapy with reduced starting doses (30-80% of standard starting dose) ... | Strong |
| azathioprine | Intermediate Metabolizer | - | Initiate therapy with reduced starting doses (30-80% of standard starting dose) ... | Strong |
| azathioprine | Possible Intermediate Metabolizer | - | Initiate therapy with reduced starting doses (30-80% of standard starting dose) ... | Strong |
| azathioprine | Indeterminate | - | Consider alternative nonthiopurine immunosuppressant therapy. | Strong |
| azathioprine | Possible Intermediate Metabolizer | - | Consider alternative nonthiopurine immunosuppressant therapy. | Strong |
| azathioprine | Intermediate Metabolizer | - | Consider alternative nonthiopurine immunosuppressant therapy. | Strong |
| azathioprine | Normal Metabolizer | - | Consider alternative nonthiopurine immunosuppressant therapy. | Strong |
| azathioprine | Poor Metabolizer | - | Consider alternative nonthiopurine immunosuppressant therapy. | Strong |
```

Four rows share the key `azathioprine | Normal Metabolizer`. One of them says to consider an alternative nonthiopurine agent and another says to start a standard dose. At `--limit 50` the section renders 30 rows on six distinct drug and phenotype keys.

CPIC returns a distinct genotype for each of those rows. Confirmed directly against the provider:

```
$ curl -s 'https://api.cpicpgx.org/v1/recommendation_view?lookupkey->>TPMT=not.is.null&drugname=eq.azathioprine&select=recommendationid,phenotypes,drugrecommendation&limit=100'
8480053 {'TPMT': 'Normal Metabolizer', 'NUDT15': 'Normal Metabolizer'} | Initiate therapy with standard starting dose ...
8480054 {'TPMT': 'Normal Metabolizer', 'NUDT15': 'Intermediate Metabolizer'} | Initiate therapy with reduced starting doses ...
8480055 {'TPMT': 'Normal Metabolizer', 'NUDT15': 'Possible Intermediate Metabolizer'} | Initiate therapy with reduced starting doses ...
8480061 {'TPMT': 'Normal Metabolizer', 'NUDT15': 'Poor Metabolizer'} | Consider alternative nonthiopurine immunosuppressant th...
8480071 {'TPMT': 'Normal Metabolizer', 'NUDT15': 'Indeterminate'} | Based on TPMT, initiate therapy with standard starting ...
8480083 {'TPMT': 'Normal Metabolizer', 'NUDT15': 'No Result'} | Based on TPMT, initiate therapy with standard starting ...
```

Row 8480061 is the row that reaches the terminal as `azathioprine | Normal Metabolizer | Consider alternative nonthiopurine immunosuppressant therapy`. Its actual genotype is TPMT Normal Metabolizer with NUDT15 Poor Metabolizer.

## Evidence: two Level A drugs are absent

```
$ biomcp get pgx TPMT interactions
| Drug | Gene | CPIC Level | PGx Testing |
|---|---|---|---|
| azathioprine | TPMT | A | Testing Recommended |
| mercaptopurine | TPMT | A | Testing Recommended |
| thioguanine | TPMT | A | Testing Recommended |
```

```
$ biomcp get pgx TPMT recommendations --limit 50 | grep -o '^| [a-z]*' | sort | uniq -c
     30 | azathioprine

$ biomcp get pgx TPMT recommendations --limit 50 --offset 30 | grep -o '^| [a-z]*' | sort | uniq -c
      5 | azathioprine
     25 | mercaptopurine
```

`--limit 50` returns 30 rows. Thioguanine appears on no page a caller would reach without knowing to keep offsetting. CPIC holds 35 rows for each of the three drugs:

```
$ curl -s 'https://api.cpicpgx.org/v1/recommendation_view?lookupkey->>TPMT=not.is.null&select=drugname&limit=200&order=drugname.asc,recommendationid.asc'
total rows: 105
Counter({'azathioprine': 35, 'mercaptopurine': 35, 'thioguanine': 35})
```

Mercaptopurine is the thiopurine used in paediatric ALL maintenance. Its absence from a table headed "Recommendations (CPIC)" reads as an absence of guidance.

## Cause

Both faults are in `src/entities/pgx.rs::map_recommendations`.

`CpicRecommendationRow.phenotypes` is a `HashMap<String, String>` keyed by gene symbol (`src/sources/cpic.rs`, line 403). `map_recommendations` reduces it to one string:

```rust
let phenotype = pick_lookup_value(&row.phenotypes, preferred_gene);
let activity_score = pick_lookup_value(&row.activityscore, preferred_gene);
let implication = pick_lookup_value(&row.implications, preferred_gene);
```

`src/entities/pgx.rs::pick_lookup_value` returns the entry for the queried gene and discards every other gene in the map. NUDT15 disappears. `PgxRecommendation.phenotype` is a single `Option<String>` (`src/entities/pgx.rs`, line 144) and the table in `templates/pgx.md.j2` line 44 has one `Phenotype` column, so nothing downstream can recover the dropped gene.

`pick_lookup_value` also carries a second fault. When the queried gene is absent from the map it falls back to `map.values().find(|v| !v.trim().is_empty())`. `HashMap` iteration order is unspecified, so that fallback picks a gene at random.

The missing drugs come from the last two lines of `map_recommendations`:

```rust
out.sort_by(|a, b| a.drugname.cmp(&b.drugname));
out.truncate(30);
```

`src/sources/cpic.rs::recommendations_by_gene_page_plan` already orders by `drugname.asc,recommendationid.asc` and `src/entities/pgx.rs::get_with_cpic` requests `limit + 1` rows. The hardcoded `out.truncate(30)` then caps the section at 30 regardless of `--limit`, and the caller's own `rows.truncate(limit)` never bites above 30. Alphabetical order puts all 35 azathioprine rows ahead of every other drug, so page one is one drug and the section says nothing about the drugs it did not reach.

## The rule

Settled here. Two changes, one to the shape of a phenotype and one to coverage.

**Genotype, not phenotype.** `PgxRecommendation.phenotype: Option<String>` becomes `genotype: Vec<(String, String)>`, one entry per gene CPIC returns, sorted with the queried gene first and the remaining genes in symbol order. `activity_score` and `implication` take the same shape for the same reason. The `Phenotype` column in `templates/pgx.md.j2` becomes `Genotype` and renders the pairs as `TPMT Normal Metabolizer; NUDT15 Poor Metabolizer`. Delete `pick_lookup_value`. It has no other caller and its HashMap-order fallback is not worth keeping.

**Coverage, not silence.** Delete `out.truncate(30)` from `map_recommendations`. The section limit is the only cap. Then make the section state its own coverage: the recommendations section issues one extra bounded CPIC request for the same `lookupkey` filter selecting `drugname` only, at limit 200, and renders one line under the table naming the drugs on this page and the drugs held for this gene that are not on this page. For TPMT at the default limit that line reads `Drugs on this page: azathioprine. Also held for TPMT: mercaptopurine, thioguanine.` The caller then knows the gap is paging rather than absence.

Do not round-robin the rows across drugs and do not add a drug filter to `get pgx`. Ordering stays as CPIC returns it.

## Done when

- `biomcp get pgx TPMT recommendations` renders a `Genotype` column, and the row carrying "Consider alternative nonthiopurine immunosuppressant therapy" shows `TPMT Normal Metabolizer; NUDT15 Poor Metabolizer`. A test pins that row against a fixture.
- No two rows in that section share an identical drug and genotype key with different recommendation text. A test pins the uniqueness of the key over the TPMT fixture.
- A single-gene recommendation, such as one keyed on CYP2D6 alone, renders one genotype pair and no trailing separator. A test pins it.
- `pick_lookup_value` no longer exists. A test pins that a recommendation row whose map lacks the queried gene renders every gene it does hold, in symbol order, with no dependence on map iteration order.
- `biomcp get pgx TPMT recommendations --limit 50` returns 50 rows. A test pins the count.
- `biomcp get pgx TPMT recommendations` names mercaptopurine and thioguanine in its coverage line while showing only azathioprine rows. A test pins the line.
- `--json` output carries the genotype pairs and the coverage list. A test pins the JSON shape.
- `spec/entity/pgx.md` "Recommendations Stay Opt-In" block is updated for the `Genotype` column, and the spec gate passes.
- The existing suite stays green.

## Boundary

May change: `src/entities/pgx.rs` (`map_recommendations`, `pick_lookup_value`, the `PgxRecommendation` struct, the recommendations branch of `get_with_cpic`), `src/sources/cpic.rs` for the added drug-coverage request plan, `templates/pgx.md.j2` recommendations table, `src/render/markdown/pgx.rs`, `spec/entity/pgx.md`, and the tests for those.

Must not change: `src/entities/pgx.rs::is_likely_gene` and the gene-versus-drug classifier, `map_pair_rows` and the interactions section, `map_frequencies`, `map_guidelines`, the annotations section, `PgxSectionPagination` and the `--offset` contract, `src/sources/cpic.rs::recommendations_by_gene_page_plan` ordering, and the `search pgx` surface.
