use std::sync::LockResult;

pub(crate) fn recover_poison<T>(result: LockResult<T>) -> T {
    result.unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn poisoned_lock_returns_the_guarded_value() {
        let value = Arc::new(Mutex::new(41));
        let worker_value = Arc::clone(&value);
        let _ = std::thread::spawn(move || {
            let mut guard = worker_value.lock().expect("lock should start healthy");
            *guard += 1;
            panic!("poison test lock");
        })
        .join();

        assert_eq!(*recover_poison(value.lock()), 42);
    }

    #[test]
    fn every_reconstructible_lock_uses_the_shared_helper() {
        let ddinter = include_str!("../sources/ddinter.rs");
        let gencc = include_str!("../sources/gencc/store.rs");
        assert_eq!(ddinter.matches("recover_poison(").count(), 3);
        assert_eq!(gencc.matches("recover_poison(").count(), 2);
    }
}
