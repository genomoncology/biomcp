//! Ordered skill routes and matcher functions for `biomcp suggest`.

use super::QuestionContext;
use super::extract::*;
use super::patterns::*;

#[cfg(test)]
pub(crate) struct SuggestRouteExample {
    pub question: &'static str,
    pub expected_skill: &'static str,
    pub expected_commands: [&'static str; 2],
}

pub(super) struct SuggestRoute {
    pub(super) slug: &'static str,
    pub(super) summary: &'static str,
    pub(super) matcher: fn(&QuestionContext<'_>) -> Option<Vec<String>>,
}

#[cfg(test)]
const ROUTE_EXAMPLES: &[SuggestRouteExample] = &[
    SuggestRouteExample {
        question: "Is variant rs113488022 pathogenic in melanoma?",
        expected_skill: "variant-pathogenicity",
        expected_commands: [
            "biomcp get variant rs113488022 clinvar predictions population",
            "biomcp get variant rs113488022 civic cgi",
        ],
    },
    SuggestRouteExample {
        question: "Follow up PMID:22663011 citations",
        expected_skill: "article-follow-up",
        expected_commands: [
            concat!("biomcp get article ", "22663011 annotations"),
            concat!("biomcp article citations ", "22663011 --limit 5"),
        ],
    },
    SuggestRouteExample {
        question: "When was imatinib approved?",
        expected_skill: "drug-regulatory",
        expected_commands: [
            "biomcp get drug imatinib regulatory",
            "biomcp get drug imatinib approvals",
        ],
    },
    SuggestRouteExample {
        question: "What pharmacogenes affect warfarin dosing?",
        expected_skill: "pharmacogene-cumulative",
        expected_commands: [
            concat!("biomcp search pgx -d ", "warfarin --limit 10"),
            concat!("biomcp get pgx ", "warfarin recommendations annotations"),
        ],
    },
    SuggestRouteExample {
        question: "Are there recruiting trials for melanoma?",
        expected_skill: "trial-recruitment",
        expected_commands: [
            "biomcp search trial -c melanoma --status recruiting --limit 5",
            "biomcp search article -d melanoma --type review --limit 5",
        ],
    },
    SuggestRouteExample {
        question: "How do I distinguish Goldberg-Shprintzen syndrome vs Shprintzen-Goldberg syndrome?",
        expected_skill: "syndrome-disambiguation",
        expected_commands: [
            "biomcp search disease \"Goldberg-Shprintzen syndrome\" --limit 5",
            "biomcp search disease \"Shprintzen-Goldberg syndrome\" --limit 5",
        ],
    },
    SuggestRouteExample {
        question: "Is Borna disease virus linked to brain tumor?",
        expected_skill: "negative-evidence",
        expected_commands: [
            "biomcp search article -k \"Borna disease virus brain tumor\" --type review --limit 5",
            "biomcp search article -k \"Borna disease virus brain tumor association\" --limit 5",
        ],
    },
    SuggestRouteExample {
        question: "What chromosome is Arnold Chiari syndrome mapped to?",
        expected_skill: "disease-locus-mapping",
        expected_commands: [
            "biomcp search article -k \"Arnold Chiari syndrome chromosome\" --type review --limit 10",
            "biomcp search article -k \"Arnold Chiari syndrome deletion duplication trisomy chromosome\" --limit 10",
        ],
    },
    SuggestRouteExample {
        question: "What pathway explains imatinib resistance?",
        expected_skill: "mechanism-pathway",
        expected_commands: [
            concat!("biomcp search drug ", "imatinib --limit 5"),
            concat!("biomcp get drug ", "imatinib targets regulatory"),
        ],
    },
    SuggestRouteExample {
        question: "Where is OPA1 localized?",
        expected_skill: "gene-function-localization",
        expected_commands: [
            "biomcp get gene OPA1 protein hpa",
            "biomcp get gene OPA1 ontology",
        ],
    },
    SuggestRouteExample {
        question: "Which variants are in PLN?",
        expected_skill: "mutation-catalog",
        expected_commands: [
            concat!("biomcp get gene ", "PLN"),
            concat!("biomcp search variant -g ", "PLN --limit 10"),
        ],
    },
    SuggestRouteExample {
        question: "How does NANOG regulate cell cycle?",
        expected_skill: "cellular-process-regulation",
        expected_commands: ["biomcp get gene NANOG", "biomcp get gene NANOG ontology"],
    },
    SuggestRouteExample {
        question: "What drugs treat melanoma?",
        expected_skill: "treatment-lookup",
        expected_commands: [
            "biomcp search drug --indication melanoma --limit 5",
            "biomcp search article -d melanoma --type review --limit 5",
        ],
    },
    SuggestRouteExample {
        question: "What symptoms are seen in Marfan syndrome?",
        expected_skill: "symptom-phenotype",
        expected_commands: [
            "biomcp get disease \"Marfan syndrome\" phenotypes",
            "biomcp search article -d \"Marfan syndrome\" --type review --limit 5",
        ],
    },
    SuggestRouteExample {
        question: "What is BRAF in melanoma?",
        expected_skill: "gene-disease-orientation",
        expected_commands: [
            "biomcp search all --gene BRAF --disease melanoma",
            "biomcp search article -g BRAF -d melanoma --type review --limit 5",
        ],
    },
];

pub(super) const ROUTES: &[SuggestRoute] = &[
    SuggestRoute {
        slug: "variant-pathogenicity",
        summary: "Use the variant pathogenicity playbook for clinical-significance and evidence checks on a specific variant.",
        matcher: route_variant_pathogenicity,
    },
    SuggestRoute {
        slug: "article-follow-up",
        summary: "Use the article follow-up playbook to expand from a known publication into annotations, citations, and related papers.",
        matcher: route_article_follow_up,
    },
    SuggestRoute {
        slug: "drug-regulatory",
        summary: "Use the drug regulatory playbook for approval, label, regional regulatory, or withdrawal questions.",
        matcher: route_drug_regulatory,
    },
    SuggestRoute {
        slug: "pharmacogene-cumulative",
        summary: "Use the pharmacogene cumulative playbook for PGx, dosing, metabolism, and recommendation questions.",
        matcher: route_pharmacogene_cumulative,
    },
    SuggestRoute {
        slug: "trial-recruitment",
        summary: "Use the trial recruitment playbook for recruiting, enrolling, or open clinical-trial questions.",
        matcher: route_trial_recruitment,
    },
    SuggestRoute {
        slug: "syndrome-disambiguation",
        summary: "Use the syndrome disambiguation playbook when the question compares similarly named syndromes or diagnoses.",
        matcher: route_syndrome_disambiguation,
    },
    SuggestRoute {
        slug: "negative-evidence",
        summary: "Use the negative evidence playbook for claims that need absence, contradiction, or weak-association checks.",
        matcher: route_negative_evidence,
    },
    SuggestRoute {
        slug: "disease-locus-mapping",
        summary: "Use the disease locus mapping playbook for chromosome, locus, mapped gene, or disease-location questions.",
        matcher: route_disease_locus_mapping,
    },
    SuggestRoute {
        slug: "mechanism-pathway",
        summary: "Use the mechanism pathway playbook for pathway, mechanism, signaling, resistance, or target questions.",
        matcher: route_mechanism_pathway,
    },
    SuggestRoute {
        slug: "gene-function-localization",
        summary: "Use the gene function localization playbook for protein function, localization, tissue, and ontology questions.",
        matcher: route_gene_function_localization,
    },
    SuggestRoute {
        slug: "mutation-catalog",
        summary: "Use the mutation catalog playbook to enumerate variants or mutations for a gene.",
        matcher: route_mutation_catalog,
    },
    SuggestRoute {
        slug: "cellular-process-regulation",
        summary: "Use the cellular process regulation playbook for questions about how a gene regulates a biological process.",
        matcher: route_cellular_process_regulation,
    },
    SuggestRoute {
        slug: "treatment-lookup",
        summary: "Use the treatment lookup playbook for therapy or approved-drug questions.",
        matcher: route_treatment_lookup,
    },
    SuggestRoute {
        slug: "symptom-phenotype",
        summary: "Use the symptom phenotype playbook for symptom, sign, phenotype, and clinical-feature questions.",
        matcher: route_symptom_phenotype,
    },
    SuggestRoute {
        slug: "gene-disease-orientation",
        summary: "Use the gene disease orientation playbook for first-pass gene-plus-disease context.",
        matcher: route_gene_disease_orientation,
    },
];

#[cfg(test)]
pub(crate) fn route_examples() -> &'static [SuggestRouteExample] {
    ROUTE_EXAMPLES
}

fn route_variant_pathogenicity(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "pathogenic",
        "pathogenicity",
        "clinical significance",
        "clinical",
        "actionable",
        "significance",
        "benign",
        "risk",
        "clinvar",
        "oncogenic",
        "civic",
    ]) {
        return None;
    }
    let variant = extract_variant_identifier(ctx)?;
    Some(vec![
        format!(
            "biomcp get variant {} clinvar predictions population",
            quote(&variant)
        ),
        format!("biomcp get variant {} civic cgi", quote(&variant)),
    ])
}

