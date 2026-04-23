//! Offline question-to-skill routing for the `biomcp suggest` first move.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

use crate::render::markdown::shell_quote_arg;

#[derive(Debug, Clone)]
pub(crate) struct SuggestArgs {
    pub question: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub(crate) struct SuggestResponse {
    pub matched_skill: Option<String>,
    pub summary: String,
    pub first_commands: Vec<String>,
    pub full_skill: Option<String>,
}

#[derive(Debug, Clone, Copy)]
#[cfg(test)]
pub(crate) struct SuggestRouteExample {
    pub question: &'static str,
    pub expected_skill: &'static str,
    pub expected_commands: [&'static str; 2],
}

struct QuestionContext<'a> {
    original: &'a str,
    normalized: String,
}

struct SuggestRoute {
    slug: &'static str,
    summary: &'static str,
    matcher: fn(&QuestionContext<'_>) -> Option<Vec<String>>,
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

const ROUTES: &[SuggestRoute] = &[
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

/// Render a suggestion for a free-text biomedical question.
///
/// # Errors
///
/// Returns an error only when JSON serialization fails.
pub(crate) fn run(args: SuggestArgs, json: bool) -> anyhow::Result<String> {
    let response = suggest_question(&args.question);
    if json {
        Ok(crate::render::json::to_pretty(&response)?)
    } else {
        Ok(render_markdown(&response))
    }
}

pub(crate) fn suggest_question(question: &str) -> SuggestResponse {
    let context = QuestionContext::new(question);
    if context.normalized.is_empty() {
        return no_match_response();
    }

    for route in ROUTES {
        if let Some(commands) = (route.matcher)(&context) {
            return matched_response(route.slug, route.summary, commands);
        }
    }

    no_match_response()
}

#[cfg(test)]
pub(crate) fn route_examples() -> &'static [SuggestRouteExample] {
    ROUTE_EXAMPLES
}

impl QuestionContext<'_> {
    fn new(question: &str) -> QuestionContext<'_> {
        QuestionContext {
            original: question.trim(),
            normalized: normalize_text(question),
        }
    }

