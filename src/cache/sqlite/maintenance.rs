//! Age and pressure cleanup keeps its writer lease only when work is due.
use super::*;

const STARTUP_INTERVAL_MS: i64 = 86_400_000;

#[derive(Clone, Copy)]
pub(crate) enum Maintenance {
    Startup,
    Pressure,
    Clean,
}

pub(crate) fn maintain(
    store: &mut Store,
    config: &crate::cache::ResolvedCacheConfig,
    max_age: Option<Duration>,
    max_size: Option<u64>,
    dry_run: bool,
    mode: Maintenance,
) -> Result<crate::cache::CleanReport> {
    let now = now_ms();
    let space =
        crate::cache::inspect_filesystem_space(&config.cache_root).map_err(io::Error::other)?;
    if matches!(mode, Maintenance::Startup)
        && max_age.is_none()
        && max_size.is_none()
        && !dry_run
        && startup_not_due(store, config, now, space)?
    {
        return Ok(crate::cache::CleanReport {
            dry_run,
            orphans_removed: 0,
            entries_removed: 0,
            bytes_freed: 0,
            provider_capture_bytes_freed: 0,
            errors: Vec::new(),
        });
    }
    let mut report=store.transaction(|tx, root| {
        let last: i64 = tx.query_row("SELECT last_clean FROM cache_meta WHERE id=1", [], |r|r.get(0)).map_err(sqlite_error)?;
        let due = match mode {Maintenance::Startup=>now.saturating_sub(last)>=STARTUP_INTERVAL_MS,Maintenance::Pressure=>false,Maintenance::Clean=>true};
        let age = max_age.or(if due { Some(config.max_age) } else { None });
        let cutoff = age.map(|age|now.saturating_sub(age.as_millis().min(i64::MAX as u128) as i64));
        let mut total: u64 = tx.query_row("SELECT optional_bytes FROM cache_meta WHERE id=1",[],|r|read_u64(r,0)).map_err(sqlite_error)?;
        let limit = max_size.unwrap_or(config.max_size);
        let expired = if due {
            let mut statement = tx.prepare("SELECT namespace,key FROM entries WHERE (namespace!='capture' AND expires IS NOT NULL AND expires<=?1) OR (namespace='http' AND ?2 IS NOT NULL AND created<=?2)").map_err(sqlite_error)?;
            statement.query_map(params![now,cutoff],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(sqlite_error)?
                .collect::<std::result::Result<Vec<_>,_>>().map_err(sqlite_error)?
        } else { Vec::new() };
        let mut selected = expired;
        let mut planned: u64 = 0;
        for (namespace,key) in &selected {
            planned += tx.query_row("SELECT bytes FROM entries WHERE namespace=?1 AND key=?2",params![namespace,key],|r|read_u64(r,0)).map_err(sqlite_error)?;
        }
        total=total.saturating_sub(planned);
        let mut limits=config.clone();limits.max_size=limit;
        let estimated_space=crate::cache::FilesystemSpace{available_bytes:space.available_bytes.saturating_add(planned),total_bytes:space.total_bytes};
        let evaluation=crate::cache::evaluate_cache_usage(crate::cache::CacheUsage{referenced_blob_bytes:total,orphan_blob_bytes:0},&limits,estimated_space);
        let target=evaluation.effective_max_size;
        let mut selected_set=selected.iter().cloned().collect::<std::collections::HashSet<_>>();
        if total>target {
            let mut statement=tx.prepare("SELECT namespace,key,bytes FROM entries WHERE namespace!='capture' ORDER BY last_access").map_err(sqlite_error)?;
            let rows=statement.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,read_u64(r,2)?))).map_err(sqlite_error)?;
            for row in rows {
                if total<=target { break; }
                let (namespace,key,bytes)=row.map_err(sqlite_error)?;
                if !selected_set.insert((namespace.clone(),key.clone())) {continue;}
                selected.push((namespace,key)); total=total.saturating_sub(bytes);planned+=bytes;
            }
        }
        let mut capture_bytes=0;
        for (namespace,key) in &selected {
            if namespace=="capture" {
                capture_bytes+=tx.query_row("SELECT bytes FROM entries WHERE namespace=?1 AND key=?2",params![namespace,key],|r|read_u64(r,0)).map_err(sqlite_error)?;
            }
            if !dry_run { delete_record(tx,root,namespace,key)?; }
        }
        if !dry_run && due && matches!(mode,Maintenance::Startup) { tx.execute("UPDATE cache_meta SET last_clean=?1 WHERE id=1",[now]).map_err(sqlite_error)?; }
        Ok(crate::cache::CleanReport { dry_run,orphans_removed:0,entries_removed:selected.len(),bytes_freed:planned,provider_capture_bytes_freed:capture_bytes,errors:Vec::new() })
    })?;
    if matches!(mode, Maintenance::Clean) {
        let captures =
            crate::cache::provider_capture::ProviderCaptureStore::new(&config.cache_root);
        let freed = captures.clean_in_store(store, dry_run)?;
        report.provider_capture_bytes_freed = freed;
        report.bytes_freed = report.bytes_freed.saturating_add(freed);
    }
    if !dry_run {
        store.collect_blobs()?;
        if report.bytes_freed > 0 {
            store.release_space()?;
        }
    }
    Ok(report)
}

fn startup_not_due(
    store: &Store,
    config: &crate::cache::ResolvedCacheConfig,
    now: i64,
    space: crate::cache::FilesystemSpace,
) -> Result<bool> {
    let (last_clean, optional_bytes, garbage): (i64, u64, bool) = store.connection().query_row(
        "SELECT last_clean,optional_bytes,EXISTS(SELECT 1 FROM blob_gc) FROM cache_meta WHERE id=1",
        [], |row| Ok((row.get(0)?, read_u64(row, 1)?, row.get(2)?)),
    ).map_err(sqlite_error)?;
    let evaluation = crate::cache::evaluate_cache_usage(
        crate::cache::CacheUsage {
            referenced_blob_bytes: optional_bytes,
            orphan_blob_bytes: 0,
        },
        config,
        space,
    );
    Ok(now.saturating_sub(last_clean) < STARTUP_INTERVAL_MS
        && !garbage
        && optional_bytes <= evaluation.effective_max_size)
}
