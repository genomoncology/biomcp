---
flow: build
priority: 6
---

# `search variant` returns a silent zero when a filter value is not the one the provider indexes

`biomcp search variant` sends every filter to MyVariant.info as a literal term. A gene symbol or a protein change that the provider does not index under that exact string matches nothing, and the command prints `Found: 0 variant(s)` with no other signal. A caller cannot tell a real absence of evidence from a query the backend never had a chance to answer. Two confirmed cases share that one cause, and one mechanism closes both.

Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01. The binary on PATH is 0.8.25 and shows the same behaviour.

## Case one: a current HGNC symbol returns zero

```
$ biomcp search variant -g H3-3A
# Variant Search Results

Found: 0 variant(s)
Query: gene=H3-3A

No variants found matching the filters.

Showing 0 of 0 results.
```

```
$ biomcp search variant -g H3F3A
# Variant Search Results

Found: 10 variant(s)
Query: gene=H3F3A
...
Showing 1-10 of 1156 results. Use --offset 10 for more.
```

`H3-3A` is the current HGNC symbol. `H3F3A` is the withdrawn symbol. `biomcp get gene H3-3A` already succeeds and already prints `H3F3A` in its Aliases block, so the mapping is available to the tool.

## Case two: literature protein numbering returns zero

```
$ biomcp search variant -g H3F3A --hgvsp K27M
Requested variant: H3F3A K27M

Resolution: Unresolved

# Variant Search Results

Found: 0 variant(s)
Query: gene=H3F3A, hgvsp=K27M

No variants found matching the filters.
```

```
$ biomcp search variant -g H3F3A --hgvsp K28M
Requested variant: H3F3A K28M

Resolution: Resolved

# Variant Search Results

Found: 1 variant(s)
Query: gene=H3F3A, hgvsp=K28M

| ID | Build | Gene | Transcript | Coding | Protein | Legacy Name | ...
| chr1:g.226252135A>T | GRCh37 | H3F3A | NM_002107.4 | c.83A>T | p.Lys28Met | H3F3A K28M | ...
```

Papers and clinicians write K27M for this lesion. dbNSFP numbers the protein from the initiator methionine and holds the same change as K28M. `Resolution: Unresolved` is the only hint, and it appears only when `--hgvsp` is present. A gene-only zero gets no resolution line at all.

## Cause

`src/sources/myvariant.rs::MyVariantClient::search_plan` builds the outbound query. The gene filter becomes one exact term:

```rust
terms.push(format!(
    "dbnsfp.genename:{}",
    Self::escape_query_value(gene)
));
```

The protein filter becomes one exact phrase:

```rust
terms.push(format!("dbnsfp.hgvsp:\"{}\"", Self::escape_query_value(&v)));
```

Neither value is resolved against anything. Confirmed directly against the provider:

```
$ curl -s 'https://myvariant.info/v1/query?q=dbnsfp.genename:H3-3A&size=1'
{"took":3,"total":0,"max_score":null,"hits":[]}
$ curl -s 'https://myvariant.info/v1/query?q=dbnsfp.genename:H3F3A&size=1'
{"took":2,"total":1156,...}
```

`src/cli/variant/dispatch.rs::normalize_search_hgvsp` is the only normalisation applied to `--hgvsp`. It calls `src/entities/variant/resolution.rs::normalize_protein_change`, which converts three-letter amino acid codes to one-letter codes and rewrites a trailing `*` to `X`. It never touches the residue position.

`src/entities/variant/search/mod.rs::search_page_with_execution` returns a `VariantSearchPage`. The page carries `resolution: Option<VariantSearchResolution>` and the value is set only on the `requested_identity` branch, so a gene-only search returns `resolution: None`. `src/cli/variant/dispatch.rs` prints the resolution line only when both `requested_variant` and `resolution` are present. Nothing in the type can carry "the gene filter matched no records" or "the protein change matched no records".

## The diagnostic rule

One mechanism, not two point patches. Both cases are the same failure: the command discards the reason for a zero. A per-defect patch would add a gene warning in one place and a protein warning in another, and the third case of the same class would arrive with no home. Build the channel once.

