use super::run_outcome_inner;
use crate::cli::Cli;
use clap::Parser;

/// Future-size probe (ticket 1243). `run_outcome_inner`'s future is the
/// biggest state machine in the CLI: every dispatch arm's locals live in
/// it, and the 8 MiB execute stack's headroom is set by its size (the
/// ticket 1225 pin). Future construction is lazy — parsing a minimal CLI
/// and calling the async fn runs no I/O — so the assert's failure
/// message reports the measured size without side effects.
///
/// Measured history (bytes, debug test profile):
///   2026-09-26 base fe691c48 (async with_no_cache, unboxed arms):
///     dispatch 225_312, run 227_856
///   2026-09-26 scoped no-cache return (item 1):
///     dispatch 113_520, run 114_512
///   2026-09-26 boxed dispatch arms and join (item 2):
///     dispatch 2_240, run 2_656
/// The ceilings carry roughly 2x margin over the boxed sizes so an
/// unrelated arm change surfaces here first.
#[test]
fn the_dispatch_future_stays_under_the_size_ceiling() {
    let cli =
        Cli::try_parse_from(["biomcp", "get", "gene", "BRAF"]).expect("minimal gene get parses");
    let fut = run_outcome_inner(cli, false);
    let bytes = std::mem::size_of_val(&fut);
    assert!(
        bytes <= 4_096,
        "run_outcome_inner's future is {bytes} bytes, over the 4_096-byte ceiling; \
         a dispatch arm grew its state machine (see the measured history above)"
    );
}

/// The `run` fallthrough wraps the same arms and must not regrow them:
/// its future is bounded separately because it constructs its own Cli.
#[test]
fn the_run_fallthrough_future_stays_under_the_size_ceiling() {
    let cli = Cli::try_parse_from(["biomcp", "cache", "path"]).expect("cache path parses");
    let fut = super::super::run(cli);
    let bytes = std::mem::size_of_val(&fut);
    assert!(
        bytes <= 8_192,
        "the run future is {bytes} bytes, over the 8_192-byte ceiling; \
         see the measured history in the dispatch-future probe"
    );
}

mod runtime_split {
    use crate::cli::Cli;
    use crate::cli::worker::ONE_SHOT_RUNTIMES_BUILT;
    use crate::cli::worker::mcp_runtime_probe;
    use clap::Parser;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    /// The MCP path drives on the caller's runtime and builds no
    /// one-shot runtime of its own. Ticket 1257 rewrote the proof:
    /// the drive goes through `execute_mcp_cli` — the real MCP entry
    /// (`mcp/shell.rs` calls it), so `for_shared_call` picks the
    /// shared drive under this ambient test runtime exactly as it
    /// does inside the server — and the assertion reads the
    /// probe-counter seam in `cli::worker` (see its comment) plus
    /// the production counter, so no parallel `run_outcome` test can
    /// move the number the test owns.
    #[tokio::test]
    async fn the_mcp_path_builds_no_one_shot_runtime() {
        let cli = Cli::try_parse_from(["biomcp", "cache", "path"]).expect("cache path parses");
        let before = ONE_SHOT_RUNTIMES_BUILT.load(Ordering::SeqCst);
        mcp_runtime_probe::arm();
        let outcome = crate::cli::execute_mcp_cli(cli).await;
        mcp_runtime_probe::disarm();
        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(
            mcp_runtime_probe::ONE_SHOTS_DURING_PROBE.load(Ordering::SeqCst),
            0,
            "the MCP path constructed a one-shot runtime inside the probe window"
        );
        assert_eq!(
            ONE_SHOT_RUNTIMES_BUILT.load(Ordering::SeqCst),
            before,
            "the shared drive must not construct a per-call runtime"
        );
    }

    /// The CLI one-shot drops its runtime without waiting for a
    /// started blocking task: an eviction-shaped sleeper must not
    /// delay the reply. `drive_one_shot` is the production drop path.
    #[test]
    fn a_one_shot_drop_does_not_wait_for_background_blocking_work() {
        // watchdog: red-side hang — the sleeper outlives the reply by
        // design; a plain 2 s bound keeps the whole test well under a
        // minute even on a loaded host.
        let started = Instant::now();
        let reply = super::super::drive_one_shot(async {
            tokio::task::spawn_blocking(|| {
                // watchdog: red-side hang — the sleeper must outlive the reply
                std::thread::sleep(Duration::from_secs(2)); // watchdog: bounded blocking-sleeper fixture
            });
            anyhow::Ok(())
        });
        assert!(reply.is_ok());
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_secs(1),
            "the one-shot reply waited {elapsed:?} for background blocking work"
        );
    }
}
