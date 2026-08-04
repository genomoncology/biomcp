# Code Review — Ticket 648

## Scope reviewed

- Read `AGENTS.md`, ticket, design draft/final, red-check evidence, code log, and applicable mustmatch/Rust/Python/testing standards.
- Confirmed `main` is an ancestor of `HEAD`; reviewed the complete `main..HEAD` diff and both ticket commits.
- Inspected the retained resolver boundary directly: `attach_not_included` uses `resolve_archive_package`; explicit `assets` and `asset` use `resolve_article_assets`.

## Audit results

- **Design completeness:** the architecture clarification implements the selected Option A boundary. The named mustmatch ratchet excludes the JATS-only handle from ordinary fulltext, while the existing explicit-assets contract retains retrieval of that handle. No runtime, CLI, fixture, or help change was required or made.
- **Forward traceability:** the single proof-matrix entry lands at `spec/entity/article.md::Fulltext Reports Assets Not Included` as the assertion that `linked-jats-s2.csv` is absent from ordinary-summary next commands.
- **Reverse traceability:** `git diff main..HEAD -- 'spec/*'` contains only that proof-matrix assertion and associated explanatory text. It adds no invented assertion and removes or relaxes none.
- **Edit discipline:** 24 changed lines across the named architecture document and spec are proportionate to the minimal documentation clarification plus discriminating public-contract ratchet. No over-edit found.
- **Quality/security:** no runtime code changed. The specification drives the public CLI against the deterministic fixture and the exact-count assertion is discriminating (one archive supplement versus one linked-only supplement), not trivia. No injection, secret, data-completeness, duplication, resource, dead-code, stale-error, or shadowing defect found.
- **Collateral / issues:** no repair was necessary; no out-of-scope issue was found.

## Validation

- `make lint` — passed
- `make test` — passed (native and 446 Python contracts; strict MkDocs build passed)
- `make spec` — passed (90 passed/3 skipped; 219 passed/2 skipped; 7 passed; 31 Python contracts passed; 10 static specs passed)
- `git diff --check main..HEAD` — passed

## Repair / commit

No defect required repair. Consequently no separate code-review commit was created.

## Residual concerns

None within ticket scope.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| — | None found | — | No defects found. |
