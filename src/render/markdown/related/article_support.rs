use super::*;

const ARTICLE_GENE_BLOCKLIST: &[&str] = &[
    "DNA", "RNA", "MRNA", "PCR", "MRI", "CT", "EEG", "ECG", "ATP", "ADP", "SNP", "UTR", "ORF",
    "CDS", "HIV", "HPV", "CMV", "HSV", "EBV", "HBV", "HCV",
];

const ARTICLE_DRUG_SUFFIXES: &[&str] = &[
    "mab", "nib", "vir", "pril", "sartan", "statin", "platin", "rubicin", "taxel", "azole",
    "dipine", "parin", "tide", "cept", "olol", "afil", "semide", "idone",
];
const ARTICLE_CROSS_ENTITY_MAX_TOKENS: usize = 3;
const ARTICLE_DISEASE_SUFFIXES: &[&str] = &[
    "cancer",
    "carcinoma",
    "deficiency",
    "disease",
    "disorder",
    "infection",
    "leukaemia",
    "leukemia",
    "lymphoma",
    "melanoma",
    "sarcoma",
    "syndrome",
    "tumor",
    "tumour",
];
const ARTICLE_DISCOVER_SUFFIXES: &[&str] = &["vaccine", "vaccines"];
const ARTICLE_QUESTION_PREFIXES: &[&str] = &[
    "how ", "what ", "when ", "where ", "which ", "who ", "why ", "list ", "show ", "find ",
    "give ", "tell ",
];

pub(super) fn exact_variant_article_follow_up(
    filters: &crate::entities::article::ArticleSearchFilters,
) -> Option<String> {
    let keyword = filters.keyword.as_deref()?.trim();
    if !matches!(
        crate::entities::variant::classify_variant_input(keyword),
        crate::entities::variant::VariantInputKind::Exact(
            crate::entities::variant::VariantIdFormat::GeneProteinChange { .. }
        )
    ) {
        return None;
    }
    Some(format!("biomcp variant articles {}", quote_arg(keyword)))
}

pub(super) fn article_annotation_command(
    bucket: ArticleAnnotationBucket,
    text: &str,
) -> Option<String> {
    let text = text.trim();
    let quoted = quote_arg(text);
    if quoted.is_empty() {
        return None;
    }

    Some(match bucket {
        ArticleAnnotationBucket::Gene => format!("biomcp search gene -q {quoted}"),
        ArticleAnnotationBucket::Disease => format!("biomcp search disease --query {quoted}"),
        ArticleAnnotationBucket::Chemical => format!("biomcp get drug {quoted}"),
        ArticleAnnotationBucket::Mutation => format!("biomcp get variant {quoted}"),
    })
}

pub(super) fn ranked_article_annotation_commands(
    title: &str,
    rows: &[AnnotationCount],
    bucket: ArticleAnnotationBucket,
    limit: usize,
) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }

    let normalized_title = normalize_match_text(title);
    let mut ranked = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let command = article_annotation_command(bucket, &row.text)?;
            let normalized_text = normalize_match_text(&row.text);
            let title_hit =
                !normalized_text.is_empty() && normalized_title.contains(normalized_text.as_str());
            Some((title_hit, row.count, index, command))
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.2.cmp(&b.2))
    });

    ranked
        .into_iter()
        .take(limit)
        .map(|(_, _, _, command)| command)
        .collect()
}

pub(super) fn trial_results_search_command(trial: &Trial) -> Option<String> {
    let nct_id = trial.nct_id.trim();
    if nct_id.is_empty() {
        return None;
    }

    let title_seed = trial
        .title
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    let seed = if title_seed.is_empty() {
        nct_id.to_string()
    } else {
        format!("{nct_id} {title_seed}")
    };
    let seed_q = format!("\"{}\"", seed.replace('\"', "\\\""));
    if seed_q.is_empty() {
        return None;
    }

    if let Some(intervention) = trial.interventions.first().map(String::as_str) {
        let intervention_q = quote_arg(intervention);
        if !intervention_q.is_empty() {
            return Some(format!(
                "biomcp search article --drug {intervention_q} -q {seed_q} --limit 5"
            ));
        }
    }

    Some(format!("biomcp search article -q {seed_q} --limit 5"))
}

