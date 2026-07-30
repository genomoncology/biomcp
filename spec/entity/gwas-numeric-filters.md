# GWAS numeric filter contracts

GWAS p-values are probabilities, so BioMCP validates their domain before a
catalog request. The structured error remains safe for scripts.

## GWAS p-values are probabilities

<!-- mustmatch-lint: skip -->

A p-value threshold narrows associations only when it is finite, greater than
zero, and at most one. Invalid thresholds fail locally instead of returning
confident emptiness or silently disabling the filter.

The native table-driven test
`search_gwas_page_rejects_invalid_probability_before_client_construction` covers
the full non-finite and out-of-range matrix. This command keeps the CLI's
script-safe error shape visible to users.

| str:value | str:label |
|---|---|
| 0 | zero |

```bash run id=invalid-gwas-p-value exit=2 each_row="GWAS p-values are probabilities"
biomcp --json search gwas BRAF --p-value={{value}} --limit 1
```

```json expect=invalid-gwas-p-value contains each_row="GWAS p-values are probabilities"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```text expect=invalid-gwas-p-value contains each_row="GWAS p-values are probabilities"
--p-value
```
