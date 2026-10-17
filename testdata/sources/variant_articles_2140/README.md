# variant_articles_2140 fixtures

Recorded replies for ticket 2140 (tie genomic aliases to the gene in variant
article search). Every file is a byte-faithful capture from the named public
provider on 2026-10-17, key-less, receipted in `../capture-receipts.json`.
The two recorded-reply tests in `src/entities/article/variant_search.rs`
(`samhd1_..._tie_the_genomic_alias_to_the_gene`,
`cdkn2a_..._tie_the_genomic_alias_to_the_gene`) serve these bytes from a local
fixture origin and route by request shape.

## `myvariant/`

- `query_samhd1_i448t_20261017.json` — the resolution hit list for
  `dbnsfp.genename:SAMHD1 AND dbnsfp.hgvsp:"p.I448T"`: one hit whose genomic id
  is `chr20:g.35533834A>G` (rs774964432, c.1343T>C, p.Ile448Thr). The source
  identity the article search builds from it carries the bare coordinate alias
  the ticket is about.
- `query_cdkn2a_a57v_20261017.json` — the same shape for
  `dbnsfp.genename:CDKN2A AND dbnsfp.hgvsp:"p.A57V"` (two hits; the retained one
  carries `chr9:g.21971188G>A`, rs372266620).
- `variant_chr20_g_35533834A_G_20261017.json`,
  `variant_chr9_g_21971188G_A_20261017.json` — the citation-hydration lookups
  the source-citation route runs for the retained hits.

## `pubmed/`

The keyword legs of the exact-lexical route (`term=<alias> NOT retracted
publication[pt]`):

- `esearch_samhd1_i448t_keyword_20261017.json` (930 hits, first 100 ids) with
  `esummary_samhd1_i448t_keyword_20261017.json` — the SAMHD1-naming papers the
  search must keep, e.g. 42857107 ("Dynamic changes in IFI44L and SAMHD1 mRNA
  levels...").
- `esearch_samhd1_chr20_bare_20261017.json` (15 ids) with
  `esummary_samhd1_chr20_bare_20261017.json` — what the bare coordinate
  `chr20:g.35533834A>G` returns today: papers matched only because they mention
  "chr20", including the cotton paper 28794480.
- `esearch_samhd1_chr20_gene_tied_20261017.json` — what
  `SAMHD1 chr20:g.35533834A>G` returns: count 0.
- `esearch_cdkn2a_a57v_keyword_20261017.json` (2 ids:
  23190892, 18509008) with `esummary_cdkn2a_a57v_keyword_20261017.json`.
- `esearch_cdkn2a_chr9_bare_20261017.json` (139 hits, first 100 ids) with
  `esummary_cdkn2a_chr9_bare_20261017.json` — includes the citrus paper 41894487
  at native position 5.
- `esearch_cdkn2a_chr9_gene_tied_20261017.json` (4 ids: 42284679, 34256839,
  33335285, 11025389) with `esummary_cdkn2a_chr9_gene_tied_20261017.json` — the
  papers a gene-tied coordinate alias keeps: about CDKN2A's chromosome 9
  neighborhood, no citrus.
- `esearch_samhd1_strict_i448t_empty_20261017.json` — a real count-0 ESearch
  reply used as the generic empty answer for every other term.

## `europepmc/`

- `search_cdkn2a_a57v_keyword_20261017.json` — six rows for the primary-alias
  keyword query, including 18178632 ("CDKN2A mutations and melanoma risk in the
  Icelandic population"), the CDKN2A-naming paper the search must keep.
- `search_zero_hit_20261017.json` — a real zero-hit REST search reply used as
  the generic empty answer (also the real reply for the strict
  `TITLE_ABS:"SAMHD1 I378T"` query).

## `pubtator/`

- `autocomplete_samhd1_i448t_20261017.json`,
  `autocomplete_cdkn2a_a57v_20261017.json` — the one variant entity each
  gene+change autocomplete returns (`@VARIANT_p.I448T_SAMHD1_human`,
  `@VARIANT_p.A57V_CDKN2A_human`), with the rsID in `db_id`.
- `search_samhd1_entity_20261017.json`,
  `search_cdkn2a_entity_20261017.json` — the annotated papers each entity id
  search returns.
- `search_zero_hit_20261017.json` — a real count-0 search reply used as the
  generic empty answer.
