# 2361 — Clear the Tokio AsyncFd advisory

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `Cargo.lock`, `CHANGELOG.md`.

## Outcome

The release's existing advisory check accepts the locked Tokio dependency after the smallest supported patched upgrade, with existing CLI and MCP behavior preserved.

## Evidence

Final source d6a72ec91910b10738f4347eb770061934e6191e failed both M5 main qualification j6391 and Gator Linux qualification j6394. Cargo-deny rejects Tokio 1.50.0 for AsyncFd I/O safety under RUSTSEC-0000-0000. Both hosts used RustSec database 6e868779ba1c38dc4a1bc8090b8c22d15dc69e56. OSV passed. Neither qualification reached tests or coverage.

The upstream fix is https://github.com/tokio-rs/tokio/pull/8587. The scanner identifies patched ranges beginning at 1.51.6 and 1.53.3. Select the smallest compatible supported patch and change only required dependency companions. This repair serves Ian's ruling to fix every known bug before 0.9.4 ships.

## Keeps

Keep the advisory policy and reviewed exceptions unchanged. Preserve existing public behavior and the full-text deadline repair. Add the release changelog bullet and correct the file claim to the actual diff.

- Proof: The existing cargo-deny advisory stage is the failing check on current main; retain j6391 and j6394 as RED evidence. Show that same stage passes with the dependency repair, then run one routine landing gate on the reviewed-ready branch through pm job. Use existing CLI and MCP tests; add a regression only if an actual compatibility defect needs one. Final source qualification and the complete untagged rehearsal remain with 2330 after landing.
