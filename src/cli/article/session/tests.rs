//! Article search session loop-breaker tests.

use super::*;

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn terms(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn search<'a>(
    token: &'a str,
    keyword: Option<&'a str>,
    pmids: &'a [String],
    next_commands: &'a [String],
    now_epoch_secs: u64,
) -> SessionSearch<'a> {
    SessionSearch {
        token,
        keyword,
        pmids,
        next_commands,
        now_epoch_secs,
    }
}

#[test]
fn token_validation_accepts_safe_local_labels_and_rejects_unsafe_input() {
    let max_len = "a".repeat(128);
    for token in ["lit-review-1", "agent.session:42", "A_Z.0-9", &max_len] {
        validate_token(token).expect("safe token should validate");
    }

    let too_long = "a".repeat(129);
    for token in [
        "",
        &too_long,
        "has space",
        "../unsafe",
        "semi;colon",
        "unicode-é",
        "line\nbreak",
    ] {
        let err = validate_token(token).expect_err("unsafe token should be rejected");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--session"));
    }
}

#[test]
fn normalization_removes_search_filler_words_and_deduplicates_terms() {
    assert_eq!(
        normalized_terms("What review articles study Oncotype DX studies?"),
        terms(&["dx", "oncotype"])
    );
    assert!(normalized_terms("the and review articles literature").is_empty());
}

#[test]
fn overlap_threshold_boundaries() {
    let previous = terms(&["dx", "oncotype"]);
    let overlapping = terms(&["dcis", "dx", "oncotype"]);
    let disjoint = terms(&["mtor", "signaling"]);
    let exact_threshold_previous = terms(&["a", "b", "c"]);
    let exact_threshold_current = terms(&["a", "b", "c", "d", "e"]);
    let below_threshold_current = terms(&["a", "b", "d", "e", "f"]);

    assert!(overlap_score(&previous, &overlapping) >= 0.60);
    assert!(overlap_score(&previous, &disjoint) < 0.60);
    assert_eq!(
        overlap_score(&exact_threshold_previous, &exact_threshold_current),
        0.60
    );
    assert!(
        overlap_score(&exact_threshold_previous, &below_threshold_current) < 0.60,
        "overlap below the threshold must not trigger"
    );
    assert_eq!(overlap_score(&BTreeSet::new(), &overlapping), 0.0);
}

#[test]
fn session_store_records_first_search_then_emits_ordered_loop_suggestions() {
    let root = crate::test_support::TempDirGuard::new("article-session-loop");
    let first_pmids = strings(&["22663011", "24200969"]);
    let second_pmids = strings(&["39073865"]);
    let next_commands = strings(&[
        "biomcp get article 39073865",
        "biomcp search article -k \"Oncotype DX DCIS study\" --year-min 2025 --year-max 2025 --limit 5",
        "biomcp list article",
    ]);

    let first = record_success_and_suggestions(
        root.path(),
        search(
            "lit-review-1",
            Some("Oncotype DX review paper"),
            &first_pmids,
            &[],
            1_000,
        ),
    );
    assert!(
        first.is_empty(),
        "first search should only record a baseline"
    );

    let second = record_success_and_suggestions(
        root.path(),
        search(
            "lit-review-1",
            Some("Oncotype DX DCIS study"),
            &second_pmids,
            &next_commands,
            1_030,
        ),
    );

    let commands = second
        .iter()
        .map(|suggestion| suggestion.command.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        vec![
            "biomcp article batch 22663011 24200969",
            "biomcp discover \"Oncotype DX DCIS study\"",
            "biomcp search article -k \"Oncotype DX DCIS study\" --year-min 2025 --year-max 2025 --limit 5",
        ]
    );
    assert!(
        second
            .iter()
            .all(|suggestion| suggestion.sections.is_empty())
    );
}

#[test]
fn expired_or_disjoint_session_state_does_not_emit_loop_suggestions() {
    let root = crate::test_support::TempDirGuard::new("article-session-expired");
    let pmids = strings(&["22663011"]);

    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("Oncotype DX review paper"),
                &pmids,
                &[],
                1_000,
            ),
        )
        .is_empty()
    );
    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("Oncotype DX DCIS study"),
                &pmids,
                &[],
                1_601,
            ),
        )
        .is_empty(),
        "entries older than 10 minutes should expire"
    );
    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("mTOR signaling pathway"),
                &pmids,
                &[],
                1_620,
            ),
        )
        .is_empty(),
        "disjoint keywords should reset the baseline without suggestions"
    );
}

