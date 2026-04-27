# API Keys

Most BioMCP workflows run without credentials.
Some enrichment and higher-rate access paths improve with API keys. For provider terms, redistribution caveats, and which sources are only indirect provenance, see the [source-licensing reference](../reference/source-licensing.md).

## Required for prediction section

### `ALPHAGENOME_API_KEY`

Used by variant prediction lookups:

Provider access: <https://deepmind.google/science/alphagenome/>

```bash
export ALPHAGENOME_API_KEY="..."
biomcp get variant "chr7:g.140453136A>T" predict
```

## Additional API keys

### `ONCOKB_TOKEN`

Used for the production `biomcp variant oncokb ...` helper. Without the
token, that helper is unavailable and reports the required environment
variable.
BioMCP keeps `ONCOKB_TOKEN` because OncoKB itself calls the credential a
token.

Register at: <https://www.oncokb.org/account/register>

```bash
export ONCOKB_TOKEN="..."
biomcp variant oncokb "BRAF V600E"
```

### `NCI_API_KEY`

Used for NCI CTS trial calls.

Request access at: <https://clinicaltrialsapi.cancer.gov/>

```bash
export NCI_API_KEY="..."
biomcp search trial -c melanoma --source nci
```

### `DISGENET_API_KEY`

Required for DisGeNET scored association sections on genes and diseases.
Without the key, those DisGeNET sections are unavailable and report the
required environment variable.

Register at: <https://www.disgenet.com/>

```bash
export DISGENET_API_KEY="..."
biomcp get gene TP53 disgenet
biomcp get disease "breast cancer" disgenet
```

### `UMLS_API_KEY`

Adds optional clinical crosswalk enrichment to `biomcp discover`. Without the
key, `discover` still runs with OLS4-only results and omits UMLS crosswalks.

Register at: <https://uts.nlm.nih.gov/uts/signup-login>

```bash
export UMLS_API_KEY="..."
biomcp discover "cystic fibrosis"
biomcp --json discover diabetes
```

### `NCBI_API_KEY`

Improves rate limits for PubTator, PubMed/efetch, PMC OA, and NCBI ID converter (3 → 10 req/sec).

Create one in My NCBI: <https://www.ncbi.nlm.nih.gov/account/settings/>

```bash
export NCBI_API_KEY="..."
biomcp search article -g BRAF --limit 5
```

### `S2_API_KEY`

Enables authenticated Semantic Scholar article requests. Use it for a dedicated
provider quota at 1 req/sec across the optional article search leg, `get article`
enrichment, `get article ... tldr`, `article citations`, `article references`,
and `article recommendations`. Without the key, those paths still work through
the shared unauthenticated pool at 1 req/2sec.

Request a key at: <https://www.semanticscholar.org/product/api>

```bash
export S2_API_KEY="..."
biomcp get article 22663011 tldr
biomcp article citations 22663011 --limit 3
```

### `OPENFDA_API_KEY`

Improves OpenFDA rate limits for drug safety lookups.

Request a key at: <https://open.fda.gov/apis/authentication/>

```bash
export OPENFDA_API_KEY="..."
biomcp search adverse-event --drug pembrolizumab --limit 5
```

## Key management guidance

- Prefer environment variables over hardcoded values.
- Do not commit secrets into source control.
- Set keys in the same environment used by your MCP client.
- Rotate keys when sharing machines or CI runners.
- `S2_API_KEY` is optional. Without it, article search/get/helper paths still work through the shared Semantic Scholar pool at 1 req/2sec. With it, BioMCP sends `x-api-key` and uses the authenticated quota at 1 req/sec for the same article search/get/helper paths.
- `UMLS_API_KEY` is optional; when absent, `discover` still works with OLS4-only results.

See also: [Source Licensing and Terms](../reference/source-licensing.md)
