//! Owned loopback HTTP fixtures, independent of the caller's Tokio runtime.

use std::future::Future;
use std::io;
use std::sync::Arc;
use std::thread;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

pub struct TestHttpServer {
    pub base: String,
    stop: Option<oneshot::Sender<()>>,
    worker: Option<thread::JoinHandle<io::Result<()>>>,
}

impl TestHttpServer {
    /// Own every accepted connection, including unfinished or deliberately held requests.
    pub fn spawn<F, Fut>(handler: F) -> io::Result<Self>
    where
        F: Fn(TcpStream) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = io::Result<()>> + Send + 'static,
    {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;
        let base = format!("http://{}", listener.local_addr()?);
        let (stop, stopped) = oneshot::channel();
        let worker = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(serve(listener, stopped, Arc::new(handler)))
        });
        Ok(Self {
            base,
            stop: Some(stop),
            worker: Some(worker),
        })
    }

    /// Read a complete request before invoking a raw-response handler.
    pub fn respond<F, Fut>(handler: F) -> io::Result<Self>
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = io::Result<Vec<u8>>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        Self::spawn(move |mut stream| {
            let handler = handler.clone();
            async move {
                if let Some(request) = read_request(&mut stream).await? {
                    stream.write_all(&handler(request).await?).await?;
                }
                Ok(())
            }
        })
    }

    /// Normal teardown must assert this result; Drop only provides unwind cleanup.
    pub fn finish(&mut self) -> io::Result<()> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        self.worker.take().map_or(Ok(()), |worker| {
            worker
                .join()
                .map_err(|_| io::Error::other("HTTP fixture worker panicked"))?
        })
    }
}

impl Drop for TestHttpServer {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

async fn serve<F, Fut>(
    listener: std::net::TcpListener,
    mut stopped: oneshot::Receiver<()>,
    handler: Arc<F>,
) -> io::Result<()>
where
    F: Fn(TcpStream) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = io::Result<()>> + Send + 'static,
{
    let listener = tokio::net::TcpListener::from_std(listener)?;
    let mut connections = JoinSet::new();
    let mut failure = None;
    loop {
        tokio::select! {
            biased;
            result = connections.join_next(), if !connections.is_empty() => {
                remember_failure(result.expect("nonempty connections"), &mut failure);
            }
            _ = &mut stopped => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    let handler = handler.clone();
                    connections.spawn(async move { handler(stream).await });
                }
                Err(error) if transient(&error) => continue,
                Err(error) => { failure.get_or_insert(error); break; }
            }
        }
    }
    connections.abort_all();
    while let Some(result) = connections.join_next().await {
        remember_failure(result, &mut failure);
    }
    failure.map_or(Ok(()), Err)
}

fn transient(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock
            | io::ErrorKind::Interrupted
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::NotConnected
    )
}

fn remember_failure(
    result: Result<io::Result<()>, tokio::task::JoinError>,
    failure: &mut Option<io::Error>,
) {
    let error = match result {
        Ok(Ok(())) => return,
        Ok(Err(error)) if transient(&error) => return,
        Ok(Err(error)) => error,
        Err(error) if error.is_cancelled() => return,
        Err(error) => io::Error::other(format!("HTTP fixture connection failed: {error}")),
    };
    failure.get_or_insert(error);
}

/// Return None when a client leaves before sending a complete request.
pub async fn read_request(stream: &mut TcpStream) -> io::Result<Option<Vec<u8>>> {
    let mut request = Vec::new();
    let mut expected = None;
    loop {
        let mut chunk = [0; 8192];
        let count = stream.read(&mut chunk).await?;
        if count == 0 {
            return Ok(None);
        }
        request.extend_from_slice(&chunk[..count]);
        if expected.is_none()
            && let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n")
        {
            let header = String::from_utf8_lossy(&request[..end]);
            let length = header
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>())
                })
                .transpose()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
                .unwrap_or(0);
            expected = Some(
                (end + 4)
                    .checked_add(length)
                    .ok_or_else(|| io::Error::other("request length overflow"))?,
            );
        }
        if expected.is_some_and(|length| request.len() >= length) {
            return Ok(Some(request));
        }
    }
}

#[cfg(test)]
mod tests;
