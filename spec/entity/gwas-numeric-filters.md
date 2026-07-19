# GWAS numeric filter contracts

GWAS p-values are probabilities, so BioMCP validates their domain before a
catalog request. The structured error remains safe for scripts.

## GWAS p-values are probabilities

<!-- mustmatch-lint: skip -->

A p-value threshold narrows associations only when it is finite, greater than
zero, and at most one. Invalid thresholds fail locally instead of returning
confident emptiness or silently disabling the filter.

| str:value | str:label |
|---|---|
| NaN | not a number |
| +inf | positive infinity |
| -inf | negative infinity |
| 1e309 | overflow |
| 0 | zero |
| -0.01 | negative |
| 1.01 | greater than one |

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