fn route_article_follow_up(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "article",
        "paper",
        "publication",
        "pubmed",
        "pmid",
        "pmcid",
        "doi",
        "citation",
        "citations",
        "reference",
        "references",
        "follow up",
        "recommendations",
    ]) {
        return None;
    }
    let article = extract_article_identifier(ctx)?;
    Some(vec![
        format!("biomcp get article {} annotations", quote(&article)),
        format!("biomcp article citations {} --limit 5", quote(&article)),
    ])
}

fn route_drug_regulatory(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "approved",
        "approval",
        "approvals",
        "regulatory",
        "label",
        "labeling",
        "licensed",
        "authorization",
        "authorized",
        "withdrawn",
        "fda",
        "ema",
        "eu",
        "who",
    ]) {
        return None;
    }
    let drug = capture_clean(regulatory_subject_re(), ctx, 1)
        .or_else(|| capture_clean(approval_for_re(), ctx, 1))
        .or_else(|| {
            content_anchor_before_terms(
                ctx,
                &[
                    "approved",
                    "approval",
                    "licensed",
                    "authorized",
                    "authorization",
                ],
            )
        })?;
    let regulatory = match detect_regulatory_region(ctx) {
        Some(region) => format!(
            "biomcp get drug {} regulatory --region {region}",
            quote(&drug)
        ),
        None => format!("biomcp get drug {} regulatory", quote(&drug)),
    };
    Some(vec![
        regulatory,
        format!("biomcp get drug {} approvals", quote(&drug)),
    ])
}

