//! Retained enrichment conversion and source-only occurrence reports.
use super::*;

pub(super) fn drug_type_from_hit(hit: &MyChemHit) -> Option<String> {
    let v = hit
        .chembl
        .as_ref()
        .and_then(|c| c.molecule_type.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase());

    let result = match v.as_deref() {
        Some("antibody") => Some("biologic".into()),
        Some("small molecule") => Some("small-molecule".into()),
        Some(other) => Some(other.to_string()),
        None => None,
    };
    hit.record_source(
        "/chembl/molecule_type",
        "enrichment",
        "normalize_molecule_type",
        "existing molecule-type map; identity adapter omits enrichment",
        result.clone(),
    );
    result
}

pub(super) fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let first = first.to_uppercase().collect::<String>();
            let rest = chars.as_str().to_ascii_lowercase();
            format!("{first}{rest}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn normalize_action_type(action: &str) -> String {
    let v = action.trim().replace('_', " ");
    if v.is_empty() {
        return String::new();
    }
    if v.chars().any(|c| c.is_ascii_lowercase()) {
        return v;
    }
    title_case_words(&v.to_ascii_lowercase())
}

pub(super) fn chembl_mechanisms_from_hit(hit: &MyChemHit) -> Vec<String> {
    let Some(chembl) = hit.chembl.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    for (index, mech) in chembl.drug_mechanisms.iter().enumerate() {
        if let Some(mechanism) = mech
            .mechanism_of_action
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        {
            hit.record_source(
                &format!("/chembl/drug_mechanisms/{index}/mechanism_of_action"),
                "enrichment",
                "select_mechanism",
                "explicit mechanism takes precedence over synthesized action/target",
                Some(mechanism.clone()),
            );
            hit.record_source(
                &format!("/chembl/drug_mechanisms/{index}/action_type"),
                "enrichment",
                "omit_synthesized_action",
                "explicit mechanism text present",
                None,
            );
            hit.record_source(
                &format!("/chembl/drug_mechanisms/{index}/target_name"),
                "enrichment",
                "omit_synthesized_target",
                "explicit mechanism text present; get target list is GtoPdb-owned",
                None,
            );
            out.push(mechanism);
            continue;
        }

        let action = mech
            .action_type
            .as_deref()
            .map(normalize_action_type)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let target = mech
            .target_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        if action.is_none() && target.is_none() {
            continue;
        }

        let mechanism = match (action, target) {
            (Some(a), Some(t)) => format!("{a} of {t}"),
            (Some(a), None) => a,
            (None, Some(t)) => t.to_string(),
            (None, None) => continue,
        };
        for field in ["action_type", "target_name"] {
            hit.record_source(
                &format!("/chembl/drug_mechanisms/{index}/{field}"),
                "enrichment",
                "synthesize_mechanism",
                "existing action and target synthesis after absent explicit mechanism",
                Some(mechanism.clone()),
            );
        }
        out.push(mechanism);
    }
    out
}

pub(super) fn fallback_mechanism_from_hit(hit: &MyChemHit) -> Option<String> {
    let classes = moa_pharm_classes(hit);
    classes
        .iter()
        .find(|class| {
            let class = class.to_ascii_lowercase();
            class.contains("kinase") || class.contains("braf") || class.contains("b-raf")
        })
        .cloned()
        .or_else(|| {
            classes.into_iter().find(|class| {
                let class = class.to_ascii_lowercase();
                !(class.contains("cytochrome p450")
                    || class.contains("metabol")
                    || class.contains("enzyme") && class.contains("induc"))
            })
        })
}

pub(super) fn normalize_approval_date(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    if v.len() == 10 {
        // Only an ASCII YYYY-MM-DD shape is accepted, so later slicing by
        // callers stays on character boundaries for any external value.
        let bytes = v.as_bytes();
        let valid = bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[4] == b'-'
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[7] == b'-'
            && bytes[8..10].iter().all(|b| b.is_ascii_digit());
        return valid.then(|| v.to_string());
    }
    if v.len() == 8 && v.chars().all(|c| c.is_ascii_digit()) {
        return Some(format!("{}-{}-{}", &v[0..4], &v[4..6], &v[6..8]));
    }
    None
}

pub(super) fn approval_date_display(raw: &str) -> Option<String> {
    let normalized = normalize_approval_date(raw)?;
    let year: i32 = normalized[0..4].parse().ok()?;
    let month: u8 = normalized[5..7].parse().ok()?;
    let day: u8 = normalized[8..10].parse().ok()?;
    let month = Month::try_from(month).ok()?;
    Some(format!("{month} {day}, {year}"))
}

