//! Test-only observation of the real client admission and consumer use boundaries.
use super::MyChemHit;
use std::sync::{Arc, Mutex, OnceLock};
type Observer = Arc<dyn Fn(&[MyChemHit], &'static str) + Send + Sync>;
static OBSERVER: OnceLock<Mutex<Option<Observer>>> = OnceLock::new();
fn slot() -> &'static Mutex<Option<Observer>> {
    OBSERVER.get_or_init(|| Mutex::new(None))
}
pub(crate) struct Guard(Option<Observer>);
pub(crate) fn observe(
    observer: impl Fn(&[MyChemHit], &'static str) + Send + Sync + 'static,
) -> Guard {
    let previous = crate::utils::sync::recover_poison(slot().lock()).replace(Arc::new(observer));
    Guard(previous)
}
pub(crate) fn record(hits: &[MyChemHit], stage: &'static str) {
    let observer = crate::utils::sync::recover_poison(slot().lock()).clone();
    if let Some(observer) = observer {
        observer(hits, stage);
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        *crate::utils::sync::recover_poison(slot().lock()) = self.0.take();
    }
}
