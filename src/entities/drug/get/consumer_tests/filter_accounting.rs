//! Bound the target-refusal contradiction within the existing adopted filter control.
use super::*;

pub(super) fn assert_target_refusal_excludes_success(
    hits: &[MyChemHit],
    wanted: &Value,
    id: &Value,
) {
    let mechanism_refusal = hits
        .iter()
        .find(|hit| hit.row.source().ordinal() == 2)
        .unwrap();
    let events = mechanism_refusal.conversion.lock().unwrap();
    for action in ["select_target_override", "discard_mechanism_filter"] {
        assert!(
            events.iter().any(|event| {
                event.stage == "search filtering"
                    && event.action == action
                    && event.origin.is_none()
            }),
            "{id}: valid target override before mechanism refusal must remain exercised"
        );
    }
    drop(events);

    let changed = hits
        .iter()
        .cloned()
        .map(|mut hit| {
            if hit.row.source().ordinal() == 1 {
                let mut events = hit.conversion.lock().unwrap().clone();
                let mut success = events
                    .iter()
                    .find(|event| {
                        event.stage == "search filtering"
                            && event.action == "discard_target_filter"
                            && event.origin.is_none()
                    })
                    .unwrap()
                    .clone();
                assert!(success.claim_index.is_none() && success.lexical_text.is_none());
                assert!(success.namespace.is_none() && success.source_term_type.is_none());
                assert!(!success.source_only && success.source_pointer.is_none());
                success.action = "select_target_override";
                success.reason = "matched requested target displayed in uppercase";
                events.push(success);
                hit.conversion = Arc::new(Mutex::new(events));
            }
            hit
        })
        .collect::<Vec<_>>();
    let rejected = std::panic::catch_unwind(|| {
        crate::sources::mychem::consumer_tests_conversion::assert_conversion(
            &changed.iter().collect::<Vec<_>>(),
            wanted,
            "search",
            id,
        );
    });
    assert!(
        rejected.is_err(),
        "{id}: target refusal must reject contradictory target success"
    );
}
