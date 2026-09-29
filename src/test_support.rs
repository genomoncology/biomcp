use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

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

/// Build a watchdog deadline duration, stretched by
/// `BIOMCP_TEST_TIMEOUT_SCALE` (read once per process, default 1.0) so a
/// slow or loaded gate host can widen every marked watchdog at once.
pub(crate) fn watchdog(seconds: u64) -> Duration {
    static SCALE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    let factor = *SCALE.get_or_init(|| {
        std::env::var("BIOMCP_TEST_TIMEOUT_SCALE")
            .ok()
            .and_then(|raw| raw.parse::<f64>().ok())
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(1.0)
    });
    watchdog_scaled(seconds, factor)
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
        // The reaped child's stdout pipe is at end-of-stream, so the
        // drain thread has exited and joining cannot block.
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
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
pub(crate) fn signal_ready_on_raw_stdout(marker: &str) {
    let mut stdout = std::io::stdout();
    stdout
        .write_all(format!("{marker}\n").as_bytes())
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