    fn has_any(&self, phrases: &[&str]) -> bool {
        phrases
            .iter()
            .any(|phrase| contains_phrase(&self.normalized, phrase))
    }
}

fn matched_response(slug: &str, summary: &str, commands: Vec<String>) -> SuggestResponse {
    let mut seen = HashSet::new();
    let mut first_commands = Vec::new();
    for command in commands {
        if seen.insert(command.to_ascii_lowercase()) {
            first_commands.push(command);
        }
    }
    assert_eq!(
        first_commands.len(),
        2,
        "suggest route {slug} must produce exactly two starter commands",
    );

    SuggestResponse {
        matched_skill: Some(slug.to_string()),
        summary: summary.to_string(),
        first_commands,
        full_skill: Some(format!("biomcp skill {slug}")),
    }
}

fn no_match_response() -> SuggestResponse {
    SuggestResponse {
        matched_skill: None,
        summary: "No confident BioMCP skill match.".to_string(),
        first_commands: Vec::new(),
        full_skill: None,
    }
}

fn render_markdown(response: &SuggestResponse) -> String {
    let matched_skill = response.matched_skill.as_deref().unwrap_or("no match");
    let full_skill = response.full_skill.as_deref().unwrap_or("none");

    let mut out = String::new();
    out.push_str("# BioMCP Suggestion\n\n");
    if response.matched_skill.is_some() {
        out.push_str(&format!("- matched_skill: `{matched_skill}`\n"));
    } else {
        out.push_str("- matched_skill: no match\n");
    }
    out.push_str(&format!("- summary: {}\n", response.summary));
    out.push_str("- first_commands:\n");
    if response.first_commands.is_empty() {
        out.push_str("  none\n");
    } else {
        for (index, command) in response.first_commands.iter().enumerate() {
            out.push_str(&format!("  {}. `{command}`\n", index + 1));
        }
    }
    if response.full_skill.is_some() {
        out.push_str(&format!("- full_skill: `{full_skill}`\n"));
    } else {
        out.push_str("- full_skill: none\n");
    }

    if response.matched_skill.is_none() {
        out.push_str(
            "\nTry `biomcp skill list` to browse playbooks or `biomcp discover \"<question>\"` \
             when you need entity resolution instead of playbook selection.\n",
        );
    }
    out
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

fn extract_variant_identifier(ctx: &QuestionContext<'_>) -> Option<String> {
    rsid_re()
        .find(ctx.original)
        .map(|m| m.as_str().to_ascii_lowercase())
        .or_else(|| {
            gene_variant_re()
                .find(ctx.original)
                .and_then(|m| clean_anchor(m.as_str()))
        })
        .or_else(|| {
            hgvs_re()
                .find(ctx.original)
                .and_then(|m| clean_anchor(m.as_str()))
        })
}

fn extract_article_identifier(ctx: &QuestionContext<'_>) -> Option<String> {
    pmid_re()
        .captures(ctx.original)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .or_else(|| {
            pmcid_re()
                .captures(ctx.original)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        })
        .or_else(|| doi_re().find(ctx.original).map(|m| m.as_str().to_string()))
        .or_else(|| {
            bare_article_id_re()
                .find(ctx.original)
                .map(|m| m.as_str().to_string())
        })
}

fn extract_gene_symbol(ctx: &QuestionContext<'_>) -> Option<String> {
    if let Some(symbol) = capture_clean(explicit_gene_re(), ctx, 1) {
        let symbol = symbol.to_ascii_uppercase();
        if !is_gene_stopword(&symbol) {
            return Some(symbol);
        }
    }

    for matched in gene_symbol_re().find_iter(ctx.original) {
        let symbol = matched.as_str();
        if is_gene_stopword(symbol) {
            continue;
        }
        return Some(symbol.to_string());
    }
    None
}

fn detect_regulatory_region(ctx: &QuestionContext<'_>) -> Option<&'static str> {
    if ctx.has_any(&["fda", "us", "u s", "united states"]) {
        return Some("us");
    }
    if ctx.has_any(&["ema", "eu", "europe", "european"]) {
        return Some("eu");
    }
    if ctx.has_any(&["who", "world health organization"]) {
        return Some("who");
    }
    None
}

fn trial_intervention_anchor(ctx: &QuestionContext<'_>) -> Option<String> {
    capture_clean_any(trial_intervention_re(), ctx, &[1, 2]).or_else(|| {
        if ctx.has_any(&["with"]) {
            capture_clean(trial_with_re(), ctx, 1)
        } else {
            None
        }
    })
}

fn content_anchor_before_terms(ctx: &QuestionContext<'_>, terms: &[&str]) -> Option<String> {
    let lower = ctx.original.to_ascii_lowercase();
    for term in terms {
        if let Some(index) = lower.find(term) {
            let before = &ctx.original[..index];
            if let Some(anchor) = clean_anchor(before) {
                return Some(anchor);
            }
        }
    }
    None
}

fn syndrome_pair(ctx: &QuestionContext<'_>) -> Option<(String, String)> {
    [
        syndrome_compare_re(),
        syndrome_difference_re(),
        syndrome_vs_re(),
        syndrome_confused_re(),
    ]
    .iter()
    .find_map(|regex| capture_pair(regex, ctx))
}

fn negative_terms(ctx: &QuestionContext<'_>) -> Option<(String, String)> {
    [linked_terms_re(), cause_terms_re(), evidence_terms_re()]
        .iter()
        .find_map(|regex| capture_pair(regex, ctx))
}

fn mechanism_drug_anchor(ctx: &QuestionContext<'_>) -> Option<String> {
    capture_clean_any(mechanism_resistance_re(), ctx, &[1, 2, 3])
        .or_else(|| capture_clean(mechanism_work_re(), ctx, 1))
        .or_else(|| capture_clean(mechanism_of_re(), ctx, 1))
        .or_else(|| capture_clean(mechanism_topic_re(), ctx, 1))
}

fn mechanism_gene_topic(ctx: &QuestionContext<'_>, gene: &str) -> String {
    cleanup_question_topic(ctx.original)
        .map(|topic| topic.replace(gene, "").trim().to_string())
        .and_then(|topic| clean_anchor(&topic))
        .unwrap_or_else(|| "pathway mechanism".to_string())
}

fn cleanup_question_topic(value: &str) -> Option<String> {
    let mut topic = clean_anchor(value)?;
    for prefix in [
        "what pathway explains ",
        "what mechanism explains ",
        "what is the mechanism of ",
        "what is the pathway for ",
        "is there evidence for ",
        "is ",
        "are ",
        "does ",
        "do ",
    ] {
        if topic.to_ascii_lowercase().starts_with(prefix) {
            topic = topic[prefix.len()..].trim().to_string();
        }
    }
    clean_anchor(&topic)
}

fn capture_clean(regex: &'static Regex, ctx: &QuestionContext<'_>, index: usize) -> Option<String> {
    let captures = regex.captures(ctx.original)?;
    clean_anchor(captures.get(index)?.as_str())
}

fn capture_clean_any(
    regex: &'static Regex,
    ctx: &QuestionContext<'_>,
    indexes: &[usize],
) -> Option<String> {
    let captures = regex.captures(ctx.original)?;
    indexes
        .iter()
        .filter_map(|index| captures.get(*index))
        .find_map(|matched| clean_anchor(matched.as_str()))
}

fn capture_pair(regex: &'static Regex, ctx: &QuestionContext<'_>) -> Option<(String, String)> {
    let captures = regex.captures(ctx.original)?;
    let first = clean_anchor(captures.get(1)?.as_str())?;
    let second = clean_anchor(captures.get(2)?.as_str())?;
    Some((first, second))
}

fn clean_anchor(raw: &str) -> Option<String> {
    let collapsed = raw
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| {
            matches!(
                c,
                '?' | '.' | ',' | ';' | ':' | '!' | '"' | '\'' | '(' | ')' | '[' | ']'
            )
        })
        .trim()
        .to_string();
    if collapsed.is_empty() {
        return None;
    }

    let mut value = collapsed;
    loop {
        let lower = value.to_ascii_lowercase();
        let Some(prefix) = ANCHOR_PREFIXES
            .iter()
            .find(|prefix| lower.starts_with(**prefix))
        else {
            break;
        };
        value = value[prefix.len()..].trim().to_string();
        if value.is_empty() {
            return None;
        }
    }

    let normalized = normalize_text(&value);
    if normalized.len() < 2 || STOP_ANCHORS.contains(&normalized.as_str()) {
        return None;
    }
    if normalized
        .split_whitespace()
        .all(|word| STOP_ANCHORS.contains(&word))
    {
        return None;
    }
    Some(value)
}

fn quote(value: &str) -> String {
    shell_quote_arg(value)
}

fn normalize_text(value: &str) -> String {
    let mut out = String::new();
    let mut previous_space = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_space = false;
        } else if !previous_space {
            out.push(' ');
            previous_space = true;
        }
    }
    out.trim().to_string()
}

