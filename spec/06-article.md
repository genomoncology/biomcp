# Article Queries

Article commands provide literature retrieval and annotation-focused enrichment for entity extraction. This spec validates both retrieval modes and PubTator annotation surfaces. Assertions are anchored to headings, IDs, and table schemas that remain stable over time.

| Section | Command focus | Why it matters |
|---|---|---|
| Gene search | `search article -g BRAF` | Confirms gene-linked literature lookup |
| Keyword search | `search article -k immunotherapy` | Confirms free-text discovery |
| Article detail | `get article 22663011` | Confirms canonical article card output |
| Annotation section | `get article ... annotations` | Confirms PubTator integration |
| Entity helper | `article entities 22663011` | Confirms entity extraction pivot |
| Batch helper | `article batch 22663011 24200969` | Confirms compact multi-article fetch |
| Semantic Scholar detail | `get article 22663011 tldr` | Confirms optional-key enrichment section |
| Semantic Scholar graph | `article citations|references 22663011` | Confirms citation graph pivots |
| Semantic Scholar recommendations | `article recommendations ...` | Confirms related-paper pivots |

## Searching by Gene

Gene-based literature search is a common evidence collection step in variant and disease workflows. We assert on heading context and table columns.

```bash
out="$(biomcp search article -g BRAF --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRAF"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Searching by Keyword

Keyword search supports broad discovery before narrowing to specific entities. The output should echo keyword context and include PMID-centric table output.

```bash
out="$(biomcp search article -k immunotherapy --limit 3)"
echo "$out" | mustmatch like "keyword=immunotherapy"
echo "$out" | mustmatch like "PMID"
```

## Getting Article Details

The article detail card should preserve stable bibliographic anchors for reproducible referencing. We assert on PMID and journal markers.

```bash
out="$(biomcp get article 22663011)"
echo "$out" | mustmatch like "PMID: 22663011"
echo "$out" | mustmatch like "Journal:"
```

## Article Annotations

Annotation output summarizes entity classes detected by PubTator. The assertions target section heading and gene summary marker.

```bash
out="$(biomcp get article 22663011 annotations)"
echo "$out" | mustmatch like "## PubTator Annotations"
echo "$out" | mustmatch like "Genes:"
```

## Article Full Text Saved Markdown

Full text remains a path-based contract on stdout. The proof needs to confirm
that BioMCP still prints `Saved to:` while the cached file now contains
structured Markdown from PMC/JATS instead of flattened XML.

```bash
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
out="$(TMPDIR="$tmpdir" biomcp get article 27083046 fulltext)"
echo "$out" | mustmatch like "## Full Text"
echo "$out" | mustmatch like "Saved to:"
path="$(printf '%s\n' "$out" | sed -n 's/^Saved to: //p' | head -n1)"
test -n "$path"
test -f "$path"
saved="$(cat "$path")"
echo "$saved" | mustmatch like "# Synaptotagmin-1 C2B domain interacts simultaneously"
echo "$saved" | mustmatch like "## Abstract"
echo "$saved" | mustmatch like "## Introduction"
echo "$saved" | mustmatch like "## References"
echo "$saved" | mustmatch like "Zhou et al., 2015"
echo "$saved" | mustmatch not like "Creative Commons Attribution License"
echo "$saved" | mustmatch not like "eLife Sciences Publications"
```

## Article to Entities

`article entities` exposes actionable next-command pivots by entity class. We check top-level heading and genes subsection marker.

```bash
out="$(biomcp article entities 22663011)"
echo "$out" | mustmatch like "# Entities in PMID 22663011"
echo "$out" | mustmatch like "## Genes"
```

## Article Batch

`article batch` returns compact numbered cards for known IDs without
changing single-article output. The markdown contract exposes a stable heading,
numbered card sections with PMID/bibliographic fields, and degrades cleanly
without Semantic Scholar TLDR data.

```bash
out="$(biomcp article batch 22663011 24200969)"
echo "$out" | mustmatch like "# Article Batch (2)"
echo "$out" | mustmatch like "## 1."
echo "$out" | mustmatch like "## 2."
echo "$out" | mustmatch like "PMID: 22663011"
echo "$out" | mustmatch like "PMID: 24200969"

json_out="$(biomcp --json article batch 22663011 24200969)"
echo "$json_out" | mustmatch like "\"requested_id\": \"22663011\""
echo "$json_out" | mustmatch like "\"pmid\": \"22663011\""
echo "$json_out" | mustmatch like "\"title\":"
echo "$json_out" | mustmatch like "\"year\":"