pub(in crate::render::markdown) fn typed_article_identifier(
    pmid: Option<&str>,
    pmcid: Option<&str>,
    doi: Option<&str>,
    arxiv_id: Option<&str>,
    semantic_scholar_id: Option<&str>,
) -> Option<String> {
    fn available(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|value| !value.is_empty())
    }
    available(pmid)
        .map(|value| format!("PMID {value}"))
        .or_else(|| available(pmcid).map(|value| format!("PMCID {value}")))
        .or_else(|| available(doi).map(|value| format!("DOI {value}")))
        .or_else(|| available(arxiv_id).map(|value| format!("arXiv {value}")))
        .or_else(|| available(semantic_scholar_id).map(|value| format!("Semantic Scholar {value}")))
}

pub(super) fn article_related_id(paper: &ArticleRelatedPaper) -> String {
    typed_article_identifier(
        paper.pmid.as_deref(),
        None,
        paper.doi.as_deref(),
        paper.arxiv_id.as_deref(),
        paper.paper_id.as_deref(),
    )
    .map(|identifier| markdown_cell(&identifier))
    .unwrap_or_else(|| "-".to_string())
}

pub(super) fn article_related_label(paper: &ArticleRelatedPaper) -> String {
    typed_article_identifier(
        paper.pmid.as_deref(),
        None,
        paper.doi.as_deref(),
        paper.arxiv_id.as_deref(),
        paper.paper_id.as_deref(),
    )
    .unwrap_or_else(|| paper.title.clone())
}

fn article_keyword_tokens(keyword: &str) -> Vec<&str> {
    keyword
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .filter(|token| !token.is_empty())
        .collect()
}

fn first_article_gene_token<'a>(tokens: &'a [&'a str]) -> Option<(&'a str, usize)> {
    tokens.iter().enumerate().find_map(|(index, token)| {
        let upper = token.to_ascii_uppercase();
        (crate::entities::discover::looks_like_gene_symbol_token(token)
            && !ARTICLE_GENE_BLOCKLIST.contains(&upper.as_str()))
        .then_some((*token, index))
    })
}

fn first_article_drug_token(tokens: &[&str]) -> Option<String> {
    tokens.iter().find_map(|token| {
        let normalized = token.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return None;
        }

        (normalized.len() >= 5
            && ARTICLE_DRUG_SUFFIXES
                .iter()
                .any(|suffix| normalized.ends_with(suffix)))
        .then_some(normalized)
    })
}

fn article_keyword_cross_entity_eligible(keyword: &str, tokens: &[&str]) -> bool {
    let keyword = keyword.trim();
    !keyword.is_empty()
        && tokens.len() <= ARTICLE_CROSS_ENTITY_MAX_TOKENS
        && !keyword.contains('?')
        && !ARTICLE_QUESTION_PREFIXES
            .iter()
            .any(|prefix| keyword.to_ascii_lowercase().starts_with(prefix))
}

fn article_keyword_last_token(tokens: &[&str]) -> Option<String> {
    tokens
        .last()
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| !token.is_empty())
}

fn article_keyword_looks_like_disease_phrase(tokens: &[&str]) -> bool {
    article_keyword_last_token(tokens)
        .is_some_and(|token| ARTICLE_DISEASE_SUFFIXES.contains(&token.as_str()))
}

fn article_keyword_looks_like_discover_phrase(tokens: &[&str]) -> bool {
    article_keyword_looks_like_disease_phrase(tokens)
        || article_keyword_last_token(tokens)
            .is_some_and(|token| ARTICLE_DISCOVER_SUFFIXES.contains(&token.as_str()))
}

