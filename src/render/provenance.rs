use std::collections::HashSet;

use serde::Serialize;

use crate::entities::adverse_event::{
    AdverseEvent, AdverseEventReport, AdverseEventSourceSearch, DeviceEvent,
};
use crate::entities::article::Article;
use crate::entities::diagnostic::{Diagnostic, diagnostic_source_label};
use crate::entities::discover::DiscoverResult;
use crate::entities::disease::Disease;
use crate::entities::drug::{Drug, DrugInteractionReport};
use crate::entities::gene::Gene;
use crate::entities::pathway::Pathway;
use crate::entities::pgx::Pgx;
use crate::entities::protein::Protein;
use crate::entities::section_outcome::{SectionOutcomeState, SectionOutcomes};
use crate::entities::trial::Trial;
use crate::entities::variant::Variant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionSource {
    pub key: String,
    pub label: String,
    pub outcome: SectionOutcomeState,
    pub sources: Vec<String>,
}

impl SectionSource {
    pub(crate) fn normalized(self) -> Option<Self> {
        let key = self.key.trim();
        let label = self.label.trim();
        let sources = normalize_sources(self.sources);
        if key.is_empty()
            || label.is_empty()
            || self.outcome == SectionOutcomeState::NotRequested
            || (sources.is_empty()
                && !matches!(
                    self.outcome,
                    SectionOutcomeState::Inapplicable
                        | SectionOutcomeState::Degraded
                        | SectionOutcomeState::Unavailable
                ))
        {
            return None;
        }
        Some(Self {
            key: key.to_string(),
            label: label.to_string(),
            outcome: self.outcome,
            sources,
        })
    }
}

fn has_text(value: &str) -> bool {
    !value.trim().is_empty()
}

fn has_opt_text(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(has_text)
}

fn normalize_sources<I, S>(sources: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for source in sources {
        let source = source.as_ref().trim();
        if source.is_empty() {
            continue;
        }
        if seen.insert(source.to_string()) {
            out.push(source.to_string());
        }
    }
    out
}

fn push_section<I, S>(
    out: &mut Vec<SectionSource>,
    present: bool,
    key: &str,
    label: &str,
    sources: I,
) where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    if !present {
        return;
    }
    if let Some(section) = (SectionSource {
        key: key.to_string(),
        label: label.to_string(),
        outcome: SectionOutcomeState::Data,
        sources: sources
            .into_iter()
            .map(|source| source.as_ref().to_string())
            .collect(),
    })
    .normalized()
    {
        out.push(section);
    }
}

fn outcome_section_sources(
    entity: &str,
    outcomes: &SectionOutcomes,
    labels: &[(&str, &str)],
) -> Vec<SectionSource> {
    labels
        .iter()
        .filter_map(|(key, label)| {
            let outcome = outcomes.get(key)?;
            assert!(
                crate::entities::source_state_registry::allows_sources(
                    entity,
                    key,
                    outcome.sources()
                ),
                "section outcome credits a provider outside the source-state registry: {entity}/{key}"
            );
            SectionSource {
                key: (*key).to_string(),
                label: (*label).to_string(),
                outcome: outcome.outcome(),
                sources: outcome.sources().to_vec(),
            }
            .normalized()
        })
        .collect()
}

pub(crate) fn discover_section_sources(result: &DiscoverResult) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let structured_sources = result
        .concepts
        .iter()
        .flat_map(|concept| concept.sources.iter().map(|source| source.source.as_str()))
        .collect::<Vec<_>>();
    push_section(
        &mut out,
        !structured_sources.is_empty(),
        "structured_concepts",
        "Structured Concepts",
        structured_sources,
    );
    push_section(
        &mut out,
        result.plain_language.is_some(),
        "plain_language",
        "Plain Language",
        ["MedlinePlus"],
    );
    out
}

pub(crate) fn diagnostic_section_sources(diagnostic: &Diagnostic) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let summary_present = has_text(&diagnostic.source)
        || has_text(&diagnostic.source_id)
        || has_text(&diagnostic.accession)
        || has_text(&diagnostic.name)
        || has_opt_text(&diagnostic.test_type)
        || has_opt_text(&diagnostic.manufacturer)
        || has_opt_text(&diagnostic.target_marker)
        || has_opt_text(&diagnostic.regulatory_version)
        || has_opt_text(&diagnostic.prequalification_year)
        || has_opt_text(&diagnostic.laboratory)
        || has_opt_text(&diagnostic.institution)
        || has_opt_text(&diagnostic.country)
        || has_opt_text(&diagnostic.clia_number)
        || has_opt_text(&diagnostic.state_licenses)
        || has_opt_text(&diagnostic.current_status)
        || has_opt_text(&diagnostic.public_status)
        || !diagnostic.method_categories.is_empty();
    let source_label = diagnostic_source_label(&diagnostic.source);
    push_section(
        &mut out,
        summary_present,
        "summary",
        "Summary",
        [source_label],
    );
    push_section(
        &mut out,
        diagnostic.genes.is_some(),
        "genes",
        "Genes",
        [source_label],
    );
    push_section(
        &mut out,
        diagnostic.conditions.is_some(),
        "conditions",
        "Conditions",
        [source_label],
    );
    push_section(
        &mut out,
        diagnostic.methods.is_some(),
        "methods",
        "Methods",
        [source_label],
    );
    out.extend(outcome_section_sources(
        "diagnostic",
        &diagnostic.section_outcomes,
        &[("regulatory", "Regulatory")],
    ));
    out
}

pub(crate) fn trial_source_label(source: Option<&str>) -> String {
    match source
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "ctgov" | "clinicaltrials" | "clinicaltrials.gov" => "ClinicalTrials.gov".to_string(),
        "nci" | "nci cts" | "nci_cts" | "cts" => "NCI CTS".to_string(),
        _ => source
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("ClinicalTrials.gov")
            .to_string(),
    }
}

pub(crate) fn pathway_source_label(source: &str) -> String {
    let source = source.trim();
    if source.eq_ignore_ascii_case("kegg") {
        "KEGG".to_string()
    } else if source.eq_ignore_ascii_case("reactome") {
        "Reactome".to_string()
    } else if source.eq_ignore_ascii_case("wikipathways") {
        "WikiPathways".to_string()
    } else if source.is_empty() {
        "Reactome".to_string()
    } else {
        source.to_string()
    }
}

