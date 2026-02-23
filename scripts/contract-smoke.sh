#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/contract-smoke.sh [--fast]

Options:
  --fast   Run one representative probe per source family
  -h       Show this help
USAGE
}

FAST=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --fast)
      FAST=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

CURL_BASE=(curl -sS -L --max-time 40)
PASS=0
FAIL=0

probe_get() {
  local name="$1"
  local code_re="$2"
  local body_re="$3"
  local url="$4"

  local tmp
  tmp=$(mktemp)
  local code
  code=$("${CURL_BASE[@]}" -o "$tmp" -w "%{http_code}" "$url" || true)
  local body
  body=$(cat "$tmp")
  rm -f "$tmp"

  local ok=1
  if [[ ! "$code" =~ $code_re ]]; then
    ok=0
  fi
  if [[ -n "$body_re" ]] && ! printf '%s' "$body" | grep -qE "$body_re"; then
    ok=0
  fi

  if [[ $ok -eq 1 ]]; then
    echo "[PASS] $name (code=$code)"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $name (code=$code)"
    echo "        url: $url"
    FAIL=$((FAIL + 1))
  fi
}

probe_post_json() {
  local name="$1"
  local code_re="$2"
  local body_re="$3"
  local url="$4"
  local payload="$5"

  local tmp
  tmp=$(mktemp)
  local code
  code=$("${CURL_BASE[@]}" -H "content-type: application/json" -X POST -d "$payload" -o "$tmp" -w "%{http_code}" "$url" || true)
  local body
  body=$(cat "$tmp")
  rm -f "$tmp"

  local ok=1
  if [[ ! "$code" =~ $code_re ]]; then
    ok=0
  fi
  if [[ -n "$body_re" ]] && ! printf '%s' "$body" | grep -qE "$body_re"; then
    ok=0
  fi

  if [[ $ok -eq 1 ]]; then
    echo "[PASS] $name (code=$code)"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $name (code=$code)"
    echo "        url: $url"
    FAIL=$((FAIL + 1))
  fi
}

echo "== contract smoke checks ($( [[ $FAST -eq 1 ]] && echo fast || echo full )) =="

# UniProt
if [[ $FAST -eq 1 ]]; then
  probe_get "UniProt fast" '^200$' 'P15056|primaryAccession' "https://rest.uniprot.org/uniprotkb/P15056.json"
else
  probe_get "UniProt happy" '^200$' 'P15056|primaryAccession' "https://rest.uniprot.org/uniprotkb/P15056.json"
  probe_get "UniProt edge" '^(200|404|400)$' '' "https://rest.uniprot.org/uniprotkb/P99999.json"
  probe_get "UniProt invalid endpoint" '^(400|404)$' '' "https://rest.uniprot.org/uniprotkb/P15056.not-real"
fi

# QuickGO
if [[ $FAST -eq 1 ]]; then
  probe_get "QuickGO fast" '^200$' 'results' "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=5"
else
  probe_get "QuickGO happy" '^200$' 'results' "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=5"
  probe_get "QuickGO edge" '^200$' 'results' "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P99999&limit=5"
  probe_get "QuickGO invalid" '^(400|422|500)$' '' "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=-1"
fi

# STRING
if [[ $FAST -eq 1 ]]; then
  probe_get "STRING fast" '^200$' '\[' "https://string-db.org/api/json/network?identifiers=BRAF&species=9606&limit=5"
else
  probe_get "STRING happy" '^200$' '\[' "https://string-db.org/api/json/network?identifiers=BRAF&species=9606&limit=5"
  probe_get "STRING edge" '^(200|404)$' '' "https://string-db.org/api/json/network?identifiers=NO_SUCH_GENE_091&species=9606&limit=5"
  probe_get "STRING invalid endpoint" '^404$' '' "https://string-db.org/api/json/not-a-real-endpoint?identifiers=BRAF&species=9606"
fi

# gnomAD GraphQL
if [[ $FAST -eq 1 ]]; then
  probe_post_json "gnomAD fast" '^200$' '__typename' "https://gnomad.broadinstitute.org/api" '{"query":"query { __typename }"}'
