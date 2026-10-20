use super::TestHttpServer;
use std::io::{self, Read, Write};
use std::net::TcpStream as Client;
use std::thread;
use std::time::Duration;

#[test]
fn parallel_fixtures_serve_after_idle_partial_and_disconnected_clients() {
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                let mut server = TestHttpServer::respond(|request| async move {
                    assert!(request.starts_with(b"GET /survivor HTTP/1.1\r\n"));
                    Ok(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
                            .to_vec(),
                    )
                })
                .unwrap();
                let address = server.base.trim_start_matches("http://");
                let _idle = Client::connect(address).unwrap();
                let mut partial = Client::connect(address).unwrap();
                partial.write_all(b"GET /").unwrap();
                for _ in 0..12 {
                    let mut disconnected = Client::connect(address).unwrap();
                    disconnected.write_all(b"GET /").unwrap();
                }
                let mut survivor = Client::connect(address).unwrap();
                survivor
                    .set_read_timeout(Some(Duration::from_secs(60)))
                    .unwrap();
                survivor.write_all(b"GET /survivor HTTP/1.1\r\n").unwrap();
                survivor.write_all(b"Host: localhost\r\n\r\n").unwrap();
                let mut response = String::new();
                survivor.read_to_string(&mut response).unwrap();
                assert!(response.ends_with("\r\n\r\nok"), "{response}");
                server.finish().unwrap();
            });
        }
    });
}

#[test]
fn teardown_reports_handler_errors_and_panics_but_drop_preserves_unwinding() {
    for panics in [false, true] {
        let (sent, received) = std::sync::mpsc::channel();
        let mut server = TestHttpServer::spawn(move |_| {
            let sent = sent.clone();
            async move {
                sent.send(()).unwrap();
                assert!(!panics, "deliberate fixture panic");
                Err(io::Error::other("deliberate fixture error"))
            }
        })
        .unwrap();
        let _client = Client::connect(server.base.trim_start_matches("http://")).unwrap();
        received.recv_timeout(Duration::from_secs(60)).unwrap();
        assert!(server.finish().is_err());
        server.finish().unwrap();
    }
    let original = std::panic::catch_unwind(|| {
        let _server = TestHttpServer::spawn(|_| async { std::future::pending().await }).unwrap();
        panic!("original test assertion");
    })
    .unwrap_err();
    assert_eq!(
        original.downcast_ref::<&str>(),
        Some(&"original test assertion")
    );
}
