use std::collections::HashSet;

use serde::Serialize;

use crate::entities::adverse_event::{AdverseEvent, AdverseEventReport, DeviceEvent};
use crate::entities::article::Article;
use crate::entities::discover::DiscoverResult;
use crate::entities::disease::Disease;
use crate::entities::drug::Drug;
use crate::entities::gene::Gene;
use crate::entities::pathway::Pathway;
use crate::entities::pgx::Pgx;
use crate::entities::protein::Protein;
use crate::entities::trial::Trial;
use crate::entities::variant::Variant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionSource {
    pub key: String,
    pub label: String,
    pub sources: Vec<String>,
}

impl SectionSource {
    pub(crate) fn normalized(self) -> Option<Self> {
        let key = self.key.trim();
        let label = self.label.trim();
        let sources = normalize_sources(self.sources);
        if key.is_empty() || label.is_empty() || sources.is_empty() {
            return None;
        }
        Some(Self {
            key: key.to_string(),
            label: label.to_string(),
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
    let mut sources = Vec::new();
    if !drug.interactions.is_empty() {
        sources.push("DrugBank".to_string());
    }
    if has_opt_text(&drug.interaction_text) {
        sources.push("OpenFDA label".to_string());
    }
    normalize_sources(sources)
}

pub(crate) fn drug_interaction_heading_label(drug: &Drug) -> String {
    let sources = drug_interaction_sources(drug);
    if sources.is_empty() {
        "Interactions".to_string()
    } else {
        format!("Interactions ({})", sources.join(" / "))
    }
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
    if let Some(pathways) = &gene.pathways {
        push_section(
            &mut out,
            true,
            "pathways",
            "Pathways",
            pathways.iter().map(|row| row.source.as_str()),
        );
    }
    push_section(
        &mut out,
        gene.ontology.is_some(),
        "ontology",
        "Ontology",
        ["Enrichr"],
    );
    push_section(
        &mut out,
        gene.diseases.is_some(),
        "diseases",
        "Diseases",
        ["Enrichr"],
    );
    push_section(
        &mut out,
        gene.protein.is_some(),
        "protein",
        "Protein",
        ["UniProt"],
    );
    push_section(&mut out, gene.go.is_some(), "go", "GO Terms", ["QuickGO"]);
    push_section(
        &mut out,
        gene.interactions.is_some(),
        "interactions",
        "Interactions",
        ["STRING"],
    );
    push_section(&mut out, gene.civic.is_some(), "civic", "CIViC", ["CIViC"]);
    push_section(
        &mut out,
        gene.expression.is_some(),
        "expression",
        "Expression",
        ["GTEx"],
    );
    push_section(
        &mut out,
        gene.hpa.is_some(),
        "hpa",
        "Human Protein Atlas",
        ["Human Protein Atlas"],
    );
    push_section(
        &mut out,
        gene.druggability.is_some(),
        "druggability",
        "Druggability",
        ["DGIdb", "Open Targets"],
    );
    push_section(
        &mut out,
        gene.clingen.is_some(),
        "clingen",
        "ClinGen",
        ["ClinGen"],
    );
    push_section(
        &mut out,
        gene.constraint.is_some(),
        "constraint",
        "Constraint",
        ["gnomAD"],
    );
    push_section(
        &mut out,
        gene.disgenet.is_some(),
        "disgenet",
        "DisGeNET",
        ["DisGeNET"],
    );
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
    push_section(
        &mut out,
        !drug.top_adverse_events.is_empty(),
        "safety",
        "Safety",
        ["OpenFDA FAERS"],
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
        !drug.targets.is_empty(),
        "targets",
        "Targets",
        ["ChEMBL", "Open Targets"],
    );
    push_section(
        &mut out,
        !drug.indications.is_empty(),
        "indications",
        "Indications",
        ["Open Targets"],
    );
    let interaction_sources = drug_interaction_sources(drug);
    push_section(
        &mut out,
        !interaction_sources.is_empty(),
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
        drug.approvals.is_some(),
        "approvals",
        "Drugs@FDA Approvals",
        ["OpenFDA Drugs@FDA"],
    );
    push_section(&mut out, drug.civic.is_some(), "civic", "CIViC", ["CIViC"]);
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
        !disease.treatment_landscape.is_empty(),
        "treatments",
        "Treatments",
        ["MyChem.info indication search"],
    );
    push_section(
        &mut out,
        disease.recruiting_trial_count.is_some(),
        "recruiting_trials",
        "Recruiting Trials",
        ["ClinicalTrials.gov"],
    );
    push_section(
        &mut out,
        !disease.associated_genes.is_empty() || !disease.gene_associations.is_empty(),
        "associated_genes",
        "Associated Genes",
        ["Monarch Initiative", "Open Targets"],
    );
    push_section(
        &mut out,
        !disease.pathways.is_empty(),
        "pathways",
        "Pathways",
        ["Reactome"],
    );
    push_section(
        &mut out,
        !disease.phenotypes.is_empty(),
        "phenotypes",
        "Phenotypes",
        ["Monarch Initiative", "HPO"],
    );
    push_section(
        &mut out,
        !disease.variants.is_empty(),
        "variants",
        "Variants",
        ["Monarch Initiative", "CIViC"],
    );
    push_section(
        &mut out,
        !disease.models.is_empty(),
        "models",
        "Models",
        ["Monarch Initiative"],
    );
    push_section(
        &mut out,
        !disease.prevalence.is_empty() || has_opt_text(&disease.prevalence_note),
        "prevalence",
        "Prevalence",
        ["HPO"],
    );
    push_section(
        &mut out,
        disease.civic.is_some(),
        "civic",
        "CIViC",
        ["CIViC"],
    );
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
        variant.prediction.is_some(),
        "prediction",
        "AlphaGenome Prediction",
        ["AlphaGenome"],
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
    push_section(
        &mut out,
        variant.civic.is_some(),
        "civic",
        "CIViC",
        ["CIViC"],
    );
    push_section(
        &mut out,
        !variant.cancer_frequencies.is_empty(),
        "cbioportal",
        "cBioPortal",
        ["cBioPortal"],
    );
    push_section(
        &mut out,
        !variant.gwas.is_empty(),
        "gwas",
        "GWAS",
        ["GWAS Catalog"],
    );
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
        ["PubMed", "Europe PMC"],
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
    push_section(
        &mut out,
        article.full_text_path.is_some() || has_opt_text(&article.full_text_note),
        "fulltext",
        "Full Text",
        ["PMC OA"],
    );
    push_section(
        &mut out,
        article.semantic_scholar.is_some(),
        "semantic_scholar",
        "Semantic Scholar",
        ["Semantic Scholar"],
    );
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
    push_section(
        &mut out,
        !pathway.genes.is_empty(),
        "genes",
        "Genes",
        source_ref,
    );
    push_section(
        &mut out,
        !pathway.events.is_empty(),
        "events",
        "Events",
        source_ref,
    );
    push_section(
        &mut out,
        !pathway.enrichment.is_empty(),
        "enrichment",
        "Enrichment",
        ["g:Profiler (Reactome enrichment)"],
    );
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
    push_section(
        &mut out,
        !protein.structures.is_empty() || protein.structure_count.is_some(),
        "structures",
        "Structures",
        ["PDB", "AlphaFold via UniProt"],
    );
    push_section(
        &mut out,
        !protein.domains.is_empty(),
        "domains",
        "Domains",
        ["InterPro"],
    );
    push_section(
        &mut out,
        !protein.interactions.is_empty(),
        "interactions",
        "Interactions",
        ["STRING"],
    );
    push_section(
        &mut out,
        !protein.complexes.is_empty(),
        "complexes",
        "Complexes",
        ["ComplexPortal"],
    );
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
        !pgx.frequencies.is_empty(),
        "frequencies",
        "Population Frequencies",
        ["CPIC"],
    );
    push_section(
        &mut out,
        !pgx.guidelines.is_empty(),
        "guidelines",
        "Guidelines",
        ["CPIC"],
    );
    push_section(
        &mut out,
        !pgx.annotations.is_empty() || has_opt_text(&pgx.annotations_note),
        "annotations",
        "PharmGKB Annotations",
        ["PharmGKB"],
    );
    out
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
    use crate::entities::pathway::Pathway;

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
}
