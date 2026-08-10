---
flow: build
priority: 9
---
# Run only the quality audit a test mutates

The Python lane passes, but fourteen negative tests each launch the complete
quality-ratchet wrapper. Every launch rescans the roughly 196,000-line Rust
tree and runs every unrelated audit. Two scans alone account for more than
thirty seconds per wrapper invocation, making this the largest deterministic
test-speed cost found in the review.

## Test contract

Give the Python quality checker a stable way to run one named audit. Mutation
tests call only the audit whose fixture they changed, preferably in process.
Keep exactly one end-to-end test that invokes the shipped shell wrapper and
proves it composes the complete audit set.

A full invocation discovers and parses tracked Rust files once and shares that
representation with audits that need it. Individual audits must not each walk
and reread the whole tree. The shell wrapper's normal command and failure
format remain compatible with CI and developers.

## Done when

- Every existing negative fixture still makes its intended audit fail and
  passes when the fixture is valid.
- An instrumented contract proves each mutation test invokes only its named
  audit.
- One wrapper integration proves all registered audits run in normal order and
  a failing audit still makes the wrapper nonzero.
- The two full-tree scans run once per full wrapper invocation, not once per
  negative fixture.
- No assertion, audit, or source-policy rule is deleted to obtain the speedup.

## Authorized test changes

Design commits may restate the wrapper-oriented mutation tests in
`tests/test_quality_ratchet_contract.py` and change the test-facing command
surface in `tools/check-quality-ratchet.py` and
`tools/check-quality-ratchet.sh`. Existing CI entry points and all individual
audit expectations stay covered.

The src line ceiling may not rise.
