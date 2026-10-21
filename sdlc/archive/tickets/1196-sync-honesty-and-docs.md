---
flow: build
priority: 4
deps: []
---

# 1196: Data sync tells the truth and the list docs catch up

## Goal

The six local-data `sync` commands report a real `changed` value in `--json`
(the GenCC initializer failure keeps the structured envelope, named `GenCC`),
the batch list page documents the modes the CLI accepts, and the CLI
reference lists `biomcp gencc sync` beside its siblings.

## Current Facts

Exit-code and envelope probes against a built binary, plus source reads:

1. `sync_outcome` in `src/cli/system/dispatch.rs` hard-coded
   `"changed": true` for ema, who, cvx, ddinter, gtr, and who-ivd; only gencc
   computed a real value. `DdinterClient::sync` already returned a real bool
   at the source layer, but dispatch discarded it.
2. `handle_gencc` in `src/cli/system/mod.rs` mapped
   `GenCcClient::new()` failure to a bare `anyhow!` string, losing the typed
   JSON error envelope.
3. `list_batch` in `src/cli/list/helpers.rs` omitted `--mode <compact|detail>`
   (article batches only) and the article compact 20-ID cap
   (`ARTICLE_BATCH_MAX_IDS = 20`, `src/cli/system/batch.rs`).
4. `docs/user-guide/cli-reference.md` "Top-level commands" listed the other
   six sync commands but not `biomcp gencc sync`.

## Scope

- `bundle_fingerprint` in `src/utils/download.rs`: sorted
  `(path, length, mtime nanos)` of the files under a bundle root. Each source
  sync root (ema, who_pq, cvx, gtr, who_ivd) wraps its existing body and
  returns `before != after`; ddinter returns its existing bool. `sync_outcome`
  takes the changed value; every handler threads it. The signal reports bundle
  write state (files added, removed, or rewritten); a byte-identical Force
  rewrite still registers as a change — recorded limitation, not a fake.
- The GenCC initializer failure becomes
  `BioMcpError::SourceUnavailable { source_name: "GenCC", ... }`, and
  `SourceProvider` gains the `GENCC` label so the envelope prints `GenCC`.
- List page and CLI reference lines above.
- No behavior change beyond those outputs; no new dependency or packaged path.

## Tests

- `bundle_fingerprint` unit tests: missing root, stable, rewritten, removed.
- `sync_json_reports_the_actual_change_flag` and
  `sync_text_output_reports_its_message_not_a_change_flag` in
  `src/cli/system/tests.rs` pin the JSON contract (false stays false).
- `list_batch_and_enrich_pages_exist` pins the new option and cap lines.

## Verification

- `cargo nextest run --no-default-features -E 'test(system) or test(sync) or test(bundle_fingerprint) or test(download)'`:
  120 passed, 0 failed.
- `cargo clippy --no-default-features -- -D warnings`: clean.
- `pytest tests/test_json_list_contract.py tests/test_source_licensing_docs_contract.py tests/test_documentation_consistency_audit_contract.py`:
  28 passed.
- Live probes: `BIOMCP_GENCC_BASE="not a url" biomcp --json gencc sync` →
  exit 1, `{"error": {"code": "source_unavailable", "source": "GenCC", ...}}`;
  who-ivd with a dead endpoint and a seeded bundle → `changed: false`
  (fallback wrote nothing); who-ivd against a served fixture → `changed: true`.
- Quality ratchet: pass (error.rs 1132 with delta 10; who_pq.rs 1123 delta 6;
  new ema.rs entry 1003 delta 3; CLI caps cleared by compacting handlers and
  page assertions).

## Complexity

- Contract score: 1 (several explicit cases: six sync commands, the JSON
  contract, the envelope)
- State and timing score: 1 (change flag observes persistent bundle state)
- Reach score: 1 (six source modules, dispatch, error registry, two docs)
- Proof score: 1 (unit, contract, and live-probe modes)
- Cost of error score: 0 (machine-readable metadata; cheap local correction)
- Total: 4
- Minimum level floor: none
- Final level: 2
- Reasons: multiple owners but mechanical plumbing with live two-direction
  proof
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Implemented and self-verified under the acceptance-report contract; parent
  retains the independent review gate.

- Code review: ACCEPT 2026-09-15 with record-only P2s, closed here: (a)
  ddinter's flag is constant true because its sync republishes the whole
  eight-file bundle into a fresh staging directory every run — a fingerprint
  wrapper would read the same; recorded as write-state semantics, and the
  ticket notes automation should not read ddinter changed:false on success.
  (b) The error.rs inventory entry now records co-ownership "1142, 1196"
  (seven lines from 1142's accepted overage, three from this ticket's
  label). (c) The GENCC legacy label also re-points failing (not just
  uninitializable) gencc sync errors to source "GenCC" with retry guidance —
  an improvement, recorded as the fifth output change.