fn contains_phrase(normalized: &str, phrase: &str) -> bool {
    let phrase = normalize_text(phrase);
    if phrase.is_empty() {
        return false;
    }
    let haystack = format!(" {normalized} ");
    let needle = format!(" {phrase} ");
    haystack.contains(&needle)
}

fn is_gene_stopword(symbol: &str) -> bool {
    let upper = symbol.to_ascii_uppercase();
    let lower = symbol.to_ascii_lowercase();
    GENE_STOPWORDS.contains(&upper.as_str()) || STOP_ANCHORS.contains(&lower.as_str())
}

fn rsid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\brs\d+\b").expect("valid rsID regex"))
}

fn gene_variant_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b[A-Z][A-Z0-9-]{1,14}\s+(?:p\.)?[A-Z][A-Za-z]{0,2}\d+[A-Z][A-Za-z]{0,2}\b")
            .expect("valid gene variant regex")
    })
}

fn hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:c|g|m|n|p|r)\.[A-Za-z0-9_>.+:-]+\b").expect("valid HGVS regex")
    })
}

fn pmid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bpmid[:\s]*([0-9]{5,})\b").expect("valid PMID regex"))
}

fn pmcid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:pmcid[:\s]*)?(PMC[0-9]+)\b").expect("valid PMCID regex")
    })
}

