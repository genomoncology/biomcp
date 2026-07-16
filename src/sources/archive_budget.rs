use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub(super) struct ArchiveLimits {
    pub(super) max_entries: u64,
    pub(super) max_member_bytes: u64,
    pub(super) max_total_bytes: u64,
    pub(super) max_metadata_bytes: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("archive resource limit or metadata policy was exceeded")]
pub(super) struct ArchiveBudgetError;

pub(super) enum ArchiveEntry {
    Metadata,
    Regular(PathBuf),
    Directory(PathBuf),
}

pub(super) struct ArchiveBudget {
    limits: ArchiveLimits,
    entries: u64,
    total_bytes: u64,
    pending_metadata: bool,
    pending_path: Option<PathBuf>,
}

impl ArchiveBudget {
    pub(super) fn new(limits: ArchiveLimits) -> Self {
        Self {
            limits,
            entries: 0,
            total_bytes: 0,
            pending_metadata: false,
            pending_path: None,
        }
    }

    pub(super) fn account<R: Read>(
        &mut self,
        entry: &mut tar::Entry<'_, R>,
    ) -> Result<ArchiveEntry, ArchiveBudgetError> {
        self.entries = self.entries.checked_add(1).ok_or(ArchiveBudgetError)?;
        if self.entries > self.limits.max_entries {
            return Err(ArchiveBudgetError);
        }

        let size = entry.header().size().map_err(|_| ArchiveBudgetError)?;
        self.total_bytes = self
            .total_bytes
            .checked_add(size)
            .ok_or(ArchiveBudgetError)?;
        if self.total_bytes > self.limits.max_total_bytes {
            return Err(ArchiveBudgetError);
        }

        let entry_type = entry.header().entry_type();
        if entry_type.is_gnu_longname() {
            self.begin_metadata(size)?;
            let path = read_metadata(entry, size)?;
            self.pending_path = Some(parse_gnu_path(&path)?);
            return Ok(ArchiveEntry::Metadata);
        }
        if entry_type.is_pax_local_extensions() {
            self.begin_metadata(size)?;
            let metadata = read_metadata(entry, size)?;
            self.pending_path = Some(parse_pax_path(&metadata)?.ok_or(ArchiveBudgetError)?);
            return Ok(ArchiveEntry::Metadata);
        }
        if entry_type.is_pax_global_extensions()
            || entry_type.is_gnu_longlink()
            || entry_type.is_gnu_sparse()
        {
            return Err(ArchiveBudgetError);
        }

        let path = match self.pending_path.take() {
            Some(path) => path,
            None => entry.path().map_err(|_| ArchiveBudgetError)?.into_owned(),
        };
        self.pending_metadata = false;

        if entry_type.is_file() {
            if size > self.limits.max_member_bytes {
                return Err(ArchiveBudgetError);
            }
            return Ok(ArchiveEntry::Regular(path));
        }
        if entry_type.is_dir() {
            return Ok(ArchiveEntry::Directory(path));
        }
        Err(ArchiveBudgetError)
    }

    fn begin_metadata(&mut self, size: u64) -> Result<(), ArchiveBudgetError> {
        if self.pending_metadata || size > self.limits.max_metadata_bytes {
            return Err(ArchiveBudgetError);
        }
        self.pending_metadata = true;
        Ok(())
    }

    pub(super) fn finish(self) -> Result<(), ArchiveBudgetError> {
        if self.pending_metadata {
            return Err(ArchiveBudgetError);
        }
        Ok(())
    }
}

fn read_metadata<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    size: u64,
) -> Result<Vec<u8>, ArchiveBudgetError> {
    let capacity = usize::try_from(size).map_err(|_| ArchiveBudgetError)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| ArchiveBudgetError)?;
    entry
        .read_to_end(&mut bytes)
        .map_err(|_| ArchiveBudgetError)?;
    if bytes.len() != capacity {
        return Err(ArchiveBudgetError);
    }
    Ok(bytes)
}

fn parse_gnu_path(bytes: &[u8]) -> Result<PathBuf, ArchiveBudgetError> {
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    let path = std::str::from_utf8(bytes).map_err(|_| ArchiveBudgetError)?;
    if path.is_empty() {
        return Err(ArchiveBudgetError);
    }
    Ok(PathBuf::from(path))
}