no_key_out="$(env -u S2_API_KEY biomcp --json article batch 22663011)"
echo "$no_key_out" | mustmatch like "\"requested_id\": \"22663011\""
echo "$no_key_out" | mustmatch like "\"title\":"
echo "$no_key_out" | mustmatch not like "\"tldr\":"
```

## Article Batch Invalid Identifier

An unsupported identifier format should fail with the existing supported
identifier guidance rather than a generic error.

```bash
out="$(biomcp article batch S1535610826000103 2>&1 || true)"
echo "$out" | mustmatch like "Unsupported identifier"
```

## Article Batch Limit Enforcement

More than 20 IDs should fail immediately, before any network work.

```bash
out="$(biomcp article batch 1000001 1000002 1000003 1000004 1000005 1000006 1000007 1000008 1000009 1000010 1000011 1000012 1000013 1000014 1000015 1000016 1000017 1000018 1000019 1000020 1000021 2>&1 || true)"
echo "$out" | mustmatch like "limited to 20"
```

## Optional-Key Get Article Path

Ordinary `get article` must still work when Semantic Scholar is unavailable. We
force the no-key path even on keyed machines and assert that the PubMed card
still renders without the Semantic Scholar section.

```bash
out="$(env -u S2_API_KEY biomcp get article 22663011)"
echo "$out" | mustmatch like "PMID: 22663011"
echo "$out" | mustmatch like "Journal:"
echo "$out" | mustmatch not like "Semantic Scholar"
```

## Article Search JSON Without Semantic Scholar Key

No-key article search must stay explicit and functional. JSON should report the
disabled state while still surfacing ranking metadata from the local relevance
policy.

```bash
out="$(env -u S2_API_KEY biomcp --json search article -g BRAF --limit 3)"
echo "$out" | mustmatch like "\"semantic_scholar_enabled\": false"
echo "$out" | mustmatch like "\"ranking\": {"
echo "$out" | mustmatch not like "\"source\": \"semanticscholar\""
```

## Article Search JSON With Semantic Scholar Key

When `S2_API_KEY` is present, article search should expose the keyed search-leg
state and merged source metadata in JSON.

```bash
out="$(biomcp --json search article -g BRAF -d melanoma --include-retracted --limit 5)"
echo "$out" | mustmatch like "\"semantic_scholar_enabled\": true"
echo "$out" | mustmatch like "\"matched_sources\": ["
echo "$out" | mustmatch like "\"ranking\": {"
```

## Article Debug Plan

The optional debug plan should expose the actual search surface, planner
markers, and sources in both markdown and JSON without changing default output.

```bash
out="$(env -u S2_API_KEY biomcp search article -g BRAF --debug-plan --limit 3)"
echo "$out" | mustmatch like "## Debug plan"
echo "$out" | mustmatch like "\"surface\": \"search_article\""
echo "$out" | mustmatch like "\"planner=federated\""

json_out="$(env -u S2_API_KEY biomcp --json search article -g BRAF --debug-plan --limit 3)"
echo "$json_out" | mustmatch like "\"debug_plan\": {"
echo "$json_out" | mustmatch like "\"surface\": \"search_article\""
echo "$json_out" | mustmatch like "\"leg\": \"article\""
echo "$json_out" | mustmatch like "\"sources\": ["
```

## Semantic Scholar TLDR Section

When `S2_API_KEY` is present, `get article ... tldr` isolates the Semantic
Scholar enrichment section and exposes stable markers for TLDR and influence
metrics.

```bash
out="$(biomcp get article 22663011 tldr)"
echo "$out" | mustmatch like "# "
echo "$out" | mustmatch like "Semantic Scholar"
echo "$out" | mustmatch like "TLDR:"
echo "$out" | mustmatch like "Influential citations:"
```

## Semantic Scholar Citations

Citation traversal should expose a graph table with contexts, intents, and the
influential flag visible to the user.

```bash
out="$(biomcp article citations 22663011 --limit 3)"
echo "$out" | mustmatch like "# Citations for"
echo "$out" | mustmatch like "| PMID | Title | Intents | Influential | Context |"
```

## Semantic Scholar References

Reference traversal should expose the same visible graph columns.

```bash
out="$(biomcp article references 22663011 --limit 3)"
echo "$out" | mustmatch like "# References for"
echo "$out" | mustmatch like "| PMID | Title | Intents | Influential | Context |"
```

## Semantic Scholar Recommendations (Single Seed)

Single-seed recommendations should render related papers with stable table
columns.

```bash
out="$(biomcp article recommendations 22663011 --limit 3)"
echo "$out" | mustmatch like "# Recommendations for"
echo "$out" | mustmatch like "| PMID | Title | Journal | Year |"
```

## Semantic Scholar Recommendations (Multi Seed)

Multi-paper recommendation requests should accept repeated positive seeds plus a
negative set and still render the recommendation table.

```bash
out="$(biomcp article recommendations 22663011 24200969 --negative 39073865 --limit 3)"
echo "$out" | mustmatch like "# Recommendations for"
echo "$out" | mustmatch like "| PMID | Title | Journal | Year |"
echo "$out" | mustmatch like "Negative seeds:"
```

## Semantic Scholar Requires API Key For Native Helpers

The new Semantic Scholar-native helper commands are explicit optional-key
surfaces. Without the key they should fail clearly instead of silently falling
back.

```bash
status=0
out="$(env -u S2_API_KEY biomcp article citations 22663011 --limit 3 2>&1)" || status=$?
test "$status" -ne 0
echo "$out" | mustmatch like "API key required"
echo "$out" | mustmatch like "S2_API_KEY"
echo "$out" | mustmatch like "Semantic Scholar"
```

## Invalid Identifier Rejection

BioMCP supports PMID, PMCID, and DOI for article lookup. Unsupported formats such as
publisher PIIs must fail fast, return a non-zero exit, and name the supported types in
the error text.

```bash
status=0
out="$(biomcp get article S1535610826000103 2>&1)" || status=$?
test "$status" -ne 0
echo "$out" | mustmatch like "PMID"
echo "$out" | mustmatch like "PMCID"
echo "$out" | mustmatch like "DOI"
echo "$out" | mustmatch like "publisher"
```

## Sort Behavior

Default article search uses relevance sort. The output header echoes the sort in effect so callers can verify the default.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search article -k melanoma --limit 3)"
echo "$out" | mustmatch like "sort=relevance"
```

Passing `--sort date` opts into date-based ordering.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search article -k melanoma --sort date --limit 3)"
echo "$out" | mustmatch like "sort=date"
```

## Federated Deep Offset Guard

Federated article search merges PubTator3 and Europe PMC before applying paging. Very deep offsets must fail fast with an explicit bound so callers do not get silently incorrect merged windows.

```bash
status=0
out="$(biomcp search article -k melanoma --limit 50 --offset 1201 2>&1)" || status=$?
test "$status" -ne 0
echo "$out" | mustmatch like "--offset + --limit must be <= 1250"
```