fn route_pharmacogene_cumulative(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "pharmacogene",
        "pharmacogenes",
        "pharmacogenomic",
        "pharmacogenomics",
        "pgx",
        "genotype",
        "dosing",
        "dose",
        "metabolism",
        "recommendation",
        "recommendations",
    ]) {
        return None;
    }
    let drug = capture_clean(pgx_drug_re(), ctx, 1)
        .or_else(|| capture_clean(generic_for_re(), ctx, 1))
        .or_else(|| content_anchor_before_terms(ctx, &["dosing", "dose", "metabolism"]))?;
    Some(vec![
        format!("biomcp search pgx -d {} --limit 10", quote(&drug)),
        format!(
            "biomcp get pgx {} recommendations annotations",
            quote(&drug)
        ),
    ])
}

fn route_trial_recruitment(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "trial",
        "trials",
        "recruiting",
        "recruitment",
        "enrolling",
        "enrollment",
        "open trial",
        "open trials",
        "clinical trial",
        "clinical trials",
    ]) {
        return None;
    }
    if let Some(intervention) = trial_intervention_anchor(ctx) {
        return Some(vec![
            format!(
                "biomcp search trial -i {} --status recruiting --limit 5",
                quote(&intervention)
            ),
            format!(
                "biomcp search article --drug {} --type review --limit 5",
                quote(&intervention)
            ),
        ]);
    }

    let condition = capture_clean(trial_condition_re(), ctx, 1)
        .or_else(|| capture_clean(generic_for_re(), ctx, 1))?;
    Some(vec![
        format!(
            "biomcp search trial -c {} --status recruiting --limit 5",
            quote(&condition)
        ),
        format!(
            "biomcp search article -d {} --type review --limit 5",
            quote(&condition)
        ),
    ])
}

fn route_syndrome_disambiguation(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "distinguish",
        "differentiate",
        "difference",
        "disambiguate",
        "confused",
        "versus",
        "vs",
        "compare",
    ]) {
        return None;
    }
    let (first, second) = syndrome_pair(ctx)?;
    Some(vec![
        format!("biomcp search disease {} --limit 5", quote(&first)),
        format!("biomcp search disease {} --limit 5", quote(&second)),
    ])
}

fn route_negative_evidence(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "linked",
        "associated",
        "association",
        "causes",
        "cause",
        "evidence for",
        "evidence against",
        "no evidence",
        "rule out",
        "absence",
        "contradict",
        "refute",
    ]) {
        return None;
    }
    let (first, second) = negative_terms(ctx)?;
    let topic = format!("{first} {second}");
    let association_topic = format!("{topic} association");
    Some(vec![
        format!(
            "biomcp search article -k {} --type review --limit 5",
            quote(&topic)
        ),
        format!(
            "biomcp search article -k {} --limit 5",
            quote(&association_topic)
        ),
    ])
}

fn route_disease_locus_mapping(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "chromosome",
        "locus",
        "loci",
        "mapped",
        "mapping",
        "deletion",
        "duplication",
        "trisomy",
        "cytogenetic",
        "cytoband",
        "genomic location",
    ]) {
        return None;
    }
    let disease = capture_clean(mapped_disease_re(), ctx, 1)
        .or_else(|| capture_clean(generic_for_re(), ctx, 1))?;
    let chromosome_topic = format!("{disease} chromosome");
    let structural_topic = format!("{disease} deletion duplication trisomy chromosome");
    Some(vec![
        format!(
            "biomcp search article -k {} --type review --limit 10",
            quote(&chromosome_topic)
        ),
        format!(
            "biomcp search article -k {} --limit 10",
            quote(&structural_topic)
        ),
    ])
}

