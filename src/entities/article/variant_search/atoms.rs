//! Checked vocabulary for the private pool format's borrowed runtime labels.
use serde::{Deserialize, Deserializer};

// The alias prevents serde from implicitly borrowing input for runtime labels.
pub(crate) type Atom = &'static str;

fn checked<E: serde::de::Error>(value: String) -> Result<&'static str, E> {
    use super::super::identity_verification::{
        CONTENT_SUBSET_VERSION, PAPER_TRANSCRIPT_ANCHOR_BASIS,
        PUBTATOR_EXPORT_FULLTEXT_TEMPLATE_VERSION, RESPONSE_SUBSET_VERSION, VERIFIER_VERSION,
    };
    const ATOMS: &[&str] = &[
        VERIFIER_VERSION,
        PAPER_TRANSCRIPT_ANCHOR_BASIS,
        PUBTATOR_EXPORT_FULLTEXT_TEMPLATE_VERSION,
        RESPONSE_SUBSET_VERSION,
        CONTENT_SUBSET_VERSION,
        "incomplete",
        "citation_count",
        "http_status",
        "decode",
        "local_io",
        "internal",
        "invalid_argument",
        "connect",
        "network",
        "tls",
        "body_limit",
        "absent",
        "abstract",
        super::UNSTATED_TRANSCRIPT_REASON,
        super::STATED_TRANSCRIPT_REASON,
        super::GENE_ALIAS_DISABLED_REASON,
        super::GENE_ALIAS_BUDGET_REASON,
        super::GENE_ALIAS_ABSENT_REASON,
        super::GENE_ALIAS_UNAVAILABLE_REASON,
        "pubmed_query_length_limit",
        "europepmc_query_length_limit",
        "connection",
        "configuration",
        "alias",
        "alias_limit",
        "annotation",
        "annotation-v1",
        "attempted",
        "available",
        "best_effort_free_text",
        "canonical_equivalence",
        "captured_abstract",
        "civic",
        "clingen_car",
        "clingen_ldh",
        "clingen_ldh_direct",
        "clingen_ldh_medium",
        "confirmed",
        "conflicting",
        "contradictory",
        "deadline_exceeded",
        "degraded",
        "disabled",
        "discovery",
        "enrichment",
        "europepmc",
        "exact",
        "exact_lexical",
        "exact_name",
        "exact_variant",
        "federated",
        "filtered_confirmed_only",
        "fulltext",
        "gene_aliases",
        "identity_inapplicable",
        "identity_verification",
        "inapplicable",
        "indeterminate",
        "invalid_typed_linkage",
        "invocation",
        "invocation_deadline",
        "ldh-medium-direct-v1",
        "litvar",
        "logical_work_cap",
        "metadata",
        "missing_typed_gene_annotation",
        "mygene-info",
        "myvariant",
        "no_provider_query_available",
        "no_variant_entity",
        "none",
        "not_applicable",
        "not_attempted",
        "not_found",
        "not_requested",
        "not_visible",
        "ok",
        "omitted",
        "provider_error",
        "provider_identity_contradiction",
        "provider_query_attempted",
        "provider_timeout",
        "pubmed",
        "pubtator",
        "pubtator3",
        "pubtator_alias_limit",
        "query_result_cap",
        "rate_limited",
        "received",
        "resolution",
        "resolution_unavailable",
        "retry_attempted",
        "semantic-scholar",
        "semanticscholar",
        "semanticscholar_alias_limit",
        "single_identity",
        "source_citation",
        "source_disabled",
        "source_unavailable",
        "stable_identifier",
        "strategy_does_not_query_alias",
        "strict",
        "structured_annotation",
        "timed_out",
        "timeout",
        "unavailable",
        "unknown",
        "unverified",
        "unverified_alias_match",
        "visible",
        "variant-article-candidate-trace-v2",
    ];
    ATOMS
        .iter()
        .copied()
        .chain(std::iter::once(super::rank::METHOD))
        .chain(super::rank::INPUTS)
        .chain([
            super::variant_article_retrieval_path(super::VariantArticleStrategy::Exact, true),
            super::variant_article_retrieval_path(super::VariantArticleStrategy::Union, true),
            super::variant_article_retrieval_path(super::VariantArticleStrategy::Lexical, true),
            super::VARIANT_ENTITY_RETRIEVAL_PATH,
            super::VARIANT_FALLBACK_RETRIEVAL_PATH,
        ])
        .chain([400, 401, 403, 404, 429, 502, 503, 500].map(|status| {
            crate::sources::literature_retry::response_class(
                reqwest::StatusCode::from_u16(status).expect("valid status"),
            )
        }))
        .find(|atom| *atom == value)
        .ok_or_else(|| E::custom("unknown label in article pool"))
}

pub(crate) fn atom<'de, D: Deserializer<'de>>(de: D) -> Result<&'static str, D::Error> {
    checked(String::deserialize(de)?)
}
pub(crate) fn optional<'de, D: Deserializer<'de>>(de: D) -> Result<Option<&'static str>, D::Error> {
    Option::<String>::deserialize(de)?.map(checked).transpose()
}
pub(crate) fn list<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<&'static str>, D::Error> {
    Vec::<String>::deserialize(de)?
        .into_iter()
        .map(checked)
        .collect()
}
