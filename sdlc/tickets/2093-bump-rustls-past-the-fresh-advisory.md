# 2093 — Bump rustls past the fresh advisory

Status: OPEN.

Milestone: 0.9.3

## Outcome

`cargo deny check advisories` passes again on main, with rustls at or past the releases that fix the fresh advisories.

## Evidence

Filed 2026-10-14 when a fresh rustls advisory (RUSTSEC, fixed in rustls >= 0.23.46; CI also shows a second rustls advisory) turned every lane's lint and CI red, including main's last three runs (38555091574, 38553661475, 3855313861), first red after main's last green at the 2069 landing.

- Starts from: Cargo.lock's rustls 0.23.45.
- Changes: `cargo update -p rustls` (and any transitive patch cargo chooses) to at least 0.23.46; one Internal changelog bullet; the dev-cycle expected-ticket set gains 2093.
- Proof: `cargo deny check advisories` passes locally and in CI; the rest of the lockfile unchanged apart from the rustls-related patch bumps; make lint green; CI green on the exact head.
- Defers: nothing.

Executed on the branch (head `768d2522a6821e53086fadbedd85cb26636368be`):

- `cargo update -p rustls` moved exactly one package: `rustls v0.23.45 -> v0.23.46`; the Cargo.lock diff is that package's version and checksum (2 lines), nothing else.
- Yellow, both locks held: `cargo deny --locked check advisories` prints `advisories ok`; `make lint` green in 148.2 s (the advisory was the only failure on main before the bump); the changelog-refresh tests pass (19 passed).
- CI run 38558549795 on that exact head: completed success, 28m11s.
