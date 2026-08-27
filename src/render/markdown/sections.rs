//! Section-filter parsing and progressive-disclosure helpers for markdown renderers.

use super::*;

const DISEASE_DISCOVERY_SECTION_NAMES: &[&str] = &[
    "genes",
    "pathways",
    "phenotypes",
    "diagnostics",
    "clinical_features",
    "survival",
    "funding",
    "variants",
    "models",
    "prevalence",
    "civic",
    "disgenet",
    "all",
];

const GENE_DISCOVERY_SECTION_NAMES: &[&str] = &[
    "pathways",
    "ontology",
    "diseases",
    "diagnostics",
    "funding",
    "protein",
    "go",
    "interactions",
    "civic",
    "expression",
    "hpa",
    "druggability",
    "clingen",
    "constraint",
    "disgenet",
    "all",
];

pub(super) fn has_all_section(requested: &[String]) -> bool {
    requested
        .iter()
        .any(|s| s.trim().eq_ignore_ascii_case("all"))
}

pub(super) fn is_section_only_requested(requested: &[String]) -> bool {
    !has_all_section(requested) && requested.iter().any(|s| !s.trim().is_empty())
}

pub(super) fn requested_section_names(requested: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for section in requested {
        let section = section.trim();
        if section.is_empty() || section.eq_ignore_ascii_case("all") {
            continue;
        }
        let normalized = section.to_ascii_lowercase();
        if out.iter().any(|v| v == &normalized) {
            continue;
        }
        out.push(normalized);
    }
    out
}

pub(super) fn section_header(entity_label: &str, requested: &[String]) -> String {
    let names = requested_section_names(requested);
    if names.is_empty() {
        entity_label.to_string()
    } else {
        format!("{entity_label} - {}", names.join(", "))
    }
}

pub(super) fn format_sections_block(entity: &str, id: &str, sections: Vec<String>) -> String {
    let commands = visible_section_commands(entity, id, &sections);
    if commands.is_empty() {
        return String::new();
    }
    let mut out = String::from("More:");
    for command in commands {
        let section = command.rsplit(' ').next().unwrap_or_default();
        let _ = write!(
            out,
            "\n  {command}   - {}",
            section_description(entity, section)
        );
    }
    let id_q = quote_arg(id);
    let _ = write!(out, "\nAll:\n  biomcp get {entity} {id_q} all");
    out
}