fn parse_pax_path(bytes: &[u8]) -> Result<Option<PathBuf>, ArchiveBudgetError> {
    let mut cursor = 0;
    let mut seen = HashSet::new();
    let mut path = None;
    while cursor < bytes.len() {
        let space = bytes[cursor..]
            .iter()
            .position(|byte| *byte == b' ')
            .map(|offset| cursor + offset)
            .ok_or(ArchiveBudgetError)?;
        let length = std::str::from_utf8(&bytes[cursor..space])
            .map_err(|_| ArchiveBudgetError)?
            .parse::<usize>()
            .map_err(|_| ArchiveBudgetError)?;
        let end = cursor.checked_add(length).ok_or(ArchiveBudgetError)?;
        if end > bytes.len() || end <= space + 1 || bytes[end - 1] != b'\n' {
            return Err(ArchiveBudgetError);
        }
        let record = &bytes[space + 1..end - 1];
        let equals = record
            .iter()
            .position(|byte| *byte == b'=')
            .ok_or(ArchiveBudgetError)?;
        let key = &record[..equals];
        let value = &record[equals + 1..];
        if !seen.insert(key.to_vec())
            || key == b"size"
            || key == b"linkpath"
            || key.starts_with(b"GNU.sparse")
        {
            return Err(ArchiveBudgetError);
        }
        if key == b"path" {
            let value = std::str::from_utf8(value).map_err(|_| ArchiveBudgetError)?;
            if value.is_empty() {
                return Err(ArchiveBudgetError);
            }
            path = Some(PathBuf::from(value));
        }
        cursor = end;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn tar_bytes(contents: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut bytes);
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "file.bin", contents)
                .expect("append entry");
            builder.finish().expect("finish tar");
        }
        bytes
    }

    fn limits(max_member_bytes: u64, max_total_bytes: u64) -> ArchiveLimits {
        ArchiveLimits {
            max_entries: 1,
            max_member_bytes,
            max_total_bytes,
            max_metadata_bytes: 16,
        }
    }

    fn raw_tar(entries: &[(&str, tar::EntryType, &[u8])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (path, entry_type, contents) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_path(path).expect("set raw entry path");
            header.set_size(contents.len() as u64);
            header.set_entry_type(*entry_type);
            header.set_mode(0o644);
            header.set_cksum();
            bytes.extend_from_slice(header.as_bytes());
            bytes.extend_from_slice(contents);
            bytes.resize(bytes.len().next_multiple_of(512), 0);
        }
        bytes.extend_from_slice(&[0; 1024]);
        bytes
    }

    fn pax_record(key: &str, value: &str) -> Vec<u8> {
        let body = format!("{key}={value}\n");
        let mut length = body.len() + 2;
        loop {
            let record = format!("{length} {body}");
            if record.len() == length {
                return record.into_bytes();
            }
            length = record.len();
        }
    }

    #[test]
    fn regular_member_accepts_exact_limits_and_rejects_max_plus_one() {
        let exact = tar_bytes(b"abc");
        let mut archive = tar::Archive::new(Cursor::new(exact));
        let mut entry = archive
            .entries()
            .unwrap()
            .raw(true)
            .next()
            .unwrap()
            .unwrap();
        let accounted = ArchiveBudget::new(limits(3, 3)).account(&mut entry);
        assert!(matches!(accounted, Ok(ArchiveEntry::Regular(_))));

        let over = tar_bytes(b"abcd");
        let mut archive = tar::Archive::new(Cursor::new(over));
        let mut entry = archive
            .entries()
            .unwrap()
            .raw(true)
            .next()
            .unwrap()
            .unwrap();
        assert!(
            ArchiveBudget::new(limits(3, 4))
                .account(&mut entry)
                .is_err()
        );
    }

    #[test]
    fn entry_and_total_accounting_reject_checked_overflow() {
        let bytes = tar_bytes(b"x");
        let mut archive = tar::Archive::new(Cursor::new(bytes));
        let mut entry = archive
            .entries()
            .unwrap()
            .raw(true)
            .next()
            .unwrap()
            .unwrap();
        let mut entry_overflow = ArchiveBudget::new(limits(1, u64::MAX));
        entry_overflow.entries = u64::MAX;
        assert!(entry_overflow.account(&mut entry).is_err());

        let bytes = tar_bytes(b"x");
        let mut archive = tar::Archive::new(Cursor::new(bytes));
        let mut entry = archive
            .entries()
            .unwrap()
            .raw(true)
            .next()
            .unwrap()
            .unwrap();
        let mut total_overflow = ArchiveBudget::new(limits(1, u64::MAX));
        total_overflow.total_bytes = u64::MAX;
        assert!(total_overflow.account(&mut entry).is_err());
    }

    #[test]
    fn aggregate_and_entry_limits_accept_exact_boundaries_and_reject_max_plus_one() {
        let bytes = raw_tar(&[
            ("one", tar::EntryType::Regular, b"a"),
            ("two", tar::EntryType::Regular, b"b"),
        ]);
        let archive_limits = ArchiveLimits {
            max_entries: 2,
            max_member_bytes: 1,
            max_total_bytes: 2,
            max_metadata_bytes: 16,
        };
        let mut archive = tar::Archive::new(Cursor::new(&bytes));
        let mut budget = ArchiveBudget::new(archive_limits);
        for entry in archive.entries().unwrap().raw(true) {
            assert!(matches!(
                budget.account(&mut entry.unwrap()),
                Ok(ArchiveEntry::Regular(_))
            ));
        }
        assert!(budget.finish().is_ok());

        for limits in [
            ArchiveLimits {
                max_entries: 1,
                ..archive_limits
            },
            ArchiveLimits {
                max_total_bytes: 1,
                ..archive_limits
            },
        ] {
            let mut archive = tar::Archive::new(Cursor::new(&bytes));
            let mut budget = ArchiveBudget::new(limits);
            let mut entries = archive.entries().unwrap().raw(true);
            assert!(
                budget
                    .account(&mut entries.next().unwrap().unwrap())
                    .is_ok()
            );
            assert!(
                budget
                    .account(&mut entries.next().unwrap().unwrap())
                    .is_err()
            );
        }
    }

    #[test]
    fn bounded_metadata_sets_one_effective_path_and_then_resets() {
        let long_path = "nested/very-long-name.txt";
        let mut long_name = long_path.as_bytes().to_vec();
        long_name.push(0);
        let bytes = raw_tar(&[
            ("././@LongLink", tar::EntryType::GNULongName, &long_name),
            ("placeholder", tar::EntryType::Regular, b"a"),
            ("ordinary.txt", tar::EntryType::Regular, b"b"),
        ]);
        let limits = ArchiveLimits {
            max_entries: 3,
            max_member_bytes: 1,
            max_total_bytes: long_name.len() as u64 + 2,
            max_metadata_bytes: long_name.len() as u64,
        };
        let mut archive = tar::Archive::new(Cursor::new(bytes));
        let mut entries = archive.entries().unwrap().raw(true);
        let mut budget = ArchiveBudget::new(limits);
        assert!(matches!(
            budget.account(&mut entries.next().unwrap().unwrap()),
            Ok(ArchiveEntry::Metadata)
        ));
        assert!(matches!(
            budget.account(&mut entries.next().unwrap().unwrap()),
            Ok(ArchiveEntry::Regular(path)) if path == std::path::Path::new(long_path)
        ));
        assert!(matches!(
            budget.account(&mut entries.next().unwrap().unwrap()),
            Ok(ArchiveEntry::Regular(path)) if path == std::path::Path::new("ordinary.txt")
        ));
        assert!(budget.finish().is_ok());
    }

    #[test]
    fn metadata_limits_ambiguity_and_unsupported_entry_types_are_rejected() {
        let metadata = b"long-name\0";
        let bytes = raw_tar(&[("././@LongLink", tar::EntryType::GNULongName, metadata)]);
        let mut archive = tar::Archive::new(Cursor::new(bytes));
        let mut entry = archive
            .entries()
            .unwrap()
            .raw(true)
            .next()
            .unwrap()
            .unwrap();
        let metadata_limits = ArchiveLimits {
            max_entries: 1,
            max_member_bytes: 1,
            max_total_bytes: metadata.len() as u64,
            max_metadata_bytes: metadata.len() as u64 - 1,
        };
        assert!(
            ArchiveBudget::new(metadata_limits)
                .account(&mut entry)
                .is_err()
        );

        let bytes = raw_tar(&[
            ("././@LongLink", tar::EntryType::GNULongName, metadata),
            ("././@LongLink", tar::EntryType::GNULongName, metadata),
        ]);
        let mut archive = tar::Archive::new(Cursor::new(bytes));
        let mut entries = archive.entries().unwrap().raw(true);
        let mut budget = ArchiveBudget::new(ArchiveLimits {
            max_entries: 2,
            max_member_bytes: 1,
            max_total_bytes: (metadata.len() * 2) as u64,
            max_metadata_bytes: metadata.len() as u64,
        });
        assert!(
            budget
                .account(&mut entries.next().unwrap().unwrap())
                .is_ok()
        );
        assert!(
            budget
                .account(&mut entries.next().unwrap().unwrap())
                .is_err()
        );

        for entry_type in [
            tar::EntryType::Link,
            tar::EntryType::Symlink,
            tar::EntryType::Char,
            tar::EntryType::Block,
        ] {
            let bytes = raw_tar(&[("unsafe", entry_type, b"")]);
            let mut archive = tar::Archive::new(Cursor::new(bytes));
            let mut entry = archive
                .entries()
                .unwrap()
                .raw(true)
                .next()
                .unwrap()
                .unwrap();
            assert!(
                ArchiveBudget::new(limits(0, 0))
                    .account(&mut entry)
                    .is_err()
            );
        }
    }

    #[test]
    fn pax_paths_are_accepted_but_size_sparse_and_duplicate_metadata_are_rejected() {
        assert_eq!(
            parse_pax_path(&pax_record("path", "nested/article.xml")).unwrap(),
            Some(PathBuf::from("nested/article.xml"))
        );
        assert!(parse_pax_path(b"10 size=1\n").is_err());
        assert!(parse_pax_path(b"20 GNU.sparse.size=1\n").is_err());
        let mut duplicate = pax_record("path", "one");
        duplicate.extend_from_slice(&pax_record("path", "two"));
        assert!(parse_pax_path(&duplicate).is_err());
    }
}
