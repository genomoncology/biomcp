use super::*;

pub(super) async fn middleware_put_settles_after_publication_before_releasing_key_lock() {
    let root = TempDirGuard::new("middleware-put-publication");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let origin = reqwest::Url::parse(&format!("http://{address}")).unwrap();
    let policy = provider_url_policy::ProviderUrlPolicy::test_fixture(
        provider_url_policy::ProviderUrlConsumer::GithubRelease,
        &origin,
    )
    .unwrap();
    let armed = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let manager = crate::cache::SizeAwareCacheManager::new(
        root.path().join("http"),
        crate::sources::tests::test_cache_config(root.path()),
    )
    .unwrap()
    .with_safe_return_observer({
        let (armed, release) = (Arc::clone(&armed), Arc::clone(&release));
        move || {
            let (armed, release) = (Arc::clone(&armed), Arc::clone(&release));
            async move {
                armed.notify_one();
                release.notified().await
            }
        }
    });
    // Occupy the process limiter slot to prove this paused clock stays isolated.
    crate::sources::rate_limit::wait_for_url_str(origin.as_str()).await;
    let client = crate::sources::finish_cached_http_client_with_limiter(
        SharedHttpClientKind::Default,
        Some(&policy),
        manager,
        crate::sources::rate_limit::RateLimitMiddleware::isolated_for_test,
    )
    .unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        let _ = stream.read(&mut request).await.unwrap();
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nCache-Control: public, max-age=60\r\nConnection: close\r\n\r\nok").await.unwrap();
    });
    let deadline = VariantArticleDeadline::from_now(std::time::Duration::from_secs(10));
    let driving = Arc::new(AtomicBool::new(true));
    let driver = tokio::spawn({
        let driving = Arc::clone(&driving);
        async move {
            while driving.load(Ordering::SeqCst) {
                tokio::task::yield_now().await
            }
        }
    });
    let request = with_variant_article_deadline(deadline.clone(), async {
        client
            .get(origin)
            .send_with_source_context(crate::error::SourceContext::narrow(
                crate::error::SourceProvider::OLS4,
            ))
            .await
    });
    tokio::pin!(request);
    tokio::select! {
        () = armed.notified() => {}
        result = &mut request => panic!("settled before publication pause: {result:?}"),
    }
    driving.store(false, Ordering::SeqCst);
    driver.await.unwrap();
    tokio::time::advance(std::time::Duration::from_secs(11)).await;
    assert!(deadline.is_exhausted());
    assert!(
        crate::cache::try_lock_cache_maintenance(root.path())
            .unwrap()
            .is_none()
    );
    assert!(deadline.run(std::future::pending::<()>()).await.is_err());
    release.notify_one();
    let result = request.await;
    assert!(result.is_ok(), "post-arm request must settle: {result:?}");
    assert!(
        crate::cache::try_lock_cache_maintenance(root.path())
            .unwrap()
            .is_some()
    );
    server.await.unwrap();
}
