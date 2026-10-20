---
base: 0e96a4e3
head: a9dee294
---

Made the six named tests prove their claims, from the 2026-09-27
review file's Tests section.

The shared-runtime probe now drives `execute_mcp_cli` — the real
MCP entry — through `for_shared_call`, so the production wiring is
what the test exercises, and its own dedicated armed-window probe
counter cannot be moved by other tests' drives; the test asserts
both the probe (zero) and the production counter (unchanged). The
module comment states the honest determinism envelope: nextest (the
gate and CI runner) isolates per process; under plain `cargo test`
the armed window spans a full blocking-thread drive — a real, named
residual, milliseconds wide, not microseconds.

The depth-scanner test now contains real CDATA carrying
markup-looking text, with 65 opens hidden in each family (comment,
CDATA, PI) so counting any one family alone exceeds the cap; the
CDATA text must survive as the root's first text child (roxmltree's
`text()` returns only the first child — review caught the original
assertion could never pass).

The release-mode panic test runs in CI as its own job (release
build, nextest, the named rmcp test, no network, no conditions),
pinned by the offline-gate contract with the exact command, and
`make verify` keeps its copy. The health allowed-base test passes
in debug and release by satisfying the loopback+signal pair the
gate demands — no weakening — and both process-variable tests carry
the repo's serial key. The TLS test grew a third call that dials
the private-CA fixture: the connection counter increments while the
handshake fails, proving the warned client was constructed and
dialed but did not trust the fixture's roots — and the record now
says plainly that `biomcp version` builds no client. GenCC cleanup
gained the wrong-owner (mode-0000 stand-in, named as such) and
not-a-directory (ENOTDIR → Invalid) tests.

Evidence: code review BLOCK once — the CDATA first-text-child bug
(P0, caught by reading roxmltree's source) and two P2s (per-family
drift sensitivity raised to 65-per-family; the window wording) —
all fixed. Gate cycles caught rustfmt; one full-suite run hit the
known GenCC load flake (recorded in its watch issue with the
three-times-alone pass evidence); green at a9dee294 on the re-run —
lint, test, spec, stress, zero failed lines.
