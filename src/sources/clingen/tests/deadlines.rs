use super::*;
use reqwest_middleware::{Middleware, Next};

struct CompletedResponse(tokio::sync::mpsc::UnboundedSender<()>);

#[async_trait::async_trait]
impl Middleware for CompletedResponse {
    async fn handle(
        &self,
        request: reqwest::Request,
        extensions: &mut http::Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let response = next.run(request, extensions).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.bytes().await?;
        let mut response = http::Response::builder().status(status);
        *response.headers_mut().unwrap() = headers;
        let response = response.body(reqwest::Body::from(bytes)).unwrap().into();
        self.0.send(()).unwrap();
        Ok(response)
    }
}

impl Fixture {
    /// Advance the deadline only after both complete bodies and the blocked
    /// route have arrived. The pending response cannot race a wall clock.
    pub(super) async fn timeout_context(&self, symbol: &str) -> GeneClinGen {
        let (completed, mut responses) = tokio::sync::mpsc::unbounded_channel();
        let client = ClinGenClient::with_client_and_base(
            reqwest_middleware::ClientBuilder::from_client(crate::sources::test_client().unwrap())
                .with(CompletedResponse(completed))
                .build(),
            self.base.clone(),
        );
        let request = client.gene_context(symbol, Duration::from_millis(40));
        tokio::pin!(request);
        let driver = tokio::spawn(async {
            loop {
                tokio::task::yield_now().await;
            }
        });
        let readiness = async {
            self.blocked_started.notified().await;
            for _ in 0..2 {
                responses.recv().await.expect("completed fixture body");
            }
        };
        tokio::select! {
            () = readiness => {}
            result = &mut request => panic!("context settled before readiness: {result:?}"),
        }
        driver.abort();
        let _ = driver.await;
        tokio::time::advance(Duration::from_millis(41)).await;
        request.await.expect("partial ClinGen context")
    }
}
