use std::collections::HashSet;

use time::Month;

use crate::entities::drug::{Drug, DrugInteraction, DrugSearchResult};
use crate::sources::mychem::{MyChemHit, MyChemNdcField, MyChemPharmClass};

fn normalize_name(value: &str) -> String {
    value.trim().trim_matches('.').to_ascii_lowercase()
}

/// Whether one whitespace token is a bare amount: digits with optional
/// decimal separators ("81", "0.3", "1,000").
fn token_is_amount(token: &str) -> bool {
    token.chars().next().is_some_and(|ch| ch.is_ascii_digit())
        && token
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '.'))
}

/// Whether one whitespace token is an amount with its unit glued on
/// ("81mg", "0.3%"), the way some product rows write strengths.
fn token_is_amount_with_unit(token: &str) -> bool {
    let digits = token
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '.'))
        .count();
    digits > 0
        && token[digits..].len() <= 4
        && token[digits..]
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || ch == '%')
}

/// A trailing strength run a product row carries after its ingredient
/// ("aspirin 81 mg", "lidocaine 4%"): amount tokens and the short unit
/// words that follow them. A name is never reduced past its last word that
/// is not part of a strength, and a run that consumed the whole name names
/// nothing.
pub(crate) fn strip_trailing_strength(name: &str) -> &str {
    let tokens: Vec<&str> = name.split_whitespace().collect();
    if tokens.len() < 2 {
        return name;
    }
    let mut end = tokens.len();
    while end > 1 {
        let token = tokens[end - 1];
        if token_is_amount(token) || token_is_amount_with_unit(token) {
            end -= 1;
            continue;
        }
        // A short unit word that follows an amount word ("81 mg").
        if token.chars().all(|ch| ch.is_ascii_alphabetic())
            && token.len() <= 3
            && end >= 2
            && (token_is_amount(tokens[end - 2]) || token_is_amount_with_unit(tokens[end - 2]))
        {
            end -= 1;
            continue;
        }
        break;
    }
    if end == tokens.len() || end == 0 {
        return name;
    }
    let stripped_len = tokens[..end].iter().map(|t| t.len()).sum::<usize>() + end - 1;
    &name[..stripped_len]
}

fn ndc_nonproprietary_names(hit: &MyChemHit) -> Vec<&str> {
    let Some(ndc) = hit.ndc.as_ref() else {
        return Vec::new();
    };
    match ndc {
        MyChemNdcField::One(v) => v.nonproprietaryname.as_deref().into_iter().collect(),
        MyChemNdcField::Many(v) => v
            .iter()
            .filter_map(|n| n.nonproprietaryname.as_deref())
            .collect(),
    }
}

fn ndc_proprietary_names(hit: &MyChemHit) -> Vec<&str> {
    let Some(ndc) = hit.ndc.as_ref() else {
        return Vec::new();
    };
    match ndc {
        MyChemNdcField::One(v) => v.proprietaryname.as_deref().into_iter().collect(),
        MyChemNdcField::Many(v) => v
            .iter()
            .filter_map(|n| n.proprietaryname.as_deref())
            .collect(),
    }
}

fn ndc_nonproprietaryname(hit: &MyChemHit) -> Option<&str> {
    ndc_nonproprietary_names(hit).into_iter().next()
}

fn ndc_pharm_classes(hit: &MyChemHit) -> Vec<&str> {
    let Some(ndc) = hit.ndc.as_ref() else {
        return Vec::new();
    };
    match ndc {
        MyChemNdcField::One(v) => v
            .pharm_classes
            .iter()
            .filter_map(MyChemPharmClass::as_str)
            .collect(),
        MyChemNdcField::Many(v) => v
            .iter()
            .flat_map(|n| n.pharm_classes.iter().filter_map(MyChemPharmClass::as_str))
            .collect(),
    }
}

fn clean_moa_class(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = trimmed.strip_suffix("[MoA]").unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix("[EPC]").unwrap_or(trimmed);
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn moa_pharm_classes(hit: &MyChemHit) -> Vec<String> {
    ndc_pharm_classes(hit)
        .into_iter()
        .filter(|v| v.contains("[MoA]"))
        .filter_map(clean_moa_class)
        .collect()
}

fn chebi_name(hit: &MyChemHit) -> Option<&str> {
    hit.chebi.as_ref().and_then(|c| c.name())
}

fn unii_display_name(hit: &MyChemHit) -> Option<&str> {
    hit.unii.as_ref().and_then(|u| u.display_name())
}

fn unii_id(hit: &MyChemHit) -> Option<&str> {
    hit.unii.as_ref().and_then(|u| u.unii())
}

fn string_or_vec_values(value: &crate::utils::serde::StringOrVec) -> Vec<&str> {
    match value {
        crate::utils::serde::StringOrVec::None => Vec::new(),
        crate::utils::serde::StringOrVec::Single(v) => vec![v.as_str()],
        crate::utils::serde::StringOrVec::Multiple(v) => v.iter().map(String::as_str).collect(),
    }
}

fn openfda_generic_names(hit: &MyChemHit) -> Vec<&str> {
    hit.openfda
        .as_ref()
        .map(|o| string_or_vec_values(&o.generic_name))
        .unwrap_or_default()
}

fn openfda_brand_names(hit: &MyChemHit) -> Vec<&str> {
    hit.openfda
        .as_ref()
        .map(|o| string_or_vec_values(&o.brand_name))
        .unwrap_or_default()
}

fn openfda_generic_name(hit: &MyChemHit) -> Option<&str> {
    openfda_generic_names(hit).into_iter().next()
}

fn openfda_brand_name(hit: &MyChemHit) -> Option<&str> {
    openfda_brand_names(hit).into_iter().next()
}

fn best_name_from_hit(hit: &MyChemHit) -> Option<String> {
    // The record's own single-source identity names outrank the merged NDC
    // product rows: a record carrying hundreds of products'
    // rows still names itself, so an unrelated product's ingredient line
    // ("diphenhydramine hydrochloride zinc acetate" on the diphenhydramine
    // record) never becomes the card's name, while a naked product row —
    // whose established line is all it has — keeps naming itself.
    let candidates: [Option<&str>; 8] = [
        openfda_generic_name(hit),
        hit.drugbank.as_ref().and_then(|d| d.name.as_deref()),
        hit.chembl.as_ref().and_then(|c| c.pref_name.as_deref()),
        hit.gtopdb.as_ref().and_then(|g| g.name.as_deref()),
        unii_display_name(hit),
        chebi_name(hit),
        ndc_nonproprietaryname(hit),
        openfda_brand_name(hit),
    ];

    candidates
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(normalize_name)
}

fn hit_all_names(hit: &MyChemHit) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for value in ndc_nonproprietary_names(hit) {
        out.push(normalize_name(value));
    }
    for value in ndc_proprietary_names(hit) {
        out.push(normalize_name(value));
    }
    for value in openfda_generic_names(hit) {
        out.push(normalize_name(value));
    }
    for value in openfda_brand_names(hit) {
        out.push(normalize_name(value));
    }
    if let Some(v) = hit.drugbank.as_ref().and_then(|d| d.name.as_deref()) {
        out.push(normalize_name(v));
    }
    if let Some(drugbank) = hit.drugbank.as_ref() {
        for synonym in &drugbank.synonyms {
            out.push(normalize_name(synonym));
        }
    }
    if let Some(drugcentral) = hit.drugcentral.as_ref() {
        // DrugCentral synonyms carry brands ("tagrisso") that no fetched
        // openFDA block holds, so a brand query selects the drug record the
        // brand lives on.
        for synonym in &drugcentral.synonyms {
            out.push(normalize_name(synonym));
        }
    }
    if let Some(v) = hit.chembl.as_ref().and_then(|c| c.pref_name.as_deref()) {
        out.push(normalize_name(v));
    }
    if let Some(v) = hit.gtopdb.as_ref().and_then(|g| g.name.as_deref()) {
        out.push(normalize_name(v));
    }
    if let Some(v) = unii_display_name(hit) {
        out.push(normalize_name(v));
    }
    if let Some(v) = chebi_name(hit) {
        out.push(normalize_name(v));
    }
    out
}

