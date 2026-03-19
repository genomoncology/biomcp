# Table S3. Normalization and notation-acceptance stress test

The finished table will summarize how BioMCP resolves alias, brand-name, and notation-heavy inputs into canonical biomedical identifiers.

## Expected columns

| Column | Meaning |
| --- | --- |
| `Category` | Type of normalization case being tested. |
| `Input` | Original user-facing query string. |
| `Expected canonical` | Canonical symbol, identifier, or normalized representation expected by reviewers. |
| `Resolved` | Canonical value BioMCP resolved during the benchmark. |
| `Match note` | Short note explaining whether the result matched and any caveats. |
