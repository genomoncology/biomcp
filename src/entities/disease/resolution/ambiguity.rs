//! The clinical-reading pointer table and the abbreviation-ambiguity
//! surface the default trial source consumes.

use super::{
    DirectNameResolution, MyDiseaseClient, holder_display_label, is_abbreviation_shaped_token,
    resolve_disease_hit_by_name_direct, single_holder_is_short_token,
};
use crate::sources::mydisease::MyDiseaseHit;

/// How strongly oncology holds a clinical reading no indexed source
/// record carries: the default reading the holders never offer, or an
/// additional reading beside holders that already name clinical
/// diseases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadingRelation {
    /// The reading is the one oncology means when no holder does. It
    /// leads the refusal's candidate list, because the holders alone
    /// would bury the disease clinicians mean under rare records
    /// (`MM` usually means multiple myeloma, not Miyoshi muscular
    /// dystrophy).
    Default,
    /// The holders already name clinical diseases, so the reading stays
    /// a pointer line and never joins the candidates (`MF` also names
    /// myelofibrosis beside mycosis fungoides and myotonia fluctuans).
    Additional,
}

impl ReadingRelation {
    fn phrase(self) -> &'static str {
        match self {
            Self::Default => "usually means",
            Self::Additional => "also names",
        }
    }
}

/// A clinical reading an abbreviation names that no indexed source
/// record holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClinicalAbbreviationReading {
    pub(crate) token: &'static str,
    pub(crate) relation: ReadingRelation,
    pub(crate) label: &'static str,
    pub(crate) ontology_id: &'static str,
}

/// Abbreviations whose clinical meaning the source cannot see, with the
/// sources checked and measured for each entry. The
/// refusal stands on the source holders alone; this table only adds a
/// reading no holder carries — as the first candidate line for a
/// default reading, or as a pointer line for an additional one.
/// `MM` → multiple myeloma (MONDO:0009693): MyDisease holds `MM` on
///   Miyoshi muscular dystrophy (MONDO:0009685) alone. Checked live
///   2026-10-08 and again 2026-10-11: `mondo.synonym.exact:"MM"` and
///   `disease_ontology.synonyms.exact:"MM"` return no myeloma record,
///   the MONDO:0009693 synonym list carries no `MM`, and the NCI
///   Thesaurus synonym list for Multiple Myeloma (NCIT:C3242, via EBI
///   OLS4) carries no `MM`. Measured on the ClinicalTrials.gov registry
///   2026-10-11: of the first 50 trials matching `query.cond=MM`, 30
///   list a myeloma condition, so myeloma is the reading the registry
///   corpus itself carries.
/// `MF` → myelofibrosis (MONDO:0009692, the record `get disease
///   "myelofibrosis"` resolves to through its Disease Ontology name):
///   MyDisease holds `MF` on mycosis fungoides (MONDO:0009691) and
///   myotonia fluctuans (MONDO:0020481) alone. Checked live
///   2026-10-11: no myelofibrosis record carries `MF` in
///   `mondo.synonym.exact` or `disease_ontology.synonyms.exact`, the
///   Disease Ontology entry (DOID:4971) lists no `MF`, and the NCI
///   Thesaurus synonym lists for Myelofibrosis (NCIT:C3248) and Primary
///   Myelofibrosis (NCIT:C2862, via EBI OLS4) list no `MF`. Measured on
///   the ClinicalTrials.gov registry 2026-10-11: of the first 50 trials
///   matching `query.cond=MF`, 28 list a myelofibrosis condition
///   against 6 for mycosis fungoides, so myelofibrosis is the reading
///   the trial corpus carries most. Curated abbreviation preferences
///   stay deferred.
const CLINICAL_ABBREVIATION_READINGS: &[ClinicalAbbreviationReading] = &[
    ClinicalAbbreviationReading {
        token: "MM",
        relation: ReadingRelation::Default,
        label: "multiple myeloma",
        ontology_id: "MONDO:0009693",
    },
    ClinicalAbbreviationReading {
        token: "MF",
        relation: ReadingRelation::Additional,
        label: "myelofibrosis",
        ontology_id: "MONDO:0009692",
    },
];

pub(super) fn clinical_reading(requested: &str) -> Option<ClinicalAbbreviationReading> {
    let requested = requested.trim();
    CLINICAL_ABBREVIATION_READINGS
        .iter()
        .copied()
        .find(|reading| reading.token.eq_ignore_ascii_case(requested))
}

/// The reading's disease label for the requested token, so a command
/// surface other than `get disease` can name the same reading in its
/// own retry.
pub(crate) fn clinical_reading_label(requested: &str) -> Option<&'static str> {
    clinical_reading(requested).map(|reading| reading.label)
}

