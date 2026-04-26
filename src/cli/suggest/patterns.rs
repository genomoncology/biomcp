//! Lazy regex factories and static text catalogs for `biomcp suggest`.

use std::sync::OnceLock;

use regex::Regex;

pub(super) fn rsid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\brs\d+\b").expect("valid rsID regex"))
}

pub(super) fn gene_variant_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b[A-Z][A-Z0-9-]{1,14}\s+(?:p\.)?[A-Z][A-Za-z]{0,2}\d+[A-Z][A-Za-z]{0,2}\b")
            .expect("valid gene variant regex")
    })
}

pub(super) fn hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:c|g|m|n|p|r)\.[A-Za-z0-9_>.+:-]+\b").expect("valid HGVS regex")
    })
}

pub(super) fn pmid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bpmid[:\s]*([0-9]{5,})\b").expect("valid PMID regex"))
}

pub(super) fn pmcid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:pmcid[:\s]*)?(PMC[0-9]+)\b").expect("valid PMCID regex")
    })
}

pub(super) fn doi_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b10\.\d{4,9}/[^\s]+\b").expect("valid DOI regex"))
}

pub(super) fn bare_article_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[0-9]{7,}\b").expect("valid bare article ID regex"))
}

pub(super) fn gene_symbol_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[A-Z][A-Z0-9-]{1,14}\b").expect("valid gene symbol regex"))
}

pub(super) fn explicit_gene_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bgene\s+([A-Za-z][A-Za-z0-9-]{1,14})\b")
            .expect("valid explicit gene regex")
    })
}

pub(super) fn regulatory_subject_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:was|is|were|are)\s+([A-Za-z0-9][A-Za-z0-9 -]{1,80}?)\s+(?:approved|licensed|authorized|withdrawn|regulated|labeled|labelled)\b",
        )
        .expect("valid regulatory subject regex")
    })
}

pub(super) fn approval_for_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:approval|approvals|label|regulatory|fda|ema|who)\s+(?:for|of)\s+(.+?)(?:\?|$)")
            .expect("valid approval-for regex")
    })
}

pub(super) fn pgx_drug_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:affect|affects|for|of|with)\s+([A-Za-z0-9][A-Za-z0-9 -]{1,80}?)\s+(?:dosing|dose|metabolism|response|recommendations?)\b")
            .expect("valid PGx drug regex")
    })
}

pub(super) fn generic_for_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:for|in|of|with|on)\s+(.+?)(?:\?|$)").expect("valid generic-for regex")
    })
}

pub(super) fn trial_condition_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:recruiting|enrolling|open)?\s*trials?\s+(?:for|in|with)\s+(.+?)(?:\?|$)",
        )
        .expect("valid trial condition regex")
    })
}

pub(super) fn trial_intervention_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\btrials?\s+(?:for|with)\s+(?:drug|intervention|therapy|treatment)\s+(.+?)(?:\?|$)|\bintervention\s+(.+?)(?:\?|$)",
        )
        .expect("valid trial intervention regex")
    })
}

pub(super) fn trial_with_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\btrials?\s+with\s+(.+?)(?:\?|$)").expect("valid trial-with regex")
    })
}

pub(super) fn syndrome_compare_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:distinguish|differentiate|disambiguate|compare)\s+(.+?)\s+(?:vs\.?|versus|from)\s+(.+?)(?:\?|$)")
            .expect("valid syndrome comparison regex")
    })
}

pub(super) fn syndrome_difference_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bdifference\s+between\s+(.+?)\s+and\s+(.+?)(?:\?|$)")
            .expect("valid syndrome difference regex")
    })
}

pub(super) fn syndrome_vs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(.+?)\s+(?:vs\.?|versus)\s+(.+?)(?:\?|$)")
            .expect("valid syndrome-vs regex")
    })
}

pub(super) fn syndrome_confused_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(.+?)\s+confused\s+with\s+(.+?)(?:\?|$)")
            .expect("valid syndrome confused-with regex")
    })
}

pub(super) fn linked_terms_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:is|are|was|were|does|do)?\s*(.+?)\s+(?:linked|associated|caus(?:e|es|ed)|related)\s+(?:to|with)\s+(.+?)(?:\?|$)")
            .expect("valid linked terms regex")
    })
}

