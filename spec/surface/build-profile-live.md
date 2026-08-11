# Feature-profile live confirmation

Release artifacts include the AlphaGenome client, while the routine profile
intentionally omits it. This opt-in check uses real upstream health probes to
confirm that the smaller binary remains honest about the capability it did not
build.

## A no-feature health report names AlphaGenome as not built

The no-default-feature binary still lists AlphaGenome, but it must report that
it was not built rather than reading a key or attempting the gRPC connection.

```bash
"${BIOMCP_SPEC_FEATURE_OFF_BIN:?spec preparation did not export feature-off CLI}" --json health --apis-only | jq '[.rows[] | select(.api == "AlphaGenome")][0] | (.status == "unavailable" and .not_built == true and .latency == "-" and (has("key_configured") | not))' | mustmatch 'true'
```

## The default release binary still connects with a configured key

With an operator-provided AlphaGenome key, the ordinary release binary retains
the real gRPC health probe. This is deliberately live verification, not a
routine credential simulation.

```bash
"${BIOMCP_SPEC_FEATURE_ON_BIN:?spec preparation did not export feature-on CLI}" --json health --apis-only | jq '[.rows[] | select(.api == "AlphaGenome")][0] | (.status == "ok" and .key_configured == true)' | mustmatch 'true'
```