pub(super) fn approval_summary(display: Option<&str>) -> Option<String> {
    display.map(|value| format!("FDA approved on {value}"))
}

pub(super) fn approval_date_from_hit(hit: &MyChemHit) -> Option<String> {
    let approvals = hit.drugcentral.as_ref().map(|d| &d.approval)?;
    let mut fda_dates: Vec<String> = approvals
        .iter()
        .filter(|a| {
            a.agency
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case("FDA"))
        })
        .filter_map(|a| a.date.as_deref())
        .filter_map(normalize_approval_date)
        .collect();

    let chosen = if !fda_dates.is_empty() {
        fda_dates.sort();
        fda_dates.first().cloned()
    } else {
        let mut dates = approvals
            .iter()
            .filter_map(|a| a.date.as_deref())
            .filter_map(normalize_approval_date)
            .collect::<Vec<_>>();
        dates.sort();
        dates.first().cloned()
    };
    let fda = !fda_dates.is_empty();
    let mut recorded = false;
    for (index, approval) in approvals.iter().enumerate() {
        let eligible = !fda
            || approval
                .agency
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case("FDA"));
        let selected = !recorded
            && eligible
            && approval
                .date
                .as_deref()
                .and_then(normalize_approval_date)
                .as_ref()
                == chosen.as_ref()
            && chosen.is_some();
        if selected {
            recorded = true;
        }
        hit.record_source(
            &format!("/drugcentral/approval/{index}/agency"),
            "enrichment",
            if selected {
                "select_approval_agency"
            } else {
                "omit_approval_agency"
            },
            if selected {
                "FDA agency makes date eligible"
            } else {
                "first earliest eligible approval supplies date"
            },
            selected.then(|| approval.agency.clone()).flatten(),
        );
        let target = if selected {
            let date = chosen.clone();
            let display = date
                .as_deref()
                .and_then(approval_date_display)
                .or_else(|| date.clone());
            Some(
                serde_json::json!({"date":date,"raw":date,"display":display,"summary":approval_summary(display.as_deref())}),
            )
        } else {
            None
        };
        hit.record_source_value(
            &format!("/drugcentral/approval/{index}/date"),
            "enrichment",
            if selected {
                "normalize_approval_date"
            } else {
                "omit_approval_date"
            },
            if selected {
                "YYYYMMDD normalizes to YYYY-MM-DD and named calendar display"
            } else {
                "first earliest eligible approval supplies date"
            },
            target,
        );
    }
    chosen
}

pub(super) fn first_target_from_hit(hit: &MyChemHit) -> Option<String> {
    if let Some(gtopdb) = hit.gtopdb.as_ref() {
        for target in &gtopdb.interaction_targets {
            let Some(symbol) = target
                .symbol
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            return Some(symbol.to_string());
        }
    }

    let chembl = hit.chembl.as_ref()?;
    for mechanism in &chembl.drug_mechanisms {
        let Some(target) = mechanism
            .target_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        return Some(target.to_string());
    }

    None
}

pub(super) fn moa_class_occurrences(hit: &MyChemHit) -> Vec<(String, String)> {
    let rows = match hit.ndc.as_ref() {
        Some(MyChemNdcField::One(row)) => vec![("/ndc".to_owned(), row)],
        Some(MyChemNdcField::Many(rows)) => rows
            .iter()
            .enumerate()
            .map(|(index, row)| (format!("/ndc/{index}"), row))
            .collect(),
        None => Vec::new(),
    };
    rows.into_iter()
        .flat_map(|(base, row)| {
            row.pharm_classes
                .iter()
                .enumerate()
                .filter_map(move |(index, value)| {
                    let value = value.as_str()?;
                    if !value.contains("[MoA]") {
                        return None;
                    }
                    Some((
                        format!("{base}/pharm_classes/{index}"),
                        clean_moa_class(value)?,
                    ))
                })
        })
        .collect()
}

pub(super) fn record_match_candidates(hit: &MyChemHit) {
    let mut fields = HashSet::new();
    for (index, claim) in hit.row.identity().claims().iter().enumerate() {
        let biodata::DrugClaimValue::Term(term) = claim.value() else {
            continue;
        };
        let field = (claim.origin().section(), claim.origin().field());
        let selected = field == ("drugbank", "synonyms") || fields.insert(field);
        hit.record(
            Some(index),
            "get name matching",
            if selected {
                "include_match_candidate"
            } else {
                "omit_match_candidate"
            },
            if selected {
                "retained first accessor or DrugBank synonym participates in name matching"
            } else {
                "later accessor occurrence does not participate in name matching"
            },
            selected.then(|| normalize_name(term.text())),
        );
    }
}
