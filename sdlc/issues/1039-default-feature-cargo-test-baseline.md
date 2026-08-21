# Default-feature cargo test baseline failures

A direct default-feature `cargo test` invocation reportedly has two unrelated baseline failures: a provider URL fixture-origin expectation and a rate-limit policy expectation. The affected logic is in `src/sources/provider_url_policy.rs` and `src/sources/rate_limit.rs`.

The required `make test` lane uses `--no-default-features` and passes. Reconcile the default-feature unit expectations or document the supported lane in a future ticket.
