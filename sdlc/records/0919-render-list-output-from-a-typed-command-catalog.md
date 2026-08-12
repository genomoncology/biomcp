---
base: b11602b2eb7310e4850d84ad48922a68f16784c8
head: d95208a4
---

`biomcp list --json` now serializes a typed catalog instead of scraping the
rendered Markdown reference. Catalog entries distinguish executable literals,
templates with typed placeholders, and prose. Literal entries are parsed
through the production Clap tree before JSON is emitted.

The catalog owns searchable/gettable classification and named-section
capabilities. It adds author, removes the false `get study` classification,
keeps study as its own command page, includes `study top-mutated`, fills the
skill command inventory, and publishes stable ordered JSON entries. The human
root inventory is projected from the same capability model, and the public CLI
documentation describes the new shape.

All 31 focused Rust list tests and 22 focused JSON, documentation, author, and
CLI-surface contracts passed. No-feature Clippy passed with warnings denied.
The implementation added 258 net `src` lines against the ticket's 280-line
ceiling.