fn doi_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b10\.\d{4,9}/[^\s]+\b").expect("valid DOI regex"))
}

fn bare_article_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[0-9]{7,}\b").expect("valid bare article ID regex"))
}

fn gene_symbol_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b[A-Z][A-Z0-9-]{1,14}\b").expect("valid gene symbol regex"))
}

fn explicit_gene_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bgene\s+([A-Za-z][A-Za-z0-9-]{1,14})\b")
            .expect("valid explicit gene regex")
    })
}

fn regulatory_subject_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:was|is|were|are)\s+([A-Za-z0-9][A-Za-z0-9 -]{1,80}?)\s+(?:approved|licensed|authorized|withdrawn|regulated|labeled|labelled)\b",
        )
        .expect("valid regulatory subject regex")
    })
}

fn approval_for_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:approval|approvals|label|regulatory|fda|ema|who)\s+(?:for|of)\s+(.+?)(?:\?|$)")
            .expect("valid approval-for regex")
    })
}

fn pgx_drug_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:affect|affects|for|of|with)\s+([A-Za-z0-9][A-Za-z0-9 -]{1,80}?)\s+(?:dosing|dose|metabolism|response|recommendations?)\b")
            .expect("valid PGx drug regex")
    })
}

fn generic_for_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:for|in|of|with|on)\s+(.+?)(?:\?|$)").expect("valid generic-for regex")
    })
}

fn trial_condition_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:recruiting|enrolling|open)?\s*trials?\s+(?:for|in|with)\s+(.+?)(?:\?|$)",
        )
        .expect("valid trial condition regex")
    })
}

fn trial_intervention_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\btrials?\s+(?:for|with)\s+(?:drug|intervention|therapy|treatment)\s+(.+?)(?:\?|$)|\bintervention\s+(.+?)(?:\?|$)",
        )
        .expect("valid trial intervention regex")
    })
}

fn trial_with_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\btrials?\s+with\s+(.+?)(?:\?|$)").expect("valid trial-with regex")
    })
}

fn syndrome_compare_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:distinguish|differentiate|disambiguate|compare)\s+(.+?)\s+(?:vs\.?|versus|from)\s+(.+?)(?:\?|$)")
            .expect("valid syndrome comparison regex")
    })
}

fn syndrome_difference_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bdifference\s+between\s+(.+?)\s+and\s+(.+?)(?:\?|$)")
            .expect("valid syndrome difference regex")
    })
}

fn syndrome_vs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(.+?)\s+(?:vs\.?|versus)\s+(.+?)(?:\?|$)")
            .expect("valid syndrome-vs regex")
    })
}

fn syndrome_confused_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(.+?)\s+confused\s+with\s+(.+?)(?:\?|$)")
            .expect("valid syndrome confused-with regex")
    })
}

fn linked_terms_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:is|are|was|were|does|do)?\s*(.+?)\s+(?:linked|associated|caus(?:e|es|ed)|related)\s+(?:to|with)\s+(.+?)(?:\?|$)")
            .expect("valid linked terms regex")
    })
}

fn cause_terms_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:does|do)\s+(.+?)\s+caus(?:e|es)\s+(.+?)(?:\?|$)")
            .expect("valid cause terms regex")
    })
}

