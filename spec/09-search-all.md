# Cross-Entity Search

`search all` gives a counts-first, multi-entity orientation card for exploratory workflows. This file validates multi-slot, counts-only, gene-only, and keyword entrypoints. Assertions check section headers and structural markers that remain stable as result sets evolve.

| Section | Command focus | Why it matters |
|---|---|---|
| Multi-slot search | `search all -g BRAF -d melanoma` | Confirms cross-entity fan-out |
| Counts-only mode | `search all -g BRAF --counts-only` | Confirms deterministic structural summaries |
| Single-slot search | `search all -g BRAF` | Confirms gene-first orientation card |
| Keyword search | `search all -k "checkpoint inhibitor"` | Confirms text-driven orientation path |

## Multi-slot Search

Combining typed slots is the recommended way to orient a workflow quickly. We assert on top-level query header and entity subsection markers.

```bash
out="$(biomcp search all -g BRAF -d melanoma --limit 3)"
echo "$out" | mustmatch like "# Search All: gene=BRAF disease=melanoma"
echo "$out" | mustmatch like "## Genes"
echo "$out" | mustmatch like "## Variants"
```

## Counts-only Mode

Counts-only output is useful for low-noise planning before fetching rows. The stable marker is the explicit row-omission line plus presence of repeated entity sections.

```bash
out="$(biomcp search all -g BRAF --counts-only)"
echo "$out" | mustmatch like "Rows omitted ("
echo "$out" | mustmatch like "--counts-only"
echo "$out" | mustmatch like "## Variants"
echo "$out" | mustmatch like "## Trials"
```

## Single Gene Search

A single typed slot should still return a multi-entity orientation card. We verify top heading and trial section presence.

```bash
out="$(biomcp search all -g BRAF --limit 3)"
echo "$out" | mustmatch like "# Search All: gene=BRAF"
echo "$out" | mustmatch like "## Trials"
```

## Keyword Search

Keyword search supports a text-first starting point when entity type is unknown. The response should echo keyword context and include the articles section.

```bash
out="$(biomcp search all -k "checkpoint inhibitor" --limit 3)"
echo "$out" | mustmatch like "# Search All: keyword=checkpoint inhibitor"
echo "$out" | mustmatch like "## Articles"
```

## JSON Search All Preserves Article Metadata

The article section in `search all` should reuse the stabilized article search
pipeline rather than flattening away row-level source and ranking metadata.

```bash
out="$(biomcp --json search all -g BRAF --limit 3)"
echo "$out" | mustmatch like "\"entity\": \"article\""
echo "$out" | mustmatch like "\"source\":"
echo "$out" | mustmatch like "\"ranking\": {"
```
