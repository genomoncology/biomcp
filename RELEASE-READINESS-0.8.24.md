# Release readiness 0.8.24

Date: 2026-06-24
Binary smoked: `target/release/biomcp`
Commit: `f324e7f9`

## Version sync

Command: `bash scripts/check-version-sync.sh`

Result: PASS — `Versions in sync: 0.8.24`.

## Release smoke

Command:

```bash
scripts/release-smoke.sh
```

Result: PASS — 23 passed, 0 failed.

Smoke items:

- PASS 240 serve-http default accepts foreign Host.
- PASS 240 serve-http `--allowed-hosts` rejects foreign Host on `/mcp`.
- PASS 239 plugin marketplace metadata is valid and wires `biomcp serve`.
- PASS 435 typed MCP `search` / `get` tools are present with entity enum schemas.
- PASS 434 MCP `get gene BRAF` includes source/provenance footer.
- PASS 436 over-cap `--limit` states the valid range.
- PASS 436 multi-word `get drug` gives search guidance.
- PASS 436 multi-word `get disease` gives search guidance.
- PASS 436 `get pathway P15056` gives a protein redirect hint.
- PASS 437 normal gene query emits no `WARN` on stderr.
- PASS 438 `gene trials BRAF --limit 1` exits promptly.
- PASS 438 `disease trials melanoma --limit 1` exits promptly.
- PASS 439 `get gene PD-L1` resolves CD274.
- PASS 439 `get gene HER2` resolves ERBB2.
- PASS 439 `get gene P53` resolves TP53.
- PASS 440 `get variant 'NM_004333.6:c.1799T>A'` resolves BRAF V600E / rs113488022.
- PASS 441 `--json get gene NOT_A_GENE_445` emits JSON error on stdout.
- PASS 441 `--json get variant bogusvar445` emits JSON error on stdout.
- PASS 443 `get pathway ENSG00000157764` gives a gene redirect hint.
- PASS 443 `get pathway BRAF` gives a gene redirect hint.
- PASS 443 `get pathway rs113488022` gives a variant redirect hint.
- PASS 444 `--version` reports 0.8.24.
- PASS 444 `version` reports the current HEAD git SHA.

Notes:

- An initial smoke-script draft checked the unrestricted health route for the
  `--allowed-hosts` rejection. The durable script now checks `/mcp`, which is the
  guarded MCP endpoint.
- One initial transcript-HGVS live call hit a transient normalization-service retry
  failure, then passed on rerun with the same release binary. No release-blocking
  failure remains.

## Gate results

- PASS `make lint`
- PASS `make test`
- PASS `make spec`
- PASS `make verify`