pub(crate) fn drug_interaction_sources(drug: &Drug) -> Vec<String> {
    let mut sources = vec!["DDInter".to_string()];
    if drug
        .interactions
        .iter()
        .any(|row| row.description.as_deref().is_some_and(has_text))
    {
        sources.push("DrugBank".to_string());
    }
    if has_opt_text(&drug.interaction_text) {
        sources.push("OpenFDA label".to_string());
    }
    normalize_sources(sources)
}

pub(crate) fn drug_interaction_heading_label(drug: &Drug) -> String {
    if drug.interactions.is_empty() && !has_opt_text(&drug.interaction_text) {
        "Interactions".to_string()
    } else {
        "Interactions (DDInter)".to_string()
    }
}

pub(crate) fn drug_interaction_note(drug: &Drug) -> Option<String> {
    if !drug.interactions.is_empty() {
        Some(
            "Structured rows come from the current DDInter download bundle. DDInter warns that missing rows do not prove no interaction exists."
                .to_string(),
        )
    } else {
        Some(
            "The current DDInter download bundle has no matching rows for this drug. DDInter warns that missing rows do not prove no interaction exists."
                .to_string(),
        )
    }
}

pub(crate) fn drug_interaction_report_section_sources(
    report: &DrugInteractionReport,
) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let mut sources = vec!["DDInter"];
    if report
        .interactions
        .iter()
        .any(|row| row.description.as_deref().is_some_and(has_text))
    {
        sources.push("DrugBank");
    }
    if report
        .label_interaction_text
        .as_deref()
        .is_some_and(has_text)
    {
        sources.push("OpenFDA label");
    }
    push_section(&mut out, true, "interactions", "Interactions", sources);
    out
}

pub(crate) fn gene_section_sources(gene: &Gene) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let identity_present = has_text(&gene.symbol)
        || has_text(&gene.name)
        || has_text(&gene.entrez_id)
        || has_opt_text(&gene.location)
        || has_opt_text(&gene.genomic_coordinates)
        || has_opt_text(&gene.uniprot_id)
        || has_opt_text(&gene.ensembl_id)
        || has_opt_text(&gene.omim_id)
        || has_opt_text(&gene.gene_type);
    push_section(
        &mut out,
        identity_present,
        "identity",
        "Identity",
        ["NCBI Gene / MyGene.info"],
    );
    push_section(
        &mut out,
        has_opt_text(&gene.summary),
        "summary",
        "Summary",
        ["NCBI Gene"],
    );
    push_section(
        &mut out,
        !gene.aliases.is_empty(),
        "aliases",
        "Aliases",
        ["NCBI Gene / MyGene.info"],
    );
    out.extend(outcome_section_sources(
        "gene",
        &gene.section_outcomes,
        &[
            ("pathways", "Pathways"),
            ("ontology", "Ontology"),
            ("diseases", "Diseases"),
            ("diagnostics", "Diagnostics"),
            ("protein", "Protein"),
            ("go", "GO Terms"),
            ("interactions", "Interactions"),
            ("civic", "CIViC"),
            ("expression", "Expression"),
            ("hpa", "Human Protein Atlas"),
            ("druggability", "Druggability"),
            ("clingen", "ClinGen"),
            ("constraint", "Constraint"),
            ("disgenet", "DisGeNET"),
            ("funding", "Funding"),
        ],
    ));
    out
}

pub(crate) fn drug_section_sources(drug: &Drug) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let overview_present = has_text(&drug.name)
        || has_opt_text(&drug.drugbank_id)
        || has_opt_text(&drug.chembl_id)
        || has_opt_text(&drug.unii)
        || has_opt_text(&drug.drug_type)
        || has_opt_text(&drug.route);
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        ["MyChem.info"],
    );
    push_section(
        &mut out,
        has_opt_text(&drug.approval_date),
        "fda_approved",
        "FDA Approved",
        ["DrugCentral"],
    );
    push_section(
        &mut out,
        !drug.brand_names.is_empty(),
        "brand_names",
        "Brand Names",
        ["DrugBank"],
    );
    let mut regulatory_sources = Vec::new();
    if drug
        .section_outcomes
        .get("approvals")
        .is_some_and(|outcome| !outcome.sources().is_empty())
    {
        regulatory_sources.push("OpenFDA Drugs@FDA".to_string());
    }
    if drug.ema_regulatory.is_some() {
        regulatory_sources.push("EMA".to_string());
    }
    if drug.who_prequalification.is_some() {
        regulatory_sources.push("WHO Prequalification".to_string());
    }
    push_section(
        &mut out,
        !regulatory_sources.is_empty(),
        "regulatory",
        "Regulatory",
        regulatory_sources.iter().map(String::as_str),
    );
    push_section(
        &mut out,
        has_opt_text(&drug.mechanism) || !drug.mechanisms.is_empty(),
        "mechanisms",
        "Mechanisms",
        ["MyChem.info", "ChEMBL"],
    );
    push_section(
        &mut out,
        !drug.variant_targets.is_empty(),
        "variant_targets",
        "Variant Targets",
        ["CIViC"],
    );
    let interaction_sources = drug_interaction_sources(drug);
    push_section(
        &mut out,
        !drug.interactions.is_empty() || has_opt_text(&drug.interaction_text),
        "interactions",
        "Interactions",
        interaction_sources.iter().map(String::as_str),
    );
    push_section(
        &mut out,
        drug.label.is_some(),
        "label",
        "FDA Label",
        ["OpenFDA label"],
    );
    push_section(
        &mut out,
        drug.shortage.is_some(),
        "shortage",
        "Shortage",
        ["OpenFDA Drug Shortages"],
    );
    push_section(
        &mut out,
        drug.ema_shortage.is_some(),
        "ema_shortage",
        "EMA Shortage",
        ["EMA"],
    );
    out.extend(outcome_section_sources(
        "drug",
        &drug.section_outcomes,
        &[
            ("approvals", "Drugs@FDA Approvals"),
            ("safety", "Safety"),
            ("targets", "Targets"),
            ("indications", "Indications"),
            ("civic", "CIViC"),
        ],
    ));
    out
}

