# Provider URL fixture-origin unit test fails in the routine feature lane

A direct `tools/with-build-identity cargo test --locked --no-default-features --lib`
fails `sources::provider_url_policy::tests::selected_fixture_origin_allows_only_exact_ip_loopback`.
The test expects an HTTP `127.0.0.1` fixture origin to be accepted, but the
policy rejects its non-HTTPS scheme. This is unrelated to ticket 1043's build
output cleaner. Reconcile the unit expectation with the fixture-origin policy
and the passing `make test` lane.

2026-08-23: reproduced in full-suite direct runs in both feature lanes; passes in
isolation, so the failure is suite-order-dependent. Folded into ticket 1049, which
declares the supported test lane in writing.
