# How To: Find Articles

This guide shows practical literature-search patterns.

## Broad start

```bash
biomcp search article -g BRAF --limit 10
```

`search article` always works without credentials. When `S2_API_KEY` is set,
BioMCP adds an optional Semantic Scholar search leg and keeps `sort=relevance`
directness-first instead of citation-first.

## Add disease context

```bash
biomcp search article -g BRAF -d melanoma --limit 10
```

## Constrain by date

```bash
biomcp search article -g BRAF --since 2024-01-01 --limit 10
```

## Exclude preprints when supported

```bash
biomcp search article -g BRAF --since 2024-01-01 --no-preprints --limit 10
```

## Pull the full text section

```bash
biomcp get article 22663011 fulltext
```

## Fetch several shortlisted papers at once

```bash
biomcp article batch 22663011 24200969 39073865
```

Use `article batch` after search when you already know the candidate PMIDs or
DOIs and want compact title/journal/year/entity cards before opening one paper
in full detail. The helper preserves input order and still works when
`S2_API_KEY` is unset.

## Inspect the ranking rationale in JSON

```bash
env -u S2_API_KEY biomcp --json search article -g BRAF --limit 3
```

Look for `semantic_scholar_enabled`, row-level `matched_sources`, and
`ranking` metadata to see why a paper ranked where it did.

## Inspect the executed search plan

Markdown:

```bash
env -u S2_API_KEY biomcp search article -g BRAF --debug-plan --limit 3
```

JSON / MCP-friendly text output:

```bash
env -u S2_API_KEY biomcp --json search article -g BRAF --debug-plan --limit 3
```

`--debug-plan` adds a top-level `debug_plan` payload in JSON and prepends the
same payload as a fenced JSON block in markdown. Request JSON+plan for MCP
callers with `--json --debug-plan`.

## Follow-up pattern

After identifying key papers, pivot to trials or variants:

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
biomcp search variant -g BRAF --limit 5
```