fn evidence_terms_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:any\s+|no\s+)?evidence\s+(?:for|against)\s+(.+?)\s+(?:and|in|with)\s+(.+?)(?:\?|$)")
            .expect("valid evidence terms regex")
    })
}

fn mapped_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:is|for|of)\s+(.+?)\s+(?:mapped|located|on chromosome|at locus)")
            .expect("valid mapped disease regex")
    })
}

fn mechanism_topic_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:pathway|mechanism|signaling)\s+(?:explains?|for|of|in)\s+(.+?)(?:\?|$)",
        )
        .expect("valid mechanism topic regex")
    })
}

fn mechanism_resistance_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\bexplains?\s+(.+?)\s+resistance\b|(?:.+?\s+)?resistance\s+(?:to|against)\s+(.+?)(?:\?|$)|\b([A-Za-z0-9][A-Za-z0-9 -]{1,80}?)\s+resistance\b",
        )
        .expect("valid mechanism resistance regex")
    })
}

fn mechanism_work_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bhow\s+(?:does|do)\s+(.+?)\s+work\b").expect("valid mechanism work regex")
    })
}

fn mechanism_of_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:mechanism|pathway|signaling|targets?)\s+(?:of|for)\s+(.+?)(?:\?|$)")
            .expect("valid mechanism-of regex")
    })
}

fn treatment_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:drugs?|medications?|therapies|treatments?)\s+(?:treat|for|against)\s+(.+?)(?:\?|$)|\btreat(?:s|ment)?\s+(?:for\s+)?(.+?)(?:\?|$)")
            .expect("valid treatment disease regex")
    })
}

fn symptom_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:symptoms?|phenotypes?|features?|signs)\b.*?\b(?:in|of|for|with)\s+(.+?)(?:\?|$)")
            .expect("valid symptom disease regex")
    })
}

fn symptom_named_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:in|of|for|with)\s+(.+?\b(?:disease|syndrome|cancer))\b")
            .expect("valid symptom named disease regex")
    })
}

fn symptom_text_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:symptoms?|phenotypes?|clinical features?|features?|signs)\s+(?:include|like|such as|with|are)?\s*(.+?)(?:\?|$)")
            .expect("valid symptom text regex")
    })
}

fn gene_disease_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:in|with|for)\s+(.+?)(?:\?|$)").expect("valid gene disease regex")
    })
}

