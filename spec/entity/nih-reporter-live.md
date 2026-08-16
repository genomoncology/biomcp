# Live NIH Reporter Funding Check

This operator-run page makes one bounded request to the public NIH Reporter service. It does not start or consume routine provider fixtures. A successful response must retain the requested query, a bounded fiscal-year window, a positive project-year count, and at least one fully shaped grant.

## Funding Results Stay Non-Empty and Structured

<!-- mustmatch-lint: skip -->

```bash run id=nih-reporter-funding-live exit=0 timeout=180
set +e
funding_payload="$(../../tools/biomcp-ci --json get disease "Marfan syndrome" funding)"
funding_status=$?
set -e
if ((funding_status != 0)); then
    exit "$funding_status"
fi
printf "%s" "$funding_payload" | python3 -c '
import json
import os
import sys

try:
    payload = json.load(sys.stdin)
except (json.JSONDecodeError, UnicodeDecodeError) as error:
    raise SystemExit(f"Malformed NIH Reporter funding JSON: {error}")

funding = payload.get("funding")
if funding is None and payload.get("funding_note") == "NIH Reporter funding data is temporarily unavailable.":
    sentinel = os.environ.get("BIOMCP_VERIFY_LIVE_PENDING_SENTINEL")
    if not sentinel:
        raise SystemExit("NIH Reporter live pending sentinel is unavailable")
    try:
        with open(sentinel, "x", encoding="utf-8") as handle:
            handle.write("biomcp-nih-reporter-unavailable-v1\n")
            handle.flush()
            os.fsync(handle.fileno())
    except OSError as error:
        raise SystemExit(f"NIH Reporter live pending sentinel could not be written: {error}")
    print("NIH Reporter funding data is temporarily unavailable.", file=sys.stderr)
    raise SystemExit(1)
if not isinstance(funding, dict):
    raise SystemExit("NIH Reporter response is missing the required funding object")

query = funding.get("query")
fiscal_years = funding.get("fiscal_years")
matching = funding.get("matching_project_years")
grants = funding.get("grants")
if not isinstance(query, str) or not query.strip():
    raise SystemExit("NIH Reporter funding query is missing")
if not isinstance(fiscal_years, list) or not fiscal_years or len(fiscal_years) > 5 or not all(isinstance(year, int) for year in fiscal_years):
    raise SystemExit("NIH Reporter fiscal_years are missing or malformed")
if not isinstance(matching, int) or isinstance(matching, bool) or matching < 1:
    raise SystemExit("NIH Reporter matching_project_years is missing or empty")
if not isinstance(grants, list) or not grants:
    raise SystemExit("NIH Reporter grants are missing or empty")
required = {"project_title", "project_num", "fiscal_year", "award_amount"}
if not all(isinstance(grant, dict) and required <= set(grant) for grant in grants):
    raise SystemExit("NIH Reporter grant rows are malformed")
print(json.dumps({"nih_reporter_funding_live": True, "query": query, "grant_count": len(grants)}, sort_keys=True))
'
```

```json expect=nih-reporter-funding-live contains
{
  "nih_reporter_funding_live": true,
  "query": "Marfan syndrome"
}
```
