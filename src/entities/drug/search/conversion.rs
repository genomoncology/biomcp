//! Source-only custody at the actual filter and target override boundaries.
use super::*;

pub(super) fn record_filter(hit: &MyChemHit, action: &'static str, target: Option<&str>) {
    if let Some(chembl) = &hit.chembl {
        for index in 0..chembl.drug_mechanisms.len() {
            for field in ["mechanism_of_action", "action_type", "target_name"] {
                hit.record_source(
                    &format!("/chembl/drug_mechanisms/{index}/{field}"),
                    "search filtering",
                    if action == "target_override" {
                        "select_mechanism_and_admit"
                    } else {
                        action
                    },
                    "actual retained target and mechanism predicates govern result admission",
                    if action == "target_override" {
                        chembl.drug_mechanisms[index]
                            .mechanism_of_action
                            .as_deref()
                            .map(str::trim)
                            .map(str::to_owned)
                    } else {
                        None
                    },
                );
            }
        }
    }
    if let Some(gtopdb) = &hit.gtopdb {
        for (index, row) in gtopdb.interaction_targets.iter().enumerate() {
            let matched = target.is_some_and(|target| {
                row.symbol
                    .as_deref()
                    .is_some_and(|symbol| symbol.trim().eq_ignore_ascii_case(target.trim()))
            });
            hit.record_source(
                &format!("/gtopdb/interaction_targets/{index}/symbol"),
                "search filtering",
                if action == "target_override" {
                    if matched {
                        "match_and_uppercase_requested_target"
                    } else {
                        "omit_initial_target"
                    }
                } else {
                    action
                },
                "actual requested-target match overrides the projected first target",
                if matched {
                    target.map(|target| target.to_ascii_uppercase())
                } else {
                    None
                },
            );
        }
    }
}