else
  probe_post_json "gnomAD happy" '^200$' '__typename' "https://gnomad.broadinstitute.org/api" '{"query":"query { __typename }"}'
  probe_post_json "gnomAD edge" '^200$' 'variant' "https://gnomad.broadinstitute.org/api" '{"query":"query { variant(variantId: \"7-1-A-T\", dataset: gnomad_r4) { variantId } }"}'
  probe_post_json "gnomAD invalid query" '^400$' 'errors' "https://gnomad.broadinstitute.org/api" '{"query":"query { variant(variantId: \"bad\") {"}'
fi

# ChEMBL
if [[ $FAST -eq 1 ]]; then
  probe_get "ChEMBL fast" '^200$' 'CHEMBL25|molecule_chembl_id' "https://www.ebi.ac.uk/chembl/api/data/molecule/CHEMBL25.json"
else
  probe_get "ChEMBL happy" '^200$' 'CHEMBL25|molecule_chembl_id' "https://www.ebi.ac.uk/chembl/api/data/molecule/CHEMBL25.json"
  probe_get "ChEMBL edge" '^200$' 'molecules|page_meta' "https://www.ebi.ac.uk/chembl/api/data/molecule/search.json?q=NOT_A_REAL_DRUG_091"
  probe_get "ChEMBL invalid endpoint" '^404$' '' "https://www.ebi.ac.uk/chembl/api/data/not-a-real-resource.json"
fi

# OpenTargets GraphQL
if [[ $FAST -eq 1 ]]; then
  probe_post_json "OpenTargets fast" '^200$' 'data|CHEMBL25' "https://api.platform.opentargets.org/api/v4/graphql" '{"query":"query { drug(chemblId: \"CHEMBL25\") { id name } }"}'
else
  probe_post_json "OpenTargets happy (chemblId)" '^200$' 'data|CHEMBL25' "https://api.platform.opentargets.org/api/v4/graphql" '{"query":"query { drug(chemblId: \"CHEMBL25\") { id name } }"}'
  probe_post_json "OpenTargets edge" '^200$' 'drug' "https://api.platform.opentargets.org/api/v4/graphql" '{"query":"query { drug(chemblId: \"CHEMBL_DOES_NOT_EXIST\") { id name } }"}'
  probe_post_json "OpenTargets invalid argument key" '^400$' 'errors' "https://api.platform.opentargets.org/api/v4/graphql" '{"query":"query { drug(efoId: \"EFO_0000311\") { id name } }"}'
fi

# Reactome
if [[ $FAST -eq 1 ]]; then
  probe_get "Reactome fast" '^200$' 'results' "https://reactome.org/ContentService/search/query?query=MAPK&species=Homo%20sapiens&pageSize=1"
else
  probe_get "Reactome happy" '^200$' 'results' "https://reactome.org/ContentService/search/query?query=MAPK&species=Homo%20sapiens&pageSize=1"
  probe_get "Reactome edge" '^(200|404)$' '' "https://reactome.org/ContentService/search/query?query=NO_SUCH_PATHWAY_091&species=Homo%20sapiens&pageSize=1"
  probe_get "Reactome invalid" '^404$' '' "https://reactome.org/ContentService/data/query/NOT_A_REAL_STABLE_ID"
fi

# g:Profiler
if [[ $FAST -eq 1 ]]; then
  probe_post_json "g:Profiler fast" '^200$' 'result' "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" '{"organism":"hsapiens","query":["BRAF","KRAS"]}'
else
  probe_post_json "g:Profiler happy" '^200$' 'result' "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" '{"organism":"hsapiens","query":["BRAF","KRAS"]}'
  probe_post_json "g:Profiler edge" '^200$' 'result' "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" '{"organism":"hsapiens","query":["NO_SUCH_GENE_091"]}'
  probe_post_json "g:Profiler invalid" '^(400|422)$' '' "https://biit.cs.ut.ee/gprofiler/api/gost/profile/" '{"query":"not-an-array"}'
fi

