---
flow: quickfix
priority: 6
---

# Replace the yanked `chacha20` dependency in the shipped graph

`Cargo.lock` resolves `chacha20` 0.10.0, which crates.io marks as yanked.
The crate was published on 2026-02-07; crates.io's version record shows that
0.10.0 and 0.10.1 were both yanked on 2026-08-27, after 0.10.2 was published
that day. A yank prevents new dependency resolution to those versions but does
not delete their published artifacts.

The routine and all-feature graphs both show the same production path:

```
chacha20 v0.10.0        <- yanked
└── rand v0.10.1
    └── rmcp v1.7.0
        ├── biomcp-cli
        └── biomcp-mcp-contract-client (development contract client)
```

The root crate enables rmcp's Streamable HTTP server support. That feature
enables rmcp's optional `rand` dependency with rand's defaults; rand's
`std_rng` feature then enables `chacha20`. The development contract client also
uses rmcp, but it is not the reason the crate is in the shipped binary.

`cargo deny check advisories` currently succeeds but prints
`warning[yanked]: detected yanked crate`, because the repository's advisory
policy reports yanks as warnings. It reports no RustSec advisory for this
dependency. Upstream's 0.10.2 changelog says that release fixes an SSE4.1
intrinsic used by the SSE2 backend for RNG and legacy variants. Neither the
changelog nor the crates.io version record explicitly says why both earlier
versions were yanked, so this ticket does not infer a security advisory or a
specific yank reason.

## Design

Update only the locked `chacha20` package from 0.10.0 to the exact supported
patch release 0.10.2:

```
cargo update -p chacha20@0.10.0 --precise 0.10.2
```

A dry run on 2026-09-04 resolves only that `chacha20` update. Both versions
require Rust 1.85 and expose the same dependency requirements used by the
locked graph, so no manifest, feature, source, documentation, or
public-contract change is expected.

## Reproduction and acceptance

Before the change, this prints the warning and finds it:

```
cargo deny check advisories 2>&1 | rg 'warning\[yanked\]'
```

Done, observably:

- `Cargo.lock` changes only the `chacha20` package version and checksum, from
  0.10.0 to 0.10.2; `rmcp`, `rand`, and every manifest remain unchanged.
- `cargo tree --locked --no-default-features --edges normal,build -i chacha20`
  and `cargo tree --locked --all-features --edges normal,build -i chacha20`
  both resolve 0.10.2 through the same `rand -> rmcp -> biomcp-cli` production
  path. Excluding development edges ensures the contract client cannot mask
  loss of the dependency from the shipped graph.
- `cargo deny check advisories` succeeds and its output contains no
  `warning[yanked]` or other yanked-crate report.
- `make lint`, `make test`, `make spec`, and `git diff --check` pass against the
  updated locked graph.

No new source test is warranted for a lockfile-only selection. The tree and
advisory assertions are the focused proof; the repository gates cover compile
and behavior.

## Boundary

Change `Cargo.lock` only. Do not change `Cargo.toml`, the contract-client
manifest, `deny.toml`, source, tests, documentation, or public metadata. Do not
upgrade `rmcp`, `rand`, or any unrelated package. If the exact update cannot
remain lockfile-only, stop and return the ticket to design rather than widening
this quick fix.

This remains a quick fix: it is a deterministic, single-package patch update
inside an already-compatible transitive requirement, with no product-code or
contract design. Moving this ticket to its record on completion is the only
roadmap bookkeeping; no live ticket depends on 1106.

## Provenance

Filed as a flowless draft by a verify stage, which is the correct behavior for
an out-of-scope problem found while exercising. It then sat unclaimed, because
a draft with no flow and no priority is never claimed and nothing reports it.
Promoted 2026-09-03 after the dependency was re-verified as still yanked and a
supported successor was confirmed to exist. Dates and yank state were
corrected from the crates.io version records on 2026-09-04.

The first design review rejected graph checks that included development edges;
the acceptance commands now prove the production path directly.

## Completed 2026-09-04

Updated only the locked `chacha20` package from 0.10.0 to 0.10.2. The lockfile
diff contains exactly the package version and checksum change; manifests,
`rmcp`, `rand`, source, tests, documentation, and policy configuration remain
unchanged. Both production-only inverse trees retain the shipped
`chacha20 -> rand -> rmcp -> biomcp-cli` path, and `cargo deny check advisories`
now reports `advisories ok` with no yanked-crate warning.

Independent design review accepted the corrected production-edge proof after
rejecting the original dev-edge-masked assertion. A distinct implementer made
the exact lock update, and a fresh code reviewer accepted the actual diff with
no findings after independently checking the crate archive checksum, packaged
manifest, dependency paths, advisory result, and diff hygiene.

Final gates passed on the reviewed tree: `make lint`; `make test`, including the
complete Rust lane, 877 Python tests passed (3 skipped), and the strict
documentation build; and `make spec`, including its static lane.
