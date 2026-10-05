# 1301 — route a gene-symbol first token in a free-text variant search

Filed 2026-10-05 by the BioMCP 0.9 lead, from the KB lead's message of 2026-10-05, reproduced 2026-10-05.

Status: OPEN.

## Build status

- Built on branch `tickets/1301-gene-symbol-first`, sha `22d15888` (2026-10-05). Two review cycles, both recorded. Cycle 1: FIX, one P1 (explicit `--hgvsp`/`--consequence` dropped in both gene-first branches); fixed by carrying the flags through `VariantSearchPlan::GeneFirstCandidate` into a `finalize` extraction shared with the standard path — a pure move, so `off` restores the old fallthrough exactly. Cycle 2: ACCEPT, all four verification points confirmed, no scope creep.
- Gene-symbol source: the existing MyGene unique canonical symbol-or-alias oracle (the same one `discover` trusts, 2.5 s shared budget), behind `BIOMCP_VARIANT_QUERY_GENE_ROUTING` (`mygene` default, `off` restores). Rationale recorded in code comments and `docs/reference/configuration.md`. Reviewer rulings: no second knob; `off` stays the sole non-default value; no MCP description change (a help example line is a separate follow-up).
- Spec page `spec/entity/variant-gene-first-routing.md` proves routed, refused-uppercase, and lowercase behavior from recorded fixtures, plus zero-row working-form output. 51 targeted tests, lint green.
- Deferred: hyphenated symbols never reach the oracle (exact-form token shape); CHANGELOG bullet owed before the next release; help example line follow-up.

## Outcome

`search variant "SCN5A Brugada"` finds the SCN5A variants tied to Brugada syndrome, because the gene-symbol first token routes to the gene filter and the rest routes to the condition. A zero result never silently swallows a gene symbol.

## Evidence

- Starts from: Reproduced on main a877443f on 2026-10-05: `search variant 'SCN5A Brugada' --significance pathogenic --limit 3` prints `Query: significance=pathogenic, condition=SCN5A Brugada` and returns 0 rows. The working form `-g SCN5A --condition Brugada` returns 3. Agents in experiment 421 typed exactly the free-text form and concluded ClinVar had nothing. Root cause: `resolve_variant_query` in `src/cli/variant/dispatch.rs` routes a positional phrase through rsID, gene+change, residue-alias, gene c.HGVS and exon-deletion-phrase parsers, and its final fallthrough (about :497) assigns the whole phrase to `condition: Some(query)`. The smart-routing precedents already exist one branch above (`parse_exon_deletion_phrase` handles "EGFR exon 19 deletion").
- Keeps: Every routing that works today; an explicit `-g` or `--condition` still wins; genuinely non-gene phrases still search as conditions.
- Changes: When a positional query's first token is a known gene symbol (the same gene-token check the exact forms use), `-g` is absent, and the phrase has more tokens, route the first token to the gene filter and the remainder to the condition. When routing is ambiguous or the symbol is unknown, keep today's condition behavior but print the working form in the output when the search returns zero rows.
- Proof: A spec table from recorded responses: "SCN5A Brugada" routes to gene+condition and returns the 3 rows the explicit form returns; a phrase whose first token is not a gene keeps condition behavior; a zero result prints the working form. Design input needed before review: no gene-validation oracle exists in the parser today (`is_exact_gene_token` accepts any uppercase word, so BRUGADA would pass a naive check) — the design must name the oracle, likely the existing gene-symbol lookup on the search path, and the spec table must include an uppercase non-gene first token that does not route.
- Defers: Teaching search to parse full HGVS in the positional slot (that path already errors helpfully).
