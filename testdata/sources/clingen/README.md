# ClinGen gene fixtures

`lookup_tp53.json`, `validity_tp53.csv`, and `dosage_tp53.csv` are minimized
public captures from 2026-09-05. The lookup capture retains only the exact TP53
row. Each CSV retains the provider metadata, separator, header, and TP53 row.
Their receipts record the public request, committed-byte hash, and the original
full-response hash before minimization.

`lookup_braf.json`, `validity.csv`, `validity_hgnc_only.csv`, and `dosage.csv`
predate receipt enforcement and remain pending verification.

Delay, failure, malformed-schema, HTML, invalid-encoding, and oversized-body
responses used by tests are derived synthetic variants. They are created by
the test fixture and are not byte-faithful provider captures.
