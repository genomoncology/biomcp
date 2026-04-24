# Article Queries

Article workflows mix typed biomedical anchors with broader keyword discovery.
These canaries keep the blocking lane honest about search structure, annotation
paths, and fulltext fallback behavior without depending on optional API keys.

## Gene Search

Gene-linked article search should still read like a literature intake surface:
clear heading, ranking note, and a PMID-first table.

```bash
out="$(../../tools/biomcp-ci search article -g BRAF --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRAF"
echo "$out" | mustmatch like "Ranking: calibrated PubMed rescue + lexical directness"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Keyword Search

Keyword search is a different planning path from typed gene search. The query
echo and source-aware table should make that distinction visible.

```bash
out="$(../../tools/biomcp-ci search article -q 'alternative microexon splicing metastasis' --source litsense2 --limit 1)"
echo "$out" | mustmatch like "keyword=alternative microexon splicing metastasis"
echo "$out" | mustmatch like "| PMID | Title | Source(s) | Date | Why | Cit. |"
```

## Search Table & Source Ranking

The JSON contract should preserve the top article follow-up and keep per-result
source identity plus ranking metadata available to automation.

```bash
json_out="$(../../tools/biomcp-ci --json search article -g BRAF --limit 3)"
echo "$json_out" | mustmatch like '"matched_sources": ['
echo "$json_out" | jq -e '._meta.next_commands[0] | test("^biomcp get article .+$")' >/dev/null
echo "$json_out" | jq -e 'all(.results[]; (.matched_sources | type) == "array" and (.ranking.mode | type == "string"))' >/dev/null
```

## PubTator Annotations

Annotations remain a first-class deepen path. The section should keep the
PubTator heading and explain that the extracted entities are normalized.

```bash
out="$(../../tools/biomcp-ci get article 22663011 annotations)"
echo "$out" | mustmatch like "## PubTator Annotations"
echo "$out" | mustmatch like "normalized entity mentions"
echo "$out" | mustmatch '/Genes: [A-Z0-9]/'
```

## Full-Text HTML Fallback

When the XML ladder misses, BioMCP should fall back to the PMC HTML article page
and still keep the saved-file contract on stdout.

```bash
bash ../fixtures/setup-article-fulltext-source-fixture.sh ../..
. ../../.cache/spec-article-fulltext-source-env
trap 'kill "${BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID:-}" 2>/dev/null || true' EXIT
out="$(../../tools/biomcp-ci get article 22663012 fulltext)"
echo "$out" | mustmatch like "## Full Text (PMC HTML)"
path="$(printf '%s\n' "$out" | sed -n 's/^Saved to: //p' | head -n1)"
test -n "$path"
saved="$(cat "$path")"
echo "$saved" | mustmatch like "PMC HTML fallback body text"
```

## PDF Fallback Is Opt-In

Semantic Scholar PDF is a last resort, not the default resolver order. The same
fixture-backed article should fail cleanly without `--pdf` and succeed with it.

```bash
bash ../fixtures/setup-article-fulltext-source-fixture.sh ../..
. ../../.cache/spec-article-fulltext-source-env
trap 'kill "${BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID:-}" 2>/dev/null || true' EXIT
default_out="$(../../tools/biomcp-ci get article 22663013 fulltext)"
echo "$default_out" | mustmatch like "XML and HTML sources did not return full text"
echo "$default_out" | mustmatch not like "Semantic Scholar PDF"
pdf_out="$(../../tools/biomcp-ci get article 22663013 fulltext --pdf)"
echo "$pdf_out" | mustmatch like "## Full Text (Semantic Scholar PDF)"
pdf_path="$(printf '%s\n' "$pdf_out" | sed -n 's/^Saved to: //p' | head -n1)"
test -n "$pdf_path"
test -f "$pdf_path"
```

## Semantic Scholar Degrades Truthfully Without a Key

The blocking lane is intentionally keyless. Article search should stay usable
and explicit about the no-key path rather than hard-failing or pretending the
keyed data plane ran.

```bash
json_out="$(../../tools/biomcp-ci --json search article -g BRAF --limit 3 2>/dev/null)"
echo "$json_out" | mustmatch like '"semantic_scholar_enabled": true'
echo "$json_out" | mustmatch like '"ranking": {'
echo "$json_out" | jq -e 'all(.results[]; .ranking.mode == "lexical")' >/dev/null
```

## Entity Follow-Up

`article entities` is the compact follow-up in this bootstrap slice. It should
still expose the gene subsection and typed follow-up commands.

```bash
out="$(../../tools/biomcp-ci article entities 22663011)"
echo "$out" | mustmatch like "# Entities in PMID 22663011"
echo "$out" | mustmatch like "## Genes ("
echo "$out" | mustmatch like '`biomcp search gene -q BRAF`'
```
