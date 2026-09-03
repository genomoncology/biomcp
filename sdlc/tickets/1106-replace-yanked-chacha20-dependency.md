---
flow: quickfix
priority: 6
---

# The shipped binary depends on a crate its author withdrew

`Cargo.lock` resolves `chacha20` 0.10.0. Its author yanked that version from the registry on 2026-02-07, and yanked 0.10.1 after it. Both are gone.

It reaches the shipped binary through the MCP protocol crate:

```
chacha20 v0.10.0        <- yanked
└── rand v0.10.1
    └── rmcp v1.7.0
        ├── biomcp-cli
        └── biomcp-mcp-contract-client
```

`cargo deny check advisories` prints `warning[yanked]: detected yanked crate` on every lint run, and has done for as long as the lock file has held this version. It is a warning rather than an error, so nothing has ever blocked on it and nothing has ever chased it.

A yanked crate is one the author withdrew. Until someone reads the reason, we do not know whether that was a vulnerability, a broken release, or a mistaken publish. What we do know is that it is a cryptographic dependency in the binary we ship, and that no supported version of it is installed.

## The fix

`chacha20` 0.10.2 was published on 2026-08-27 and is not yanked. It is the successor to both withdrawn versions.

```
cargo update -p chacha20
```

Keep the locked, offline routine gates working. Do not widen any version requirement in `Cargo.toml` to achieve this — the lock file is what needs to move.

## Reproduction

```
test -z "$(cargo deny check advisories 2>&1 | grep -i yanked)"
```

Exits 1 on `0.9.0-dev.6` at `ede62277`, because the yanked warning is present. Exits 0 once the lock file resolves a supported version.

## Done, observably

- The reproduction command above exits 0.
- `Cargo.lock` resolves `chacha20` at a version the registry still serves.
- `cargo deny check advisories` reports no yanked crate.
- The offline routine gates still pass against the locked graph.

## Boundary

Change `Cargo.lock` only, plus whatever minimum is required to make the update resolve. Do not upgrade `rmcp` or `rand` to achieve this unless the update cannot resolve otherwise, and say so if that turns out to be necessary — that is a larger change than this ticket covers.

Do not silence the warning by configuring `cargo deny` to ignore it.

## Provenance

Filed as a flowless draft by a verify stage, which is the correct behavior for an out-of-scope problem found while exercising. It then sat unclaimed, because a draft with no flow and no priority is never claimed and nothing reports it. Promoted 2026-09-03 after the dependency was re-verified as still yanked and a supported successor was confirmed to exist.