pub(crate) fn disease_section_sources(disease: &Disease) -> Vec<SectionSource> {
    let mut out = Vec::new();
    push_section(
        &mut out,
        has_opt_text(&disease.definition),
        "definition",
        "Definition",
        ["MyDisease.info"],
    );
    push_section(
        &mut out,
        !disease.synonyms.is_empty(),
        "synonyms",
        "Synonyms",
        ["MONDO / Disease Ontology via MyDisease.info"],
    );
    push_section(
        &mut out,
        !disease.parents.is_empty(),
        "parents",
        "Parents",
        ["MONDO / Disease Ontology via MyDisease.info"],
    );
    push_section(
        &mut out,
        !disease.top_genes.is_empty() || !disease.top_gene_scores.is_empty(),
        "top_genes",
        "Genes",
        ["Open Targets"],
    );
    push_section(
        &mut out,
        !disease.associated_genes.is_empty() || !disease.gene_associations.is_empty(),
        "associated_genes",
        "Associated Genes",
        ["Monarch Initiative", "Open Targets"],
    );
    out.extend(outcome_section_sources(
        "disease",
        &disease.section_outcomes,
        &[
            ("treatments", "Treatments"),
            ("recruiting_trials", "Recruiting Trials"),
            ("genes", "Genes"),
            ("pathways", "Pathways"),
            ("phenotypes", "Phenotypes"),
            ("diagnostics", "Diagnostics"),
            ("variants", "Variants"),
            ("models", "Models"),
            ("prevalence", "Prevalence"),
            ("survival", "Survival"),
            ("funding", "Funding"),
            ("civic", "CIViC"),
        ],
    ));
    push_section(
        &mut out,
        disease.disgenet.is_some(),
        "disgenet",
        "DisGeNET",
        ["DisGeNET"],
    );
    out
}

pub(crate) fn variant_section_sources(variant: &Variant) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let identity_present = has_text(&variant.gene)
        || has_text(&variant.id)
        || has_opt_text(&variant.hgvs_p)
        || has_opt_text(&variant.hgvs_c)
        || has_opt_text(&variant.rsid)
        || has_opt_text(&variant.cosmic_id)
        || has_opt_text(&variant.significance)
        || has_opt_text(&variant.consequence);
    push_section(
        &mut out,
        identity_present,
        "identity",
        "Identity",
        ["MyVariant.info", "ClinVar"],
    );

    push_section(
        &mut out,
        has_opt_text(&variant.clinvar_id)
            || !variant.conditions.is_empty()
            || !variant.clinvar_conditions.is_empty()
            || variant.clinvar_condition_reports.is_some()
            || variant.clinvar_review_stars.is_some()
            || has_opt_text(&variant.clinvar_review_status),
        "clinvar",
        "ClinVar",
        ["ClinVar"],
    );
    push_section(
        &mut out,
        variant.gnomad_af.is_some() || variant.population_breakdown.is_some(),
        "population",
        "Population",
        ["gnomAD via MyVariant.info"],
    );
    push_section(
        &mut out,
        variant.conservation.is_some(),
        "conservation",
        "Conservation",
        ["MyVariant.info"],
    );
    push_section(
        &mut out,
        !variant.expanded_predictions.is_empty()
            || variant.cadd_score.is_some()
            || has_opt_text(&variant.sift_pred)
            || has_opt_text(&variant.polyphen_pred),
        "expanded_predictions",
        "Expanded Predictions",
        ["MyVariant.info"],
    );
    push_section(
        &mut out,
        has_opt_text(&variant.cosmic_id) || variant.cosmic_context.is_some(),
        "cosmic",
        "COSMIC",
        ["COSMIC"],
    );
    push_section(
        &mut out,
        !variant.cgi_associations.is_empty(),
        "cgi",
        "CGI Drug Associations",
        ["Cancer Genome Interpreter"],
    );
    out.extend(outcome_section_sources(
        "variant",
        &variant.section_outcomes,
        &[
            ("predict", "Prediction"),
            ("cancerhotspots", "Cancer Hotspots"),
            ("civic", "CIViC"),
            ("cbioportal", "cBioPortal"),
            ("gwas", "GWAS"),
        ],
    ));
    out
}

pub(crate) fn article_section_sources(article: &Article) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let bibliography_present = has_opt_text(&article.pmid)
        || has_opt_text(&article.pmcid)
        || has_opt_text(&article.doi)
        || has_text(&article.title)
        || has_opt_text(&article.journal)
        || has_opt_text(&article.date)
        || article.citation_count.is_some()
        || has_opt_text(&article.publication_type)
        || article.open_access.is_some();
    push_section(
        &mut out,
        bibliography_present,
        "bibliography",
        "Bibliography",
        ["PubMed", "Europe PMC"],
    );
    push_section(
        &mut out,
        !article.authors.is_empty(),
        "authors",
        "Authors",
        [article.author_source.display_name()],
    );
    push_section(
        &mut out,
        has_opt_text(&article.abstract_text),
        "abstract",
        "Abstract",
        ["PubMed", "Europe PMC"],
    );
    push_section(
        &mut out,
        article.annotations.is_some(),
        "annotations",
        "PubTator Annotations",
        ["PubTator3"],
    );
    out.extend(outcome_section_sources(
        "article",
        &article.section_outcomes,
        &[
            ("fulltext", "Full Text"),
            ("indexing", "Article Indexing"),
            ("tldr", "Semantic Scholar"),
        ],
    ));
    out
}