#[test]
fn empty_normalized_keyword_resets_baseline_without_loop_suggestions() {
    let root = crate::test_support::TempDirGuard::new("article-session-empty-keyword");
    let pmids = strings(&["22663011"]);
    let next_commands = strings(&[
        "biomcp search article -k \"Oncotype DX DCIS study\" --year-min 2025 --year-max 2025 --limit 5",
    ]);

    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("Oncotype DX review paper"),
                &pmids,
                &[],
                1_000,
            ),
        )
        .is_empty()
    );
    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("the and review articles literature"),
                &pmids,
                &[],
                1_030,
            ),
        )
        .is_empty(),
        "post-stopword-empty searches should emit no suggestions"
    );
    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("Oncotype DX DCIS study"),
                &pmids,
                &next_commands,
                1_060,
            ),
        )
        .is_empty(),
        "empty baseline should prevent a later overlap trigger"
    );
}

#[test]
fn unavailable_loop_ladder_rungs_are_omitted() {
    let root = crate::test_support::TempDirGuard::new("article-session-omit-rungs");
    let no_pmids = Vec::new();
    let current_pmids = strings(&["39073865"]);
    let next_commands = strings(&["biomcp get article 39073865", "biomcp list article"]);

    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("Oncotype DX review paper"),
                &no_pmids,
                &[],
                1_000,
            ),
        )
        .is_empty()
    );

    let suggestions = record_success_and_suggestions(
        root.path(),
        search(
            "lit-review-1",
            Some("Oncotype DX DCIS study"),
            &current_pmids,
            &next_commands,
            1_030,
        ),
    );
    let commands = suggestions
        .iter()
        .map(|suggestion| suggestion.command.as_str())
        .collect::<Vec<_>>();
    assert_eq!(commands, vec!["biomcp discover \"Oncotype DX DCIS study\""]);
}

#[test]
fn lock_contention_fails_open_without_rewriting_existing_baseline() {
    let root = crate::test_support::TempDirGuard::new("article-session-lock-contention");
    let pmids = strings(&["22663011"]);
    assert!(
        record_success_and_suggestions(
            root.path(),
            search(
                "lit-review-1",
                Some("Oncotype DX review paper"),
                &pmids,
                &[],
                1_000,
            ),
        )
        .is_empty()
    );

    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path(root.path()))
        .expect("open lock");
    lock.lock_exclusive().expect("hold session store lock");

    let suggestions = record_success_and_suggestions(
        root.path(),
        search(
            "lit-review-1",
            Some("Oncotype DX DCIS study"),
            &pmids,
            &[],
            1_030,
        ),
    );
    assert!(
        suggestions.is_empty(),
        "store lock contention should fail open"
    );
    lock.unlock().expect("release session store lock");

    let contents =
        std::fs::read_to_string(store_path(root.path())).expect("store should still exist");
    let value: serde_json::Value = serde_json::from_str(&contents).expect("store should parse");
    assert_eq!(
        value["sessions"]["lit-review-1"]["keyword"],
        serde_json::Value::String("Oncotype DX review paper".into())
    );
}

#[test]
fn malformed_store_recovers_as_empty_and_writes_valid_json() {
    let root = crate::test_support::TempDirGuard::new("article-session-malformed");
    let path = store_path(root.path());
    std::fs::create_dir_all(path.parent().expect("store file should have parent"))
        .expect("create store dir");
    std::fs::write(&path, "{not json").expect("write malformed store");

    let pmids = strings(&["22663011"]);
    let suggestions = record_success_and_suggestions(
        root.path(),
        search(
            "lit-review-1",
            Some("Oncotype DX review paper"),
            &pmids,
            &[],
            1_000,
        ),
    );
    assert!(suggestions.is_empty());

    let repaired = std::fs::read_to_string(&path).expect("store should be rewritten");
    let value: serde_json::Value =
        serde_json::from_str(&repaired).expect("repaired store should parse");
    assert_eq!(value["version"], serde_json::json!(1));
    assert!(value["sessions"]["lit-review-1"].is_object());
}

#[test]
fn capacity_pruning_keeps_newest_1024_sessions() {
    let root = crate::test_support::TempDirGuard::new("article-session-capacity");
    let pmids = strings(&["22663011"]);

    for index in 0..1_030 {
        let token = format!("loop-{index:04}");
        assert!(
            record_success_and_suggestions(
                root.path(),
                search(
                    &token,
                    Some("Oncotype DX review paper"),
                    &pmids,
                    &[],
                    1_000 + index / 4,
                ),
            )
            .is_empty()
        );
    }

    let contents = std::fs::read_to_string(store_path(root.path())).expect("store should exist");
    let value: serde_json::Value = serde_json::from_str(&contents).expect("store should parse");
    let sessions = value["sessions"]
        .as_object()
        .expect("sessions should be an object");
    assert_eq!(sessions.len(), 1_024);
    assert!(!sessions.contains_key("loop-0000"));
    assert!(!sessions.contains_key("loop-0005"));
    assert!(sessions.contains_key("loop-0006"));
    assert!(sessions.contains_key("loop-1029"));
}