pub(super) fn cause_terms_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:does|do)\s+(.+?)\s+caus(?:e|es)\s+(.+?)(?:\?|$)")
            .expect("valid cause terms regex")
    })
}

pub(super) fn evidence_terms_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:any\s+|no\s+)?evidence\s+(?:for|against)\s+(.+?)\s+(?:and|in|with)\s+(.+?)(?:\?|$)")
            .expect("valid evidence terms regex")
    })
}

pub(super) fn mapped_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:is|for|of)\s+(.+?)\s+(?:mapped|located|on chromosome|at locus)")
            .expect("valid mapped disease regex")
    })
}

pub(super) fn mechanism_topic_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:pathway|mechanism|signaling)\s+(?:explains?|for|of|in)\s+(.+?)(?:\?|$)",
        )
        .expect("valid mechanism topic regex")
    })
}

pub(super) fn mechanism_resistance_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\bexplains?\s+(.+?)\s+resistance\b|(?:.+?\s+)?resistance\s+(?:to|against)\s+(.+?)(?:\?|$)|\b([A-Za-z0-9][A-Za-z0-9 -]{1,80}?)\s+resistance\b",
        )
        .expect("valid mechanism resistance regex")
    })
}

pub(super) fn mechanism_work_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bhow\s+(?:does|do)\s+(.+?)\s+work\b").expect("valid mechanism work regex")
    })
}

pub(super) fn mechanism_of_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:mechanism|pathway|signaling|targets?)\s+(?:of|for)\s+(.+?)(?:\?|$)")
            .expect("valid mechanism-of regex")
    })
}

pub(super) fn treatment_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:drugs?|medications?|therapies|treatments?)\s+(?:treat|for|against)\s+(.+?)(?:\?|$)|\btreat(?:s|ment)?\s+(?:for\s+)?(.+?)(?:\?|$)")
            .expect("valid treatment disease regex")
    })
}

pub(super) fn symptom_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:symptoms?|phenotypes?|features?|signs)\b.*?\b(?:in|of|for|with)\s+(.+?)(?:\?|$)")
            .expect("valid symptom disease regex")
    })
}

pub(super) fn symptom_named_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:in|of|for|with)\s+(.+?\b(?:disease|syndrome|cancer))\b")
            .expect("valid symptom named disease regex")
    })
}

pub(super) fn symptom_text_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:symptoms?|phenotypes?|clinical features?|features?|signs)\s+(?:include|like|such as|with|are)?\s*(.+?)(?:\?|$)")
            .expect("valid symptom text regex")
    })
}

pub(super) fn gene_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:in|with|for)\s+(.+?)(?:\?|$)").expect("valid gene disease regex")
    })
}

pub(super) const ANCHOR_PREFIXES: &[&str] = &[
    "what drugs treat ",
    "what drugs are used for ",
    "what medications treat ",
    "what treatments are used for ",
    "what treatment is used for ",
    "what symptoms are seen in ",
    "what symptoms occur in ",
    "what phenotype is seen in ",
    "what phenotypes are seen in ",
    "which variants are in ",
    "which mutations are in ",
    "what pathway explains ",
    "which pathway explains ",
    "what mechanism explains ",
    "which mechanism explains ",
    "how does ",
    "how do ",
    "what is ",
    "what are ",
    "where is ",
    "when was ",
    "when were ",
    "is ",
    "are ",
    "was ",
    "were ",
    "the ",
    "a ",
    "an ",
];

pub(super) const STOP_ANCHORS: &[&str] = &[
    "what",
    "when",
    "where",
    "which",
    "who",
    "how",
    "is",
    "are",
    "was",
    "were",
    "gene",
    "genes",
    "drug",
    "drugs",
    "disease",
    "variant",
    "variants",
    "mutation",
    "mutations",
    "cancer",
    "syndrome",
    "trial",
    "trials",
    "treatment",
    "therapy",
    "pathway",
    "mechanism",
    "approved",
    "approval",
    "x",
];

pub(super) const GENE_STOPWORDS: &[&str] = &[
    "AND", "ARE", "DNA", "DOI", "EMA", "FDA", "HGVS", "MCP", "OR", "PMC", "PMID", "RNA", "WHO",
];