pub(crate) fn trial_section_sources(trial: &Trial) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let source = trial_source_label(trial.source.as_deref());
    let source_ref = [source.as_str()];
    let overview_present = has_text(&trial.nct_id)
        || has_text(&trial.title)
        || has_text(&trial.status)
        || has_opt_text(&trial.phase)
        || has_opt_text(&trial.study_type)
        || has_opt_text(&trial.age_range)
        || has_opt_text(&trial.sponsor)
        || trial.enrollment.is_some()
        || has_opt_text(&trial.start_date)
        || has_opt_text(&trial.completion_date);
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        source_ref,
    );
    push_section(
        &mut out,
        !trial.conditions.is_empty(),
        "conditions",
        "Conditions",
        source_ref,
    );
    push_section(
        &mut out,
        !trial.interventions.is_empty(),
        "interventions",
        "Interventions",
        source_ref,
    );
    push_section(
        &mut out,
        has_opt_text(&trial.summary),
        "summary",
        "Summary",
        source_ref,
    );
    push_section(
        &mut out,
        has_opt_text(&trial.eligibility_text),
        "eligibility",
        "Eligibility",
        source_ref,
    );
    push_section(
        &mut out,
        trial.locations.is_some(),
        "locations",
        "Locations",
        source_ref,
    );
    push_section(
        &mut out,
        trial.outcomes.is_some(),
        "outcomes",
        "Outcomes",
        source_ref,
    );
    push_section(&mut out, trial.arms.is_some(), "arms", "Arms", source_ref);
    push_section(
        &mut out,
        trial.references.is_some(),
        "references",
        "References",
        source_ref,
    );
    out
}

pub(crate) fn pathway_section_sources(pathway: &Pathway) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let source = pathway_source_label(&pathway.source);
    let source_ref = [source.as_str()];
    let identity_present =
        has_text(&pathway.id) || has_text(&pathway.name) || has_opt_text(&pathway.species);
    push_section(
        &mut out,
        identity_present,
        "identity",
        "Identity",
        source_ref,
    );
    push_section(
        &mut out,
        has_opt_text(&pathway.summary),
        "summary",
        "Summary",
        source_ref,
    );
    out.extend(outcome_section_sources(
        "pathway",
        &pathway.section_outcomes,
        &[
            ("genes", "Genes"),
            ("events", "Events"),
            ("enrichment", "Enrichment"),
        ],
    ));
    out
}

pub(crate) fn protein_section_sources(protein: &Protein) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let identity_present = has_text(&protein.accession)
        || has_text(&protein.name)
        || has_opt_text(&protein.entry_id)
        || has_opt_text(&protein.gene_symbol)
        || has_opt_text(&protein.organism)
        || protein.length.is_some();
    push_section(
        &mut out,
        identity_present,
        "identity",
        "Identity",
        ["UniProt"],
    );
    push_section(
        &mut out,
        has_opt_text(&protein.function),
        "function",
        "Function",
        ["UniProt"],
    );
    out.extend(outcome_section_sources(
        "protein",
        &protein.section_outcomes,
        &[
            ("structures", "Structures"),
            ("domains", "Domains"),
            ("interactions", "Interactions"),
            ("complexes", "Complexes"),
        ],
    ));
    out
}

pub(crate) fn pgx_section_sources(pgx: &Pgx) -> Vec<SectionSource> {
    let mut out = Vec::new();
    push_section(
        &mut out,
        !pgx.interactions.is_empty(),
        "interactions",
        "Interactions",
        ["CPIC"],
    );
    push_section(
        &mut out,
        !pgx.recommendations.is_empty(),
        "recommendations",
        "Recommendations",
        ["CPIC"],
    );

    push_section(
        &mut out,
        !pgx.guidelines.is_empty(),
        "guidelines",
        "Guidelines",
        ["CPIC"],
    );
    out.extend(outcome_section_sources(
        "pgx",
        &pgx.section_outcomes,
        &[
            ("frequencies", "Population Frequencies"),
            ("annotations", "PharmGKB Annotations"),
        ],
    ));
    out
}

pub(crate) fn adverse_event_source_search_section_sources(
    search: &AdverseEventSourceSearch,
) -> Vec<SectionSource> {
    outcome_section_sources(
        "adverse_event",
        &search.section_outcomes,
        &[("faers", "OpenFDA FAERS"), ("vaers", "CDC CVX/VAERS")],
    )
}

pub(crate) fn adverse_event_section_sources(event: &AdverseEvent) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let overview_present = has_text(&event.report_id)
        || has_text(&event.drug)
        || has_opt_text(&event.patient)
        || has_opt_text(&event.reporter_type)
        || has_opt_text(&event.reporter_country)
        || has_opt_text(&event.indication)
        || has_opt_text(&event.date);
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        !event.reactions.is_empty(),
        "reactions",
        "Reactions",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        !event.outcomes.is_empty(),
        "outcomes",
        "Outcomes",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        !event.concomitant_medications.is_empty(),
        "concomitant_drugs",
        "Concomitant Drugs",
        ["OpenFDA"],
    );
    out
}

pub(crate) fn device_event_section_sources(event: &DeviceEvent) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let overview_present = has_text(&event.report_id)
        || has_text(&event.device)
        || has_opt_text(&event.report_number)
        || has_opt_text(&event.manufacturer)
        || has_opt_text(&event.event_type)
        || has_opt_text(&event.date);
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        has_opt_text(&event.description),
        "description",
        "Description",
        ["OpenFDA"],
    );
    out
}

