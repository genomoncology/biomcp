//! Authoritative inventory of optional source-backed entity sections.
//!
//! The quality ratchet also reads these static rows directly, so selector policy
//! fields are intentionally retained even when runtime code only queries keys.

// dead-code reason: static source-state registry is also consumed by the quality contract
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Aggregation {
    Additive,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorClass {
    Canonical,
    Alias,
    Aggregate,
    Local,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceStateRow {
    pub entity: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub providers: &'static [&'static str],
    pub aggregation: Aggregation,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SelectorRow {
    pub entity: &'static str,
    pub selector: &'static str,
    pub class: SelectorClass,
    pub canonical: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryRoute {
    Section {
        cli_entity: &'static str,
        section: &'static str,
    },
    BaseCard {
        cli_entity: &'static str,
    },
    VariantStructure,
}

const fn state(
    entity: &'static str,
    key: &'static str,
    label: &'static str,
    providers: &'static [&'static str],
    aggregation: Aggregation,
) -> SourceStateRow {
    SourceStateRow {
        entity,
        key,
        label,
        providers,
        aggregation,
    }
}

const fn selector(
    entity: &'static str,
    selector: &'static str,
    class: SelectorClass,
    canonical: Option<&'static str>,
) -> SelectorRow {
    SelectorRow {
        entity,
        selector,
        class,
        canonical,
    }
}

pub(crate) const SOURCE_STATE_ROWS: &[SourceStateRow] = &[
    state(
        "gene",
        "pathways",
        "Pathways",
        &["Reactome", "KEGG"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "ontology",
        "Ontology",
        &["Enrichr"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "diseases",
        "Diseases",
        &["Enrichr"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "diagnostics",
        "Diagnostics",
        &["NCBI Genetic Testing Registry", "WHO Prequalified IVD"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "protein",
        "Protein",
        &["UniProt"],
        Aggregation::Fallback,
    ),
    state(
        "gene",
        "go",
        "GO Terms",
        &["QuickGO"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "interactions",
        "Interactions",
        &["STRING"],
        Aggregation::Additive,
    ),
    state("gene", "civic", "CIViC", &["CIViC"], Aggregation::Additive),
    state(
        "gene",
        "expression",
        "Expression",
        &["GTEx"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "hpa",
        "Human Protein Atlas",
        &["Human Protein Atlas"],
        Aggregation::Fallback,
    ),
    state(
        "gene",
        "druggability",
        "Druggability",
        &["DGIdb", "Open Targets"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "clingen",
        "ClinGen",
        &["ClinGen"],
        Aggregation::Fallback,
    ),
    state(
        "gene",
        "constraint",
        "Constraint",
        &["gnomAD"],
        Aggregation::Fallback,
    ),
    state(
        "gene",
        "disgenet",
        "DisGeNET",
        &["DisGeNET"],
        Aggregation::Additive,
    ),
    state(
        "gene",
        "funding",
        "Funding",
        &["NIH Reporter"],
        Aggregation::Additive,
    ),
    state(
        "article",
        "fulltext",
        "Full Text",
        &[
            "Europe PMC",
            "NCBI EFetch",
            "PMC OA",
            "PMC",
            "Semantic Scholar",
        ],
        Aggregation::Fallback,
    ),
    state(
        "article",
        "indexing",
        "Indexing",
        &["PubMed"],
        Aggregation::Fallback,
    ),
    state(
        "article",
        "tldr",
        "Semantic Scholar",
        &["Semantic Scholar"],
        Aggregation::Fallback,
    ),
    state(
        "pathway",
        "genes",
        "Genes",
        &["Reactome", "KEGG", "WikiPathways", "MyGene.info"],
        Aggregation::Fallback,
    ),
    state(
        "pathway",
        "events",
        "Events",
        &["Reactome"],
        Aggregation::Additive,
    ),
    state(
        "pathway",
        "enrichment",
        "Enrichment",
        &["g:Profiler"],
        Aggregation::Additive,
    ),
    state(
        "protein",
        "domains",
        "Domains",
        &["InterPro"],
        Aggregation::Additive,
    ),
    state(
        "protein",
        "interactions",
        "Interactions",
        &["STRING"],
        Aggregation::Additive,
    ),
    state(
        "protein",
        "complexes",
        "Complexes",
        &["Complex Portal"],
        Aggregation::Additive,
    ),
    state(
        "protein",
        "structures",
        "Structures",
        &["PDBe"],
        Aggregation::Additive,
    ),
    state(
        "drug",
        "approvals",
        "Drugs@FDA Approvals",
        &["OpenFDA Drugs@FDA"],
        Aggregation::Additive,
    ),
    state(
        "drug",
        "safety",
        "Safety",
        &["OpenFDA FAERS", "OpenFDA label", "EMA"],
        Aggregation::Additive,
    ),
    state(
        "drug",
        "targets",
        "Targets",
        &["Guide to PHARMACOLOGY", "ChEMBL", "Open Targets"],
        Aggregation::Additive,
    ),
    state(
        "drug",
        "indications",
        "Indications",
        &["DrugCentral", "Open Targets"],
        Aggregation::Additive,
    ),
    state(
        "drug",
        "interactions",
        "Interactions",
        &["DDInter", "DrugBank", "OpenFDA label"],
        Aggregation::Additive,
    ),
    state("drug", "civic", "CIViC", &["CIViC"], Aggregation::Fallback),
    state(
        "adverse_event",
        "faers",
        "FAERS",
        &["OpenFDA FAERS"],
        Aggregation::Additive,
    ),
    state(
        "adverse_event",
        "vaers",
        "VAERS",
        &["CDC CVX", "CDC VAERS"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "treatments",
        "Treatments",
        &["MyChem.info indication search"],
        Aggregation::Fallback,
    ),
    state(
        "disease",
        "recruiting_trials",
        "Recruiting Trials",
        &["ClinicalTrials.gov"],
        Aggregation::Fallback,
    ),
    state(
        "disease",
        "genes",
        "Genes",
        &["Monarch Initiative", "CIViC", "Open Targets"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "pathways",
        "Pathways",
        &["Reactome"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "phenotypes",
        "Phenotypes",
        &["MyDisease.info", "Monarch Initiative", "HPO"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "diagnostics",
        "Diagnostics",
        &["NCBI Genetic Testing Registry", "WHO Prequalified IVD"],
        Aggregation::Fallback,
    ),
    state(
        "disease",
        "variants",
        "Variants",
        &["CIViC"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "models",
        "Models",
        &["Monarch Initiative"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "prevalence",
        "Prevalence",
        &["Open Targets"],
        Aggregation::Fallback,
    ),
    state(
        "disease",
        "survival",
        "Survival",
        &["SEER Explorer"],
        Aggregation::Fallback,
    ),
    state(
        "disease",
        "funding",
        "Funding",
        &["NIH Reporter"],
        Aggregation::Additive,
    ),
    state(
        "disease",
        "civic",
        "CIViC",
        &["CIViC"],
        Aggregation::Fallback,
    ),
    state(
        "variant_structure",
        "domains",
        "Domains",
        &["InterPro"],
        Aggregation::Fallback,
    ),
    state(
        "variant_structure",
        "cancerhotspots",
        "Cancer Hotspots",
        &["cancerhotspots.org"],
        Aggregation::Fallback,
    ),
    state(
        "variant",
        "predict",
        "Prediction",
        &["AlphaGenome"],
        Aggregation::Fallback,
    ),
    state(
        "variant",
        "population",
        "Population",
        &["dbSNP", "gnomAD v4"],
        Aggregation::Fallback,
    ),
    state(
        "variant",
        "cancerhotspots",
        "Cancer Hotspots",
        &["cancerhotspots.org"],
        Aggregation::Fallback,
    ),
    state(
        "variant",
        "civic",
        "CIViC",
        &["CIViC"],
        Aggregation::Fallback,
    ),
    state(
        "variant",
        "cbioportal",
        "cBioPortal",
        &["cBioPortal"],
        Aggregation::Fallback,
    ),
    state(
        "variant",
        "gwas",
        "GWAS",
        &["GWAS Catalog"],
        Aggregation::Fallback,
    ),
    state(
        "pgx",
        "frequencies",
        "Frequencies",
        &["CPIC"],
        Aggregation::Additive,
    ),
    state(
        "pgx",
        "annotations",
        "Annotations",
        &["PharmGKB"],
        Aggregation::Fallback,
    ),
    state(
        "diagnostic",
        "regulatory",
        "Regulatory",
        &["OpenFDA Device 510(k) / PMA"],
        Aggregation::Fallback,
    ),
];

pub(crate) const SELECTOR_ROWS: &[SelectorRow] = &[
    selector(
        "gene",
        "pathways",
        SelectorClass::Canonical,
        Some("pathways"),
    ),
    selector(
        "gene",
        "ontology",
        SelectorClass::Canonical,
        Some("ontology"),
    ),
    selector(
        "gene",
        "diseases",
        SelectorClass::Canonical,
        Some("diseases"),
    ),
    selector(
        "gene",
        "diagnostics",
        SelectorClass::Canonical,
        Some("diagnostics"),
    ),
    selector("gene", "protein", SelectorClass::Canonical, Some("protein")),
    selector("gene", "go", SelectorClass::Canonical, Some("go")),
    selector(
        "gene",
        "interactions",
        SelectorClass::Canonical,
        Some("interactions"),
    ),
    selector("gene", "civic", SelectorClass::Canonical, Some("civic")),
    selector(
        "gene",
        "expression",
        SelectorClass::Canonical,
        Some("expression"),
    ),
    selector("gene", "hpa", SelectorClass::Canonical, Some("hpa")),
    selector(
        "gene",
        "druggability",
        SelectorClass::Canonical,
        Some("druggability"),
    ),
    selector("gene", "clingen", SelectorClass::Canonical, Some("clingen")),
    selector(
        "gene",
        "constraint",
        SelectorClass::Canonical,
        Some("constraint"),
    ),
    selector(
        "gene",
        "disgenet",
        SelectorClass::Canonical,
        Some("disgenet"),
    ),
    selector("gene", "funding", SelectorClass::Canonical, Some("funding")),
    selector("gene", "all", SelectorClass::Aggregate, None),
    selector(
        "article",
        "fulltext",
        SelectorClass::Canonical,
        Some("fulltext"),
    ),
    selector("article", "annotations", SelectorClass::Local, None),
    selector("article", "tldr", SelectorClass::Canonical, Some("tldr")),
    selector(
        "article",
        "indexing",
        SelectorClass::Canonical,
        Some("indexing"),
    ),
    selector("article", "assets", SelectorClass::Local, None),
    selector("article", "asset", SelectorClass::Alias, Some("assets")),
    selector("article", "all", SelectorClass::Aggregate, None),
    selector("pathway", "genes", SelectorClass::Canonical, Some("genes")),
    selector(
        "pathway",
        "events",
        SelectorClass::Canonical,
        Some("events"),
    ),
    selector(
        "pathway",
        "enrichment",
        SelectorClass::Canonical,
        Some("enrichment"),
    ),
    selector("pathway", "all", SelectorClass::Aggregate, None),
    selector(
        "protein",
        "domains",
        SelectorClass::Canonical,
        Some("domains"),
    ),
    selector(
        "protein",
        "interactions",
        SelectorClass::Canonical,
        Some("interactions"),
    ),
    selector(
        "protein",
        "complexes",
        SelectorClass::Canonical,
        Some("complexes"),
    ),
    selector(
        "protein",
        "structures",
        SelectorClass::Canonical,
        Some("structures"),
    ),
    selector("protein", "all", SelectorClass::Aggregate, None),
    selector("drug", "label", SelectorClass::Local, None),
    selector("drug", "regulatory", SelectorClass::Aggregate, None),
    selector("drug", "safety", SelectorClass::Canonical, Some("safety")),
    selector("drug", "shortage", SelectorClass::Local, None),
    selector("drug", "targets", SelectorClass::Canonical, Some("targets")),
    selector(
        "drug",
        "indications",
        SelectorClass::Canonical,
        Some("indications"),
    ),
    selector(
        "drug",
        "interactions",
        SelectorClass::Canonical,
        Some("interactions"),
    ),
    selector("drug", "civic", SelectorClass::Canonical, Some("civic")),
    selector(
        "drug",
        "approvals",
        SelectorClass::Canonical,
        Some("approvals"),
    ),
    selector("drug", "all", SelectorClass::Aggregate, None),
    selector("adverse_event", "reactions", SelectorClass::Local, None),
    selector("adverse_event", "outcomes", SelectorClass::Local, None),
    selector("adverse_event", "concomitant", SelectorClass::Local, None),
    selector("adverse_event", "guidance", SelectorClass::Local, None),
    selector("adverse_event", "all", SelectorClass::Aggregate, None),
    selector("disease", "genes", SelectorClass::Canonical, Some("genes")),
    selector(
        "disease",
        "pathways",
        SelectorClass::Canonical,
        Some("pathways"),
    ),
    selector(
        "disease",
        "phenotypes",
        SelectorClass::Canonical,
        Some("phenotypes"),
    ),
    selector(
        "disease",
        "diagnostics",
        SelectorClass::Canonical,
        Some("diagnostics"),
    ),
    selector(
        "disease",
        "variants",
        SelectorClass::Canonical,
        Some("variants"),
    ),
    selector(
        "disease",
        "models",
        SelectorClass::Canonical,
        Some("models"),
    ),
    selector(
        "disease",
        "prevalence",
        SelectorClass::Canonical,
        Some("prevalence"),
    ),
    selector(
        "disease",
        "survival",
        SelectorClass::Canonical,
        Some("survival"),
    ),
    selector(
        "disease",
        "funding",
        SelectorClass::Canonical,
        Some("funding"),
    ),
    selector("disease", "civic", SelectorClass::Canonical, Some("civic")),
    selector("disease", "disgenet", SelectorClass::Local, None),
    selector(
        "disease",
        "clinical_features",
        SelectorClass::Alias,
        Some("phenotypes"),
    ),
    selector("disease", "all", SelectorClass::Aggregate, None),
    selector(
        "variant",
        "predict",
        SelectorClass::Canonical,
        Some("predict"),
    ),
    selector("variant", "predictions", SelectorClass::Local, None),
    selector("variant", "clinvar", SelectorClass::Local, None),
    selector(
        "variant",
        "population",
        SelectorClass::Canonical,
        Some("population"),
    ),
    selector(
        "variant",
        "population-details",
        SelectorClass::Alias,
        Some("population"),
    ),
    selector("variant", "conservation", SelectorClass::Local, None),
    selector("variant", "cosmic", SelectorClass::Local, None),
    selector("variant", "cgi", SelectorClass::Local, None),
    selector("variant", "civic", SelectorClass::Canonical, Some("civic")),
    selector(
        "variant",
        "cbioportal",
        SelectorClass::Canonical,
        Some("cbioportal"),
    ),
    selector("variant", "gwas", SelectorClass::Canonical, Some("gwas")),
    selector("variant", "all", SelectorClass::Aggregate, None),
    selector("pgx", "interactions", SelectorClass::Local, None),
    selector("pgx", "recommendations", SelectorClass::Local, None),
    selector(
        "pgx",
        "frequencies",
        SelectorClass::Canonical,
        Some("frequencies"),
    ),
    selector("pgx", "guidelines", SelectorClass::Local, None),
    selector(
        "pgx",
        "annotations",
        SelectorClass::Canonical,
        Some("annotations"),
    ),
    selector("pgx", "all", SelectorClass::Aggregate, None),
    selector("diagnostic", "genes", SelectorClass::Local, None),
    selector("diagnostic", "conditions", SelectorClass::Local, None),
    selector("diagnostic", "methods", SelectorClass::Local, None),
    selector(
        "diagnostic",
        "regulatory",
        SelectorClass::Canonical,
        Some("regulatory"),
    ),
    selector("diagnostic", "all", SelectorClass::Aggregate, None),
    selector("trial", "eligibility", SelectorClass::Local, None),
    selector("trial", "contacts", SelectorClass::Local, None),
    selector("trial", "locations", SelectorClass::Local, None),
    selector("trial", "outcomes", SelectorClass::Local, None),
    selector("trial", "arms", SelectorClass::Local, None),
    selector("trial", "references", SelectorClass::Local, None),
    selector("trial", "all", SelectorClass::Aggregate, None),
];

pub(crate) fn allows_sources(entity: &str, key: &str, sources: &[String]) -> bool {
    SOURCE_STATE_ROWS
        .iter()
        .find(|row| row.entity == entity && row.key == key)
        .is_some_and(|row| {
            sources
                .iter()
                .all(|source| row.providers.contains(&source.as_str()))
        })
}

pub(crate) fn outcome_keys(entity: &str) -> Vec<&'static str> {
    SOURCE_STATE_ROWS
        .iter()
        .filter(|row| row.entity == entity)
        .map(|row| row.key)
        .collect()
}

pub(crate) fn labels(entity: &str) -> Vec<(&'static str, &'static str)> {
    SOURCE_STATE_ROWS
        .iter()
        .filter(|row| row.entity == entity)
        .map(|row| (row.key, row.label))
        .collect()
}

/// Resolve the one public read route that owns a recoverable source-state row.
///
/// Canonical selectors use their registered target. Rows that intentionally
/// have no selector are enumerated here so rendering cannot invent CLI syntax.
pub(crate) fn recovery_route(entity: &str, key: &str) -> Option<RecoveryRoute> {
    if entity == "variant_structure" && matches!(key, "domains" | "cancerhotspots") {
        return Some(RecoveryRoute::VariantStructure);
    }
    let cli_entity = match entity {
        "adverse_event" => "adverse-event",
        "article" => "article",
        "diagnostic" => "diagnostic",
        "disease" => "disease",
        "drug" => "drug",
        "gene" => "gene",
        "pathway" => "pathway",
        "pgx" => "pgx",
        "protein" => "protein",
        "variant" => "variant",
        _ => return None,
    };
    let canonical = SELECTOR_ROWS.iter().find_map(|row| {
        (row.entity == entity && row.selector == key && row.class == SelectorClass::Canonical)
            .then_some(row.canonical)
            .flatten()
            .map(|section| RecoveryRoute::Section {
                cli_entity,
                section,
            })
    });
    canonical.or(match (entity, key) {
        ("disease", "treatments" | "recruiting_trials")
        | ("variant", "cancerhotspots")
        | ("adverse_event", "faers" | "vaers") => Some(RecoveryRoute::BaseCard { cli_entity }),
        _ => None,
    })
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn every_source_state_row_has_one_callable_recovery_route() {
        for row in SOURCE_STATE_ROWS {
            let canonical_routes = SELECTOR_ROWS
                .iter()
                .filter(|selector| {
                    selector.entity == row.entity
                        && selector.selector == row.key
                        && selector.class == SelectorClass::Canonical
                        && selector.canonical.is_some()
                })
                .count();
            let exceptional_routes = usize::from(matches!(
                (row.entity, row.key),
                ("disease", "treatments" | "recruiting_trials")
                    | ("variant", "cancerhotspots")
                    | ("adverse_event", "faers" | "vaers")
                    | ("variant_structure", "domains" | "cancerhotspots")
            ));
            assert_eq!(canonical_routes + exceptional_routes, 1);
            assert!(
                recovery_route(row.entity, row.key).is_some(),
                "missing recovery route for {}/{}",
                row.entity,
                row.key
            );
        }
        assert_eq!(recovery_route("synthetic", "unmapped"), None);
        assert_eq!(
            recovery_route("disease", "treatments"),
            Some(RecoveryRoute::BaseCard {
                cli_entity: "disease"
            })
        );
        assert_eq!(
            recovery_route("variant_structure", "domains"),
            Some(RecoveryRoute::VariantStructure)
        );
    }
}