const ANCHOR_PREFIXES: &[&str] = &[
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

const STOP_ANCHORS: &[&str] = &[
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

const GENE_STOPWORDS: &[&str] = &[
    "AND", "ARE", "DNA", "DOI", "EMA", "FDA", "HGVS", "MCP", "OR", "PMC", "PMID", "RNA", "WHO",
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use clap::Parser;

    use super::*;
    use crate::cli::Cli;

    fn parse_cmd(cmd: &str) {
        let args = shlex::split(cmd).unwrap_or_else(|| panic!("shlex failed on: {cmd}"));
        Cli::try_parse_from(args).unwrap_or_else(|err| panic!("failed to parse {cmd}: {err}"));
    }

    #[test]
    fn route_examples_cover_shipped_skill_slugs() {
        let shipped = crate::cli::skill::list_use_case_refs()
            .expect("skills")
            .into_iter()
            .map(|case| case.slug.to_string())
            .collect::<BTreeSet<_>>();
        let routed = ROUTES
            .iter()
            .map(|route| route.slug.to_string())
            .collect::<BTreeSet<_>>();

        assert_eq!(routed, shipped);
    }

    #[test]
    fn route_examples_match_expected_skills_commands_and_parse() {
        for example in ROUTE_EXAMPLES {
            let response = suggest_question(example.question);
            assert_eq!(
                response.matched_skill.as_deref(),
                Some(example.expected_skill),
                "{}",
                example.question
            );
            assert_eq!(response.first_commands.len(), 2, "{}", example.question);
            assert_eq!(
                response.full_skill.as_deref(),
                Some(format!("biomcp skill {}", example.expected_skill).as_str())
            );
            assert_eq!(response.first_commands, example.expected_commands);
            for command in response.first_commands {
                parse_cmd(&command);
            }
        }
    }

    #[test]
    fn ticket_examples_keep_exact_commands_and_response_shape() {
        let treatment = suggest_question("What drugs treat melanoma?");
        assert_eq!(
            treatment,
            SuggestResponse {
                matched_skill: Some("treatment-lookup".to_string()),
                summary:
                    "Use the treatment lookup playbook for therapy or approved-drug questions."
                        .to_string(),
                first_commands: vec![
                    "biomcp search drug --indication melanoma --limit 5".to_string(),
                    "biomcp search article -d melanoma --type review --limit 5".to_string(),
                ],
                full_skill: Some("biomcp skill treatment-lookup".to_string()),
            }
        );

        let variant = suggest_question("Is variant rs113488022 pathogenic in melanoma?");
        assert_eq!(
            variant.first_commands,
            vec![
                "biomcp get variant rs113488022 clinvar predictions population",
                "biomcp get variant rs113488022 civic cgi",
            ]
        );

        let json = crate::render::json::to_pretty(&variant).expect("json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let keys = value
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            keys,
            BTreeSet::from([
                "first_commands".to_string(),
                "full_skill".to_string(),
                "matched_skill".to_string(),
                "summary".to_string(),
            ])
        );
    }

    #[test]
    fn no_match_is_successful_with_null_json_fields() {
        let response = suggest_question("What is x?");
        assert_eq!(
            response,
            SuggestResponse {
                matched_skill: None,
                summary: "No confident BioMCP skill match.".to_string(),
                first_commands: vec![],
                full_skill: None,
            }
        );

        let json = crate::render::json::to_pretty(&response).expect("json");
        assert!(json.contains("\"matched_skill\": null"));
        assert!(json.contains("\"full_skill\": null"));
        assert!(json.contains("\"first_commands\": []"));
    }

    #[test]
    fn markdown_exposes_labels_and_no_match_guidance() {
        let matched = render_markdown(&suggest_question("What drugs treat melanoma?"));
        assert!(matched.contains("# BioMCP Suggestion"));
        assert!(matched.contains("- matched_skill: `treatment-lookup`"));
        assert!(matched.contains("- summary: "));
        assert!(matched.contains("- first_commands:"));
        assert!(matched.contains("- full_skill: `biomcp skill treatment-lookup`"));

        let no_match = render_markdown(&suggest_question("What is x?"));
        assert!(no_match.contains("- matched_skill: no match"));
        assert!(no_match.contains("No confident BioMCP skill match."));
        assert!(no_match.contains("biomcp skill list"));
        assert!(no_match.contains("biomcp discover \"<question>\""));
    }

    #[test]
    fn guardrails_avoid_common_false_positives() {
        assert_eq!(suggest_question("What is x?").matched_skill, None);
        assert_eq!(
            suggest_question("Which gene is responsible for disease?").matched_skill,
            None
        );
        assert_ne!(
            suggest_question("Find article evidence from 2024 about melanoma")
                .matched_skill
                .as_deref(),
            Some("article-follow-up")
        );
        assert_eq!(
            suggest_question("Tell me about variant rs113488022").matched_skill,
            None
        );
        assert_eq!(
            suggest_question("What does gene brca1 do?")
                .matched_skill
                .as_deref(),
            Some("gene-function-localization")
        );
    }

    #[test]
    fn generated_commands_quote_user_derived_multiword_and_shell_metacharacter_anchors() {
        let response = suggest_question("What drugs treat lung cancer?");
        assert_eq!(response.matched_skill.as_deref(), Some("treatment-lookup"));
        assert!(response.first_commands[0].contains("\"lung cancer\""));
        assert!(response.first_commands[1].contains("\"lung cancer\""));

        let response = suggest_question("What drugs treat lung cancer; rm -rf /?");
        assert_eq!(response.matched_skill.as_deref(), Some("treatment-lookup"));
        assert!(response.first_commands[0].contains("\"lung cancer; rm -rf /\""));
        parse_cmd(&response.first_commands[0]);
    }

    #[test]
    fn mechanism_resistance_to_drug_prefers_drug_anchor() {
        let response = suggest_question("What is the mechanism of resistance to imatinib?");
        assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
        assert_eq!(
            response.first_commands,
            vec![
                concat!("biomcp search drug ", "imatinib --limit 5").to_string(),
                concat!("biomcp get drug ", "imatinib targets regulatory").to_string(),
            ]
        );
    }

    #[test]
    fn mechanism_resistance_against_prefers_drug_anchor() {
        let response = suggest_question("What is the mechanism of resistance against imatinib?");
        assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
        assert_eq!(
            response.first_commands,
            vec![
                concat!("biomcp search drug ", "imatinib --limit 5").to_string(),
                concat!("biomcp get drug ", "imatinib targets regulatory").to_string(),
            ]
        );
    }

    #[test]
    fn mechanism_drug_branch_handles_imatinib_resistance_develop() {
        let response = suggest_question("How does imatinib resistance develop?");
        assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
        assert_eq!(
            response.first_commands,
            vec![
                concat!("biomcp search drug ", "imatinib --limit 5").to_string(),
                concat!("biomcp get drug ", "imatinib targets regulatory").to_string(),
            ]
        );
    }

    #[test]
    fn mechanism_resistance_to_drug_quotes_shell_sensitive_anchor() {
        let response =
            suggest_question("What is the mechanism of resistance to imatinib; rm -rf /?");
        assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
        assert_eq!(
            response.first_commands,
            vec![
                format!(
                    "biomcp search drug {} --limit 5",
                    quote("imatinib; rm -rf /")
                ),
                format!(
                    "biomcp get drug {} targets regulatory",
                    quote("imatinib; rm -rf /")
                ),
            ]
        );
        parse_cmd(&response.first_commands[0]);
    }

    #[test]
    fn route_specific_contract_edges_match_design() {
        let regulatory = suggest_question("When was imatinib approved by FDA?");
        assert_eq!(
            regulatory.first_commands[0],
            "biomcp get drug imatinib regulatory --region us"
        );

        let intervention = suggest_question("Are there recruiting trials with imatinib?");
        assert_eq!(
            intervention.first_commands,
            [
                "biomcp search trial -i imatinib --status recruiting --limit 5",
                "biomcp search article --drug imatinib --type review --limit 5",
            ]
        );

        let symptom = suggest_question("symptoms include seizure and developmental delay");
        assert_eq!(
            symptom.first_commands,
            [
                "biomcp discover \"seizure and developmental delay\"",
                "biomcp search phenotype \"seizure and developmental delay\" --limit 5",
            ]
        );

        let bare_vs =
            suggest_question("Goldberg-Shprintzen syndrome vs Shprintzen-Goldberg syndrome");
        assert_eq!(
            bare_vs.matched_skill.as_deref(),
            Some("syndrome-disambiguation")
        );

        let difference = suggest_question(
            "What is the difference between Goldberg-Shprintzen syndrome and Shprintzen-Goldberg syndrome?",
        );
        assert_eq!(
            difference.matched_skill.as_deref(),
            Some("syndrome-disambiguation")
        );

        let no_evidence = suggest_question("No evidence for aspirin and melanoma?");
        assert_eq!(
            no_evidence.matched_skill.as_deref(),
            Some("negative-evidence")
        );
        assert_eq!(
            no_evidence.first_commands,
            [
                "biomcp search article -k \"aspirin melanoma\" --type review --limit 5",
                "biomcp search article -k \"aspirin melanoma association\" --limit 5",
            ]
        );
    }
}
