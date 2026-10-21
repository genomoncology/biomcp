---
base: 2583566f800b56e47aa6c0808285128a339354af
head: 66bab1ebb5cdce5f0a126b68f5b0e0e1ba565a0c
---

# Add dbSNP population coordinate resolution

Eligible identifier-based population requests now use a uniquely matched,
allele-compatible GRCh38 dbSNP placement before querying gnomAD v4. The
response identifies the resolved coordinate and both data sources, while direct
GRCh38 coordinates retain their existing gnomAD-only path.