/// One canonical identity name with its provenance: a merged NDC product
/// row's established name (`product_row`) belongs to one of the many
/// products MyChem folded into the ingredient's record, while every other
/// canonical name is the record's own single-source name. A secondary
/// database label (`secondary`: GtoPdb and ChEBI titles such as
/// "5-fluorouracil" on the fluorouracil record) is the record's own name
/// too, but never names the card: the establishment fields — openFDA
/// generic, DrugBank, ChEMBL, UNII — state the ingredient's plain name, and
/// a request that matches only a secondary label is a synonym the record
/// knows under its plain name.
struct CanonicalName {
    name: String,
    product_row: bool,
    secondary: bool,
}

/// The hit's own identity names, in canonical preference order: every NDC
/// established name (MyChem merges product rows into the ingredient's
/// record, so one record carries many products' names), every openFDA
/// generic name, the DrugBank name, the ChEMBL preferred name, the GtoPdb
/// name, the UNII display name, and the ChEBI name. Brand names are kept
/// out: a brand query canonicalizes to the generic identity.
fn hit_canonical_names(hit: &MyChemHit) -> Vec<CanonicalName> {
    let mut out: Vec<CanonicalName> = Vec::new();
    for value in ndc_nonproprietary_names(hit) {
        out.push(CanonicalName {
            name: normalize_name(value),
            product_row: true,
            secondary: false,
        });
    }
    for value in openfda_generic_names(hit) {
        out.push(CanonicalName {
            name: normalize_name(value),
            product_row: false,
            secondary: false,
        });
    }
    let established: [Option<&str>; 3] = [
        hit.drugbank.as_ref().and_then(|d| d.name.as_deref()),
        hit.chembl.as_ref().and_then(|c| c.pref_name.as_deref()),
        unii_display_name(hit),
    ];
    for value in established.into_iter().flatten() {
        out.push(CanonicalName {
            name: normalize_name(value),
            product_row: false,
            secondary: false,
        });
    }
    let secondary: [Option<&str>; 2] = [
        hit.gtopdb.as_ref().and_then(|g| g.name.as_deref()),
        chebi_name(hit),
    ];
    for value in secondary.into_iter().flatten() {
        out.push(CanonicalName {
            name: normalize_name(value),
            product_row: false,
            secondary: true,
        });
    }
    out
}

/// The names the record itself states as its own product and identity
/// names: `hit_all_names` without the DrugBank and DrugCentral synonym
/// lists. A foreign synonym may extend the query ("Terfenadine
/// carboxylate" is fexofenadine's synonym, never terfenadine's salt form),
/// so the qualified-form tier reads only names the record owns.
fn hit_own_names(hit: &MyChemHit) -> Vec<String> {
    let mut out: Vec<String> = hit_canonical_names(hit)
        .into_iter()
        .map(|entry| entry.name)
        .collect();
    for value in ndc_proprietary_names(hit) {
        out.push(normalize_name(value));
    }
    for value in openfda_brand_names(hit) {
        out.push(normalize_name(value));
    }
    out
}

fn drug_type_from_hit(hit: &MyChemHit) -> Option<String> {
    let v = hit
        .chembl
        .as_ref()
        .and_then(|c| c.molecule_type.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase());

    match v.as_deref() {
        Some("antibody") => Some("biologic".into()),
        Some("small molecule") => Some("small-molecule".into()),
        Some(other) => Some(other.to_string()),
        None => None,
    }
}

