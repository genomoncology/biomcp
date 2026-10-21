//! One live SQLite connection per canonical cache root in this process.
use super::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

type Registry = HashMap<PathBuf, Weak<Mutex<ManagedConnection>>>;
pub(super) type SharedConnection = Arc<Mutex<ManagedConnection>>;

pub(super) struct ManagedConnection(Option<Connection>);

impl Drop for ManagedConnection {
    fn drop(&mut self) {
        let _span = crate::cache::timing::Span::new("close");
        if let Some(connection) = self.0.take() {
            let _ = connection.close();
        }
    }
}

pub(crate) struct ConnectionAccess<'a>(MutexGuard<'a, ManagedConnection>);

impl Deref for ConnectionAccess<'_> {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.0.0.as_ref().expect("live cache connection")
    }
}

impl DerefMut for ConnectionAccess<'_> {
    fn deref_mut(&mut self) -> &mut Connection {
        self.0.0.as_mut().expect("live cache connection")
    }
}

pub(super) fn lock(connection: &SharedConnection) -> ConnectionAccess<'_> {
    ConnectionAccess(crate::utils::sync::recover_poison(connection.lock()))
}

pub(super) fn open(
    root: &Path,
    create: impl FnOnce() -> Result<Connection>,
) -> Result<SharedConnection> {
    static CONNECTIONS: OnceLock<Mutex<Registry>> = OnceLock::new();
    let mut registry =
        crate::utils::sync::recover_poison(CONNECTIONS.get_or_init(Mutex::default).lock());
    if let Some(connection) = registry.get(root).and_then(Weak::upgrade) {
        return Ok(connection);
    }
    registry.retain(|_, connection| connection.strong_count() > 0);
    let connection = Arc::new(Mutex::new(ManagedConnection(Some(create()?))));
    registry.insert(root.to_owned(), Arc::downgrade(&connection));
    Ok(connection)
}
