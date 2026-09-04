# How to: use cross-entity pivots

Cross-entity pivot helpers let you move from one BioMCP entity to a related one
without rebuilding the next query from scratch. The grammar is:
`biomcp <entity> <helper> <id>` for helper families and
`biomcp get <entity> <id> <section>` for sectioned pivots such as diagnostics.
Use these when you already know the entity you want to investigate and need the
built-in related lookup.

## When to use a pivot helper vs. a fresh search

Use a pivot helper when you already have a specific entity identifier or label
and want the built-in related lookup. Use `search` when you are still
exploring or need richer downstream filters such as `--status`, `--phase`, or
`--since`.

| If you need to... | Use this | Why |
|---|---|---|
| Move from a known entity into its standard related view | Pivot helper | The helper carries the entity context for you |
| Find the right entity first | `search` | Discovery is broader than helper workflows |
| Add trial filters like recruiting status or phase | `search trial` | Helpers do not expose the full trial filter surface |
| Add literature filters like date windows | `search article` | Helpers do not expose article-only filters like `--since` |
| Page through a built-in related lookup | Pivot helper | Helpers support paging-style options such as `--limit` and `--offset` |

Current boundary: helper subcommands accept the pivot identifier plus paging or
source-style options, but they do not replace the full search surfaces for
trials, articles, or other entities. Diagnostic pivots from gene and disease
are `get` sections, not new helper subcommands.

## Variant pivots

Variant pivots are useful when you already have a mutation call and want the
next clinical or literature surface immediately.

```bash
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E" --limit 5
```

Use these helpers when the mutation is already known and you want treatment or
literature context without retyping gene or keyword filters. If you need trial
filters such as `--status recruiting` or article filters such as `--since`,
switch back to `search`.

## Drug pivots

Drug pivots are useful when you want to follow a therapy into trials or adverse
event reporting.

```bash
biomcp drug trials pembrolizumab --limit 5
biomcp drug adverse-events pembrolizumab --limit 5
```

`drug trials` reuses the intervention context. `drug adverse-events` is useful
for a quick safety review, but the output depends on OpenFDA availability and
rate limits.

## Disease pivots

Disease pivots are useful when you want to move from a diagnosis into the most
common next surfaces: trials, drugs, articles, and diagnostic tests.

```bash
biomcp disease trials melanoma --limit 5
biomcp disease drugs melanoma --limit 5
biomcp disease articles "Lynch syndrome" --limit 5
biomcp get disease tuberculosis diagnostics
```

These helpers keep the disease context intact. Most article pivots are best-effort:
the mix of article sources can vary, so rely on the heading and table shape
rather than a specific provider subsection. Resolved `variant articles` calls
union exact annotation, normalized-alias, and source-citation routes before one
ranking/pagination pass. Unresolved input labels its discovery route as
best-effort free text; `--strategy annotation|lexical` isolates exact routes for
diagnosis. The diagnostics pivot is an opt-in
`get disease` section that can combine GTR and WHO IVD local rows for the
resolved condition. It is capped at 10 rows and prints a `See also:` command
for `biomcp search diagnostic --disease <query> --source all --limit 50`; use
`--offset` on `search diagnostic` for later pages.

## Gene pivots

Gene pivots are useful when a biomarker or target is the center of the session
and you want the standard downstream clinical, diagnostic, and pathway views.

```bash
biomcp gene trials BRAF --limit 5
biomcp gene drugs BRAF --limit 5
biomcp gene articles BRCA1 --limit 5
biomcp gene pathways BRAF --limit 5
biomcp get gene BRCA1 diagnostics
```

`gene drugs` pivots into target-based drug lookup. `gene pathways` returns the
source-labelled pathway table for the gene, which is useful when you want
to expand from a biomarker into pathway context without leaving the CLI. The
diagnostics pivot is an opt-in `get gene` section backed by GTR diagnostic
tests for the gene.

## Other pivot helpers

The same pattern extends beyond the main four families:

```bash
biomcp pathway drugs R-HSA-5673001 --limit 5
biomcp protein structures P15056
biomcp article entities 22663011
biomcp article references 22663011 --limit 3
```

Use these when you already have the pathway ID, protein accession, or article
PMID in hand and want the default related lookup.

## Multi-step investigation example

A pivot workflow is most useful when you want to keep one piece of context as
you move through several entities.

Start from a variant and move directly into candidate trials, then literature:

```bash
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E" --limit 5
```

Start from a gene target and move into therapies, then pathway context:

```bash
biomcp gene drugs BRAF --limit 5
biomcp gene pathways BRAF --limit 5
```

If the next step needs richer filters than the helper exposes, keep the entity
you discovered and switch back to a fresh `search` command for that downstream
surface.
