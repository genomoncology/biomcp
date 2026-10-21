//! Snapshot reads validate the selected body without taking a writer lease.
use super::*;

impl Store {
    pub(crate) fn get(&mut self, namespace: &str, key: &str) -> Result<Option<Record>> {
        let _span = crate::cache::timing::Span::new("get");
        for attempt in 0..=1 {
            let begin = crate::cache::timing::Span::new("begin");
            let mut connection = self.connection();
            let tx = connection
                .transaction_with_behavior(TransactionBehavior::Deferred)
                .map_err(sqlite_error)?;
            drop(begin);
            let result = read_snapshot(&tx, &self.root, namespace, key);
            // An evictor may unlink an external blob after this WAL snapshot
            // selects its row. Drop the snapshot and revalidate once against the
            // current row; never combine one row's metadata with another body.
            if result
                .as_ref()
                .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
            {
                drop(tx);
                if attempt == 0 {
                    continue;
                }
                return Ok(None);
            }
            let record = result?;
            tx.commit().map_err(sqlite_error)?;
            if let Some((record, last_access)) = record {
                let now = now_ms();
                if last_access <= now.saturating_sub(3_600_000) {
                    // LRU accounting does not upgrade the read snapshot. A
                    // contended advisory touch cannot invalidate a verified hit.
                    let _ = connection.execute(
                        "UPDATE entries SET last_access=?3 WHERE namespace=?1 AND key=?2 AND last_access<=?3-3600000",
                        params![namespace, key, now],
                    );
                }
                return Ok(Some(record));
            }
            return Ok(None);
        }
        unreachable!("the bounded read retry always returns")
    }
}

pub(crate) fn read_record(
    tx: &Transaction<'_>,
    root: &Path,
    namespace: &str,
    key: &str,
) -> Result<Option<Record>> {
    let record = read_snapshot(tx, root, namespace, key)?;
    if record.is_some() {
        tx.execute("UPDATE entries SET last_access=?3 WHERE namespace=?1 AND key=?2 AND last_access<=?3-3600000",
            params![namespace, key, now_ms()]).map_err(sqlite_error)?;
    }
    Ok(record.map(|(record, _)| record))
}

fn read_snapshot(
    tx: &Transaction<'_>,
    root: &Path,
    namespace: &str,
    key: &str,
) -> Result<Option<(Record, i64)>> {
    let _span = crate::cache::timing::Span::new("read");
    let record = tx
        .query_row(
            "SELECT metadata, body, blob, body_hash, last_access FROM entries WHERE namespace=?1 AND key=?2",
            params![namespace, key],
            |r| {
                Ok((
                    r.get::<_, Vec<u8>>(0)?,
                    r.get::<_, Option<Vec<u8>>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()
        .map_err(sqlite_error)?;
    let Some((metadata, body, blob, body_hash, last_access)) = record else {
        return Ok(None);
    };
    let body = match (body, blob) {
        (Some(body), None) => body,
        (None, Some(blob)) => blobs::read(root, &blob)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid cache body",
            ));
        }
    };
    if format!("{:x}", Sha256::digest(&body)) != body_hash {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cache body integrity",
        ));
    }
    Ok(Some((
        Record {
            key: key.to_owned(),
            metadata,
            body,
        },
        last_access,
    )))
}
