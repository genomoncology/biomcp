use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Capture warning output for the future without changing the process subscriber.
pub(crate) async fn capture_warnings<F: std::future::Future>(future: F) -> (F::Output, String) {
    use tracing::instrument::WithSubscriber;

    #[derive(Clone)]
    struct Writer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("warning capture lock").write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let bytes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = Writer(std::sync::Arc::clone(&bytes));
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .without_time()
        .with_writer(move || writer.clone())
        .finish();
    let result = future.with_subscriber(subscriber).await;
    let logs = String::from_utf8(bytes.lock().expect("warning capture lock").clone())
        .expect("UTF-8 warnings");
    (result, logs)
}

pub(crate) struct TempDirGuard {
    inner: tempfile::TempDir,
}

impl TempDirGuard {
    pub(crate) fn new(label: &str) -> Self {
        let inner = tempfile::Builder::new()
            .prefix(&format!("biomcp-test-{label}-"))
            .tempdir()
            .expect("create temp dir");
        Self { inner }
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        self.inner.path()
    }
}

pub(crate) const FIXTURE_MIN_DISK_FREE_BYTES: u64 = 1;

/// One shared environment guard for source-fixture tests:
/// every `set` restores on drop, and the guard itself always points the
/// shared HTTP client's cache at a fresh scratch directory with cache mode
/// off, so a test never reads the machine's own cache — earlier trial
/// tests read a 183 MB host cache and hit the suite's time limit before
/// this guard existed. Tests that do not construct this guard still never
/// read the machine's cache: a test build's default cache root is a
/// per-process folder under the OS temp dir, while the
/// hand-written env guards that remain in the tree control cache mode and
/// source bases, not isolation.
pub(crate) struct FixtureEnv {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _cache_dir: TempDirGuard,
    _cache_mode: Option<crate::sources::TestCacheModeGuard>,
}

impl FixtureEnv {
    pub(crate) fn new() -> Self {
        Self::scratch_cache(Some(crate::sources::test_cache_mode::off()))
    }

    /// A guard for tests that exercise the cache itself: the scratch cache
    /// directory still isolates the test from the machine's cache, but the
    /// cache mode stays whatever the environment selects, so a test can
    /// seed its own entries and serve them back stale.
    pub(crate) fn scratch_cache_only() -> Self {
        Self::scratch_cache(None)
    }

    fn scratch_cache(cache_mode: Option<crate::sources::TestCacheModeGuard>) -> Self {
        let cache_dir = TempDirGuard::new("fixture-cache");
        let mut env = Self {
            saved: Vec::new(),
            _cache_dir: cache_dir,
            _cache_mode: cache_mode,
        };
        let root = env._cache_dir.path().to_path_buf();
        env.set_cache_root(root);
        env
    }

    /// Select a fixture root and pin its disk floor without changing cache mode.
    pub(crate) fn set_cache_root(&mut self, root: impl AsRef<std::ffi::OsStr>) {
        self.save_and_set("BIOMCP_CACHE_DIR", root);
        self.save_and_set(
            "BIOMCP_CACHE_MIN_DISK_FREE",
            format!("{FIXTURE_MIN_DISK_FREE_BYTES}B"),
        );
    }

    pub(crate) fn set(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        if key == "BIOMCP_CACHE_DIR" {
            self.set_cache_root(value);
        } else {
            self.save_and_set(key, value);
        }
    }

    fn save_and_set(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        self.saved.push((key, std::env::var_os(key)));
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe { std::env::set_var(key, value) };
    }

    pub(crate) fn remove(&mut self, key: &'static str) {
        self.saved.push((key, std::env::var_os(key)));
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe { std::env::remove_var(key) };
    }
}

impl Drop for FixtureEnv {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..).rev() {
            // SAFETY: this test holds the serial-test process-wide
            // environment lock.
            unsafe {
                if let Some(value) = value {
                    std::env::set_var(key, value)
                } else {
                    std::env::remove_var(key)
                }
            }
        }
    }
}

/// Build a watchdog deadline duration, stretched by
/// `BIOMCP_TEST_TIMEOUT_SCALE` (read once per process, default 1.0) so a
/// slow or loaded gate host can widen every marked watchdog at once.
pub(crate) fn watchdog(seconds: u64) -> Duration {
    watchdog_scaled(seconds, timeout_scale())
}

/// Join a test worker within the shared watchdog, preserving its panic payload.
/// A timeout returns an error too, so cleanup may ignore it without panicking
/// again while another assertion unwinds.
pub(crate) fn bounded_join<T>(worker: std::thread::JoinHandle<T>) -> std::thread::Result<T> {
    let deadline = std::time::Instant::now() + watchdog(60);
    while !worker.is_finished() {
        if std::time::Instant::now() >= deadline {
            return Err(Box::new(String::from(
                "test worker exceeded the join watchdog",
            )));
        }
        std::thread::sleep(Duration::from_millis(10)); // watchdog: bounded test-worker cleanup
    }
    worker.join()
}

/// A sub-second watchdog window under the same stretch, for the
/// deadline-transition proofs whose semantic windows are milliseconds.
pub(crate) fn watchdog_millis(millis: u64) -> Duration {
    Duration::from_millis((millis as f64 * timeout_scale()).round() as u64)
}

