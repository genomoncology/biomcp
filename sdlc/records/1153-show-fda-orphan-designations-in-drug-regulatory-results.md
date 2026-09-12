---
flow: build
priority: 6
deps: []
---

# Show FDA orphan designations in drug regulatory results

BioMCP drug regulatory results now include a bounded, U.S.-only FDA Orphan
Drug Designations and Approvals overlay. The source is attempted only for the
`regulatory` section in the `us` and `all` regions. It uses exact aliases from
anchored MyChem identities, bounded POST concurrency, a single total deadline,
strict HTML-table validation, and a private normalized-result cache that
honors normal, off, and infinite cache modes.

The public envelope distinguishes data, empty, degraded, and unavailable
outcomes. It keeps designation, approval, withdrawal, marketing, and
exclusivity facts separate, so the March 11, 2024 eflornithine hydrochloride
designation for Bachmann-Bupp syndrome is reported without being mislabeled as
an approved use. JSON, Markdown, raw MCP, typed MCP, provenance, health,
source inventory, configuration, and operator documentation share the same
source semantics. The package remains self-contained BioMCP 0.9 code with no
BioData or BioMCP 1.0 dependency.

## Evidence

A fresh SOL design review rejected four ambiguities in the draft: MyChem-hit
eligibility, exact cache-mode behavior, fixture provenance, and proof layering.
The ticket was amended to specify anchored identity, normal/off/infinite cache
semantics, an honest provider-shaped fixture, and exact layered acceptance.
The same reviewer then accepted the Level 3 design.

The initial SOL implementation was independently rejected because it bypassed
managed cache finalization, could detach blocking lock work after a deadline,
validated malformed nonmatching rows, materialized more than the 500-row wire
limit, and lacked material negative/cache/cancellation/health proof. After
remediation, review found two history/health defects: a form-key cache stored a
candidate-filtered subset, and the health probe accepted malformed nonempty
rows. The implementation now caches fully normalized candidate-independent
rows and filters only after reads, while health validates every nonempty row.
The same independent reviewer accepted those corrections with no remaining
material findings.

Root full testing then found a ticket-introduced stack overflow in an existing
raw-MCP trial test at the fixed 8 MiB command-worker stack. The exact test
passed on the base revision. Heap-backing the optional orphan envelope reduced
`Drug`'s inline stack footprint without changing Serde output, rendering,
provenance, cache, or error behavior; no stack limit was raised. The exact test
then passed on the branch, and independent SOL review accepted that focused
fix. A subsequent complete gate exposed three fail-closed documentation and
transport-inventory omissions. The source URL, test-only base override, and
both HTTP-client constructors are now classified explicitly, and a final
independent review accepted those changes without weakening any scanner.

On the exact accepted revision, `make lint`, `make test`, `make spec`, and
`make full-feature-check` passed under Node 22. The shared host used eight
nextest workers: all 3,438 Rust tests passed; 955 Python contracts passed with
three intentional skips; strict documentation and every offline executable
and static specification group passed. All-feature Clippy, six AlphaGenome
behavior tests, the optimized all-feature build, and PNG/SVG/terminal artifact
smoke passed. `cargo package --list --allow-dirty --locked --offline
--no-verify` reports exactly 1,300 paths, and `git diff --check` passes.