pub(crate) fn adverse_event_report_section_sources(
    report: &AdverseEventReport,
) -> Vec<SectionSource> {
    match report {
        AdverseEventReport::Faers(event) => adverse_event_section_sources(event),
        AdverseEventReport::Device(event) => device_event_section_sources(event),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::article::{ArticleAuthorCompleteness, ArticleSource};
    use crate::entities::pathway::Pathway;
    use crate::entities::section_outcome::SectionOutcome;
    use crate::entities::variant::Variant;

    #[test]
    fn pathway_source_label_maps_known_sources() {
        assert_eq!(pathway_source_label("WikiPathways"), "WikiPathways");
        assert_eq!(pathway_source_label("wikipathways"), "WikiPathways");
        assert_eq!(pathway_source_label("KEGG"), "KEGG");
        assert_eq!(pathway_source_label("kegg"), "KEGG");
        assert_eq!(pathway_source_label("Reactome"), "Reactome");
        assert_eq!(pathway_source_label("reactome"), "Reactome");
    }

    #[test]
    fn pathway_source_label_passes_through_unknown_non_empty_source() {
        assert_eq!(pathway_source_label("SomeOtherDB"), "SomeOtherDB");
    }

    #[test]
    fn pathway_source_label_falls_back_to_reactome_for_empty() {
        assert_eq!(pathway_source_label(""), "Reactome");
        assert_eq!(pathway_source_label("   "), "Reactome");
    }

    #[test]
    fn pathway_section_sources_emits_wikipathways_not_reactome_for_wp_card() {
        let pathway = Pathway {
            section_outcomes: {
                let mut outcomes =
                    SectionOutcomes::with_keys(crate::entities::pathway::PATHWAY_OUTCOME_KEYS);
                outcomes.complete(
                    "genes",
                    crate::entities::section_outcome::SectionOutcome::data("WikiPathways"),
                );
                outcomes
            },
            source: "WikiPathways".to_string(),
            id: "WP254".to_string(),
            name: "Apoptosis".to_string(),
            species: Some("Homo sapiens".to_string()),
            summary: None,
            genes: vec!["TP53".to_string()],
            events: Vec::new(),
            enrichment: Vec::new(),
        };

        let sections = pathway_section_sources(&pathway);
        for section in &sections {
            for source in &section.sources {
                assert_ne!(
                    source, "Reactome",
                    "section '{}' incorrectly attributed to Reactome for a WikiPathways card",
                    section.key
                );
                assert_eq!(source, "WikiPathways");
            }
        }
        let keys: Vec<&str> = sections.iter().map(|s| s.key.as_str()).collect();
        assert!(keys.contains(&"identity"), "identity section expected");
        assert!(keys.contains(&"genes"), "genes section expected");
    }

    #[test]
    fn diagnostic_unavailable_regulatory_outcome_has_no_source_credit() {
        let diagnostic: crate::entities::diagnostic::Diagnostic =
            serde_json::from_value(serde_json::json!({
                "section_outcomes": {
                    "regulatory": {
                        "outcome": "unavailable",
                        "sources": [],
                        "message": "OpenFDA diagnostic regulatory data is temporarily unavailable."
                    }
                },
                "source": "gtr",
                "source_id": "GTR000000001.1",
                "accession": "GTR000000001.1",
                "name": "Example",
                "test_type": null,
                "manufacturer": null,
                "target_marker": null,
                "regulatory_version": null,
                "prequalification_year": null,
                "laboratory": null,
                "institution": null,
                "country": null,
                "clia_number": null,
                "state_licenses": null,
                "current_status": null,
                "public_status": null,
                "method_categories": [],
                "genes": null,
                "conditions": null,
                "methods": null,
                "regulatory": []
            }))
            .expect("diagnostic fixture");

        let source = diagnostic_section_sources(&diagnostic)
            .into_iter()
            .find(|source| source.key == "regulatory")
            .expect("regulatory projection");
        assert_eq!(source.outcome, SectionOutcomeState::Unavailable);
        assert!(source.sources.is_empty());
    }

    #[test]
    fn variant_provenance_includes_gwas_when_requested_section_is_unavailable() {
        let variant = Variant {
            section_outcomes: {
                let mut outcomes = crate::entities::variant::default_variant_section_outcomes();
                outcomes.complete(
                    "gwas",
                    crate::entities::section_outcome::SectionOutcome::unavailable(
                        "GWAS association data is temporarily unavailable.",
                    ),
                );
                outcomes
            },
            gene: String::new(),
            id: "rs7903146".to_string(),
            hgvs_p: None,
            legacy_name: None,
            hgvs_c: None,
            rsid: Some("rs7903146".to_string()),
            cosmic_id: None,
            significance: None,
            clinvar_id: None,
            clinvar_review_status: None,
            clinvar_review_stars: None,
            conditions: Vec::new(),
            gnomad_af: None,
            allele_frequency_raw: None,
            allele_frequency_percent: None,
            consequence: None,
            cadd_score: None,
            sift_pred: None,
            polyphen_pred: None,
            conservation: None,
            expanded_predictions: Vec::new(),
            population_breakdown: None,
            cosmic_context: None,
            cgi_associations: Vec::new(),
            civic: None,
            clinvar_conditions: Vec::new(),
            clinvar_condition_reports: None,
            top_disease: None,
            cancerhotspots: None,
            cancer_frequencies: Vec::new(),
            cancer_frequency_source: None,
            gwas: Vec::new(),
            gwas_unavailable_reason: Some("GWAS association data temporarily unavailable.".into()),
            supporting_pmids: None,
            prediction: None,
        };

        let sources = variant_section_sources(&variant);
        assert!(sources.iter().any(|source| source.key == "gwas"));
    }

    #[test]
    fn drug_provenance_emits_variant_targets_when_present() {
        let drug = Drug {
            section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
            name: "rindopepimut".to_string(),
            drugbank_id: None,
            chembl_id: None,
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            approval_date_raw: None,
            approval_date_display: None,
            approval_summary: None,
            brand_names: Vec::new(),
            route: None,
            targets: vec!["EGFR".to_string()],
            variant_targets: vec!["EGFRvIII".to_string()],
            target_family: None,
            target_family_name: None,
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            interaction_pagination: None,
            interaction_bundle_freshness: None,
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            faers_query: None,
            label: None,
            label_set_id: None,
            shortage: None,
            approvals: None,
            us_safety_warnings: None,
            ema_regulatory: None,
            ema_safety: None,
            ema_shortage: None,
            who_prequalification: None,
            civic: None,
        };

        let sources = drug_section_sources(&drug);
        assert!(sources.iter().any(|source| {
            source.key == "variant_targets"
                && source.label == "Variant Targets"
                && source.sources == vec!["CIViC".to_string()]
        }));
    }

    #[test]
    fn drug_provenance_adds_who_to_regulatory_sources() {
        let drug = Drug {
            section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
            name: "trastuzumab".to_string(),
            drugbank_id: None,
            chembl_id: None,
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            approval_date_raw: None,
            approval_date_display: None,
            approval_summary: None,
            brand_names: Vec::new(),
            route: None,
            targets: Vec::new(),
            variant_targets: Vec::new(),
            target_family: None,
            target_family_name: None,
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            interaction_pagination: None,
            interaction_bundle_freshness: None,
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            faers_query: None,
            label: None,
            label_set_id: None,
            shortage: None,
            approvals: None,
            us_safety_warnings: None,
            ema_regulatory: None,
            ema_safety: None,
            ema_shortage: None,
            who_prequalification: Some(vec![crate::entities::drug::WhoPrequalificationEntry {
                kind: crate::entities::drug::WhoPrequalificationKind::FinishedPharma,
                who_reference_number: Some("BT-ON001".to_string()),
                inn: "Trastuzumab".to_string(),
                presentation: Some(
                    "Trastuzumab Powder for concentrate for solution for infusion 150 mg"
                        .to_string(),
                ),
                dosage_form: Some("Powder for concentrate for solution for infusion".to_string()),
                product_type: "Biotherapeutic Product".to_string(),
                therapeutic_area: "Oncology".to_string(),
                applicant: "Samsung Bioepis NL B.V.".to_string(),
                listing_basis: Some("Prequalification - Abridged".to_string()),
                alternative_listing_basis: None,
                prequalification_date: Some("2019-12-18".to_string()),
                who_product_id: None,
                grade: None,
                confirmation_document_date: None,
                vaccine_type: None,
                commercial_name: None,
                dose_count: None,
                manufacturer: None,
                responsible_nra: None,
            }]),
            civic: None,
        };

        let sources = drug_section_sources(&drug);
        assert!(sources.iter().any(|source| {
            source.key == "regulatory" && source.sources == vec!["WHO Prequalification".to_string()]
        }));
    }

    #[test]
    fn drug_section_sources_omit_interactions_when_no_interaction_data_is_present() {
        let drug = Drug {
            section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
            name: "pembrolizumab".to_string(),
            drugbank_id: Some("DB09037".to_string()),
            chembl_id: None,
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            approval_date_raw: None,
            approval_date_display: None,
            approval_summary: None,
            brand_names: Vec::new(),
            route: None,
            targets: Vec::new(),
            variant_targets: Vec::new(),
            target_family: None,
            target_family_name: None,
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            interaction_pagination: None,
            interaction_bundle_freshness: None,
            pharm_classes: vec!["PD-1 inhibitors".to_string()],
            top_adverse_events: Vec::new(),
            faers_query: None,
            label: None,
            label_set_id: None,
            shortage: None,
            approvals: None,
            us_safety_warnings: None,
            ema_regulatory: None,
            ema_safety: None,
            ema_shortage: None,
            who_prequalification: None,
            civic: None,
        };

        let sources = drug_section_sources(&drug);
        assert!(!sources.iter().any(|source| source.key == "interactions"));
    }

    #[test]
    fn drug_interaction_report_section_sources_include_drugbank_when_descriptions_present() {
        let report = DrugInteractionReport {
            name: "warfarin".to_string(),
            drugbank_id: Some("DB00682".to_string()),
            chembl_id: None,
            interactions: vec![crate::entities::drug::DrugInteraction {
                drug: "aspirin".to_string(),
                level: Some("Major".to_string()),
                description: Some("May increase bleeding risk.".to_string()),
                partner_classes: vec!["antiplatelets".to_string()],
            }],
            pagination: crate::entities::drug::interactions::DrugInteractionPagination {
                total: 1,
                count: 1,
                offset: 0,
                limit: 25,
                next_command: None,
            },
            bundle_freshness:
                crate::entities::drug::interactions::DrugInteractionBundleFreshness {
                    status:
                        crate::entities::drug::interactions::DrugInteractionFreshnessStatus::Fresh,
                },
            coverage_status:
                crate::entities::drug::interactions::DrugInteractionCoverageStatus::InDdinterCoverage,
            source_note: None,
            coverage_note: None,
            label_interaction_text: None,
        };

        let sources = drug_interaction_report_section_sources(&report);
        assert_eq!(sources.len(), 1);
        assert_eq!(
            sources[0].sources,
            vec!["DDInter".to_string(), "DrugBank".to_string()]
        );
    }

    #[test]
    fn disease_section_sources_include_survival_when_note_present() {
        let disease = Disease {
            id: "MONDO:0007947".to_string(),
            name: "Marfan syndrome".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            clinical_features: Vec::new(),
            key_features: Vec::new(),
            variants: Vec::new(),
            top_variant: None,
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            survival: None,
            survival_note: Some("SEER survival data not available for this condition.".into()),
            civic: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: None,
            diagnostics_note: None,
            section_outcomes: {
                let mut outcomes = crate::entities::disease::default_disease_section_outcomes();
                outcomes.complete(
                    "survival",
                    crate::entities::section_outcome::SectionOutcome::empty("SEER Explorer"),
                );
                outcomes
            },
            xrefs: std::collections::HashMap::new(),
        };

        let sources = disease_section_sources(&disease);
        assert!(sources.iter().any(|source| {
            source.key == "survival"
                && source.label == "Survival"
                && source.sources == vec!["SEER Explorer".to_string()]
        }));
    }

    #[test]
    fn gene_section_sources_include_funding_when_present() {
        let gene = Gene {
            section_outcomes: {
                let mut outcomes =
                    SectionOutcomes::with_keys(crate::entities::gene::GENE_OUTCOME_KEYS);
                outcomes.complete(
                    "funding",
                    crate::entities::section_outcome::SectionOutcome::data("NIH Reporter"),
                );
                outcomes
            },
            symbol: "ERBB2".to_string(),
            name: "erb-b2 receptor tyrosine kinase 2".to_string(),
            entrez_id: "2064".to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
            funding: Some(crate::sources::nih_reporter::NihReporterFundingSection {
                query: "ERBB2".to_string(),
                fiscal_years: vec![2022, 2023, 2024, 2025, 2026],
                matching_project_years: 1,
                grants: vec![crate::sources::nih_reporter::NihReporterGrant {
                    project_title: "Example grant".to_string(),
                    project_num: "P-1".to_string(),
                    core_project_num: Some("CORE-1".to_string()),
                    project_detail_url: None,
                    pi_name: None,
                    organization: None,
                    fiscal_year: 2026,
                    award_amount: 10,
                }],
            }),
            funding_note: None,
            diagnostics: None,
            diagnostics_note: None,
        };

        let sources = gene_section_sources(&gene);
        assert!(sources.iter().any(|source| {
            source.key == "funding"
                && source.label == "Funding"
                && source.sources == vec!["NIH Reporter".to_string()]
        }));
    }

    #[test]
    fn gene_section_sources_include_diagnostics_from_rows() {
        let gene = Gene {
            section_outcomes: {
                let mut outcomes =
                    SectionOutcomes::with_keys(crate::entities::gene::GENE_OUTCOME_KEYS);
                outcomes.complete(
                    "diagnostics",
                    crate::entities::section_outcome::SectionOutcome::data(
                        "NCBI Genetic Testing Registry",
                    ),
                );
                outcomes
            },
            symbol: "BRCA1".to_string(),
            name: "BRCA1 DNA repair associated".to_string(),
            entrez_id: "672".to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: Some(vec![crate::entities::diagnostic::DiagnosticSearchResult {
                source: "gtr".to_string(),
                accession: "GTR000000001.1".to_string(),
                name: "BRCA1 Hereditary Cancer Panel".to_string(),
                test_type: Some("Clinical".to_string()),
                manufacturer_or_lab: Some("Example Lab".to_string()),
                genes: vec!["BRCA1".to_string()],
                conditions: vec!["breast cancer".to_string()],
            }]),
            diagnostics_note: None,
        };

        let sources = gene_section_sources(&gene);
        assert!(sources.iter().any(|source| {
            source.key == "diagnostics"
                && source.label == "Diagnostics"
                && source.sources == vec!["NCBI Genetic Testing Registry".to_string()]
        }));
    }

    #[test]
    fn gene_section_sources_marks_unavailable_diagnostics_without_source_credit() {
        let gene = Gene {
            section_outcomes: {
                let mut outcomes = SectionOutcomes::with_keys(
                    crate::entities::gene::GENE_OUTCOME_KEYS,
                );
                outcomes.complete(
                    "diagnostics",
                    crate::entities::section_outcome::SectionOutcome::unavailable(
                        "Gene diagnostics are unavailable.",
                    ),
                );
                outcomes
            },
            symbol: "BRCA1".to_string(),
            name: "BRCA1 DNA repair associated".to_string(),
            entrez_id: "672".to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: None,
            diagnostics_note: Some(
                "Diagnostic local data is unavailable. Run `biomcp gtr sync` to enable gene diagnostic pivots."
                    .to_string(),
            ),
        };

        let sources = gene_section_sources(&gene);
        assert!(sources.iter().any(|source| {
            source.key == "diagnostics"
                && source.label == "Diagnostics"
                && source.outcome == SectionOutcomeState::Unavailable
                && source.sources.is_empty()
        }));
    }

    #[test]
    fn disease_section_sources_include_funding_when_note_present() {
        let disease = Disease {
            id: "MONDO:0007947".to_string(),
            name: "Marfan syndrome".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            clinical_features: Vec::new(),
            key_features: Vec::new(),
            variants: Vec::new(),
            top_variant: None,
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            survival: None,
            survival_note: None,
            civic: None,
            disgenet: None,
            funding: None,
            funding_note: Some("No NIH funding data found for this query.".into()),
            diagnostics: None,
            diagnostics_note: None,
            section_outcomes: {
                let mut outcomes = crate::entities::disease::default_disease_section_outcomes();
                outcomes.complete("funding", SectionOutcome::empty("NIH Reporter"));
                outcomes
            },
            xrefs: std::collections::HashMap::new(),
        };

        let sources = disease_section_sources(&disease);
        assert!(sources.iter().any(|source| {
            source.key == "funding"
                && source.label == "Funding"
                && source.sources == vec!["NIH Reporter".to_string()]
        }));
    }

    #[test]
    fn disease_section_sources_include_diagnostics_from_rows() {
        let disease = Disease {
            id: "MONDO:0005105".to_string(),
            name: "melanoma".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            clinical_features: Vec::new(),
            key_features: Vec::new(),
            variants: Vec::new(),
            top_variant: None,
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            survival: None,
            survival_note: None,
            civic: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: Some(vec![
                crate::entities::diagnostic::DiagnosticSearchResult {
                    source: "gtr".to_string(),
                    accession: "GTR000000001.1".to_string(),
                    name: "BRCA1 Hereditary Cancer Panel".to_string(),
                    test_type: Some("Clinical".to_string()),
                    manufacturer_or_lab: Some("Example Lab".to_string()),
                    genes: vec!["BRCA1".to_string()],
                    conditions: vec!["melanoma".to_string()],
                },
                crate::entities::diagnostic::DiagnosticSearchResult {
                    source: "who-ivd".to_string(),
                    accession: "ITPW00000".to_string(),
                    name: "Example IVD".to_string(),
                    test_type: Some("Molecular".to_string()),
                    manufacturer_or_lab: Some("WHO Lab".to_string()),
                    genes: Vec::new(),
                    conditions: vec!["melanoma".to_string()],
                },
            ]),
            diagnostics_note: None,
            section_outcomes: {
                let mut outcomes = crate::entities::disease::default_disease_section_outcomes();
                outcomes.complete(
                    "diagnostics",
                    SectionOutcome::data_sources([
                        "NCBI Genetic Testing Registry",
                        "WHO Prequalified IVD",
                    ]),
                );
                outcomes
            },
            xrefs: std::collections::HashMap::new(),
        };

        let sources = disease_section_sources(&disease);
        assert!(sources.iter().any(|source| {
            source.key == "diagnostics"
                && source.label == "Diagnostics"
                && source.sources
                    == vec![
                        "NCBI Genetic Testing Registry".to_string(),
                        "WHO Prequalified IVD".to_string(),
                    ]
        }));
    }

    #[test]
    fn disease_section_sources_include_clinical_features() {
        let disease = Disease {
            id: "MONDO:0004277".to_string(),
            name: "uterine leiomyoma".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            clinical_features: vec![crate::entities::disease::DiseasePhenotype {
                hpo_id: "HP:0000132".to_string(),
                name: Some("Menorrhagia".to_string()),
                evidence: Some("IEA".to_string()),
                frequency: None,
                frequency_qualifier: None,
                onset_qualifier: None,
                sex_qualifier: None,
                stage_qualifier: None,
                qualifiers: Vec::new(),
                source: Some("infores:hpo-annotations".to_string()),
            }],
            key_features: Vec::new(),
            variants: Vec::new(),
            top_variant: None,
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            survival: None,
            survival_note: None,
            civic: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: None,
            diagnostics_note: None,
            section_outcomes: {
                let mut outcomes = crate::entities::disease::default_disease_section_outcomes();
                outcomes.complete("phenotypes", SectionOutcome::data("HPO"));
                outcomes
            },
            xrefs: std::collections::HashMap::new(),
        };

        let sources = disease_section_sources(&disease);
        assert!(sources.iter().any(|source| {
            source.key == "phenotypes"
                && source.label == "Phenotypes"
                && source.sources == vec!["HPO".to_string()]
        }));
    }

    #[test]
    fn disease_section_sources_include_diagnostics_note_sources() {
        let disease = Disease {
            id: "MONDO:0018076".to_string(),
            name: "tuberculosis".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            clinical_features: Vec::new(),
            key_features: Vec::new(),
            variants: Vec::new(),
            top_variant: None,
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            survival: None,
            survival_note: None,
            civic: None,
            disgenet: None,
            funding: None,
            funding_note: None,
            diagnostics: None,
            diagnostics_note: Some(
                "Diagnostic local data is unavailable. Run `biomcp gtr sync` and `biomcp who-ivd sync` to enable disease diagnostic pivots."
                    .to_string(),
            ),
            section_outcomes: {
                let mut outcomes = crate::entities::disease::default_disease_section_outcomes();
                outcomes.complete(
                    "diagnostics",
                    SectionOutcome::unavailable("Diagnostic local data is unavailable."),
                );
                outcomes
            },
            xrefs: std::collections::HashMap::new(),
        };

        let sources = disease_section_sources(&disease);
        assert!(sources.iter().any(|source| {
            source.key == "diagnostics"
                && source.label == "Diagnostics"
                && source.outcome == SectionOutcomeState::Unavailable
                && source.sources.is_empty()
        }));
    }

    #[test]
    fn article_section_sources_uses_resolved_fulltext_and_indexing_providers() {
        let mut article = Article {
            section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
                crate::entities::article::ARTICLE_OUTCOME_KEYS,
            ),
            pmid: Some("22663011".to_string()),
            pmcid: Some("PMC123456".to_string()),
            doi: Some("10.1000/example".to_string()),
            title: "Example article".to_string(),
            authors: vec!["Example Author".to_string()],
            author_count: 1,
            author_completeness: ArticleAuthorCompleteness::SourceLimited,
            author_source: ArticleSource::EuropePmc,
            journal: Some("Example Journal".to_string()),
            date: Some("2024-01-01".to_string()),
            citation_count: Some(12),
            publication_type: Some("Journal Article".to_string()),
            open_access: Some(true),
            abstract_text: Some("Abstract text.".to_string()),
            full_text_path: Some(std::path::PathBuf::from("/tmp/fulltext.md")),
            full_text_note: None,
            full_text_source: Some(crate::entities::article::ArticleFulltextSource {
                kind: crate::entities::article::ArticleFulltextKind::JatsXml,
                label: "Europe PMC XML".to_string(),
                source: "Europe PMC".to_string(),
            }),
            full_text_manifest: None,
            full_text_coverage: None,
            not_included: None,
            europepmc_license: None,
            europepmc_retracted: None,
            annotations: None,
            indexing: None,
            semantic_scholar: None,
            pubtator_fallback: false,
        };
        article.section_outcomes.complete(
            "fulltext",
            crate::entities::section_outcome::SectionOutcome::data("Europe PMC"),
        );

        let sources = article_section_sources(&article);
        assert!(sources.iter().any(|source| {
            source.key == "fulltext"
                && source.label == "Full Text"
                && source.sources == vec!["Europe PMC".to_string()]
        }));
        assert!(sources.iter().any(|source| {
            source.key == "authors" && source.sources == vec!["Europe PMC".to_string()]
        }));

        article.indexing = Some(crate::entities::article::ArticleIndexing {
            status: crate::entities::article::ArticleIndexingStatus::Unavailable,
            source: ArticleSource::PubMed,
            authors: Vec::new(),
            mesh_headings: Vec::new(),
            failure: None,
        });
        article.section_outcomes.complete(
            "indexing",
            crate::entities::section_outcome::SectionOutcome::unavailable(
                "PubMed indexing is temporarily unavailable.",
            ),
        );
        let sources = article_section_sources(&article);
        assert!(sources.iter().any(|source| {
            source.key == "indexing"
                && source.label == "Article Indexing"
                && source.outcome
                    == crate::entities::section_outcome::SectionOutcomeState::Unavailable
                && source.sources.is_empty()
        }));
    }

    #[test]
    fn article_section_sources_projects_unavailable_fulltext_without_sources() {
        let mut outcomes = crate::entities::section_outcome::SectionOutcomes::with_keys(
            crate::entities::article::ARTICLE_OUTCOME_KEYS,
        );
        outcomes.complete(
            "fulltext",
            crate::entities::section_outcome::SectionOutcome::unavailable(
                "Full text is unavailable because a source failed.",
            ),
        );
        let article = Article {
            section_outcomes: outcomes,
            pmid: Some("22663011".to_string()),
            pmcid: Some("PMC123456".to_string()),
            doi: Some("10.1000/example".to_string()),
            title: "Example article".to_string(),
            authors: Vec::new(),
            author_count: 0,
            author_completeness: ArticleAuthorCompleteness::Unavailable,
            author_source: ArticleSource::PubTator,
            journal: Some("Example Journal".to_string()),
            date: Some("2024-01-01".to_string()),
            citation_count: Some(12),
            publication_type: Some("Journal Article".to_string()),
            open_access: Some(true),
            abstract_text: Some("Abstract text.".to_string()),
            full_text_path: None,
            full_text_note: Some("Full text not available: API error".to_string()),
            full_text_source: None,
            full_text_manifest: None,
            full_text_coverage: None,
            not_included: None,
            europepmc_license: None,
            europepmc_retracted: None,
            annotations: None,
            indexing: None,
            semantic_scholar: None,
            pubtator_fallback: false,
        };

        let sources = article_section_sources(&article);
        assert!(sources.iter().any(|source| {
            source.key == "fulltext"
                && source.outcome
                    == crate::entities::section_outcome::SectionOutcomeState::Unavailable
                && source.sources.is_empty()
        }));
    }
}