pub(super) fn article_keyword_cross_entity_markdown_hints(
    filters: &ArticleSearchFilters,
) -> Vec<String> {
    let keyword = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|keyword| !keyword.is_empty());
    let Some(keyword) = keyword else {
        return Vec::new();
    };

    let tokens = article_keyword_tokens(keyword);
    if tokens.is_empty() || !article_keyword_cross_entity_eligible(keyword, &tokens) {
        return Vec::new();
    }

    let has_gene_token = first_article_gene_token(&tokens).is_some();
    let has_drug_token = first_article_drug_token(&tokens).is_some();
    if has_gene_token || has_drug_token {
        return Vec::new();
    }

    let mut out = Vec::new();
    if filters.disease.is_none() && article_keyword_looks_like_discover_phrase(&tokens) {
        out.push(format!("biomcp discover {}", shell_quote_arg(keyword)));
    }

    dedupe_markdown_commands(out)
}

fn article_result_year(result: &ArticleSearchResult) -> Option<u16> {
    let year = result.date.as_deref()?.get(..4)?;
    year.chars()
        .all(|ch| ch.is_ascii_digit())
        .then(|| year.parse().ok())?
}

fn push_article_filter_arg(args: &mut Vec<String>, flag: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        args.push(format!("{flag} {}", shell_quote_arg(value)));
    }
}

fn article_year_refinement_command(
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    min_year: u16,
    max_year: u16,
) -> Option<String> {
    let mut args = vec!["biomcp search article".to_string()];
    push_article_filter_arg(&mut args, "-g", filters.gene.as_deref());
    push_article_filter_arg(&mut args, "-d", filters.disease.as_deref());
    push_article_filter_arg(&mut args, "--drug", filters.drug.as_deref());
    push_article_filter_arg(&mut args, "-a", filters.author.as_deref());
    if let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|keyword| !keyword.is_empty())
    {
        args.push(format!("-k {}", shell_quote_arg(keyword)));
    }
    push_article_filter_arg(&mut args, "--journal", filters.journal.as_deref());
    push_article_filter_arg(&mut args, "--type", filters.article_type.as_deref());
    if source_filter != crate::entities::article::ArticleSourceFilter::All {
        args.push(format!("--source {}", source_filter.as_str()));
    }
    if filters.open_access {
        args.push("--open-access".to_string());
    }
    if !filters.exclude_retracted {
        args.push("--include-retracted".to_string());
    }
    args.push(format!("--year-min {min_year:04}"));
    args.push(format!("--year-max {max_year:04}"));
    args.push("--limit 5".to_string());
    (args.len() > 3).then(|| args.join(" "))
}

pub(super) fn article_date_refinement_hint(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
) -> Vec<String> {
    if results.is_empty() || filters.date_from.is_some() || filters.date_to.is_some() {
        return Vec::new();
    }
    let mut years = results.iter().filter_map(article_result_year);
    let Some(first_year) = years.next() else {
        return Vec::new();
    };
    let (min_year, max_year) = years
        .fold((first_year, first_year), |(min_year, max_year), year| {
            (min_year.min(year), max_year.max(year))
        });
    article_year_refinement_command(filters, source_filter, min_year, max_year)
        .into_iter()
        .collect()
}

pub(super) fn related_article(article: &Article) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(pmid) = article
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        && article.annotations.is_some()
    {
        out.push(format!("biomcp article entities {pmid}"));
    }
    if let Some(ann) = article.annotations.as_ref() {
        out.extend(ranked_article_annotation_commands(
            &article.title,
            &ann.genes,
            ArticleAnnotationBucket::Gene,
            2,
        ));
        out.extend(ranked_article_annotation_commands(
            &article.title,
            &ann.diseases,
            ArticleAnnotationBucket::Disease,
            2,
        ));
        out.extend(ranked_article_annotation_commands(
            &article.title,
            &ann.chemicals,
            ArticleAnnotationBucket::Chemical,
            2,
        ));
    }
    if let Some(pmid) = article
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push(format!("biomcp article references {pmid} --limit 3"));
        out.push(format!("biomcp article citations {pmid} --limit 3"));
        out.push(format!("biomcp article recommendations {pmid} --limit 3"));
    }
    dedupe_markdown_commands(out)
}
