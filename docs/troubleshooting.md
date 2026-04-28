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

If `--no-cache` works while cached mode fails repeatedly, `biomcp cache clean`
is the safe first step before any manual wipe. If you override the cache root
with `BIOMCP_CACHE_DIR` or `cache.toml`, run `biomcp cache path` first to print
the managed HTTP cache path under the resolved cache root.

```bash
biomcp cache clean
biomcp cache clean --dry-run
biomcp cache clean --max-age 7d
```

If targeted cleanup still does not recover the cache, use
`biomcp cache clear --yes` for a full wipe of the managed `http/` tree tied to
the resolved cache root you printed with `biomcp cache path`. `cache clear`
never touches the sibling `downloads/` directory, so the choice is explicit:
start with `cache clean` for targeted GC, escalate to `cache clear --yes` only
when you need the destructive full wipe.

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

## 4) OpenFDA FAERS / CDC WONDER VAERS / recall limits

OpenFDA-backed adverse-event, recall, and device searches are currently capped
by BioMCP at `--limit <= 50` per request. CDC WONDER VAERS is a separate
aggregate-only path: use plain vaccine query text, avoid FAERS-only filters on
that route, and use `biomcp health --apis-only` if the VAERS row looks
unavailable.

```bash
biomcp search adverse-event --drug pembrolizumab --limit 50
biomcp search adverse-event "MMR vaccine" --source vaers --limit 10
biomcp search adverse-event --type recall --classification "Class I" --limit 50
```

If you use `--source all` with unsupported VAERS filters such as `--reaction`,
`--outcome`, `--serious`, date filters, age filters, `--suspect-only`,
`--sex`, `--reporter`, or `--count`, BioMCP should keep the FAERS result and
record the skipped VAERS status in JSON instead of failing the whole search.

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

`biomcp update` fails closed when the release SHA256 checksum sidecar is missing
or invalid, before replacing the current binary. If a legitimate release is
missing its sidecar, `biomcp update --allow-missing-checksum` is an UNSAFE
per-invocation override.

If `biomcp update` cannot replace the current binary (e.g. permission issues),
re-run the installer:

