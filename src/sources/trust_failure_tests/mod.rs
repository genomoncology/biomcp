//! Unit tests for the no-trust-failure retry strategy's error-chain
//! walk (ticket 1268 follow-up, 2026-09-29 review: the strategy had
//! no test; the tests live in their own file so mod.rs keeps only the
//! delegation, the module declaration and the plain-send trust stop,
//! and its size-inventory baseline rose accordingly — the gate reads
//! the exact numbers from tools/rust-source-size-inventory.json).

use super::error_chain_carries;

/// A synthetic error chain: each level wraps the next the way
/// hyper/tls errors nest inside a reqwest_middleware error.
#[derive(Debug)]
struct Wrapped(
    &'static str,
    Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
);

impl std::fmt::Display for Wrapped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Wrapped {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The Send+Sync box coerces to the plain trait object here.
        self.1
            .as_ref()
            .map(|inner| inner.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// A certificate rejection anywhere in the error chain is a trust
/// failure, so the strategy returns no retry for it; a connect
/// error with no trust marker is not.
#[test]
fn trust_markers_at_any_depth_of_the_error_chain_count() {
    let surface = Wrapped(
        "error sending request",
        Some(Box::new(Wrapped(
            "hyper::Error",
            Some(Box::new(Wrapped(
                "certificate verify failed: self-signed certificate",
                None,
            ))),
        ))),
    );
    assert!(error_chain_carries(&surface));

    let shallow = Wrapped("invalid peer certificate: chain incomplete", None);
    assert!(error_chain_carries(&shallow));

    let ordinary = Wrapped(
        "error sending request",
        Some(Box::new(Wrapped(
            "tcp connect error: connection refused",
            None,
        ))),
    );
    assert!(!error_chain_carries(&ordinary));

    let empty = Wrapped("", None);
    assert!(!error_chain_carries(&empty));
}

/// Driving the real trait method (the 2026-09-29 second review:
/// deleting the early return in `handle` passed every test).
/// `reqwest_middleware::Error::Middleware` is public, so a synthetic
/// marker-carrying error rides the exact path `handle` inspects.
#[test]
fn handle_returns_no_retry_for_a_trust_failure_error() {
    use reqwest_retry::RetryableStrategy;

    let strategy = super::NoTrustFailureStrategy;
    let trust_err = || {
        reqwest_middleware::Error::Middleware(anyhow::Error::new(Wrapped(
            "invalid peer certificate: chain incomplete",
            None,
        )))
    };
    let verdict = strategy.handle(&Err(trust_err()));
    assert!(
        verdict.is_none(),
        "a certificate rejection must not be retried"
    );

    // A non-trust middleware error keeps the default strategy's
    // verdict (whatever it is) — the wrapper only strips trust
    // failures. The observable contract: the trust marker changes
    // the answer from the default's to no-retry.
    let ordinary_err = || {
        reqwest_middleware::Error::Middleware(anyhow::Error::new(Wrapped(
            "tcp connect error: connection refused",
            None,
        )))
    };
    let default_some = reqwest_retry::DefaultRetryableStrategy
        .handle(&Err(ordinary_err()))
        .is_some();
    assert_eq!(
        strategy.handle(&Err(ordinary_err())).is_some(),
        default_some,
        "a non-trust error keeps the default retry verdict"
    );
}

/// The plain-send retry loop (Enrichr, UniProt) stops at the first
/// trust failure instead of burning its retries (2026-09-29 second
/// review): one closure call, then the error surfaces.
#[tokio::test]
async fn plain_send_retry_stops_at_a_trust_failure() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let calls = AtomicUsize::new(0);
    let context = crate::error::SourceContext::retry(crate::error::SourceProvider::ENRICHR);
    let result = crate::sources::retry_middleware_send(context, 3, || {
        calls.fetch_add(1, Ordering::SeqCst);
        async {
            Err(reqwest_middleware::Error::Middleware(anyhow::Error::new(
                Wrapped("invalid peer certificate: chain incomplete", None),
            )))
        }
    })
    .await;
    assert!(
        result.is_err(),
        "the trust failure must surface as an error"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a trust failure must not be retried on the plain-send path"
    );
}