`VariantSearchPage` gains `diagnostics: Vec<SearchDiagnostic>`. Every `search variant` call populates it. Markdown renders a `## Filter diagnostics` section above the results table when the vector is non-empty. JSON renders a `diagnostics` array on the search envelope. The three producers are fixed:

1. **Gene symbol.** Run the dbNSFP query with the requested symbol. On zero hits, resolve the symbol through MyGene.info with `src/entities/gene.rs::mygene_query_term` and collect the matched hit's symbol and aliases. Retry the dbNSFP query with each alias in turn, in the order MyGene returns them, and stop at the first alias that returns hits. Emit `gene H3-3A matched no dbNSFP records; retried as H3F3A and matched 1156` and return the retried results. When no alias returns hits, emit `gene H3-3A matched no dbNSFP records under any known symbol or alias` and return zero. The substitution is never silent. Do not attempt the retry when the first query returns hits.

2. **Protein change.** Never renumber a position. On zero hits with a gene that matched, run one bounded probe for the same gene with the position left open, then report the positions dbNSFP holds for that reference and alternate residue pair. The probe form is verified:

   ```
   $ curl -s --get 'https://myvariant.info/v1/query' \
       --data-urlencode 'q=dbnsfp.genename:H3F3A AND dbnsfp.hgvsp:p.K*M' --data 'size=5'
   {"took":17,"total":5,...}
   ```

   It returns p.K19M, p.K28M, p.K37M and p.K57M. The diagnostic reads `no dbNSFP record for H3F3A p.K27M; dbNSFP holds K to M at positions 19, 28, 37, 57`. The caller sees the off-by-one and picks. Run the probe only on the zero path, and only when the reference residue, position and alternate residue all parse.

3. **A true empty.** When every filter matched something and the intersection is still empty, emit `filters applied; no record matched` so zero reads as an answer rather than a fault.

`Resolution:` stays exactly as it is. It answers a different question and it must not absorb this one.

## Done when

- `biomcp search variant -g H3-3A` returns the same rows as `-g H3F3A` and prints a diagnostic naming the retry. A test pins the row count and the diagnostic text.
- `biomcp search variant -g H3F3A` still returns its rows and prints no gene diagnostic. A test pins that the alias retry does not run when the first query returns hits.
- A gene symbol that matches nothing under any alias returns zero and prints the "no records under any known symbol or alias" diagnostic. A test pins it.
- `biomcp search variant -g H3F3A --hgvsp K27M` returns zero rows and prints a diagnostic naming position 28. A test pins the positions listed.
- `biomcp search variant -g H3F3A --hgvsp K28M` still resolves and still prints `Resolution: Resolved`. A test pins that the probe does not run on a non-zero result.
- A search whose filters all match, and whose intersection is empty, prints the `filters applied; no record matched` diagnostic. A test pins it.
- `--json` output carries the same diagnostics as a `diagnostics` array. A test pins the JSON shape.
- `spec/entity/variant.md` gains a block for the diagnostics section, and the spec gate passes.
- The existing suite stays green.

## Boundary

May change: `src/entities/variant/search/mod.rs` (the `VariantSearchPage` type, `search_page`, `search_page_with_execution`), `src/sources/myvariant.rs::MyVariantClient::search_plan` and the alias-probe request plans beside it, `src/cli/variant/dispatch.rs` where the search outcome is assembled, `src/render/markdown/variant.rs::variant_search_markdown_with_context`, `src/render/json.rs::with_variant_search_resolution`, `spec/entity/variant.md`, and the tests for those.

Must not change: `VariantResolutionStatus`, `src/entities/variant/search/mod.rs::resolution_status`, `finalize_exact_page` or the exact-identity retention path; `src/entities/variant/resolution.rs::normalize_protein_change`; the `protein_alias` residue-alias search and its query terms; the existing search results table columns; the `get variant` path; `src/entities/gene.rs::resolve_unique_canonical_alias` and the `get gene` alias-fallback behaviour.