fn title_case_words(value: &str) -> String {
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

fn normalize_action_type(action: &str) -> String {
    let v = action.trim().replace('_', " ");
    if v.is_empty() {
        return String::new();
    }
    if v.chars().any(|c| c.is_ascii_lowercase()) {
        return v;
    }
    title_case_words(&v.to_ascii_lowercase())
}

fn chembl_mechanisms_from_hit(hit: &MyChemHit) -> Vec<String> {
    let Some(chembl) = hit.chembl.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    for mech in &chembl.drug_mechanisms {
        if let Some(mechanism) = mech
            .mechanism_of_action
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        {
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
        out.push(mechanism);
    }
    out
}

fn fallback_mechanism_from_hit(hit: &MyChemHit) -> Option<String> {
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

fn normalize_approval_date(value: &str) -> Option<String> {
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

fn approval_date_display(raw: &str) -> Option<String> {
    let normalized = normalize_approval_date(raw)?;
    let year: i32 = normalized[0..4].parse().ok()?;
    let month: u8 = normalized[5..7].parse().ok()?;
    let day: u8 = normalized[8..10].parse().ok()?;
    let month = Month::try_from(month).ok()?;
    Some(format!("{month} {day}, {year}"))
}

fn approval_summary(display: Option<&str>) -> Option<String> {
    display.map(|value| format!("FDA approved on {value}"))
}

fn approval_date_from_hit(hit: &MyChemHit) -> Option<String> {
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

    if !fda_dates.is_empty() {
        fda_dates.sort();
        return fda_dates.first().cloned();
    }

    let mut any_dates = approvals
        .iter()
        .filter_map(|a| a.date.as_deref())
        .filter_map(normalize_approval_date)
        .collect::<Vec<_>>();
    any_dates.sort();
    any_dates.first().cloned()
}

fn first_target_from_hit(hit: &MyChemHit) -> Option<String> {
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

fn name_matches_requested(candidate: &str, requested: &str) -> bool {
    if candidate == requested {
        return true;
    }
    if names_second_ingredient(candidate) {
        return false;
    }
    candidate.starts_with(&format!("{requested} "))
        || candidate.ends_with(&format!(" {requested}"))
        || candidate.contains(&format!(" {requested} "))
}

/// Whether a product name names a combination: a second ingredient joined
/// by "and" or a comma-separated list. The names here arrive already
/// normalized to lowercase. A combination is another product, never a
/// qualified form or canonical name of one requested ingredient — salts
/// and hydrates qualify ("niraparib tosylate"), combinations do not
/// ("niraparib tosylate monohydrate and abiraterone acetate").
fn names_second_ingredient(candidate: &str) -> bool {
    candidate.contains(" and ") || candidate.contains(',')
}

/// Whether a card name names a combination product line rather than one
/// ingredient: a second ingredient joined by "and" or a
/// comma-separated list. The card name arrives normalized.
pub(crate) fn card_names_combination(name: &str) -> bool {
    names_second_ingredient(&name.trim().to_ascii_lowercase())
}

/// The record's own spellings of a multi-word requested brand whose words
/// match the request in a different order or casing: MyChem
/// carries "TYLENOL Extra Strength" on the record a "Extra Strength
/// Tylenol" request resolved through, and the label lookup follows that
/// spelling to reach the record the request's own phrase order never
/// matches. Single-word brands never reorder, so they contribute nothing.
pub(crate) fn brand_spellings_for_request(hits: &[&MyChemHit], requested: &str) -> Vec<String> {
    let requested_words: std::collections::BTreeSet<String> = normalize_name(requested)
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    if requested_words.len() < 2 {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for hit in hits {
        for brand in ndc_proprietary_names(hit)
            .into_iter()
            .chain(openfda_brand_names(hit))
        {
            let brand = brand.trim();
            if brand.is_empty() || normalize_name(brand) == normalize_name(requested) {
                continue;
            }
            let words: std::collections::BTreeSet<String> = normalize_name(brand)
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .filter(|t| !t.is_empty())
                .map(str::to_string)
                .collect();
            if words == requested_words && !out.iter().any(|seen| seen.eq_ignore_ascii_case(brand))
            {
                out.push(brand.to_string());
                if out.len() >= 3 {
                    return out;
                }
            }
        }
    }
    out
}

/// A qualified form of the requested drug: the request followed by more
/// words (salt, hydrate, strength). The request merely appearing inside a
/// longer name does not identify the drug — "Zinc Oxide, Ferric Oxide Red,
/// and Pramoxine Hydrochloride" is not ferric oxide's record
/// and neither does a combination name.
fn name_extends_request(candidate: &str, requested: &str) -> bool {
    candidate.starts_with(&format!("{requested} ")) && !names_second_ingredient(candidate)
}

fn json_first_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.trim().to_string()).filter(|v| !v.is_empty()),
        serde_json::Value::Array(arr) => arr.iter().find_map(json_first_string),
        serde_json::Value::Object(obj) => obj.values().find_map(json_first_string),
        _ => None,
    }
}

fn interactions_from_hit(hit: &MyChemHit) -> Vec<DrugInteraction> {
    let Some(drugbank) = hit.drugbank.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<DrugInteraction> = Vec::new();
    for row in &drugbank.drug_interactions {
        let Some(obj) = row.as_object() else { continue };
        let drug = obj
            .get("name")
            .or_else(|| obj.get("drug"))
            .or_else(|| obj.get("drug_name"))
            .or_else(|| obj.get("drugbank_name"))
            .and_then(json_first_string);
        let Some(drug) = drug else { continue };
        let description = obj
            .get("description")
            .or_else(|| obj.get("interaction"))
            .or_else(|| obj.get("comment"))
            .and_then(json_first_string);
        out.push(DrugInteraction {
            drug,
            ddinter_id: None,
            level: None,
            description,
            partner_classes: Vec::new(),
        });
    }

    out
}

pub fn from_mychem_search_hit(hit: &MyChemHit) -> Option<DrugSearchResult> {
    let name = best_name_from_hit(hit)?;
    let mechanisms = chembl_mechanisms_from_hit(hit);
    let mechanism = mechanisms
        .first()
        .cloned()
        .or_else(|| fallback_mechanism_from_hit(hit));
    let target = first_target_from_hit(hit);

    let drugbank_id = hit
        .drugbank
        .as_ref()
        .and_then(|d| d.id.clone())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    Some(DrugSearchResult {
        name,
        drugbank_id,
        drug_type: drug_type_from_hit(hit),
        mechanism,
        target,
    })
}

pub fn select_hits_for_name<'a>(hits: &'a [MyChemHit], name: &str) -> Vec<&'a MyChemHit> {
    let target = normalize_name(name);
    // A MyChem full-text query returns any drug whose record
    // mentions the term. Only a hit whose own name, brand, or synonym equals
    // the query identifies the drug; when no exact name exists, a qualified
    // form of the same drug (salt, hydrate: "edetate disodium anhydrous")
    // still does, and a foreign synonym never does. Nothing else is
    // admitted, and there is no all-hits fallback: the caller refuses
    // honestly instead of merging another drug's record.
    // Several unrelated drugs can share one product name
    // ("Pain Relief" brands acetaminophen, lidocaine and ibuprofen products
    // alike), and a name several drugs share names no one drug. Every
    // identity-bearing hit votes for the drug its own record says the name
    // means; disagreeing votes refuse the whole tier, so one drug's
    // identifiers never merge with another's targets or label, and records
    // that agree (one product's ingredient rows) still merge.
    // A query that equals one record's own established name
    // (a non-product canonical name: "acetaminophen" is the acetaminophen
    // record's DrugBank name) selects that record over identity-bearing
    // records that only carry the query on another product's row, and a
    // vote that names a combination product line abstains: the phenylephrine record's
    // "Acetaminophen, ..." row is a product mention, not an identity claim,
    // so it never outvotes the record that is the ingredient.
    let mut out: Vec<&MyChemHit> = hits
        .iter()
        .filter(|h| hit_all_names(h).iter().any(|n| n == &target))
        .collect();
    if out.is_empty() {
        out = hits
            .iter()
            .filter(|h| {
                hit_own_names(h)
                    .iter()
                    .any(|n| name_extends_request(n, &target))
            })
            .collect();
    } else {
        // Only identity-bearing records compete for the name, and a bare
        // product row merges with whatever tier holds it. Records of the
        // same drug's qualified forms stay in the tier with the record the
        // query names (the erlotinib hydrochloride record merges with
        // erlotinib's), while records of other drugs that only carry the
        // query on a product row drop out.
        let self_named: Vec<&MyChemHit> = out
            .iter()
            .copied()
            .filter(|h| hit_has_identity(h) && hit_names_itself(h, &target))
            .collect();
        if !self_named.is_empty() {
            let self_names: Vec<String> = self_named
                .iter()
                .flat_map(|hit| {
                    hit_canonical_names(hit)
                        .into_iter()
                        .filter(|entry| !entry.product_row && !entry.secondary)
                        .map(|entry| entry.name)
                        .collect::<Vec<_>>()
                })
                .collect();
            out.retain(|h| {
                !hit_has_identity(h)
                    || hit_names_itself(h, &target)
                    || hit_names_a_qualified_form_of(h, &self_names)
            });
        }
        match shared_name_vote(&out, name) {
            SharedNameVote::Disagree => return Vec::new(),
            SharedNameVote::Agree { plain: Some(vote) } => {
                out.retain(|h| {
                    !hit_has_identity(h) || hit_resolution_vote(h, name).as_ref() == Some(&vote)
                });
            }
            SharedNameVote::Agree { plain: None } => {}
        }
    }

    // Prefer richer hits first (more sources).
    out.sort_by_key(|h| {
        let mut score: i32 = 0;
        if h.drugbank.as_ref().and_then(|d| d.id.as_deref()).is_some() {
            score -= 100;
        }
        if h.chembl
            .as_ref()
            .and_then(|c| c.molecule_chembl_id.as_deref())
            .is_some()
        {
            score -= 50;
        }
        if unii_id(h).is_some() {
            score -= 25;
        }
        if ndc_nonproprietaryname(h).is_some() {
            score -= 10;
        }
        score
    });

    out
}

/// The card name a hit contributes for a request: the first canonical name
/// that itself matches the request, so an NDC combination-product row
/// ("Analgesic", "urea, glycerin, aloe, disodium edta") never renames the
/// drug the query actually matched. Brand names are not
/// candidates; a brand that matches while no canonical name does falls back
/// to the generic identity instead.
/// A merged NDC product row's name counts as the record's own only when it
/// equals the request, when the request itself names a product or brand the
/// record carries: one record holds many products' rows, so a
/// request naming one product ("advil", "headache relief") takes that
/// product's own pairing instead of another row's wider name ("advil pm",
/// "tension headache relief"). An ingredient request keeps the
/// qualified-form rule ("imatinib mesylate" still matches "imatinib"),
/// and every non-product canonical name keeps the loose rule.
/// A card name never carries a product row's strength: the
/// request and the row names compare with their trailing strength run
/// stripped, and a secondary database label never names the card (the
/// "5-fluorouracil" GtoPdb/ChEBI title on the fluorouracil record stays a
/// synonym).
fn canonical_name_matching_request(hit: &MyChemHit, requested: &str) -> Option<String> {
    let target = normalize_name(strip_trailing_strength(requested.trim()));
    let request_names_a_carried_product = ndc_proprietary_names(hit)
        .iter()
        .chain(openfda_brand_names(hit).iter())
        .any(|name| normalize_name(name) == target);
    hit_canonical_names(hit)
        .into_iter()
        .find(|entry| {
            if entry.secondary {
                return false;
            }
            let entry_name = strip_trailing_strength(&entry.name);
            if entry.product_row && request_names_a_carried_product {
                entry_name == target
            } else {
                name_matches_requested(entry_name, &target)
            }
        })
        .map(|entry| strip_trailing_strength(&entry.name).to_string())
}

/// The nonproprietary name MyChem pairs with the requested brand on one
/// NDC row. MyChem merges many products' rows into one ingredient record,
/// and the rows themselves carry the pairing: proprietaryname ZEJULA sits on
/// rows whose nonproprietaryname is niraparib, while the Akeega combination
/// rows sit first. A brand card takes the name its own product row carries,
/// never another product's row. The row name arrives with its
/// trailing strength stripped: a product row's strength is the product's,
/// never the card's ("aspirin 81 mg" pairs the brand with aspirin).
fn brand_paired_nonproprietary_name(hit: &MyChemHit, requested: &str) -> Option<String> {
    let target = normalize_name(strip_trailing_strength(requested.trim()));
    let ndc = hit.ndc.as_ref()?;
    let rows = match ndc {
        MyChemNdcField::One(v) => std::slice::from_ref(v),
        MyChemNdcField::Many(v) => v.as_slice(),
    };
    rows.iter().find_map(|row| {
        let brand = row
            .proprietaryname
            .as_deref()
            .map(normalize_name)
            .map(|value| strip_trailing_strength(&value).to_string())
            .filter(|plain| !plain.is_empty())?;
        let plain = row
            .nonproprietaryname
            .as_deref()
            .map(normalize_name)
            .map(|value| strip_trailing_strength(&value).to_string())
            .filter(|plain| !plain.is_empty())?;
        (brand == target).then_some(plain)
    })
}

/// A hit with its own stable identifier: an identity-bearing drug record,
/// not a bare NDC product row. Only such a record's pairing may name the
/// card: a naked product row shares a record with nothing,
/// so its row name arrives through the generic fallback anyway, and letting
/// it rename the card would take a salt form such as "amivantamab-vmjw"
/// from a Rybrevant row over the identity record's own name.
fn hit_has_identity(hit: &MyChemHit) -> bool {
    hit.drugbank
        .as_ref()
        .and_then(|d| d.id.as_deref())
        .is_some()
        || hit
            .chembl
            .as_ref()
            .and_then(|c| c.molecule_chembl_id.as_deref())
            .is_some()
        || unii_id(hit).is_some()
}

/// The drug this hit says the requested name means: the ingredient its own
/// NDC row pairs with the requested product name, else the first canonical
/// name that matches the request, else the record's leading canonical
/// identity name. Every value arrives normalized, so votes
/// compare by equality.
fn hit_resolution_vote(hit: &MyChemHit, requested: &str) -> Option<String> {
    brand_paired_nonproprietary_name(hit, requested)
        .or_else(|| canonical_name_matching_request(hit, requested))
        .or_else(|| {
            hit_canonical_names(hit)
                .into_iter()
                .next()
                .map(|entry| entry.name)
        })
}

/// Whether the query equals one of the hit's own established names: a
/// non-product, non-secondary canonical name. A product row
/// that happens to equal the query is one product's line among many, not
/// the record's identity.
fn hit_names_itself(hit: &MyChemHit, target: &str) -> bool {
    hit_canonical_names(hit)
        .iter()
        .any(|entry| !entry.product_row && !entry.secondary && entry.name == target)
}

/// Whether one of the hit's own established names is a qualified form of
/// one of the given names, or the reverse: salt and base records of one
/// drug name the same drug either way ("erlotinib hydrochloride" and
/// "erlotinib"), so they merge instead of competing: the qualified-form
/// rule, kept through the self-named narrowing.
fn hit_names_a_qualified_form_of(hit: &MyChemHit, names: &[String]) -> bool {
    hit_canonical_names(hit).iter().any(|entry| {
        !entry.product_row
            && !entry.secondary
            && names.iter().any(|name| {
                name_extends_request(&entry.name, name) || name_extends_request(name, &entry.name)
            })
    })
}

/// What the exact tier's identity-bearing hits say the requested name means
///Only identity-bearing hits vote: a bare product
/// row shares a record with nothing, so it never decides the name, and zero
/// voters (a naked-row-only tier, the Keytruda shape) never disagree. The
/// plain votes decide first: a combination vote is a product mention on
/// another ingredient's record, not an identity claim, so it abstains while
/// a plain vote exists — one agreed plain vote narrows the tier to the
/// records that mean it, so abstaining records of other drugs never merge
/// into the card (the acetaminophen record decides "Tylenol"; the
/// phenylephrine record's combination row never outvotes it). With no plain
/// vote at all, the combination votes decide as ingredient word sets: one
/// shared combination line names one product the records agree on
/// (Lonsurf's two ingredient records, Motrin's PM pairing), and distinct
/// sets are distinct products sharing the name, which names no one drug
/// ("Cold and Flu").
enum SharedNameVote {
    Disagree,
    Agree { plain: Option<String> },
}

fn vote_ingredient_words(vote: &str) -> std::collections::BTreeSet<String> {
    vote.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty() && *token != "and")
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn shared_name_vote(out: &[&MyChemHit], requested: &str) -> SharedNameVote {
    let mut first: Option<String> = None;
    for hit in out {
        if !hit_has_identity(hit) {
            continue;
        }
        let Some(vote) = hit_resolution_vote(hit, requested) else {
            continue;
        };
        if names_second_ingredient(&vote) {
            continue;
        }
        match &first {
            None => first = Some(vote),
            Some(seen) if seen == &vote => {}
            Some(_) => return SharedNameVote::Disagree,
        }
    }
    if first.is_some() {
        return SharedNameVote::Agree { plain: first };
    }
    let mut first_words: Option<std::collections::BTreeSet<String>> = None;
    for hit in out {
        if !hit_has_identity(hit) {
            continue;
        }
        let Some(vote) = hit_resolution_vote(hit, requested) else {
            continue;
        };
        let words = vote_ingredient_words(&vote);
        if words.is_empty() {
            continue;
        }
        match &first_words {
            None => first_words = Some(words),
            Some(seen) if *seen == words => {}
            Some(_) => return SharedNameVote::Disagree,
        }
    }
    SharedNameVote::Agree { plain: None }
}

pub fn merge_mychem_hits(hits: &[&MyChemHit], requested_name: &str) -> Drug {
    // A brand query names the card for the drug its own
    // product row pairs. A canonical name that merely equals the brand
    // string (a CHEBI record named "Lonsurf") or no canonical match at all
    // yields to that pairing; a canonical name that extends the query
    // ("imatinib mesylate" for imatinib) keeps naming the card, and the
    // pairing counts only on identity-bearing records.
    let requested = normalize_name(requested_name);
    let canonical = hits
        .iter()
        .find_map(|hit| canonical_name_matching_request(hit, requested_name));
    let brand_paired = hits.iter().find_map(|hit| {
        hit_has_identity(hit)
            .then(|| brand_paired_nonproprietary_name(hit, requested_name))
            .flatten()
    });
    let mut name = match (canonical, brand_paired) {
        (Some(canonical), _) if canonical != requested => canonical,
        (_, Some(brand_paired)) => brand_paired,
        (Some(canonical), None) => canonical,
        (None, None) => hits
            .iter()
            .find_map(|hit| openfda_generic_name(hit).map(normalize_name))
            .or_else(|| hits.iter().find_map(|hit| best_name_from_hit(hit)))
            .unwrap_or_else(|| requested.clone()),
    };
    // A combination card shows product-level identifiers or
    // none. The merged records are one ingredient's records, so their
    // stable identifiers, approval dates and synonym lists are that
    // ingredient's facts, not the combination product's, and one
    // ingredient's identity must never stand in for the whole product
    // ("Headache Relief" never shows acetaminophen's DrugBank id,
    // approval date or chemical synonyms). The product's own names reach
    // the card through the openFDA label metadata instead.
    let combination_card = names_second_ingredient(&name);
    let mut drugbank_id: Option<String> = None;
    let mut chembl_id: Option<String> = None;
    let mut unii: Option<String> = None;
    let mut drug_type: Option<String> = None;
    let mut mechanisms: Vec<String> = Vec::new();
    let mut mechanisms_seen: HashSet<String> = HashSet::new();
    let brand_names: Vec<String> = Vec::new();
    let mut synonyms_seen: HashSet<String> = HashSet::new();
    let mut ddinter_synonyms: Vec<String> = Vec::new();
    // Only the chosen anchor hit feeds the synonym fold:
    // a pooled combination-product synonym would widen DDInter identity
    // to a real interaction partner, and interactions aggregation then
    // skips that row silently because both sides match the anchor.
    let anchor_index = hits
        .iter()
        .position(|hit| best_name_from_hit(hit).is_some());

    let mut targets: Vec<String> = Vec::new();
    let mut indications: Vec<String> = Vec::new();
    let mut pharm_classes: Vec<String> = Vec::new();
    let mut interactions: Vec<DrugInteraction> = Vec::new();

    let mut targets_seen: HashSet<String> = HashSet::new();
    let mut indications_seen: HashSet<String> = HashSet::new();
    let mut classes_seen: HashSet<String> = HashSet::new();
    let mut interactions_seen: HashSet<String> = HashSet::new();
    let mut approval_date: Option<String> = None;

    for (hit_index, hit) in hits.iter().enumerate() {
        if name.is_empty()
            && let Some(n) = best_name_from_hit(hit)
        {
            name = n;
        }

        if drugbank_id.is_none() && !combination_card {
            drugbank_id = hit
                .drugbank
                .as_ref()
                .and_then(|d| d.id.clone())
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if chembl_id.is_none() && !combination_card {
            chembl_id = hit
                .chembl
                .as_ref()
                .and_then(|c| c.molecule_chembl_id.clone())
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if unii.is_none() && !combination_card {
            unii = unii_id(hit)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if drug_type.is_none() {
            drug_type = drug_type_from_hit(hit);
        }
        if approval_date.is_none() && !combination_card {
            approval_date = approval_date_from_hit(hit);
        }

        if !combination_card
            && Some(hit_index) == anchor_index
            && let Some(drugbank) = hit.drugbank.as_ref()
        {
            for synonym in &drugbank.synonyms {
                let synonym = synonym.trim();
                if synonym.is_empty() {
                    continue;
                }
                if synonym.eq_ignore_ascii_case(&name)
                    || synonym.eq_ignore_ascii_case(requested_name)
                {
                    continue;
                }
                let key = synonym.to_ascii_lowercase();
                if !synonyms_seen.insert(key) {
                    continue;
                }
                // The synonym list feeds DDInter identity matching only:
                // a synonym is not a brand the drug's own labels carry, so
                // chemical names and aliases never reach brand_names, and
                // the loop does not break at any cap.
                if ddinter_synonyms.len() < 32 {
                    ddinter_synonyms.push(synonym.to_string());
                }
            }
        }

        if mechanisms.len() < 3 {
            for mechanism in chembl_mechanisms_from_hit(hit) {
                let key = mechanism.to_ascii_lowercase();
                if !mechanisms_seen.insert(key) {
                    continue;
                }
                mechanisms.push(mechanism);
                if mechanisms.len() >= 3 {
                    break;
                }
            }
        }

        if let Some(gtopdb) = hit.gtopdb.as_ref() {
            for t in &gtopdb.interaction_targets {
                let Some(sym) = t.symbol.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
                    continue;
                };
                let sym = sym.to_string();
                if targets_seen.insert(sym.clone()) {
                    targets.push(sym);
                }
            }
        }

        if let Some(dc) = hit.drugcentral.as_ref()
            && let Some(use_) = dc.drug_use.as_ref()
        {
            for ind in &use_.indication {
                let Some(name) = ind
                    .concept_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                let key = name.to_ascii_lowercase();
                if indications_seen.insert(key) {
                    indications.push(name.to_string());
                }
            }
        }

        for cls in moa_pharm_classes(hit) {
            let key = cls.to_ascii_lowercase();
            if classes_seen.insert(key) {
                pharm_classes.push(cls);
            }
        }

        if !combination_card && interactions.len() < 15 {
            for row in interactions_from_hit(hit) {
                let key = row.drug.to_ascii_lowercase();
                if !interactions_seen.insert(key) {
                    continue;
                }
                interactions.push(row);
                if interactions.len() >= 15 {
                    break;
                }
            }
        }
    }

    targets.sort();
    indications.sort();
    interactions.sort_by(|a, b| a.drug.cmp(&b.drug));
    pharm_classes.truncate(6);
    targets.truncate(8);
    indications.truncate(6);

    if mechanisms.is_empty() {
        for hit in hits {
            if let Some(mechanism) = fallback_mechanism_from_hit(hit) {
                mechanisms.push(mechanism);
                break;
            }
        }
    }

    let mechanism = mechanisms.first().cloned();
    let approval_date_raw = approval_date.clone();
    let approval_date_display = approval_date_raw
        .as_deref()
        .and_then(approval_date_display)
        .or_else(|| approval_date_raw.clone());
    let approval_summary = approval_summary(approval_date_display.as_deref());

    Drug {
        section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
        name,
        drugbank_id,
        chembl_id,
        unii,
        drug_type,
        mechanism,
        mechanisms,
        approval_date,
        approval_date_raw,
        approval_date_display,
        approval_summary,
        brand_names,
        route: None,
        targets,
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications,
        interactions,
        interaction_text: None,
        interaction_pagination: None,
        interaction_bundle_freshness: None,
        interaction_coverage_status: None,
        ddinter_synonyms,
        pharm_classes,
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        fda_orphan_designations: None,
        us_safety_warnings: None,
        us_boxed_warning: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        who_note: None,
        civic: None,
        cell_lines: None,
    }
}

#[cfg(test)]
pub(crate) mod name_resolution_tests;

#[cfg(test)]
pub(crate) mod shared_name_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_date_rejects_multibyte_input_with_matching_byte_length() {
        assert_eq!(normalize_approval_date("β23456789"), None);
    }

    #[test]
    fn search_mechanism_ranking_prefers_kinase_moa_and_rejects_metabolism_only() {
        let fixtures = [
            serde_json::json!({
                "_id": "dabrafenib",
                "drugbank": {"name": "Dabrafenib"},
                "ndc": {"pharm_classes": [
                    "Cytochrome P450 2C9 Inducers [MoA]",
                    "Protein Kinase Inhibitors [MoA]"
                ]}
            }),
            serde_json::json!({
                "_id": "vemurafenib",
                "drugbank": {"name": "Vemurafenib"},
                "ndc": {"pharm_classes": [
                    "Inhibitor of Serine/threonine-protein kinase B-raf [MoA]"
                ]}
            }),
            serde_json::json!({
                "_id": "metabolism-only",
                "drugbank": {"name": "Example drug"},
                "ndc": {"pharm_classes": [
                    "Cytochrome P450 2C9 Inducers [MoA]"
                ]}
            }),
        ];

        let mechanisms = fixtures
            .into_iter()
            .map(|fixture| {
                let hit: MyChemHit = serde_json::from_value(fixture).expect("valid search hit");
                from_mychem_search_hit(&hit).and_then(|row| row.mechanism)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            mechanisms,
            vec![
                Some("Protein Kinase Inhibitors".to_string()),
                Some("Inhibitor of Serine/threonine-protein kinase B-raf".to_string()),
                None,
            ]
        );
    }

    #[test]
    fn search_mechanism_prefers_chembl_over_ranked_moa_fallback() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "dabrafenib-with-chembl",
            "drugbank": {"name": "Dabrafenib"},
            "chembl": {"drug_mechanisms": [{
                "mechanism_of_action": "BRAF inhibitor"
            }]},
            "ndc": {"pharm_classes": ["Protein Kinase Inhibitors [MoA]"]}
        }))
        .expect("valid search hit");

        let row = from_mychem_search_hit(&hit).expect("named hit should render");
        assert_eq!(row.mechanism.as_deref(), Some("BRAF inhibitor"));
    }

    #[test]
    fn merge_mychem_hits_collects_deduped_mechanisms() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 1.0,
            "chembl": {
                "molecule_chembl_id": "CHEMBL1",
                "molecule_type": "Small molecule",
                "pref_name": "test",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "BRAF"},
                    {"action_type": "INHIBITOR", "target_name": "BRAF"},
                    {"action_type": "AGONIST", "target_name": "TP53"},
                    {"action_type": "ANTAGONIST", "target_name": "EGFR"},
                    {"action_type": "BLOCKER", "target_name": "ALK"}
                ]
            }
        }))
        .expect("valid JSON");

        let drug = merge_mychem_hits(&[&hit], "test");
        assert_eq!(drug.mechanisms.len(), 3, "mechanisms should be limited");
        assert_eq!(drug.mechanisms[0], "Inhibitor of BRAF");
        assert_eq!(drug.mechanisms[1], "Agonist of TP53");
        assert_eq!(drug.mechanisms[2], "Antagonist of EGFR");
        assert_eq!(drug.mechanism.as_deref(), Some("Inhibitor of BRAF"));
    }

    #[test]
    fn merge_mychem_hits_keeps_synonyms_off_the_brand_list() {
        // More than three DrugBank synonyms: every one still reaches
        // ddinter_synonyms for interaction identity, and none reaches
        // brand_names — a chemical name or alias is not a brand the drug's
        // own labels carry. The interactions seam still sees it.
        let anchor: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 10.0,
            "drugbank": {
                "name": "Aspirin",
                "id": "DB00945",
                "synonyms": [
                    "Bayer",
                    "ECM",
                    "2-Acetoxybenzoic acid",
                    "acetylsalicylic acid",
                    "Acenterine"
                ]
            }
        }))
        .expect("valid anchor hit");

        let drug = merge_mychem_hits(&[&anchor], "aspirin");
        assert!(
            drug.ddinter_synonyms
                .iter()
                .any(|synonym| synonym.eq_ignore_ascii_case("acetylsalicylic acid"))
        );
        assert!(
            drug.brand_names.is_empty(),
            "a DrugBank synonym never lists as a brand: {:?}",
            drug.brand_names
        );

        let identity =
            crate::entities::drug::interactions::ddinter_identity_for_anchor("aspirin", &drug);
        assert!(identity.terms().contains(
            &crate::sources::ddinter::normalize_name_key("acetylsalicylic acid").expect("key")
        ));
    }

    #[test]
    fn merge_mychem_hits_does_not_pool_synonyms_across_hits() {
        // A combination product's synonym must not widen the anchor's
        // DDInter identity: pooled, "dipyridamole" would name a real
        // interaction partner and the aggregation would skip that row
        // because both sides match the anchor.
        let anchor: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 10.0,
            "drugbank": {
                "name": "Aspirin",
                "synonyms": ["Bayer", "acetylsalicylic acid"]
            }
        }))
        .expect("valid anchor hit");
        let combination: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "2",
            "_score": 5.0,
            "drugbank": {
                "name": "Aspirin/dipyridamole",
                "synonyms": ["dipyridamole", "Aggrenox"]
            }
        }))
        .expect("valid combination-product hit");

        let drug = merge_mychem_hits(&[&anchor, &combination], "aspirin");
        assert!(
            drug.ddinter_synonyms
                .iter()
                .any(|synonym| synonym.eq_ignore_ascii_case("acetylsalicylic acid"))
        );
        assert!(
            drug.ddinter_synonyms
                .iter()
                .all(|synonym| !synonym.eq_ignore_ascii_case("dipyridamole"))
        );
        assert!(
            drug.brand_names
                .iter()
                .all(|synonym| !synonym.eq_ignore_ascii_case("dipyridamole"))
        );

        let identity =
            crate::entities::drug::interactions::ddinter_identity_for_anchor("aspirin", &drug);
        assert!(
            !identity.terms().contains(
                &crate::sources::ddinter::normalize_name_key("dipyridamole").expect("key")
            )
        );
    }

    #[test]
    fn select_hits_for_name_prefers_exact_hits_and_admits_salt_forms_only_when_alone() {
        // An exact name match identifies the drug, so only
        // exact hits merge. A qualified form of the same drug (salt, hydrate)
        // still matches when it alone leads with the query, and a query no
        // hit names refuses instead of falling back to every hit.
        let base: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 1.0,
            "drugbank": {"name": "Dabrafenib"}
        }))
        .expect("valid base hit");

        let salt: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "2",
            "_score": 1.0,
            "chembl": {
                "pref_name": "DABRAFENIB MESYLATE",
                "molecule_chembl_id": "CHEMBL2105729",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "BRAF"}
                ]
            }
        }))
        .expect("valid salt hit");

        let foreign: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "3",
            "_score": 5.0,
            "drugbank": {
                "id": "DBFOREIGN",
                "name": "Dabrafenib carboxylate"
            }
        }))
        .expect("valid foreign hit");

        let exact_and_beyond = [base.clone(), salt.clone(), foreign];
        let selected = select_hits_for_name(&exact_and_beyond, "dabrafenib");
        assert_eq!(selected.len(), 1);
        assert_eq!(
            selected[0]
                .drugbank
                .as_ref()
                .and_then(|d| d.name.as_deref()),
            Some("Dabrafenib")
        );

        let salt_alone = [salt];
        let selected = select_hits_for_name(&salt_alone, "dabrafenib");
        assert_eq!(selected.len(), 1);

        let unrelated: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "4",
            "_score": 1.0,
            "drugbank": {"id": "DBOTHER", "name": "Nivolumab"}
        }))
        .expect("valid unrelated hit");
        let nothing_matches = [unrelated];
        assert!(select_hits_for_name(&nothing_matches, "dabrafenib").is_empty());
    }

    #[test]
    fn merge_mychem_hits_keeps_the_matching_canonical_name_over_the_first_ndc_row() {
        // MyChem merges NDC product rows into the ingredient's
        // record, and the first row can be another product's name. The card
        // is named for the field that matched the request.
        let mannitol: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "FBPFZTCFMRRESA-KVTDHHQDSA-N",
            "_score": 30.6,
            "drugbank": {"id": "DB00742", "name": "Mannitol"},
            "ndc": [
                {"nonproprietaryname": "Analgesic"},
                {"nonproprietaryname": "mannitol"},
                {"nonproprietaryname": "MANNITOL"}
            ]
        }))
        .expect("valid mannitol hit");

        let drug = merge_mychem_hits(&[&mannitol], "mannitol");
        assert_eq!(drug.name, "mannitol");
        assert_eq!(drug.drugbank_id.as_deref(), Some("DB00742"));
        assert!(
            !serde_json::to_string(&drug)
                .expect("card serializes")
                .to_ascii_lowercase()
                .contains("analgesic")
        );
    }

    #[test]
    fn merge_mychem_hits_collects_drug_interactions() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 1.0,
            "drugbank": {
                "id": "DB0001",
                "name": "warfarin",
                "drug_interactions": [
                    {"name": "Aspirin", "description": "May increase bleeding risk."},
                    {"name": "Clopidogrel", "description": "Monitor for bleeding."}
                ]
            }
        }))
        .expect("valid JSON");

        assert_eq!(
            hit.drugbank
                .as_ref()
                .map(|d| d.drug_interactions.len())
                .unwrap_or_default(),
            2
        );
        let drug = merge_mychem_hits(&[&hit], "warfarin");
        assert_eq!(drug.interactions.len(), 2);
        assert_eq!(drug.interactions[0].drug, "Aspirin");
    }

    #[test]
    fn drug_sections_maps_osimertinib() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "DB09330",
            "_score": 1.0,
            "drugbank": {"id": "DB09330", "name": "osimertinib"},
            "chembl": {
                "molecule_chembl_id": "CHEMBL3353410",
                "molecule_type": "Small molecule",
                "pref_name": "OSIMERTINIB",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "EGFR"}
                ]
            },
            "gtopdb": {
                "interaction_targets": [{"symbol": "EGFR"}]
            },
            "drugcentral": {
                "approval": [{"agency": "FDA", "date": "20151113"}],
                "drug_use": {"indication": [{"concept_name": "Non-small cell lung cancer"}]}
            }
        }))
        .expect("valid osimertinib hit");

        let drug = merge_mychem_hits(&[&hit], "osimertinib");
        assert_eq!(drug.name, "osimertinib");
        assert_eq!(drug.targets.first().map(String::as_str), Some("EGFR"));
        assert_eq!(drug.drug_type.as_deref(), Some("small-molecule"));
        assert_eq!(drug.approval_date.as_deref(), Some("2015-11-13"));
        assert_eq!(drug.approval_date_raw.as_deref(), Some("2015-11-13"));
        assert_eq!(
            drug.approval_date_display.as_deref(),
            Some("November 13, 2015")
        );
        assert_eq!(
            drug.approval_summary.as_deref(),
            Some("FDA approved on November 13, 2015")
        );
    }

    #[test]
    fn drug_sections_maps_imatinib() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "DB00619",
            "_score": 1.0,
            "drugbank": {"id": "DB00619", "name": "imatinib"},
            "chembl": {
                "molecule_chembl_id": "CHEMBL941",
                "molecule_type": "Small molecule",
                "pref_name": "IMATINIB",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "ABL1"}
                ]
            },
            "gtopdb": {
                "interaction_targets": [{"symbol": "ABL1"}]
            }
        }))
        .expect("valid imatinib hit");

        let drug = merge_mychem_hits(&[&hit], "imatinib");
        assert_eq!(drug.name, "imatinib");
        assert_eq!(drug.targets.first().map(String::as_str), Some("ABL1"));
        assert!(
            drug.mechanism
                .as_deref()
                .is_some_and(|v| v.to_ascii_lowercase().contains("inhibitor"))
        );
    }

    #[test]
    fn from_mychem_search_hit_uses_openfda_names_when_other_sources_are_missing() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "openfda-only",
            "_score": 42.0,
            "openfda": {
                "brand_name": "Keytruda",
                "generic_name": "pembrolizumab"
            }
        }))
        .expect("valid openfda-only hit");

        let row = from_mychem_search_hit(&hit).expect("openfda names should produce a row");
        assert_eq!(row.name, "pembrolizumab");
    }

    #[test]
    fn approval_date_display_formats_month_name() {
        assert_eq!(
            approval_date_display("2014-09-04").as_deref(),
            Some("September 4, 2014")
        );
        assert_eq!(
            approval_date_display("20140904").as_deref(),
            Some("September 4, 2014")
        );
    }

    #[test]
    fn select_hits_for_name_matches_openfda_brand_name() {
        let keytruda: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "brand-hit",
            "_score": 10.0,
            "openfda": {
                "brand_name": "Keytruda",
                "generic_name": "pembrolizumab"
            }
        }))
        .expect("valid brand hit");

        let unrelated: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "other-hit",
            "_score": 1.0,
            "drugbank": {"name": "nivolumab"}
        }))
        .expect("valid unrelated hit");

        let hits = [keytruda, unrelated];
        let selected = select_hits_for_name(&hits, "keytruda");
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn merge_mychem_hits_prefers_canonical_name_from_brand_hit() {
        let keytruda: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "brand-hit",
            "_score": 10.0,
            "openfda": {
                "brand_name": "Keytruda",
                "generic_name": "pembrolizumab"
            }
        }))
        .expect("valid brand hit");

        let drug = merge_mychem_hits(&[&keytruda], "keytruda");
        assert_eq!(drug.name, "pembrolizumab");
    }
}
