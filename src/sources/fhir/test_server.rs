//! A tiny HTTP/1.1 server that answers FHIR requests from a route function
//! and records what it received. Tests only; synthetic bundles only.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// One canned response.
#[derive(Debug, Clone)]
pub(crate) struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Reply {
    pub(crate) fn json(status: u16, body: serde_json::Value) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".into(), "application/fhir+json".into())],
            body: body.to_string(),
        }
    }

    pub(crate) fn redirect(location: impl Into<String>) -> Self {
        Self {
            status: 302,
            headers: vec![("Location".into(), location.into())],
            body: String::new(),
        }
    }
}

/// One request the server saw.
#[derive(Debug, Clone)]
pub(crate) struct Seen {
    pub target: String,
    pub cache_control: Option<String>,
    pub prefer: Option<String>,
}

type Route = dyn Fn(&str) -> Reply + Send + Sync;

pub(crate) struct FixtureServer {
    pub base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
    task: tokio::task::JoinHandle<()>,
}

impl FixtureServer {
    /// Starts a server on 127.0.0.1. `route` gets the request target
    /// (path and query) and returns the reply.
    pub(crate) async fn start(route: impl Fn(&str) -> Reply + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let base = format!("http://{}", listener.local_addr().expect("address"));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let route: Arc<Route> = Arc::new(route);
        let task_seen = Arc::clone(&seen);
        let task = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    return;
                };
                let route = Arc::clone(&route);
                let seen = Arc::clone(&task_seen);
                tokio::spawn(async move {
                    let mut request = Vec::new();
                    let mut chunk = [0_u8; 4096];
                    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                        match stream.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(read) => request.extend_from_slice(&chunk[..read]),
                        }
                    }
                    let text = String::from_utf8_lossy(&request);
                    let mut lines = text.split("\r\n");
                    let target = lines
                        .next()
                        .and_then(|line| line.split(' ').nth(1))
                        .unwrap_or_default()
                        .to_string();
                    let headers = lines.collect::<Vec<_>>();
                    let header = |wanted: &str| {
                        headers.iter().find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case(wanted)
                                .then(|| value.trim().to_string())
                        })
                    };
                    seen.lock().expect("seen lock").push(Seen {
                        target: target.clone(),
                        cache_control: header("cache-control"),
                        prefer: header("prefer"),
                    });
                    let reply = route(&target);
                    let mut head = format!(
                        "HTTP/1.1 {} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n",
                        reply.status,
                        reply.body.len()
                    );
                    for (name, value) in &reply.headers {
                        head.push_str(&format!("{name}: {value}\r\n"));
                    }
                    head.push_str("\r\n");
                    let _ = stream.write_all(head.as_bytes()).await;
                    let _ = stream.write_all(reply.body.as_bytes()).await;
                    let _ = stream.shutdown().await;
                });
            }
        });
        Self { base, seen, task }
    }

    pub(crate) fn seen(&self) -> Vec<Seen> {
        self.seen.lock().expect("seen lock").clone()
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A searchset Bundle with `entries` as Condition matches and an optional next link.
pub(crate) fn condition_bundle(
    entries: &[serde_json::Value],
    next: Option<&str>,
) -> serde_json::Value {
    let mut bundle = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "entry": entries
            .iter()
            .map(|resource| serde_json::json!({"resource": resource, "search": {"mode": "match"}}))
            .collect::<Vec<_>>(),
    });
    if let Some(next) = next {
        bundle["link"] = serde_json::json!([{"relation": "next", "url": next}]);
    }
    bundle
}

/// A CapabilityStatement whose server Patient resource lists `params`.
pub(crate) fn capability(params: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "resourceType": "CapabilityStatement",
        "rest": [{"mode": "server", "resource": [{
            "type": "Patient",
            "searchParam": params
                .iter()
                .map(|name| serde_json::json!({"name": name, "type": "token"}))
                .collect::<Vec<_>>(),
        }]}],
    })
}

/// A synthetic Condition with an active clinical status.
pub(crate) fn condition(id: &str, text: &str) -> serde_json::Value {
    serde_json::json!({
        "resourceType": "Condition",
        "id": id,
        "clinicalStatus": {"coding": [{
            "system": "http://terminology.hl7.org/CodeSystem/condition-clinical",
            "code": "active"
        }]},
        "code": {"text": text, "coding": [{"system": "http://snomed.info/sct", "code": "000000", "display": text}]},
        "onsetDateTime": "2020-01-01"
    })
}