fn route_mechanism_pathway(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "pathway",
        "mechanism",
        "mechanisms",
        "signaling",
        "resistance",
        "target",
        "targets",
        "work",
        "works",
        "causes through",
    ]) {
        return None;
    }

    if let Some(drug) = mechanism_drug_anchor(ctx)
        && extract_gene_symbol(ctx)
            .as_deref()
            .is_none_or(|gene| gene != drug.as_str())
    {
        return Some(vec![
            format!("biomcp search drug {} --limit 5", quote(&drug)),
            format!("biomcp get drug {} targets regulatory", quote(&drug)),
        ]);
    }

    let gene = extract_gene_symbol(ctx)?;
    let topic = mechanism_gene_topic(ctx, &gene);
    Some(vec![
        format!("biomcp get gene {} pathways protein", quote(&gene)),
        format!(
            "biomcp search article -g {} -k {} --type review --limit 5",
            quote(&gene),
            quote(&topic)
        ),
    ])
}

fn route_gene_function_localization(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if ctx.has_any(&[
        "regulate",
        "regulates",
        "regulated",
        "regulation",
        "cell cycle",
        "cellular process",
        "differentiation",
        "apoptosis",
        "g1 s",
    ]) {
        return None;
    }
    if !ctx.has_any(&[
        "localized",
        "localization",
        "located",
        "where is",
        "function",
        "does",
        "do",
        "ontology",
        "tissue",
        "protein",
    ]) {
        return None;
    }
    let gene = extract_gene_symbol(ctx)?;
    Some(vec![
        format!("biomcp get gene {} protein hpa", quote(&gene)),
        format!("biomcp get gene {} ontology", quote(&gene)),
    ])
}

fn route_mutation_catalog(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "variant",
        "variants",
        "mutation",
        "mutations",
        "catalog",
        "hotspot",
        "hotspots",
    ]) {
        return None;
    }
    let gene = extract_gene_symbol(ctx)?;
    Some(vec![
        format!("biomcp get gene {}", quote(&gene)),
        format!("biomcp search variant -g {} --limit 10", quote(&gene)),
    ])
}

fn route_cellular_process_regulation(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "regulate",
        "regulates",
        "regulated",
        "regulation",
        "affect",
        "affects",
        "cell cycle",
        "cellular process",
        "differentiation",
        "apoptosis",
        "g1 s",
        "control",
        "controls",
        "process",
    ]) {
        return None;
    }
    let gene = extract_gene_symbol(ctx)?;
    Some(vec![
        format!("biomcp get gene {}", quote(&gene)),
        format!("biomcp get gene {} ontology", quote(&gene)),
    ])
}

fn route_treatment_lookup(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "treat",
        "treats",
        "treatment",
        "treatments",
        "therapy",
        "therapies",
        "drug",
        "drugs",
        "medication",
        "medications",
    ]) {
        return None;
    }
    let disease = capture_clean_any(treatment_disease_re(), ctx, &[1, 2])
        .or_else(|| capture_clean(generic_for_re(), ctx, 1))?;
    Some(vec![
        format!(
            "biomcp search drug --indication {} --limit 5",
            quote(&disease)
        ),
        format!(
            "biomcp search article -d {} --type review --limit 5",
            quote(&disease)
        ),
    ])
}

fn route_symptom_phenotype(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    if !ctx.has_any(&[
        "symptom",
        "symptoms",
        "phenotype",
        "phenotypes",
        "sign",
        "signs",
        "feature",
        "features",
        "clinical features",
    ]) {
        return None;
    }
    if let Some(disease) = capture_clean(symptom_disease_re(), ctx, 1)
        .or_else(|| capture_clean(symptom_named_disease_re(), ctx, 1))
    {
        return Some(vec![
            format!("biomcp get disease {} phenotypes", quote(&disease)),
            format!(
                "biomcp search article -d {} --type review --limit 5",
                quote(&disease)
            ),
        ]);
    }

    let symptom = capture_clean(symptom_text_re(), ctx, 1)
        .or_else(|| cleanup_question_topic(ctx.original))?;
    Some(vec![
        format!("biomcp discover {}", quote(&symptom)),
        format!("biomcp search phenotype {} --limit 5", quote(&symptom)),
    ])
}

fn route_gene_disease_orientation(ctx: &QuestionContext<'_>) -> Option<Vec<String>> {
    let gene = extract_gene_symbol(ctx)?;
    let disease = capture_clean(gene_disease_re(), ctx, 1)?;
    Some(vec![
        format!(
            "biomcp search all --gene {} --disease {}",
            quote(&gene),
            quote(&disease)
        ),
        format!(
            "biomcp search article -g {} -d {} --type review --limit 5",
            quote(&gene),
            quote(&disease)
        ),
    ])
}