pub(super) fn section_description(entity: &str, section: &str) -> &'static str {
    match (entity, section) {
        ("gene", "pathways") => "Reactome/KEGG pathway context",
        ("gene", "ontology") => "GO-style functional enrichment",
        ("gene", "diseases") => "disease associations",
        ("gene", "diagnostics") => "diagnostic tests for this gene from GTR",
        ("gene", "protein") => "UniProt function and localization detail",
        ("gene", "expression") => "GTEx tissue expression",
        ("gene", "hpa") => "Human Protein Atlas tissue expression and localization",
        ("gene", "go") => "QuickGO term annotations",
        ("gene", "interactions") => "STRING interaction partners",
        ("gene", "civic") => "CIViC clinical evidence",
        ("gene", "druggability") => "DGIdb interactions and tractability",
        ("gene", "clingen") => "ClinGen validity and dosage sensitivity",
        ("gene", "constraint") => "gnomAD gene constraint metrics",
        ("gene", "disgenet") => "DisGeNET scored disease links",
        ("gene", "funding") => "NIH Reporter grant support",
        ("variant", "population") => "compact gnomAD population frequencies",
        ("variant", "population-details") => "full gnomAD population frequency tables",
        ("article", "annotations") => "PubTator normalized entity mentions",
        ("article", "fulltext") => "cached full text when available",
        ("article", "tldr") => "Semantic Scholar summary and influence",
        ("diagnostic", "genes") => "diagnostic-associated gene names",
        ("diagnostic", "conditions") => {
            "condition, pathogen, or marker names reported for this diagnostic"
        }
        ("diagnostic", "methods") => "reported testing methods for this diagnostic",
        ("diagnostic", "regulatory") => "FDA 510(k) clearance and PMA approval records",
        ("disease", "genes") => "associated genes",
        ("disease", "pathways") => "pathways from associated genes",
        ("disease", "phenotypes") => "HPO phenotype annotations",
        ("disease", "diagnostics") => "diagnostic tests for this condition from GTR and WHO IVD",
        ("disease", "clinical_features") => "Monarch/HPO phenotype rows as clinical features",
        ("disease", "variants") => "disease-associated variants",
        ("disease", "models") => "model-organism evidence",
        ("disease", "prevalence") => "prevalence and epidemiology context",
        ("disease", "survival") => "SEER Explorer cancer survival rates",
        ("disease", "funding") => "NIH Reporter grant support",
        ("disease", "civic") => "CIViC disease-context evidence",
        ("disease", "disgenet") => "DisGeNET scored disease-gene links",
        ("drug", "label") => "approved-indication and FDA label detail beyond the base card",
        ("drug", "regulatory") => {
            "approval and supplement history; use only if the base card lacks approval context"
        }
        ("drug", "safety") => {
            "regulatory safety detail; use `biomcp drug adverse-events <name>` first when you want post-marketing signal"
        }
        ("drug", "targets") => "ChEMBL and OpenTargets targets",
        ("drug", "indications") => "OpenTargets indication evidence",
        ("drug", "interactions") => "label interactions and public-data fallback",
        ("drug", "civic") => "CIViC therapy evidence",
        ("drug", "approvals") => "Drugs@FDA approval history",
        ("trial", "eligibility") => "inclusion and exclusion criteria",
        ("trial", "contacts") => "central and site contact details",
        ("trial", "locations") => "site list and contact details",
        ("trial", "outcomes") => "endpoint measures and time frames",
        ("trial", "arms") => "study arms and assigned interventions",
        ("trial", "references") => "linked publications and PMID citations",
        _ => "additional detail",
    }
}

pub(super) fn sections_for(requested: &[String], available: &[&str]) -> Vec<String> {
    if has_all_section(requested) {
        return Vec::new();
    }

    let requested_set: HashSet<String> = requested
        .iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    available
        .iter()
        .copied()
        .filter(|s| *s != "all")
        .filter(|s| !requested_set.contains(&s.to_ascii_lowercase()))
        .map(|section| section.to_string())
        .collect()
}

fn visible_section_limit(entity: &str) -> usize {
    match entity {
        "disease" => 6,
        "gene" => 4,
        "diagnostic" => 4,
        _ => 3,
    }
}

fn visible_section_commands(entity: &str, id: &str, sections: &[String]) -> Vec<String> {
    let id_q = quote_arg(id);
    if id_q.is_empty() {
        return Vec::new();
    }

    sections
        .iter()
        .take(visible_section_limit(entity))
        .map(|section| format!("biomcp get {entity} {id_q} {section}"))
        .collect()
}

pub(crate) fn disease_next_commands(
    disease: &Disease,
    requested_sections: &[String],
) -> Vec<String> {
    let mut out = visible_section_commands(
        "disease",
        &disease.id,
        &sections_disease(disease, requested_sections),
    );
    out.extend(related_disease(disease));
    dedupe_markdown_commands(out)
}

pub(crate) fn diagnostic_next_commands(
    diagnostic: &Diagnostic,
    requested_sections: &[String],
) -> Vec<String> {
    let mut out = visible_section_commands(
        "diagnostic",
        &diagnostic.accession,
        &sections_diagnostic(diagnostic, requested_sections),
    );
    if out.is_empty() && is_section_only_requested(requested_sections) {
        let accession = quote_arg(&diagnostic.accession);
        if !accession.is_empty() {
            out.push(format!("biomcp get diagnostic {accession}"));
        }
    }
    out.extend(related_diagnostic(diagnostic));
    dedupe_markdown_commands(out)
}