# InterPro
if [[ $FAST -eq 1 ]]; then
  probe_get "InterPro fast" '^200$' 'results|entries|metadata' "https://www.ebi.ac.uk/interpro/api/protein/uniprot/P15056/entry/interpro/?page_size=5"
else
  probe_get "InterPro happy" '^200$' 'results|entries|metadata' "https://www.ebi.ac.uk/interpro/api/protein/uniprot/P15056/entry/interpro/?page_size=5"
  probe_get "InterPro edge" '^(200|404)$' '' "https://www.ebi.ac.uk/interpro/api/protein/uniprot/P99999/entry/interpro/?page_size=5"
  probe_get "InterPro invalid endpoint" '^404$' '' "https://www.ebi.ac.uk/interpro/api/protein/uniprot/P15056/not-a-real-resource/"
fi

# Existing variant-support sources
if [[ $FAST -eq 1 ]]; then
  probe_get "ClinicalTrials.gov fast" '^200$' 'studies' "https://clinicaltrials.gov/api/v2/studies?query.term=BRAF%20V600E&pageSize=1"
else
  probe_get "ClinicalTrials.gov happy" '^200$' 'studies' "https://clinicaltrials.gov/api/v2/studies?query.term=BRAF%20V600E&pageSize=1"
  probe_get "ClinicalTrials.gov edge" '^200$' 'studies' "https://clinicaltrials.gov/api/v2/studies?query.term=NO_SUCH_TERM_091&pageSize=1"
  probe_get "ClinicalTrials.gov invalid" '^(400|422)$' '' "https://clinicaltrials.gov/api/v2/studies?query.term=melanoma&pageSize=bad"
fi

if [[ $FAST -eq 1 ]]; then
  probe_get "cBioPortal fast" '^200$' '\[' "https://www.cbioportal.org/api/studies?projection=SUMMARY&pageSize=1"
else
  probe_get "cBioPortal happy" '^200$' '\[' "https://www.cbioportal.org/api/studies?projection=SUMMARY&pageSize=1"
  probe_get "cBioPortal edge" '^(200|404)$' '' "https://www.cbioportal.org/api/molecular-profiles?studyId=NO_SUCH_STUDY"
  probe_get "cBioPortal invalid endpoint" '^404$' '' "https://www.cbioportal.org/api/not-a-real-resource"
fi

# OncoKB demo endpoint (no token required)
if [[ $FAST -eq 1 ]]; then
  probe_get "OncoKB demo fast" '^200$' '"hugoSymbol":"BRAF".*"oncogenic":"Oncogenic"' "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E"
else
  probe_get "OncoKB demo BRAF V600E" '^200$' '"hugoSymbol":"BRAF".*"oncogenic":"Oncogenic"' "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E"
  probe_get "OncoKB demo EGFR L858R" '^200$' '"hugoSymbol":"EGFR".*"geneExist":false' "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=EGFR&alteration=L858R"
  probe_get "OncoKB demo TP53 R175H" '^200$' '"hugoSymbol":"TP53".*"oncogenic":"Oncogenic"' "https://demo.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=TP53&alteration=R175H"
fi

if [[ -n "${ONCOKB_API_TOKEN:-}" ]]; then
  local_tmp=$(mktemp)
  code=$(
    "${CURL_BASE[@]}" -H "Authorization: Bearer $ONCOKB_API_TOKEN" \
      -o "$local_tmp" -w "%{http_code}" \
      "https://www.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E" || true
  )
  body=$(cat "$local_tmp")
  rm -f "$local_tmp"
  if [[ "$code" =~ ^200$ ]] && printf '%s' "$body" | grep -qE 'geneExist|variantExist|oncogenic'; then
    echo "[PASS] OncoKB happy (token)"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] OncoKB happy (token) code=$code"
    FAIL=$((FAIL + 1))
  fi
else
  echo "[SKIP] OncoKB probes (set ONCOKB_API_TOKEN to enable)"
fi

echo
echo "Summary: pass=$PASS fail=$FAIL"

if [[ $FAIL -ne 0 ]]; then
  exit 1
fi