/// The pointer phrase both the `get disease` refusal and the default
/// trial source's keyword-search note use, so one reading
/// keeps one wording across surfaces.
pub(crate) fn clinical_reading_phrase(requested: &str) -> Option<String> {
    clinical_reading(requested).map(|reading| {
        format!(
            "Clinical reading: '{}' {} {} ({})",
            requested.trim(),
            reading.relation.phrase(),
            reading.label,
            reading.ontology_id
        )
    })
}

/// The refusal's pointer line, with the `get disease` retry that names
/// the reading.
pub(super) fn clinical_reading_line(requested: &str) -> Option<String> {
    clinical_reading(requested).map(|reading| {
        let requested = requested.trim();
        format!(
            "Clinical reading: '{requested}' {} {} ({}); \
try `biomcp get disease \"{}\"`.",
            reading.relation.phrase(),
            reading.label,
            reading.ontology_id,
            reading.label
        )
    })
}

/// The default reading's disease as the first line under the refusal's
/// `Candidates:`, in the same `- label (ontology ID)` form the holders
/// use. Only the default reading leads: additional readings stay
/// pointer lines, so tokens like `MF` keep the holders' list intact.
pub(super) fn default_reading_candidate_line(requested: &str) -> Option<String> {
    clinical_reading(requested)
        .filter(|reading| reading.relation == ReadingRelation::Default)
        .map(|reading| format!("- {} ({})\n", reading.label, reading.ontology_id))
}

/// The `get disease` refusal's retry sentence.
pub(super) fn get_disease_retry(requested: &str) -> String {
    format!(
        "Retry `biomcp get disease` with one candidate's ontology ID or full name, \
or run `biomcp search disease -q \"{requested}\"` to see every match."
    )
}

/// Why the requested token cannot name one disease: several holders, or
/// a token too short for even one holder to be evidence.
pub(super) fn ambiguity_reason(
    requested: &str,
    holders: &[MyDiseaseHit],
    short_token: bool,
) -> String {
    if short_token {
        format!(
            "the source holds it on {} disease{}, but a token this short cannot name one disease reliably",
            holders.len(),
            if holders.len() == 1 { "" } else { "s" },
        )
    } else if is_abbreviation_shaped_token(requested) {
        format!(
            "{} diseases hold it as an exact name or synonym",
            holders.len()
        )
    } else {
        format!("{} diseases carry it as their exact name", holders.len())
    }
}

/// One refusal candidate as a display line: the label the search
/// response offers, or the ontology label, with the record's ID.
pub(super) fn holder_display_line(hit: &MyDiseaseHit) -> String {
    match holder_display_label(hit) {
        Some(label) => format!("{label} ({})", hit.id),
        None => format!("{} (no label in the search response)", hit.id),
    }
}

/// What the refusal knows about an ambiguous abbreviation, exposed so
/// the default ClinicalTrials.gov trial source can say plainly what its
/// keyword search ran on instead of leaving the user to guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AbbreviationAmbiguity {
    pub(crate) requested: String,
    /// Why the token cannot name one disease (the refusal's reason text).
    pub(crate) reason: String,
    /// The named holders, one `label (ontology id)` per entry.
    pub(crate) holders: Vec<String>,
}

/// The refusal the default trial source would get from `get disease`,
/// as data instead of an error: `Some` when the token is an
/// abbreviation-shaped condition the disease resolver refuses (several
/// holders, or a holder too short to count), `None` for anything else.
/// Only abbreviation-shaped tokens reach the resolver, so ordinary
/// condition words add no grounding request.
pub(crate) async fn resolve_abbreviated_disease_ambiguity(
    client: &MyDiseaseClient,
    requested: &str,
) -> Result<Option<AbbreviationAmbiguity>, crate::error::BioMcpError> {
    let requested = requested.trim();
    if !is_abbreviation_shaped_token(requested) {
        return Ok(None);
    }
    match resolve_disease_hit_by_name_direct(client, requested).await? {
        DirectNameResolution::AmbiguousAbbreviation { requested, holders } => {
            // Several holders explain the refusal on their own; the
            // short-token reason is for the single-holder case like `MM`.
            let short_token = single_holder_is_short_token(holders.len(), &requested);
            Ok(Some(AbbreviationAmbiguity {
                reason: ambiguity_reason(&requested, &holders, short_token),
                holders: holders.iter().map(holder_display_line).collect(),
                requested,
            }))
        }
        DirectNameResolution::Resolved(_) | DirectNameResolution::Unresolved => Ok(None),
    }
}
