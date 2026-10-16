# Trial Numeric Filters

## Zero distance is rejected before ClinicalTrials.gov work

A zero-mile radius is invalid input, not a ClinicalTrials.gov failure. The CLI
must return the local JSON error, exit `2`, and make no request to the replay
server's ClinicalTrials.gov route.

<!-- mustmatch-lint: skip -->

```bash run id=zero-trial-distance exit=2
output="$(mktemp)"
trap 'rm -f "$output"' EXIT
log="$BIOMCP_REPLAY_REQUEST_LOG"
before_ctgov_requests="$(grep -Fc ' /ctgov/' "$log" || true)"
set +e
../../tools/biomcp-ci --json search trial -c melanoma --lat=1 --lon=2 --distance=0 >"$output"
status=$?
set -e
test "$status" -eq 2
test "$(( $(grep -Fc ' /ctgov/' "$log" || true) - before_ctgov_requests ))" -eq 0
cat "$output"
exit "$status"
```

```json expect=zero-trial-distance contains
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```text expect=zero-trial-distance contains
--distance
```
