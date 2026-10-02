//! Bind the finite authored vocabulary to source-pinned consumer decisions.
//! Group summaries retain their children; every independent effect owns an event.
use super::MyChemHit;
use super::conversion::DrugConversion;
use serde_json::{Value, json};
mod policy;
pub(crate) use policy::known;

pub(crate) fn assert_conversion(hits: &[&MyChemHit], wanted: &Value, boundary: &str, id: &Value) {
    assert_conversion_with_signals(hits, &[], wanted, boundary, id)
}
pub(crate) fn assert_conversion_with_signals(
    hits: &[&MyChemHit],
    signals: &[DrugConversion],
    wanted: &Value,
    boundary: &str,
    id: &Value,
) {
    let mut events = hits
        .iter()
        .flat_map(|hit| crate::utils::sync::recover_poison(hit.conversion.lock()).clone())
        .collect::<Vec<_>>();
    events.extend_from_slice(signals);
    let mut obligations = Vec::new();
    flatten(wanted, boundary, id, &mut obligations);
    let mut consumed = vec![false; events.len()];
    let mut summaries = Vec::new();
    for (expected, stage, action, reason, count) in &obligations {
        let key = json!({"custody":custody_key(expected),"stage":stage,"action":action,"reason":reason,"target":expected.get("target"),"retained_result":expected.get("retained_result"),"prior_match_kind":expected.get("prior_match_kind"),"deduplication_key":expected.get("deduplication_key")});
        if summaries.contains(&key) {
            assert!(
                expected["summary_reference"] == true,
                "{id}: repeated independent obligation {expected}"
            );
            continue;
        }
        summaries.push(key);
        let candidates = events
            .iter()
            .enumerate()
            .filter(|(_, event)| {
                custody(event, expected)
                    && event.stage == *stage
                    && same_family(stage, event.action, action)
            })
            .collect::<Vec<_>>();
        for event in events.iter().filter(|event| custody(event, expected)) {
            let prohibited = match (*stage, *action) {
                ("get merge synonyms", "omit" | "deduplicate") => {
                    matches!(event.stage, "card brands" | "DDInter synonyms")
                        && event.action == "select"
                }
                ("card brands" | "DDInter synonyms", "select") => {
                    event.stage == "get merge synonyms"
                        && matches!(event.action, "omit" | "deduplicate")
                }
                ("search selection", "omit_limit") => matches!(
                    event.stage,
                    "search projection"
                        | "search filtering"
                        | "search deduplication"
                        | "ranked match classification"
                ),
                ("search selection", "select") => {
                    event.stage == "search deduplication" && event.action == "omit_duplicate"
                }
                ("search deduplication", "omit_duplicate") => {
                    event.stage == "search selection" && event.action == "select"
                }
                _ => false,
            };
            assert!(
                !prohibited,
                "{id}: contradictory split-boundary effect {expected}: {event:?}"
            );
        }
        assert_eq!(
            candidates.len(),
            *count,
            "{id}: exact event multiplicity/contradictory branches for {expected}: {candidates:?}"
        );
        for (index, event) in candidates {
            assert!(
                !consumed[index],
                "{id}: event reused for distinct effects: {expected}"
            );
            assert_eq!(event.action, *action, "{id}: wrong branch {expected}");
            assert_eq!(
                event.reason, *reason,
                "{id}: wrong policy reason {expected}"
            );
            assert!(
                target(event, expected),
                "{id}: wrong complete effect target {expected}: {event:?}"
            );
            consumed[index] = true;
        }
    }
}
type Obligation = (Value, &'static str, &'static str, &'static str, usize);
fn flatten(wanted: &Value, boundary: &str, id: &Value, output: &mut Vec<Obligation>) {
    for expected in wanted.as_array().unwrap() {
        let stage = expected["stage"].as_str().unwrap_or("");
        let action = expected["action"].as_str().unwrap();
        let reason = expected["reason"].as_str().unwrap();
        assert!(
            known(stage, action, reason),
            "{id}: unknown authored policy {expected}"
        );
        if let Some(children) = expected.get("claim_occurrences") {
            for child in children.as_array().unwrap() {
                assert_eq!(
                    child["response_digest"], expected["response_digest"],
                    "{id}: grouped digest"
                );
                assert_eq!(
                    child["ordinal"], expected["ordinal"],
                    "{id}: grouped ordinal"
                );
            }
            if action == "tier_upgrade_keep_first_row" {
                for child in children.as_array().unwrap() {
                    let mut child = child.clone();
                    for key in [
                        "retained_result",
                        "prior_match_kind",
                        "deduplication_key",
                        "retained_digest",
                        "retained_ordinal",
                    ] {
                        child[key] = expected[key].clone();
                    }
                    // The parent describes the tier; children describe the same row's name.
                    child["target"] = expected["target"].clone();
                    bind(&child, boundary, id, output);
                }
                continue;
            }
            let mut children = children.clone();
            if matches!(action, "select" | "omit_page") {
                for child in children.as_array_mut().unwrap() {
                    if child["action"] != action {
                        child["summary_reference"] = json!(true);
                    }
                }
            }
            flatten(&children, boundary, id, output);
            if action == "omit_page" {
                if children
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|child| child["action"] == "omit_page")
                {
                    continue;
                }
                for child in children.as_array().unwrap() {
                    let mut omission = child.clone();
                    omission["action"] = expected["action"].clone();
                    omission["reason"] = expected["reason"].clone();
                    omission["stage"] = json!("ranked pagination");
                    omission.as_object_mut().unwrap().remove("target");
                    bind(&omission, boundary, id, output);
                }
                continue;
            }
            let mut row = expected.clone();
            row.as_object_mut().unwrap().remove("claim_occurrences");
            row["claim_index"] = Value::Null;
            for key in [
                "origin",
                "lexical_text",
                "namespace",
                "source_term_type",
                "source_pointer",
            ] {
                row[key] = Value::Null;
            }
            row["source_only"] = json!(false);
            bind(&row, boundary, id, output);
        } else {
            bind(expected, boundary, id, output);
        }
    }
}
fn bind(expected: &Value, boundary: &str, id: &Value, output: &mut Vec<Obligation>) {
    let stage = expected["stage"].as_str().unwrap_or("");
    let action = expected["action"].as_str().unwrap();
    let reason = expected["reason"].as_str().unwrap();
    let display_stage = match stage {
        "row projection" | "search projection" | "Search candidate projection" => {
            "search projection"
        }
        "initial Get selection and merge"
        | "base identity contribution to alias resolution"
        | "base selection, merge and candidate extraction" => "get merge",
        "" => {
            if matches!(boundary, "search" | "ranking") {
                "search projection"
            } else {
                "get merge"
            }
        }
        _ => "invalid authored display stage",
    };
    let mut add = |stage: &'static str,
                   action: &'static str,
                   reason: &'static str,
                   target_override: Option<Value>| {
        let mut expected = expected.clone();
        if let Some(target) = target_override {
            expected["target"] = target;
        }
        // P3-04 performs one merge and has no authored search_result.
        // Its enrichment events occur once. Separate search stages stay separate.
        output.push((expected, stage, action, reason, 1));
    };
    match action {
        "ascii_lowercase"
        | "select_and_ascii_lowercase"
        | "trim_edge_dots_ascii_lowercase"
        | "normalize_to_empty"
        | "select_display_ascii_lowercase"
        | "match_and_ascii_lowercase" => add(
            display_stage,
            "select_display",
            "display precedence; trim, strip edge dots and ASCII lowercase",
            None,
        ),
        "ascii_lowercase_and_match_active_substance" => {
            add(
                "search projection",
                "select_display",
                "display precedence; trim, strip edge dots and ASCII lowercase",
                None,
            );
            add(
                "ranked match classification",
                "select_active_substance_match",
                "retained source field equals normalized query at selected match tier",
                Some(json!("active_substance")),
            );
        }
        "select_first_code" => add(
            display_stage,
            "select_first_code",
            if display_stage == "get merge" {
                "first identifier; trim lexical code"
            } else {
                "first DrugBank identifier"
            },
            None,
        ),
        "omit_code" => add(
            "get merge",
            "omit_code",
            "first code already selected; occurrence retained",
            None,
        ),
        "select" if stage == "final ranked slice" => add(
            "search pagination",
            "select",
            "within page after complete ranking",
            None,
        ),
        "select" if stage == "result insertion" => add(
            "search selection",
            "select",
            "unique row within requested limit",
            None,
        ),
        "select" if stage == "alias eligibility and insertion" => add(
            "alias eligibility and insertion",
            "select",
            "source-priority eligibility and lexical tie-break",
            None,
        ),
        "select" if boundary == "EMA" => add(
            "EMA identity",
            "select",
            "allowed source field contributes EMA identity",
            None,
        ),
        "select" if expected["namespace"].is_string() => add(
            "get merge",
            "select_first_code",
            "first identifier; trim lexical code",
            None,
        ),
        "select" if matches!(reason, "anchor synonym" | "anchor card and DDInter") => {
            add("card brands", "select", "three-brand card policy", None);
            add(
                "DDInter synonyms",
                "select",
                "anchor-only 32-synonym policy",
                None,
            );
        }
        "select" => add(
            display_stage,
            "select_display",
            "display precedence; trim, strip edge dots and ASCII lowercase",
            None,
        ),
        "omit_display" => add(
            display_stage,
            "omit_display",
            if reason == "anchor already selected" {
                "anchor already selected"
            } else {
                "first accessor or higher priority source supplies display"
            },
            None,
        ),
        "include_match_candidate" => add(
            "get name matching",
            "include_match_candidate",
            "retained first accessor or DrugBank synonym participates in name matching",
            None,
        ),
        "omit_match_candidate" => add(
            "get name matching",
            "omit_match_candidate",
            "later accessor occurrence does not participate in name matching",
            None,
        ),
        "select_match_candidate" => add(
            "get name matching",
            "select_match_candidate",
            "retained first accessor or DrugBank synonym participates before normalization",
            None,
        ),
        "omit_display_candidate" => {
            add(
                "get merge",
                "omit_display",
                "first accessor or higher priority source supplies display",
                Some(Value::Null),
            );
            add(
                "get name matching",
                "include_match_candidate",
                "retained first accessor or DrugBank synonym participates in name matching",
                None,
            );
        }
        "discard_get_row" => add(
            "get selection",
            "omit",
            if expected["claim_index"].is_number() {
                "name selection excludes row"
            } else {
                "name matching, all-hit fallback and stable richness order"
            },
            None,
        ),
        "requested_name_fallback" => add(
            "get merge",
            "requested_name_fallback",
            "no supplied name",
            None,
        ),
        "select_synonym" | "select_card_and_ddinter" | "select_card_and_ddinter_synonym" => {
            add("card brands", "select", "three-brand card policy", None);
            add(
                "DDInter synonyms",
                "select",
                "anchor-only 32-synonym policy",
                None,
            );
        }
        "select_ddinter_omit_card" | "omit_card_keep_ddinter" => {
            add(
                "card brands",
                "cap",
                "three-brand card policy",
                Some(Value::Null),
            );
            add(
                "DDInter synonyms",
                "select",
                "anchor-only 32-synonym policy",
                None,
            );
        }
        "cap" => {
            add(
                "card brands",
                "cap",
                "three-brand card policy",
                Some(Value::Null),
            );
            add(
                "DDInter synonyms",
                "cap",
                "anchor-only 32-synonym policy",
                None,
            );
        }
        "omit_requested_synonym" => add(
            "get merge synonyms",
            "omit",
            "equals requested or displayed name",
            None,
        ),
        "deduplicate" if boundary == "EMA" => add(
            "EMA identity",
            "deduplicate",
            "normalized identity term already inserted",
            None,
        ),
        "deduplicate" if stage == "name deduplication" => add(
            "search deduplication",
            "omit_duplicate",
            "first normalized display row retained",
            None,
        ),
        "deduplicate" => add(
            "get merge synonyms",
            "deduplicate",
            "case insensitive duplicate",
            None,
        ),
        "discard_search_row" => add(
            "search projection",
            "discard",
            "display normalizes to empty",
            None,
        ),
        "discard" => add(
            "search projection",
            "discard",
            "row has no display name",
            None,
        ),
        "omit_unvisited_limit" => add(
            "search selection",
            "omit_limit",
            "requested result limit reached before projection",
            None,
        ),
        "select_match_tier" => add(
            "ranked match classification",
            if expected["target"] == "product_name" {
                "select_product_match"
            } else {
                "select_active_substance_match"
            },
            "retained source field equals normalized query at selected match tier",
            None,
        ),
        "select_alias_match" => add(
            "ranked match classification",
            "select_alias_match",
            "retained source field equals normalized query at selected match tier",
            None,
        ),
        "tier_upgrade_keep_first_row" => add(
            "search ranking",
            "tier_upgrade_keep_first_row",
            "later stronger match; original result row retained",
            None,
        ),
        "omit_page" => add(
            "search pagination",
            "omit_page",
            match reason {
                "offset after complete ranking" => "offset after complete ranking",
                "limit after complete ranking" => "limit after complete ranking",
                _ => panic!("{id}: unknown page reason"),
            },
            None,
        ),
        "trim_and_deduplicate" => add(
            "alias eligibility and insertion",
            "trim_and_deduplicate",
            "trimmed case-insensitive candidate already inserted",
            None,
        ),
        "omit_unvisited_alias_cap" => add(
            "alias eligibility and insertion",
            "omit_unvisited_alias_cap",
            "three newly inserted provider aliases reached before candidate deduplication",
            None,
        ),
        "exclude" if stage == "alias eligibility and insertion" => add(
            "alias eligibility and insertion",
            "exclude",
            if reason == "free-base descriptor" {
                "free-base descriptor"
            } else {
                "source alias fails simple-name or investigational-code eligibility"
            },
            None,
        ),
        "exclude" => add(
            "get merge",
            "exclude",
            "combination hit is not the named anchor",
            None,
        ),
        "exclude_alias_row" => add(
            "orphan alias admission",
            "exclude_alias_row",
            if reason == "later UNII conflict despite earlier matching code" {
                "later UNII conflict despite earlier matching code"
            } else {
                "conflicting populated identifiers"
            },
            None,
        ),
        "exclude_ema_field" => add(
            "EMA identity",
            "exclude_ema_field",
            "source field excluded from EMA allowed identity fields",
            None,
        ),
        "ascii_lowercase_retry_label" => add(
            "get fallback",
            "ascii_lowercase_retry_label",
            "one different nonblank search name selects detail request",
            None,
        ),
        "omit_candidate_identifier" => add(
            "get fallback",
            "omit_candidate_identifier",
            "search result establishes lookup label; final GET alone supplies product code",
            None,
        ),
        "replace_lookup_display" => add(
            "get fallback",
            "replace_lookup_display",
            "accepted sparse initial name is replaced by accepted final candidate",
            None,
        ),
        "select_retry_label" => add(
            "get fallback",
            "select_retry_label",
            "first generic label supplies different canonical lookup",
            None,
        ),
        "discard_sparse_product" => add(
            "get fallback",
            "discard_sparse_product",
            "ambiguous canonical discovery refuses successful sparse result",
            None,
        ),
        "select_mechanism_and_admit"
        | "match_and_uppercase_requested_target"
        | "omit_initial_target"
        | "discard_target_filter"
        | "discard_mechanism_filter" => {
            let actual_action = match action {
                "select_mechanism_and_admit" => "select_mechanism_and_admit",
                "match_and_uppercase_requested_target" => "match_and_uppercase_requested_target",
                "omit_initial_target" => "omit_initial_target",
                "discard_target_filter" => "discard_target_filter",
                _ => "discard_mechanism_filter",
            };
            add(
                "search filtering",
                actual_action,
                if expected
                    .get("source_pointer")
                    .is_none_or(|pointer| !pointer.is_string())
                {
                    if action == "discard_target_filter" {
                        "requested target absent from retained source enrichment"
                    } else {
                        "requested mechanism absent from retained source enrichment"
                    }
                } else if expected["source_pointer"]
                    .as_str()
                    .is_some_and(|pointer| pointer.contains("/gtopdb/"))
                {
                    "actual requested-target match overrides the projected first target"
                } else {
                    "actual retained target and mechanism predicates govern result admission"
                },
                None,
            );
        }
        "normalize_molecule_type" => add(
            "enrichment",
            "normalize_molecule_type",
            "existing molecule-type map; identity adapter omits enrichment",
            None,
        ),
        "select_mechanism" => add(
            "enrichment",
            "select_mechanism",
            "explicit mechanism takes precedence over synthesized action/target",
            None,
        ),
        "omit_synthesized_action" => add(
            "enrichment",
            "omit_synthesized_action",
            "explicit mechanism text present",
            None,
        ),
        "omit_synthesized_target" => add(
            "enrichment",
            "omit_synthesized_target",
            "explicit mechanism text present; get target list is GtoPdb-owned",
            None,
        ),
        "select_target" => add(
            "enrichment",
            "select_target",
            "first unique trimmed GtoPdb symbol",
            None,
        ),
        "strip_moa_suffix" => add(
            "enrichment",
            "strip_moa_suffix",
            "pharmacologic class keeps text before [MoA]",
            None,
        ),
        "select_approval_agency" => add(
            "enrichment",
            "select_approval_agency",
            "FDA agency makes date eligible",
            None,
        ),
        "normalize_approval_date" => add(
            "enrichment",
            "normalize_approval_date",
            "YYYYMMDD normalizes to YYYY-MM-DD and named calendar display",
            None,
        ),
        "select_indication" => add(
            "enrichment",
            "select_indication",
            "first unique trimmed indication name",
            None,
        ),
        "select_interaction_partner" => add(
            "enrichment",
            "select_interaction_partner",
            "name owns interaction partner under existing field precedence",
            None,
        ),
        "select_interaction_description" => add(
            "enrichment",
            "select_interaction_description",
            "description owns interaction text",
            None,
        ),
        "omit_get_atc" => add(
            "enrichment",
            "omit_get_atc",
            "ATC remains source-only; Get profile does not request it and no merged Drug field consumes it",
            None,
        ),
        _ => panic!(
            "{id}: decision belongs to a complete owning-object comparison, or lacks a source binding: {expected}"
        ),
    }
}
fn same_family(stage: &str, left: &str, right: &str) -> bool {
    // Projection normalizes the display before its separate empty-row refusal.
    if stage == "search projection" {
        return match right {
            "select_display" | "omit_display" => !matches!(left, "select_first_code" | "discard"),
            "select_first_code" => !matches!(left, "select_display" | "omit_display" | "discard"),
            "discard" => !matches!(
                left,
                "select_display" | "omit_display" | "select_first_code"
            ),
            _ => true,
        };
    }
    // Target override can precede a separate mechanism-filter rejection.
    if stage == "search filtering"
        && right == "discard_mechanism_filter"
        && matches!(
            left,
            "match_and_uppercase_requested_target" | "omit_initial_target"
        )
    {
        return false;
    }
    if stage == "search filtering"
        && left == "discard_mechanism_filter"
        && matches!(
            right,
            "match_and_uppercase_requested_target" | "omit_initial_target"
        )
    {
        return false;
    }
    if stage == "search filtering"
        && left == "select_target_override"
        && right != "select_target_override"
    {
        return false;
    }
    // Name matching retains the accessor before its normalized participation.
    if stage != "get name matching" {
        return true;
    }
    if right == "select_match_candidate" {
        !matches!(left, "include_match_candidate" | "omit_match_candidate")
    } else {
        left != "select_match_candidate"
    }
}
fn custody_key(expected: &Value) -> Value {
    let mut key = serde_json::Map::new();
    for name in [
        "response_digest",
        "ordinal",
        "claim_index",
        "origin",
        "lexical_text",
        "namespace",
        "source_term_type",
        "source_only",
        "source_pointer",
    ] {
        if let Some(value) = expected.get(name) {
            key.insert(name.to_owned(), value.clone());
        }
    }
    json!(key)
}
fn target(event: &DrugConversion, expected: &Value) -> bool {
    expected.get("target").is_none_or(|value| {
        let target = event.target.as_ref().unwrap_or(&Value::Null);
        if event.action == "tier_upgrade_keep_first_row" && target.is_object() {
            if expected.get("retained_result").is_some() {
                &target["match_kind"] == value
                    && [
                        "retained_result",
                        "prior_match_kind",
                        "deduplication_key",
                        "retained_digest",
                        "retained_ordinal",
                    ]
                    .iter()
                    .all(|key| {
                        expected
                            .get(*key)
                            .is_none_or(|wanted| &target[*key] == wanted)
                    })
            } else {
                &target["name"] == value
            }
        } else {
            target == value
        }
    })
}
fn custody(event: &DrugConversion, expected: &Value) -> bool {
    let value = json!(event);
    if expected.get("claim_index").is_none()
        && expected.get("origin").is_none()
        && expected.get("source_pointer").is_none()
    {
        if event.claim_index.is_some()
            || event.origin.is_some()
            || event.lexical_text.is_some()
            || event.namespace.is_some()
            || event.source_term_type.is_some()
            || event.source_pointer.is_some()
            || event.source_only
        {
            return false;
        }
    }
    if expected.get("source_pointer").is_some_and(Value::is_string) != event.source_only {
        return false;
    }
    [
        "response_digest",
        "ordinal",
        "claim_index",
        "origin",
        "lexical_text",
        "namespace",
        "source_term_type",
        "source_only",
        "source_pointer",
    ]
    .into_iter()
    .all(|key| {
        expected
            .get(key)
            .is_none_or(|wanted| value.get(key).unwrap_or(&Value::Null) == wanted)
    })
}