pub(crate) fn gene_next_commands(gene: &Gene, requested_sections: &[String]) -> Vec<String> {
    let mut out = visible_section_commands(
        "gene",
        &gene.symbol,
        &sections_gene(gene, requested_sections),
    );
    out.extend(related_gene(gene));
    dedupe_markdown_commands(out)
}

pub(super) fn sections_gene(gene: &Gene, requested: &[String]) -> Vec<String> {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return Vec::new();
    }

    sections_for(requested, GENE_DISCOVERY_SECTION_NAMES)
}

pub(super) fn sections_variant(variant: &Variant, requested: &[String]) -> Vec<String> {
    let id = quote_arg(preferred_variant_follow_up_id(variant));
    if id.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::variant::VARIANT_SECTION_NAMES)
}

pub(super) fn sections_article(article: &Article, requested: &[String]) -> Vec<String> {
    let key = article
        .pmid
        .as_deref()
        .or(article.pmcid.as_deref())
        .or(article.doi.as_deref())
        .unwrap_or("");
    let key = quote_arg(key);
    if key.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::article::ARTICLE_SECTION_NAMES)
}

const COMPLETED_TRIAL_SECTION_NAMES: &[&str] = &[
    "outcomes",
    "references",
    "arms",
    "eligibility",
    "contacts",
    "locations",
    "all",
];

pub(super) fn is_completed_or_terminated_trial_status(status: &str) -> bool {
    let status = status.trim();
    status.eq_ignore_ascii_case("COMPLETED") || status.eq_ignore_ascii_case("TERMINATED")
}

pub(super) fn sections_trial(trial: &Trial, requested: &[String]) -> Vec<String> {
    let nct_id = trial.nct_id.trim();
    if nct_id.is_empty() {
        return Vec::new();
    }
    let available = if is_completed_or_terminated_trial_status(&trial.status) {
        COMPLETED_TRIAL_SECTION_NAMES
    } else {
        crate::entities::trial::TRIAL_SECTION_NAMES
    };
    sections_for(requested, available)
}

pub(super) fn sections_drug(drug: &Drug, requested: &[String]) -> Vec<String> {
    let name = quote_arg(&drug.name);
    if name.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::drug::DRUG_SECTION_NAMES)
}

pub(super) fn sections_disease(disease: &Disease, requested: &[String]) -> Vec<String> {
    let key = quote_arg(&disease.id);
    if key.is_empty() {
        return Vec::new();
    }
    sections_for(requested, DISEASE_DISCOVERY_SECTION_NAMES)
}

pub(super) fn sections_diagnostic(diagnostic: &Diagnostic, requested: &[String]) -> Vec<String> {
    let accession = quote_arg(&diagnostic.accession);
    if accession.is_empty() {
        return Vec::new();
    }
    sections_for(
        requested,
        crate::entities::diagnostic::supported_diagnostic_sections_for_source(&diagnostic.source),
    )
}

pub(super) fn sections_pgx(pgx: &Pgx, requested: &[String]) -> Vec<String> {
    if pgx.query.trim().is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::pgx::PGX_SECTION_NAMES)
}

pub(super) fn sections_pathway(pathway: &Pathway, requested: &[String]) -> Vec<String> {
    let id = quote_arg(&pathway.id);
    if id.is_empty() {
        return Vec::new();
    }
    sections_for(
        requested,
        crate::entities::pathway::supported_pathway_sections_for_source(&pathway.source),
    )
}

pub(super) fn sections_protein(protein: &Protein, requested: &[String]) -> Vec<String> {
    let accession = quote_arg(&protein.accession);
    if accession.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::protein::PROTEIN_SECTION_NAMES)
}

pub(super) fn sections_adverse_event(event: &AdverseEvent, requested: &[String]) -> Vec<String> {
    let report_id = quote_arg(&event.report_id);
    if report_id.is_empty() {
        return Vec::new();
    }
    sections_for(
        requested,
        crate::entities::adverse_event::ADVERSE_EVENT_SECTION_NAMES,
    )
}

#[cfg(test)]
mod tests;
