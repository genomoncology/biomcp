//! Separate processes exercise blob publication, eviction and snapshot readers.
use super::*;

#[test]
#[ignore = "invoked only as the evictor child of the shared-cache contract"]
fn eviction_child_process() {
    let Some(root) = std::env::var_os("BIOMCP_TEST_EVICTOR_ROOT") else {
        return;
    };
    let mut store = Store::open(Path::new(&root)).unwrap();
    crate::test_support::signal_ready_on_raw_stdout("EVICTOR_READY");
    crate::test_support::block_until_stdin_closes();
    for generation in 1_u8..=48 {
        let body = vec![generation; blobs::INLINE_LIMIT + 1];
        store
            .put("http", "shared", "shared", &[generation], &body, None)
            .unwrap();
        if generation % 2 == 0 {
            store.delete("http", "shared").unwrap();
        }
    }
}

#[test]
fn separate_process_eviction_returns_only_coherent_bodies_or_misses() {
    let root = crate::test_support::TempDirGuard::new("sqlite-process-eviction");
    let mut reader = Store::open(root.path()).unwrap();
    reader
        .put(
            "http",
            "shared",
            "shared",
            &[0],
            &vec![0; blobs::INLINE_LIMIT + 1],
            None,
        )
        .unwrap();
    let mut child = crate::test_support::SignaledChild::spawn(
        &std::env::current_exe().unwrap(),
        &[
            "--exact",
            "cache::sqlite::stress_tests::eviction_child_process",
            "--ignored",
            "--nocapture",
        ],
        [("BIOMCP_TEST_EVICTOR_ROOT", root.path().as_os_str())],
        "EVICTOR_READY",
    );
    assert_eq!(reader.get("http", "shared").unwrap().unwrap().metadata, [0]);
    child.release();
    let mut hits = 0;
    let mut misses = 0;
    for _ in 0..192 {
        match reader
            .get("http", "shared")
            .expect("eviction must yield a verified row or a miss")
        {
            Some(record) => {
                assert_eq!(record.metadata.len(), 1);
                assert_eq!(record.body.len(), blobs::INLINE_LIMIT + 1);
                assert!(record.body.iter().all(|byte| *byte == record.metadata[0]));
                hits += 1;
            }
            None => misses += 1,
        }
    }
    assert!(
        child.child.wait().unwrap().success(),
        "evictor process failed"
    );
    assert_eq!(hits + misses, 192);
    println!("cross-process verified hits={hits} misses={misses}");
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