fn timeout_scale() -> f64 {
    static SCALE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *SCALE.get_or_init(|| {
        std::env::var("BIOMCP_TEST_TIMEOUT_SCALE")
            .ok()
            .and_then(|raw| raw.parse::<f64>().ok())
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(1.0)
    })
}

fn watchdog_scaled(seconds: u64, factor: f64) -> Duration {
    Duration::from_secs_f64(seconds as f64 * factor)
}

/// A spawned child whose readiness is signaled by one exact line on its
/// real stdout, and whose release is the parent dropping the stdin pipe.
///
/// The handshake line scan replaces clock polling: the wait IS the read.
/// The child is expected to block reading stdin after signaling, so a
/// parent that dies (panic unwind drops the pipe, SIGKILL closes it at
/// the kernel) releases the child by end-of-input and no orphan survives.
pub(crate) struct SignaledChild {
    pub(crate) child: Child,
    stdin: Option<ChildStdin>,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl SignaledChild {
    /// Spawn `program` and wait until it prints the exact `marker` line
    /// on stdout. A child that never signals fails here inside a scaled
    /// watchdog instead of hanging the gate; a child that exits first
    /// fails with its exit status.
    pub(crate) fn spawn<K, V>(
        program: &std::path::Path,
        args: &[&str],
        envs: impl IntoIterator<Item = (K, V)>,
        marker: &str,
    ) -> Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        let mut child = Command::new(program)
            .args(args)
            .envs(envs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn signaled child");
        let stdout = child
            .stdout
            .take()
            .expect("piped stdout for signaled child");
        let (ready_tx, ready_rx) = mpsc::channel::<()>();
        let needle = marker.to_string();
        let moved_needle = needle.clone();
        // Drains past the marker too, so a chatty child can never block on
        // a full stdout pipe before it exits.
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(text) if text.trim() == moved_needle => {
                        let _ = ready_tx.send(());
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });
        let stdin = child.stdin.take().expect("piped stdin for signaled child");
        // watchdog: handshake failure means a broken child, not slow load.
        match ready_rx.recv_timeout(watchdog(60)) {
            Ok(()) => Self {
                child,
                stdin: Some(stdin),
                reader: Some(reader),
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("signaled child never reported `{needle}` within the watchdog window");
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let status = child.wait();
                panic!("signaled child exited before reporting `{needle}`: {status:?}");
            }
        }
    }

    /// Close the stdin pipe. A well child sees end-of-input and exits by
    /// itself; call [`Child::wait`] afterwards for its exit status.
    pub(crate) fn release(&mut self) {
        drop(self.stdin.take());
    }
}

impl Drop for SignaledChild {
    fn drop(&mut self) {
        // Closing stdin first lets a well child exit on end-of-input; the
        // kill catches a child stuck anywhere else so it never outlives
        // the test. Both are no-ops on an already-reaped child.
        self.release();
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Bound the drain thread too: descendants may still own the stdout pipe.
        if let Some(reader) = self.reader.take() {
            let _ = bounded_join(reader);
        }
    }
}

/// Read stdin to end-of-input. Used by handshake children to block until
/// the parent releases them; a dead parent closes the pipe at the kernel.
pub(crate) fn block_until_stdin_closes() -> String {
    let mut received = String::new();
    let _ = std::io::stdin().read_to_string(&mut received);
    received
}

/// Write one readiness line on the real stdout fd. libtest captures the
/// `print!` family, so the parent scans for this exact line instead.
/// Start a new line because libtest may leave its test-name prefix on stdout.
pub(crate) fn signal_ready_on_raw_stdout(marker: &str) {
    let mut stdout = std::io::stdout();
    stdout
        .write_all(format!("\n{marker}\n").as_bytes())
        .expect("write readiness marker");
    stdout.flush().expect("flush readiness marker");
}

#[test]
fn watchdog_defaults_to_one_and_stretches_by_the_factor() -> Result<(), std::env::VarError> {
    // The factor is read once per process, so the environment path pins
    // the default and the pure builder pins the stretch itself.
    let guard = std::env::var("BIOMCP_TEST_TIMEOUT_SCALE");
    assert_eq!(watchdog_scaled(30, 1.0), Duration::from_secs(30));
    assert_eq!(watchdog_scaled(30, 10.0), Duration::from_secs(300));
    assert_eq!(watchdog_scaled(30, 0.5), Duration::from_millis(15_000));
    assert_eq!(watchdog_scaled(1, 0.05), Duration::from_millis(50));
    match guard {
        Err(std::env::VarError::NotPresent) => {
            assert_eq!(watchdog(30), Duration::from_secs(30));
            Ok(())
        }
        Err(other) => Err(other),
        Ok(_) => Ok(()), // an outer runner may set the factor; the builder is pinned above
    }
}

#[test]
fn signaled_child_spawn_fails_when_the_child_never_signals() {
    // Red-side proof for the handshake: a child that exits without ever
    // printing the marker fails the spawn immediately (the reader hits
    // end-of-stream and the channel disconnects) instead of passing a
    // silent child through. A spawn that returns here is the failure.
    let result = std::panic::catch_unwind(|| {
        SignaledChild::spawn(
            std::path::Path::new("/bin/sh"),
            &["-c", "exit 0"],
            Option::<(String, String)>::None,
            "entered",
        )
    });
    assert!(
        result.is_err(),
        "a child that never signals must fail the handshake spawn"
    );
}
