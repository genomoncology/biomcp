---
base: 9259bc2c4ab8379602f666322ebb82dbe4a9692b
head: 65b9bd497ada449c363ec8f44cbfc475cade02c4
---

`scripts/run-specs.sh` is now the sole complete registry for routine, static,
and live specification pages. Make targets delegate to that registry instead
of maintaining duplicate path arrays, and verification consumes the runner's
exported live list while keeping the NIH Reporter lane explicit.

Architecture contracts prove the ownership boundary and isolated runner tests
remain self-contained when optional provider fixture scripts are absent. The
complete specification gate passed with 92 routine scenarios, all page-level
contracts, 32 isolation contracts, and 9 static scenarios.
