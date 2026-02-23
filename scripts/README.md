# Source Contract Checks

This folder tracks API contract probes for the 091 expansion scope.
Each source has three probes:

- happy path: known-good request that should return useful data
- edge path: valid request expected to be empty or low-signal
- invalid path: intentionally bad request expected to fail clearly

These checks are intentionally lightweight and source-facing.
They are not a replacement for unit tests or VV docs.

## Files

- `source-contracts.md`: command inventory and expected outcomes
- `contract-smoke.sh`: optional runner for selected live probes
- `genegpt-demo.sh`: paper-style GeneGPT reproduction flow
- `geneagent-demo.sh`: paper-style GeneAgent reproduction flow

## Run

```bash
cd biomcp
./scripts/contract-smoke.sh
```

The script exits non-zero if one or more probes do not match expected HTTP behavior.
