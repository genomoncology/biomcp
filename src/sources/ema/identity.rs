use std::collections::HashSet;

use crate::entities::drug::DrugSearchMatchKind;

use super::{clean_text, field_matches_terms, normalize_term};

#[derive(Debug, Clone)]
pub(crate) struct EmaDrugIdentity {
    terms: Vec<EmaIdentityTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmaIdentitySource {
    Query,
    OpenFdaGenericName,
    NdcNonproprietaryName,
    DrugbankName,
    ChemblPrefName,
    OpenFdaBrandName,
    CvxShortDescription,
    CvxFullVaccineName,
}

impl EmaIdentitySource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::OpenFdaGenericName => "openfda.generic_name",
            Self::NdcNonproprietaryName => "ndc.nonproprietaryname",
            Self::DrugbankName => "drugbank.name",
            Self::ChemblPrefName => "chembl.pref_name",
            Self::OpenFdaBrandName => "openfda.brand_name",
            Self::CvxShortDescription => "cvx_short_description",
            Self::CvxFullVaccineName => "cvx_full_vaccine_name",
        }
    }

    fn is_cvx(self) -> bool {
        matches!(self, Self::CvxShortDescription | Self::CvxFullVaccineName)
    }
}

#[derive(Debug, Clone)]
struct EmaIdentityTerm {
    text: String,
    normalized: String,
    source: EmaIdentitySource,
}

impl EmaDrugIdentity {
    pub(crate) fn new(primary: &str) -> Self {
        Self::from_typed_terms(primary, Vec::new())
    }

    pub(crate) fn with_aliases(primary: &str, canonical: Option<&str>, aliases: &[String]) -> Self {
        let mut terms = Vec::new();
        if let Some(canonical) = canonical {
            terms.push((canonical.to_string(), EmaIdentitySource::DrugbankName));
        }
        terms.extend(
            aliases
                .iter()
                .cloned()
                .map(|term| (term, EmaIdentitySource::OpenFdaBrandName)),
        );
        Self::from_typed_terms(primary, terms)
    }

    pub(crate) fn from_typed_terms(primary: &str, terms: Vec<(String, EmaIdentitySource)>) -> Self {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (term, source) in
            std::iter::once((primary.to_string(), EmaIdentitySource::Query)).chain(terms)
        {
            let Some(text) = clean_identity_text(&term) else {
                continue;
            };
            let normalized = text.to_ascii_lowercase();
            if seen.insert(normalized.clone()) {
                out.push(EmaIdentityTerm {
                    text,
                    normalized,
                    source,
                });
            }
        }
        Self { terms: out }
    }

    pub(super) fn term_set(&self) -> HashSet<String> {
        self.terms
            .iter()
            .map(|term| term.normalized.clone())
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn terms_for_test(&self) -> Vec<(&str, &str)> {
        self.terms
            .iter()
            .map(|term| (term.text.as_str(), term.source.as_str()))
            .collect()
    }
}

fn clean_identity_text(value: &str) -> Option<String> {
    clean_text(value.trim().trim_matches('.'))
}

pub(super) fn classify_ema_match(
    identity: &EmaDrugIdentity,
    normalized_name: Option<&str>,
    normalized_active: Option<&str>,
    fields: &[&str; 3],
) -> Option<(DrugSearchMatchKind, String, EmaIdentitySource)> {
    let primary = identity.terms.first()?;
    if normalized_name == Some(primary.normalized.as_str()) {
        return Some((
            DrugSearchMatchKind::ProductName,
            primary.text.clone(),
            EmaIdentitySource::Query,
        ));
    }
    if normalized_active == Some(primary.normalized.as_str()) {
        return Some((
            DrugSearchMatchKind::ActiveSubstance,
            primary.text.clone(),
            EmaIdentitySource::Query,
        ));
    }
    for term in identity
        .terms
        .iter()
        .skip(1)
        .filter(|term| !term.source.is_cvx())
    {
        if normalized_name == Some(term.normalized.as_str())
            || normalized_active == Some(term.normalized.as_str())
        {
            return Some((DrugSearchMatchKind::Alias, term.text.clone(), term.source));
        }
    }
    for term in identity
        .terms
        .iter()
        .skip(1)
        .filter(|term| term.source.is_cvx())
    {
        if cvx_description_matches(&term.text, fields) {
            return Some((DrugSearchMatchKind::Alias, term.text.clone(), term.source));
        }
    }
    let primary_terms = HashSet::from([primary.normalized.clone()]);
    if fields
        .iter()
        .any(|field| field_matches_terms(field, &primary_terms))
    {
        return Some((
            DrugSearchMatchKind::BroadText,
            primary.text.clone(),
            EmaIdentitySource::Query,
        ));
    }
    None
}

const CVX_FORMULATION_WORDS: &[&str] = &[
    "vaccine",
    "vaccines",
    "human",
    "virus",
    "viral",
    "live",
    "inactivated",
    "attenuated",
    "recombinant",
    "conjugate",
    "polysaccharide",
    "adsorbed",
    "injectable",
    "split",
    "valent",
    "quadrivalent",
    "trivalent",
    "dose",
    "pf",
    "preservative",
    "with",
    "without",
    "free",
];

pub(super) fn cvx_signature(description: &str) -> Option<Vec<String>> {
    let retained = description
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .filter(|token| !CVX_FORMULATION_WORDS.contains(&token.to_ascii_lowercase().as_str()))
        .collect::<Vec<_>>();
    let qualifies = retained.iter().any(|token| {
        let lower = token.to_ascii_lowercase();
        (token.bytes().all(|byte| byte.is_ascii_alphabetic()) && lower.len() >= 4)
            || is_cvx_initialism(token)
    });
    qualifies.then(|| {
        retained
            .into_iter()
            .map(|token| token.to_ascii_lowercase())
            .collect()
    })
}

fn is_cvx_initialism(token: &str) -> bool {
    let letter_count = token
        .bytes()
        .take_while(|byte| byte.is_ascii_uppercase())
        .count();
    letter_count >= 3
        && token.as_bytes()[letter_count..]
            .iter()
            .all(u8::is_ascii_digit)
}

fn compact_ascii_alphanumeric(value: &str) -> String {
    value
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect()
}

pub(super) fn cvx_description_matches(description: &str, fields: &[&str; 3]) -> bool {
    let Some(signature) = cvx_signature(description) else {
        return false;
    };
    let normalized_description = normalize_term(description);
    fields.iter().any(|field| {
        if normalize_term(field) == normalized_description {
            return true;
        }
        let compact_field = compact_ascii_alphanumeric(field);
        let mut cursor = 0;
        signature.iter().all(|token| {
            let compact_token = compact_ascii_alphanumeric(token);
            let Some(relative) = compact_field[cursor..].find(&compact_token) else {
                return false;
            };
            cursor += relative + compact_token.len();
            true
        })
    })
}
