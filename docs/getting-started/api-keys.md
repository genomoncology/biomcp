# API Keys

Most BioMCP workflows run without credentials.
Some enrichment and higher-rate access paths improve with API keys.

## Required for prediction section

### `ALPHAGENOME_API_KEY`

Used by variant prediction lookups:

```bash
export ALPHAGENOME_API_KEY="..."
biomcp get variant "chr7:g.140453136A>T" predict
```

## Optional enrichment keys

### `ONCOKB_TOKEN`

Used for production OncoKB enrichment.
If omitted, BioMCP uses the public/demo path when possible.

```bash
export ONCOKB_TOKEN="..."
biomcp get variant "BRAF V600E"
```

### `NCI_API_KEY`

Used for NCI CTS trial calls.

```bash
export NCI_API_KEY="..."
biomcp search trial -c melanoma --source nci
```

### `NCBI_API_KEY`

Improves rate limits for PubTator, PMC OA, and NCBI ID converter (3 → 10 req/sec).

```bash
export NCBI_API_KEY="..."
biomcp search article -g BRAF --limit 5
```

### `OPENFDA_API_KEY`

Improves OpenFDA rate limits for drug safety lookups.

```bash
export OPENFDA_API_KEY="..."
biomcp search adverse-event --drug pembrolizumab --limit 5
```

## Key management guidance

- Prefer environment variables over hardcoded values.
- Do not commit secrets into source control.
- Set keys in the same environment used by your MCP client.
- Rotate keys when sharing machines or CI runners.
