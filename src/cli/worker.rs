//! How the bounded execute thread drives a command future.
//! Split from `cli/outcome.rs` under the 700-line CLI cap.

/// Blocking-thread stack headroom above the XML depth cap (the cap is
/// the control; this is margin for the recursive JATS and ClinVar
/// walkers that run on blocking threads). Ticket 1243.
const BLOCKING_STACK_BYTES: usize = 4 * 1024 * 1024;

/// Per-process count of one-shot runtime constructions. The MCP path
/// must drive on the shared server runtime and build none; the counter
/// is the seam the dispatch test asserts against.
pub(super) static ONE_SHOT_RUNTIMES_BUILT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Test-only seam for the MCP-path runtime probe (ticket 1257). The
/// production counter is process-wide, so under a single-process test
/// runner (`cargo test --lib`) any parallel `run_outcome` test also
/// bumps it and a delta assertion on it can fail at random. The probe
/// counts one-shot constructions only while armed, and the probe test
/// arms it for exactly its own `execute_mcp_cli` drive: under nextest
/// (one process per test) the window is fully isolated, and under
/// cargo test a false failure needs another test's one-shot drive to
/// land inside the armed window — which spans a whole blocking-thread
/// spawn plus command run (milliseconds, not microseconds), so it is
/// a real residual flake under the plain runner; the gate and CI both
/// use nextest, where no other test shares the process. `drive_one_shot` increments it
/// alongside the production counter; the test asserts both so the
/// seam cannot drift from what production counts.
/// How the dedicated execute thread drives the command future
/// (ticket 1243).
pub(super) enum WorkerDrive {
    /// The server's long-lived runtime: every MCP tool call reuses it,
    /// so pooled connections and timers survive between calls instead
    /// of dying with a per-call runtime.
    Shared(tokio::runtime::Handle),
    /// A per-call runtime dropped with `shutdown_background`, so the
    /// reply never waits for background blocking work.
    OneShot,
}

impl WorkerDrive {
    /// The shared handle when an ambient runtime exists (the server
    /// process, or a test), one-shot otherwise.
    pub(super) fn for_shared_call() -> Self {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => Self::Shared(handle),
            Err(_) => Self::OneShot,
        }
    }
}

/// Build the per-call runtime, drive the future, and drop the runtime
/// without waiting for background work. Safety invariant: every cache
/// put is awaited inline in the request path
/// (src/cache/manager.rs put at :267); the only background work is
/// eviction (spawn_eviction_task, src/cache/manager.rs at :441),
/// and a partially run eviction leaves atomic cacache removals, never
/// corruption. Fire-and-forget puts would be lost under
/// `shutdown_background` + `process::exit` — do not add them.
pub(super) fn drive_one_shot<F>(fut: F) -> anyhow::Result<F::Output>
where
    F: std::future::Future,
{
    ONE_SHOT_RUNTIMES_BUILT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    #[cfg(test)]
    mcp_runtime_probe::count_increment();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_stack_size(BLOCKING_STACK_BYTES)
        .build()?;
    let output = runtime.block_on(fut);
    runtime.shutdown_background();
    Ok(output)
}

#[cfg(test)]
pub(crate) mod mcp_runtime_probe {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    static ARMED: AtomicBool = AtomicBool::new(false);
    pub(crate) static ONE_SHOTS_DURING_PROBE: AtomicUsize = AtomicUsize::new(0);

    pub(crate) fn arm() {
        ARMED.store(true, Ordering::SeqCst);
        ONE_SHOTS_DURING_PROBE.store(0, Ordering::SeqCst);
    }

    pub(crate) fn disarm() {
        ARMED.store(false, Ordering::SeqCst);
    }

    pub(crate) fn count_increment() {
        if ARMED.load(Ordering::SeqCst) {
            ONE_SHOTS_DURING_PROBE.fetch_add(1, Ordering::SeqCst);
        }
    }
}
