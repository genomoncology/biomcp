Decision: approved
Operator verify pending: no

## Checkpoint Summary

Ticket 680 makes the three VAERS LocalOnly unit tests deterministic without changing the shipped CVX root-selection or VAERS behavior. All focused clean-home tests and the single routine full-blocking gate are green.

## Planning/FAQ Watch Results — relevant watching/answered entries probed

The BioMCP FAQ currently has no `watching` entries. Relevant answered entry 17 requires internal behavior in native tests and shipped CLI behavior in mustmatch specs; this seam correctly remains native-unit coverage. The relevant VAERS specification continues to document the unchanged public source-selection, non-vaccine, and aggregate behavior. No new safety boundary or recurring quality concern was found.

## Exercise Results — ran, inputs, observations

- Ran each named contract with `BIOMCP_CVX_DIR` unset and a fresh empty `XDG_DATA_HOME`: ibuprofen non-vaccine classification, combined-source non-vaccine status, and influenza VAERS bridge mapping all passed.
- Ran `cargo nextest run --locked entities::adverse_event::tests`: all 31 adverse-event tests passed.
- Exercised `biomcp search adverse-event --help`: the existing FAERS/VAERS/all source selection remained discoverable.
- Exercised an invalid `--source` value: it exited 2 with clap's invalid-value error, without a network call or filesystem mutation.

## Exploratory Verification — change-aware probes tried; high-signal probes; noisy/not-worth-repeating probes; recommended improved tests (`spec`, `test`, `lint`, `gate`, `verify-group`, docs/help, FAQ watching, experiment/harness); agent/tool-cost friction if applicable

The highest-signal probe was the three individual clean-home runs because it proves the former machine-local dependency is gone while retaining the real CVX parser and VAERS bridge. The adverse-event module regression run checked neighboring resolver behavior. Help and invalid-source probes confirmed the unchanged user-facing surface remains clear and safe.

The full repository gate is necessarily broad and took substantially longer than the focused proof; it remains the correct one-time regression gate, not a probe to repeat. No durable test addition is needed: the existing three semantic tests now directly exercise the injected local fixture root and catch the original regression. No agent-facing discovery or tool-cost friction was found.

## Edge Cases Tested — specific cases, results

- Missing CVX user-data prerequisite: fresh empty `XDG_DATA_HOME`, with `BIOMCP_CVX_DIR` unset — all three named tests passed.
- Non-vaccine input: `ibuprofen` remained `QueryNotVaccine` in direct and combined-source paths.
- Vaccine-family input: `influenza vaccine` retained its bridge mapping.
- Invalid CLI enum input: rejected before work is attempted, exit 2.

## Spec Audit — specs reviewed, gaps found, counts before/after, spec-only result

Reviewed `spec/entity/vaers.md`. Its source-selection, vaccine-only truthfulness, typed combined outcomes, and live aggregate canary assert public behavior; none is changed by this test-only seam. The native unit assertions are the correct layer because documenting an injected test root as CLI behavior would be false.

The changed interface has no missing user-visible contract. The three unit assertions are semantic: they distinguish non-vaccine classification, combined status preservation, and influenza bridge identity. No behavioral assertion was added or removed during verify. The routine `make spec` portion of full-blocking passed; there are no `lane: check` entries, so this is the required green spec-only confirmation.

Assertion-quality delta: no changed mustmatch assertion required relaxation or rewrite. No syntactic red found.

## Verify Group — `lane: verify` entries exercised (each: assertion, red_command, observed_status); operator-pending list explicit if credentials unavailable

There are no `lane: verify` entries in `.march/contract-red-check.json`; no verify-group command or operator action is required.

## Regression Results — existing features verified

The isolated adverse-event module passed. The full routine gate covered the Rust suite, Python CLI/MCP/docs contracts, strict MkDocs build, routine mustmatch specs, and static spec contracts.

## Test Suite — full-blocking result

Ran the authoritative full-blocking profile exactly once as `make lint && make test && make spec`. It passed: lint and quality ratchet, Rust tests, 448 Python contracts, strict MkDocs build, routine mustmatch specs (91 passed/3 skipped; 220 passed/2 skipped; 7 passed), 31 surface contracts, and 10 static checks. The three `lane: unit` entries are green under that `make test` run.

## Documentation — parity audit of docs/help/examples

No shipped behavior, CLI help, docs, or examples changed. `spec/entity/vaers.md` remains accurate for the unchanged public behavior, and strict MkDocs passed.

## Issues Found and Fixed — fixes + proof

No new defects found in verification. The ticket implementation itself repairs the ambient user-data dependency; the three clean-home focused tests and full `make test` prove it.

## Issues Filed — list with paths

None.

## Planning Updates — concrete issues filed or FAQ watching proposal

None.

## UX Quality — CLI/UI assessment (if applicable)

The unchanged adverse-event help clearly exposes source choice. Invalid source input is rejected before a potentially costly lookup. The internal seam adds no user-facing surface.

Issues filed: 0
