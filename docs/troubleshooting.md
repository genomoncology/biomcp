# Troubleshooting

BioMCP depends on multiple public biomedical APIs, so transient source failures are expected.
This guide focuses on practical triage for API failures, slow responses, and environment issues.
Start with health checks, then narrow to the affected entity and source.

## 1) Validate connectivity first

Run API-level checks before debugging entity-specific commands:

```bash
biomcp health --apis-only
```

If one source fails while others pass, the issue is usually upstream availability and not your local install.

## 2) MyGene / MyVariant intermittent failures

Gene and variant lookups rely on BioThings services (`mygene.info`, `myvariant.info`).
These services can intermittently return 5xx or timeout responses during peak traffic.

What BioMCP already does:

- Uses shared HTTP client timeouts (`connect_timeout=10s`, `timeout=30s`)
- Retries transient failures with exponential backoff (up to 3 retries)
- Uses HTTP cache to reduce repeat upstream calls

What to do when failures persist:

```bash
biomcp --no-cache search gene -q BRAF --limit 3
biomcp --no-cache get variant rs113488022
```

If `--no-cache` works while cached mode fails repeatedly, clear cache and retry:

```bash
rm -rf ~/.cache/biomcp/http-cacache
```

## 3) ClinicalTrials.gov API v2 quirks

ClinicalTrials.gov search behavior can vary with complex query combinations and pagination tokens.
The most common symptoms are empty pages after filters or unstable totals between repeated calls.

Recommended steps:

- Start with minimal filters (`-c`, `--limit`) and add one filter at a time.
- Use explicit `--source ctgov` or `--source nci` to isolate source behavior.
- Avoid changing both geographic and ESSIE filters in the same troubleshooting step.

Examples:

```bash
biomcp search trial -c melanoma --source ctgov --limit 5
biomcp search trial -c melanoma --source nci --limit 5
```

## 4) OpenFDA FAERS / recall pagination limits

OpenFDA-backed searches are currently capped by BioMCP at `--limit <= 50` per request.
If you need more data, page your own workflow with narrower queries and repeated commands.

```bash
biomcp search adverse-event --drug pembrolizumab --limit 50
biomcp search adverse-event --type recall --classification "Class I" --limit 50
```

If you have an OpenFDA API key, export it to increase quota stability:

```bash
export OPENFDA_API_KEY="..."
```

## 5) PubTator annotation timeouts

Article annotation sections (`annotations`) require PubTator3 and can be slower than base PubMed retrieval.
When this section is slow or fails, verify basic article lookup first.

```bash
biomcp get article 22663011
biomcp get article 22663011 annotations
```

If base lookup succeeds but annotations fail repeatedly, retry later and keep the workflow moving with base metadata.

## 6) AlphaGenome prediction connection issues

`get variant <id> predict` uses AlphaGenome over gRPC and requires an API key.
Missing key or TLS/connectivity issues are the two most common failure paths.

Checklist:

- Confirm `ALPHAGENOME_API_KEY` is set
- Validate outbound access to `gdmscience.googleapis.com`
- Retry with a known-good variant

```bash
export ALPHAGENOME_API_KEY="..."
biomcp get variant "chr7:g.140453136A>T" predict
```

## 7) NCI CTS API authentication failures

Trial searches with `--source nci` require `NCI_API_KEY`.
If this key is missing, BioMCP returns an explicit `ApiKeyRequired` error.

```bash
export NCI_API_KEY="..."
biomcp search trial -c melanoma --source nci --limit 5
```

## 8) OncoKB enrichment appears limited

Without `ONCOKB_TOKEN`, BioMCP uses the OncoKB demo endpoint with reduced capability.
Set a production token for full enrichment behavior.

```bash
export ONCOKB_TOKEN="..."
biomcp get variant "BRAF V600E"
```

## 9) Date validation errors

Invalid dates are rejected before API calls. Use ISO format `YYYY-MM-DD` and valid calendar dates.

Examples that should fail immediately:

```bash
biomcp search article -g BRAF --since 2024-13-01 --limit 1
biomcp search article -g BRAF --since 2024-02-30 --limit 1
```

## 10) Install/update ownership conflicts

If `biomcp update` cannot replace the current binary (e.g. permission issues),
re-run the installer:

```bash
curl -fsSL https://raw.githubusercontent.com/genomoncology/biomcp/main/install.sh | bash
```

## 11) Local build missing protoc

The project includes gRPC clients and requires `protoc` to build from source.

- macOS: `brew install protobuf`
- Ubuntu/Debian: `apt-get install protobuf-compiler`

## 12) Still blocked

Capture one failing command with full stderr and include:

- BioMCP version (`biomcp version`)
- Command and flags
- Whether `--no-cache` changes behavior
- Source-specific API key state (set or unset)
