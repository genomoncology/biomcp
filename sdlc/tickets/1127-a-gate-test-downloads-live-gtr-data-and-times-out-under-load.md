---
flow: quickfix
priority: 10
---

# The regulatory section overruns its own eight-second guard and stops the channel

`biomcp get diagnostic <accession> regulatory` takes about fifteen seconds against an endpoint that refuses connections instantly. Its declared guard is eight seconds. Every other section of the same command returns in thirty milliseconds.

That overrun is what makes `tests/test_public_example_accessions.py::test_public_gtr_examples_resolve_against_live_gtr_bundle` fail in the factory's `before` stage. The test caps each command at sixty seconds and loops over more than six of them. Fifteen seconds on an idle machine, under a factory running four channels of Rust builds at once, crosses sixty.

When it crosses, `make test` returns non-zero, `before` hands the ticket back as ready without consuming an attempt, and the ticket is claimed again. Ticket 1125 was claimed four times and spent about seventy-six minutes in `before` before one attempt got through. The board showed a ticket that kept looking ready and nothing else said why.

## Reproduction

Prepare the bundle the way the test prepares it, then run the section against a dead endpoint:

```
D=$(mktemp -d)
python3 - "$D" <<'PY'
import gzip, shutil, sys
from pathlib import Path
t = Path(sys.argv[1]) / "gtr"
shutil.copytree("spec/fixtures/gtr", t)
p = t / "test_condition_gene.txt"
p.write_text(p.read_text(encoding="utf-8").replace("GTR000000001.1", "GTR000006692.3"), encoding="utf-8")
v = t / "test_version.gz"
with gzip.open(v, "rt", encoding="utf-8") as fh: payload = fh.read()
with gzip.open(v, "wt", encoding="utf-8") as fh: fh.write(payload.replace("GTR000000001.1", "GTR000006692.3"))
PY

time env BIOMCP_GTR_DIR="$D/gtr" BIOMCP_OPENFDA_BASE=http://127.0.0.1:9 \
  ./target/debug/biomcp get diagnostic GTR000006692.3 regulatory
```

Measured on `0.9.0-dev.6` on 2026-09-02, three consecutive runs: 14.34s, 15.82s, 14.96s. Port 9 refuses immediately, so none of that time is a slow network. The same bundle with `regulatory` replaced by `all`, `genes`, `conditions` or `methods` returns in 0.03s.

## Cause

`src/entities/diagnostic/get.rs:21` declares the bound:

```rust
const OPTIONAL_REGULATORY_TIMEOUT: Duration = Duration::from_secs(8);
```

and `:556` applies it:

```rust
match tokio::time::timeout(OPTIONAL_REGULATORY_TIMEOUT, fetch_fda_regulatory(ctx)).await {
```

The measured wall time is roughly twice that bound, so work is happening outside what the timeout wraps. The OpenFDA client retries transient failures, and a connection refused is transient, so the retry ladder and its backoff are the first place to look. The environment override at `src/sources/openfda.rs:25` is honoured, so the test's stub does take effect and the overrun is not a live call sneaking through.

The section is optional by design. `:568` already has the language for its absence: `OpenFDA diagnostic regulatory data is temporarily unavailable.` An optional section that costs fifteen seconds to decline is not optional in any way a caller can feel.

## Done, observably

- The reproduction above completes within the declared eight-second guard. A test pins the bound rather than a wall-clock figure, so a future retry change cannot quietly reintroduce it.
- The section still degrades rather than failing. An unreachable OpenFDA still produces the `temporarily unavailable` line and a zero exit, and a test pins that.
- A reachable OpenFDA still returns real regulatory data. A test pins that the fast path is unchanged.
- `sh sdlc/scripts/test` passes.

## Boundary

Do not delete or skip `test_public_example_accessions.py`. It covers the public accessions named in the tool's own help, and that coverage is why it exists.

Do not raise the sixty-second cap in that test. The command should fit the bound it already declares.

Do not change what a successful `regulatory` section returns, and do not change the other four sections.

Whether the fix is to move the timeout so it wraps the retry ladder, to stop retrying a refused connection, or to bound the ladder itself, is a design choice. Any of them satisfies this ticket.

## Correction to this ticket's first draft

The first version of this ticket blamed a stale GTR bundle triggering a network refresh. That was wrong. A bundle prepared the way the test prepares it is honoured, and the command succeeds with the network unreachable. The `Refreshing stale GTR data...` line seen while investigating came from the default cache directory, not from the test's path. The timing evidence above replaces that account.
