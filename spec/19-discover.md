# Discover

`discover` is the free-text entrypoint for concept resolution before the user
knows which typed BioMCP command to run. These checks validate the approved
examples against stable structural markers and suggestion contracts.

| Section | Command focus | Why it matters |
|---|---|---|
| Gene Alias | `discover ERBB1` | Confirms alias resolution and gene suggestion |
| Drug Brand Name | `discover Keytruda` | Confirms brand-name normalization to generic drug |
| Symptom Query | `discover "chest pain"` | Confirms symptom-safe suggestions and MedlinePlus overlay |
| Ambiguous Query | `discover diabetes` | Confirms ambiguity guidance is explicit |
| Pathway Query | `discover "MAPK signaling"` | Confirms pathway-oriented suggestion generation |
| Underspecified Variant | `discover V600E` | Confirms the command avoids false gene certainty |
| OLS4-only Mode | `env -u UMLS_API_KEY discover BRCA1` | Confirms truthful degradation without UMLS |
| JSON Metadata | `--json discover Keytruda` | Confirms discover-specific `_meta` contract |
| UMLS Crosswalks | `--json discover "cystic fibrosis"` | Confirms optional clinical crosswalk enrichment |

## Gene Alias

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" discover ERBB1)"
echo "$out" | mustmatch like "# Discover: ERBB1"
echo "$out" | mustmatch like "EGFR"
echo "$out" | mustmatch like "biomcp get gene EGFR"
```

## Drug Brand Name

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" discover Keytruda)"
echo "$out" | mustmatch like "# Discover: Keytruda"
echo "$out" | mustmatch like "pembrolizumab"
echo "$out" | mustmatch like "biomcp get drug \"pembrolizumab\""
```

## Symptom Query

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" discover "chest pain")"
echo "$out" | mustmatch like "## Plain Language"
echo "$out" | mustmatch like "MedlinePlus"
echo "$out" | mustmatch like "biomcp search disease -q \"chest pain\" --limit 10"
echo "$out" | mustmatch like "biomcp search trial -c \"chest pain\" --limit 5"
echo "$out" | mustmatch like "biomcp search article -k \"chest pain\" --limit 5"
```

## Ambiguous Query

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" discover diabetes)"
echo "$out" | mustmatch like "## Concepts"
echo "$out" | mustmatch like "1."
echo "$out" | mustmatch like "Type 1"
echo "$out" | mustmatch like "Type 2"
```

## Pathway Query

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" discover "MAPK signaling")"
echo "$out" | mustmatch like "Pathway"
echo "$out" | mustmatch like "biomcp search pathway -q \"MAPK signaling\" --limit 5"
```

## Underspecified Variant

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" discover V600E)"
echo "$out" | mustmatch like "Variant"
echo "$out" | mustmatch not like "biomcp get gene "
echo "$out" | mustmatch not like "## Plain Language"
```

## OLS4-only Mode

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$(env -u UMLS_API_KEY "$bin" discover BRCA1)"
echo "$out" | mustmatch like "BRCA1"
echo "$out" | mustmatch like "UMLS enrichment unavailable"
```

## JSON Metadata

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" --json discover Keytruda)"
echo "$out" | mustmatch like '"concepts": ['
echo "$out" | mustmatch like '"next_commands": ['
echo "$out" | mustmatch like '"section_sources": ['
echo "$out" | mustmatch like '"discovery_sources": ['
echo "$out" | mustmatch like '"evidence_urls": ['
```

## UMLS Crosswalks

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" --json discover "cystic fibrosis")"
echo "$out" | grep -Eq '"(ICD10CM|SNOMEDCT|RXNORM)"'
```
