# How To: Find Articles

This guide shows practical literature-search patterns.

## Broad start

```bash
biomcp search article -g BRAF --limit 10
```

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

## Follow-up pattern

After identifying key papers, pivot to trials or variants:

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
biomcp search variant -g BRAF --limit 5
```
