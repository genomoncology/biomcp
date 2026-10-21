---
base: 7e2c45c2f2755046be2a680a5e9d500104fa7d56
head: 040817ca12a0e7b749fa1df3edf950c18456307c
---

Contributor guidance now identifies `make test` as the supported offline,
no-default-features nextest lane. It names the known order-dependent direct-run
failure and records that reconciliation was declined, so developers can recover
without treating bare `cargo test` as supported.
