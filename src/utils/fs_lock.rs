//! Non-blocking std file locks with fs2's error shape: contention reports
//! `io::ErrorKind::WouldBlock`, so every deadline poll that matches on the
//! kind shares this one mapping instead of restating it per call site. The
//! deadline and contention tests in the cache, article-search and GenCC
//! homes cover the mapping through their real entry points.

use std::fs::File;
use std::fs::TryLockError;
use std::io;

pub(crate) fn try_lock_exclusive(file: &File) -> io::Result<()> {
    file.try_lock().map_err(contention)
}

pub(crate) fn try_lock_shared(file: &File) -> io::Result<()> {
    file.try_lock_shared().map_err(contention)
}

fn contention(error: TryLockError) -> io::Error {
    match error {
        TryLockError::WouldBlock => io::Error::from(io::ErrorKind::WouldBlock),
        TryLockError::Error(error) => error,
    }
}
