# 2078 — Move article HTML to one current HTML stack

Status: OPEN.

Milestone: 0.9.3

## Outcome

Article HTML parsing and readability run on one current html5ever stack, and the 33-crate html5ever 0.26 stack, syn 1 and the `fxhash` advisory exception are gone.

## Evidence

- `readability-rust 0.1` has one release and pins `scraper 0.18`, which pulls html5ever 0.26 beside htmd's 0.38: 33 crates, 34.5 s of release CPU, 24.8 s dev (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, step 7).
- `scraper 0.27` and a maintained Readability port such as `dom_smoothie` (MIT) share html5ever 0.39.

- Starts from: three scraper call sites (`fda_orphan.rs`, `transform/article/html.rs`, `variant/erepo.rs`) and one Readability call site.
- Keeps: article full-text output, checked against the article fixtures.
- Changes: scraper 0.27; replace readability-rust; use `uuid` for the gencc temp-name nonce and drop the direct `getrandom`.
- Proof: `cargo tree -i syn@1` is empty; `cargo tree -d` shows at most two html5ever versions; the article fixtures and full-text spec pages pass, and any output change is shown in the review.
- Defers: moving htmd to the same html5ever.
