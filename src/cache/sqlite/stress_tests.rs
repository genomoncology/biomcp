//! Separate processes exercise blob publication, eviction and snapshot readers.
use super::*;

#[test]
#[ignore = "invoked only as the evictor child of the shared-cache contract"]
fn eviction_child_process() {
    use std::io::{Read, Write};
    let Some(root) = std::env::var_os("BIOMCP_TEST_EVICTOR_ROOT") else {
        return;
    };
    let mut store = Store::open(Path::new(&root)).unwrap();
    let address = std::env::var("BIOMCP_TEST_EVICTOR_CONTROL").unwrap();
    let mut control = std::net::TcpStream::connect(address).unwrap();
    publish_generation(&mut store, 1);
    crate::test_support::signal_ready_on_raw_stdout("EVICTOR_READY");
    let mut command = [0];
    control.read_exact(&mut command).unwrap();
    assert_eq!(command, [1]);
    store.delete("http", "shared").unwrap();
    publish_generation(&mut store, 2);
    control.write_all(&[2]).unwrap();
    control.read_exact(&mut command).unwrap();
    assert_eq!(command, [2]);
    for generation in 3_u8..=50 {
        publish_generation(&mut store, generation);
        if generation % 2 == 0 {
            store.delete("http", "shared").unwrap();
        }
    }
    publish_generation(&mut store, 51);
    control.write_all(&[51]).unwrap();
    control.read_exact(&mut command).unwrap();
    assert_eq!(command, [3]);
}

fn publish_generation(store: &mut Store, generation: u8) {
    let body = vec![generation; blobs::INLINE_LIMIT + 1];
    store
        .put("http", "shared", "shared", &[generation], &body, None)
        .unwrap();
}

fn assert_generation(record: &Record, expected: u8) {
    assert_eq!(record.metadata, [expected]);
    assert_eq!(record.body.len(), blobs::INLINE_LIMIT + 1);
    assert!(record.body.iter().all(|byte| *byte == expected));
}

#[test]
fn separate_process_eviction_returns_only_coherent_bodies_or_misses() {
    use std::io::{Read, Write};
    let root = crate::test_support::TempDirGuard::new("sqlite-process-eviction");
    let mut reader = Store::open(root.path()).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let mut child = crate::test_support::SignaledChild::spawn(
        &std::env::current_exe().unwrap(),
        &[
            "--exact",
            "cache::sqlite::stress_tests::eviction_child_process",
            "--ignored",
            "--nocapture",
        ],
        [
            ("BIOMCP_TEST_EVICTOR_ROOT", root.path().as_os_str()),
            (
                "BIOMCP_TEST_EVICTOR_CONTROL",
                std::ffi::OsStr::new(&address),
            ),
        ],
        "EVICTOR_READY",
    );
    let (mut control, _) = listener.accept().unwrap();
    let mut replacement = control.try_clone().unwrap();
    let _hook = read::before_next_blob_read(move || {
        replacement.write_all(&[1]).unwrap();
        let mut acknowledgement = [0];
        replacement.read_exact(&mut acknowledgement).unwrap();
        assert_eq!(acknowledgement, [2]);
    });
    // The old snapshot selected generation1. Its blob is removed before open,
    // so returning coherent generation2 requires the fresh-snapshot retry.
    assert_generation(&reader.get("http", "shared").unwrap().unwrap(), 2);
    assert!(child.child.try_wait().unwrap().is_none());
    control.write_all(&[2]).unwrap();
    for _ in 0..192 {
        match reader
            .get("http", "shared")
            .expect("eviction must yield a verified row or a miss")
        {
            Some(record) => {
                assert_generation(&record, record.metadata[0]);
            }
            None => {}
        }
    }
    let mut acknowledgement = [0];
    control.read_exact(&mut acknowledgement).unwrap();
    assert_eq!(acknowledgement, [51]);
    assert_generation(&reader.get("http", "shared").unwrap().unwrap(), 51);
    assert!(child.child.try_wait().unwrap().is_none());
    control.write_all(&[3]).unwrap();
    child.release();
    assert!(
        child.child.wait().unwrap().success(),
        "evictor process failed"
    );
    assert_eq!(
        reader
            .connection()
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
#[ignore = "invoked only by the writer-held read contract"]
fn writer_child_process() {
    let Some(root) = std::env::var_os("BIOMCP_TEST_EVICTOR_ROOT") else {
        return;
    };
    let store = Store::open(Path::new(&root)).unwrap();
    let mut connection = store.connection();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    transaction
        .execute("DELETE FROM entries WHERE key='body'", [])
        .unwrap();
    crate::test_support::signal_ready_on_raw_stdout("WRITER_READY");
    crate::test_support::block_until_stdin_closes();
    drop(transaction);
}