```bash
curl -fsSL https://biomcp.org/install.sh | bash
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

## 13) EU drug data not available

EU drug regulatory, safety, and shortage sections depend on the local EMA
human-medicines JSON batch. BioMCP auto-downloads that data on first use for
`--region eu|all` drug workflows and for the default plain-name
U.S.+EU+WHO drug search, but full `biomcp health` is still the right
readiness view when you need to debug the local EMA state:

```bash
biomcp health
```

Interpret the EMA row like this:

- `configured`: `BIOMCP_EMA_DIR` is set and all required EMA JSON files are present
- `configured (stale)`: `BIOMCP_EMA_DIR` is set and the EMA batch is present, but older than the 72-hour refresh window
- `available (default path)`: BioMCP found a complete EMA batch in the default platform data directory
- `available (default path, stale)`: the default-path EMA batch is complete but older than the 72-hour refresh window
- `not configured`: no EMA batch was found at the default path, so EU drug features are currently unavailable but the install is not considered broken
- `error (missing: ...)`: BioMCP found a partial EMA batch; install the missing files or point `BIOMCP_EMA_DIR` at a complete batch

If a refresh fails, retry explicitly:

```bash
biomcp ema sync
```

If you need to override the default path:

```bash
export BIOMCP_EMA_DIR="/path/to/ema-human"
biomcp health
```

Manual preseed remains supported for offline or controlled environments. A
complete EMA root must contain:

- `medicines.json`
- `post_authorisation.json`
- `referrals.json`
- `psusas.json`
- `dhpcs.json`
- `shortages.json`

## 14) WHO drug data not available

WHO regional regulatory search/get flows depend on the local WHO
finished-pharmaceutical-products CSV, the active-pharmaceutical-ingredients
CSV, and the vaccine CSV. BioMCP auto-downloads all three exports on first use
for `--region who|all` drug workflows, explicit WHO vaccine search
(`--product-type vaccine`), and the default plain-name U.S.+EU+WHO drug search:

```bash
biomcp health
```

Interpret the WHO row like this:

- `configured`: `BIOMCP_WHO_DIR` is set and all three WHO exports are present
- `configured (stale)`: `BIOMCP_WHO_DIR` is set and complete, but at least one WHO export is older than the 72-hour refresh window
- `available (default path)`: BioMCP found all three WHO exports in the default platform data directory
- `available (default path, stale)`: the default-path WHO root is complete, but at least one export is older than the 72-hour refresh window
- `not configured`: no complete WHO root was found at the default path, so WHO regional drug features are currently unavailable but the install is not considered broken
- `error (missing: ...)`: BioMCP found a partial WHO root or unreadable file; install all three exports or point `BIOMCP_WHO_DIR` at a complete root

If a refresh fails, retry explicitly:

```bash
biomcp who sync
```

If you need to override the default path:

```bash
export BIOMCP_WHO_DIR="/path/to/who-pq"
biomcp health
```

Manual preseed remains supported for offline or controlled environments. A
complete WHO root must contain:

- `who_pq.csv`
- `who_api.csv`
- `who_vaccines.csv`

## 15) CDC CVX/MVX vaccine bridge data not available

Default plain-name vaccine searches that include the EU path and explicit
`search drug <brand> --region eu|all` vaccine searches can use the local CDC
CVX/MVX bundle after MyChem identity resolution misses. Explicit WHO vaccine
name/brand searches with `--region who --product-type vaccine` can use the same
bridge. Full `biomcp health` is the right readiness surface when you need to
debug the local CDC state:

```bash
biomcp health
```

Interpret the CDC row like this:

- `configured`: `BIOMCP_CVX_DIR` is set and all three CDC files are present
- `configured (stale)`: `BIOMCP_CVX_DIR` is set and complete, but at least one CDC file is older than the 30-day refresh window
- `available (default path)`: BioMCP found a complete CDC CVX/MVX bundle in the default platform data directory
- `available (default path, stale)`: the default-path CDC bundle is complete, but at least one file is older than the 30-day refresh window
- `not configured`: no complete CDC root was found at the default path, so the EMA/default/WHO vaccine-brand bridge is currently unavailable but the install is not considered broken
- `error (missing: ...)`: BioMCP found a partial CDC root or unreadable file; install the missing files or point `BIOMCP_CVX_DIR` at a complete bundle

If a refresh fails, retry explicitly:

```bash
biomcp cvx sync
```

If you need to override the default path:

```bash
export BIOMCP_CVX_DIR="/path/to/cvx"
biomcp health
```

Manual preseed remains supported for offline or controlled environments. A
complete CDC root must contain:

- `cvx.txt`
- `TRADENAME.txt`
- `mvx.txt`

The CDC bundle augments the EMA/default vaccine identity path plus explicit WHO
vaccine name/brand search when `--product-type vaccine` is set. It does not
change WHO finished-pharma/API lookups or `get drug`, and pure `--region us`
searches do not use the CVX root.

## 16) Diagnostic local data not available

Diagnostic search/get flows depend on two local bundles:

- the NCBI GTR bundle for gene-centric diagnostics
- the WHO IVD CSV for infectious-disease diagnostics

BioMCP auto-downloads both on first use, but full `biomcp health` is still the
right readiness surface when you need to debug the local diagnostic state:

```bash
biomcp health
```

Interpret the rows like this:

- `GTR local data (...)`: local GTR readiness for gene-capable diagnostic search/get
- `WHO IVD local data (...)`: local WHO infectious-disease diagnostic readiness
- `configured`: the corresponding env var is set and the required file set is present
- `configured (stale)`: the configured root is complete but older than the source-specific refresh window
- `available (default path)`: the default platform data directory contains a complete root
- `available (default path, stale)`: the default root is complete but stale
- `not configured`: no complete root exists yet at the default path
- `error (missing: ...)`: BioMCP found a partial root or unreadable file

If a refresh fails, retry explicitly:

```bash
biomcp gtr sync
biomcp who-ivd sync
```

If you need to override the default path:

```bash
export BIOMCP_GTR_DIR="/path/to/gtr"
export BIOMCP_WHO_IVD_DIR="/path/to/who-ivd"
biomcp health
```

Manual preseed remains supported for offline or controlled environments. A
complete GTR root must contain:

- `test_version.gz`
- `test_condition_gene.txt`

A complete WHO IVD root must contain:

- `who_ivd.csv`
