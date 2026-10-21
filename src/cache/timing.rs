//! Opt-in cache timings contain stage names and durations, never request data.
use std::cell::Cell;
use std::time::{Duration, Instant};

thread_local! {
    static BUSY_CALLS: Cell<u64> = const { Cell::new(0) };
}

pub(super) struct Span {
    stage: &'static str,
    start: Option<Instant>,
    busy_calls: u64,
}

impl Span {
    pub(super) fn new(stage: &'static str) -> Self {
        Self {
            stage,
            start: enabled().then(Instant::now),
            busy_calls: BUSY_CALLS.get(),
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            tracing::debug!(target: "biomcp::cache::timing", stage = self.stage,
                elapsed_us = start.elapsed().as_micros() as u64,
                busy_calls = BUSY_CALLS.get().saturating_sub(self.busy_calls),
                "cache stage");
        }
    }
}

pub(super) fn enabled() -> bool {
    tracing::enabled!(target: "biomcp::cache::timing", tracing::Level::DEBUG)
}

/// Match SQLite's default 100 ms busy-timeout delays while counting callbacks.
pub(super) fn busy_handler(count: i32) -> bool {
    const DELAYS: [u64; 12] = [1, 2, 5, 10, 15, 20, 25, 25, 25, 50, 50, 100];
    const TOTALS: [u64; 12] = [0, 1, 3, 8, 18, 33, 53, 78, 103, 128, 178, 228];
    BUSY_CALLS.set(BUSY_CALLS.get().saturating_add(1));
    let index = usize::try_from(count).unwrap_or(DELAYS.len());
    let elapsed = TOTALS.get(index).copied().unwrap_or(100);
    let delay = DELAYS
        .get(index)
        .copied()
        .unwrap_or(100)
        .min(100_u64.saturating_sub(elapsed));
    if delay == 0 {
        return false;
    }
    // SQLite specifies timeout-based retries rather than a wakeup signal;
    // this callback preserves its existing total wait bounded at 100 ms.
    #[allow(clippy::disallowed_methods)]
    std::thread::sleep(Duration::from_millis(delay));
    true
}
