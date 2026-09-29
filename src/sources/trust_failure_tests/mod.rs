//! Unit tests for the no-trust-failure retry strategy's error-chain
//! walk (ticket 1268 follow-up, 2026-09-29 review: the strategy had
//! no test; the tests live in their own file so mod.rs keeps only the
//! delegation and the module declaration, and its size-inventory
//! baseline rose by exactly that: 2779 to 2786, delta 932 to 939).

use super::error_chain_carries;

/// A synthetic error chain: each level wraps the next the way
/// hyper/tls errors nest inside a reqwest_middleware error.
#[derive(Debug)]
struct Wrapped(&'static str, Option<Box<dyn std::error::Error + 'static>>);

impl std::fmt::Display for Wrapped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Wrapped {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.1.as_ref().map(|inner| inner.as_ref())
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
