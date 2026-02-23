# Article Keyword Search

This reference documents keyword query behavior for `biomcp search article`.

## Keyword behavior

`--keyword` (`-k`) is treated as escaped free text and no longer auto-quotes
whitespace-containing values.

This allows multi-word keyword retrieval such as:

```bash
biomcp search article -k "large language model clinical trials" --limit 5
```

## Phrase behavior for entity filters

Entity-oriented filters retain phrase quoting behavior:

- `--gene`
- `--disease`
- `--drug`
- `--author`

Example:

```bash
biomcp search article -g "BRAF V600E" --author "Jane Doe" --limit 5
```

## Combined filters

Filters can be combined with date and preprint controls:

```bash
biomcp search article -g BRAF -k "immune checkpoint blockade" --since 2024-01-01 --no-preprints --limit 10
```
