//! Pure positional-query resolution for `search variant`.
//!
//! Ticket 1301 keeps the parser chain in one module: the exact forms parse
//! first, a gene-symbol-shaped first token becomes an oracle-validated
//! candidate, and everything else falls through to a condition search.

use super::ResolvedVariantQuery;
use super::VariantSearchPlan;
use crate::cli::normalize_cli_query;

pub(super) fn parse_simple_gene_change(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }

    let gene = parts[0].trim();
    let change = parts[1]
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.");
    if gene.is_empty() || change.is_empty() {
        return None;
    }

    let candidate = format!("{gene} {change}");
    match crate::entities::variant::parse_variant_id(&candidate).ok()? {
        crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change } => {
            Some((gene, change))
        }
        _ => None,
    }
}

pub(super) fn parse_gene_c_hgvs(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }

    let gene = parts[0].trim();
    let change = parts[1].trim();
    if gene.is_empty() || change.is_empty() || !crate::sources::is_valid_gene_symbol(gene) {
        return None;
    }
    if !change.starts_with("c.") && !change.starts_with("C.") {
        return None;
    }
    Some((gene.to_string(), format!("c.{}", change[2..].trim())))
}

pub(super) fn parse_exon_deletion_phrase(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }

    let gene = parts[0].trim();
    if !crate::sources::is_valid_gene_symbol(gene)
        || !parts[1].eq_ignore_ascii_case("exon")
        || parts[2].parse::<u32>().ok().is_none()
        || !parts[3].eq_ignore_ascii_case("deletion")
    {
        return None;
    }

    Some((gene.to_string(), "inframe_deletion".to_string()))
}

/// Split a free-text variant query whose first token has the exact-form
/// gene-token shape into a gene-first candidate (ticket 1301).
///
/// The shape alone does not make the token a gene: `is_exact_gene_token`
/// accepts any uppercase word, so the caller must confirm the token against
/// the gene-symbol oracle before routing anything.
pub(super) fn split_gene_first_candidate(query: &str) -> Option<(String, String)> {
    let mut tokens = query.split_whitespace();
    let gene = tokens.next()?;
    let remainder = tokens.collect::<Vec<_>>().join(" ");
    if remainder.is_empty() || !crate::entities::variant::is_exact_gene_token(gene) {
        return None;
    }
    Some((gene.to_string(), remainder))
}

/// Split a protein change off the front of a gene-first remainder
/// (ticket 2022), so `BRAF V600E melanoma` reads as gene=BRAF,
/// hgvsp=V600E, condition=melanoma instead of condition=`V600E melanoma`.
/// A remainder that is only the protein change never reaches this split;
/// the exact "GENE CHANGE" form parses it earlier in the chain.
pub(super) fn split_leading_protein_change(remainder: &str) -> Option<(String, String)> {
    let mut tokens = remainder.split_whitespace();
    let change = tokens.next()?;
    let change = crate::entities::variant::normalize_protein_change(change)?;
    let condition = tokens.collect::<Vec<_>>().join(" ");
    (!condition.is_empty()).then_some((change, condition))
}

/// What a zero-row gene-first search prints (tickets 1301 and 2022).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GeneFirstNote {
    /// The oracle refused the first token: the whole phrase ran as a
    /// condition, so the hint names the condition search without the
    /// leading word, the form the phrase search never tried (the gene
    /// filter needs the official symbol the oracle did not confirm).
    Refused { gene: String, condition: String },
    /// The oracle confirmed the official symbol: the phrase was routed, so
    /// the hint states the parsed form and the same filters with the
    /// condition dropped, which differs from the command that ran.
    Routed { parsed: String, alternative: String },
}

/// The explicit filter flags a gene-first phrase keeps, stated in the
/// command line's own spelling (`--significance benign`, `--max-frequency
/// 0.01`, ...) so a zero-row hint repeats exactly what the caller typed
/// (ticket 2022's promise; ticket 2038 finding 7 restored it). Used behind
/// a reference, like a slice.
pub(super) type ExplicitFilterFlags = [(&'static str, String)];

/// Apply the gene-symbol oracle verdict to a gene-first candidate (tickets
/// 1301 and 2022).
///
/// A confirmed official symbol routes the first token to the gene filter, a
/// leading protein change to the hgvsp filter, and the rest to the
/// condition. Refusal, ambiguity, and preference `off` all keep today's
/// whole-phrase condition search and remember the phrase so a zero-row
/// result can print the condition search without the leading word. Both
/// branches attach the leftover `--hgvsp` and `--consequence` flags exactly
/// as the whole-phrase fallthrough did, and `explicit_filters` carries every
/// other explicit flag into the hints, so no explicit filter is dropped.
pub(super) fn apply_gene_first_routing(
    gene: String,
    protein_change: Option<String>,
    condition: String,
    confirmed_symbol: Option<String>,
    hgvsp_flag: Option<String>,
    consequence_flag: Option<String>,
    explicit_filters: &ExplicitFilterFlags,
) -> (ResolvedVariantQuery, Option<GeneFirstNote>) {
    match confirmed_symbol {
        Some(symbol) => {
            let parsed = gene_first_parsed_form(&symbol, protein_change.as_deref(), &condition);
            let hgvsp = protein_change.or(hgvsp_flag);
            // The alternative keeps every filter the routed search applied
            // except the condition, parsed and explicit flags included, so
            // it differs from the command that ran by exactly the dropped
            // condition (ticket 2033, finding 13; ticket 2038 finding 7).
            let alternative = gene_first_alternative_form(
                &symbol,
                hgvsp.as_deref(),
                consequence_flag.as_deref(),
                explicit_filters,
            );
            (
                VariantSearchPlan::finalize(ResolvedVariantQuery {
                    gene: Some(symbol),
                    hgvsp,
                    consequence: consequence_flag,
                    condition: Some(condition),
                    ..Default::default()
                }),
                Some(GeneFirstNote::Routed {
                    parsed,
                    alternative,
                }),
            )
        }
        None => {
            // Refusal keeps the whole phrase, protein change included, so
            // the condition search and the working form see the original
            // remainder.
            let remainder = match protein_change.as_deref() {
                Some(change) => format!("{change} {condition}"),
                None => condition.clone(),
            };
            (
                VariantSearchPlan::finalize(ResolvedVariantQuery {
                    hgvsp: hgvsp_flag,
                    consequence: consequence_flag,
                    condition: Some(format!("{gene} {remainder}")),
                    ..Default::default()
                }),
                Some(GeneFirstNote::Refused {
                    gene,
                    condition: remainder,
                }),
            )
        }
    }
}

/// The filters a routed phrase parsed into, stated in the query summary's
/// spelling so the hint and the query line agree.
pub(super) fn gene_first_parsed_form(gene: &str, hgvsp: Option<&str>, condition: &str) -> String {
    let mut parts = vec![format!("gene={gene}")];
    if let Some(hgvsp) = hgvsp {
        parts.push(format!("hgvsp={hgvsp}"));
    }
    parts.push(format!("condition={condition}"));
    parts.join(", ")
}

/// The explicit form a refused gene-first phrase suggests: the phrase as a
/// condition without the leading word. The search that returned nothing ran
/// the whole phrase as the condition, and the gene filter needs an official
/// symbol the oracle did not confirm, so the condition alone is the looser
/// form that can still return rows (ticket 2038, following 2022 and 2035's
/// smaller items).
pub(super) fn gene_first_working_form(
    condition: &str,
    explicit_filters: &ExplicitFilterFlags,
) -> String {
    let command = crate::next_command::NextCommand::biomcp().args([
        "search",
        "variant",
        "--condition",
        condition,
    ]);
    append_explicit_filters(command, explicit_filters).render_shell()
}

/// Append the caller's explicit filter flags to a hint command, in flag
/// spelling, so the hint repeats every filter the search applied. An empty
/// value renders the flag alone (`--lof`).
fn append_explicit_filters(
    command: crate::next_command::NextCommand,
    explicit_filters: &ExplicitFilterFlags,
) -> crate::next_command::NextCommand {
    let mut command = command;
    for (flag, value) in explicit_filters {
        if value.is_empty() {
            command = command.arg(*flag);
        } else {
            command = command.args([*flag, value.as_str()]);
        }
    }
    command
}

/// The alternative a routed zero-row search suggests: every filter the
/// routed search applied, with the condition dropped. The routed search
/// already applied every parsed filter, so repeating them repeats the empty
/// result; the resolved `--hgvsp` filter and any explicit `--consequence`
/// flag stay beside the caller's other explicit filters, so the command
/// differs by exactly the dropped condition (tickets 2022, 2033 and 2038
/// finding 7).
pub(super) fn gene_first_alternative_form(
    gene: &str,
    hgvsp: Option<&str>,
    consequence: Option<&str>,
    explicit_filters: &ExplicitFilterFlags,
) -> String {
    let mut command =
        crate::next_command::NextCommand::biomcp().args(["search", "variant", "-g", gene]);
    if let Some(hgvsp) = hgvsp {
        command = command.args(["--hgvsp", hgvsp]);
    }
    if let Some(consequence) = consequence {
        command = command.args(["--consequence", consequence]);
    }
    append_explicit_filters(command, explicit_filters).render_shell()
}

const VARIANT_QUERY_GENE_ROUTING_ENV: &str = "BIOMCP_VARIANT_QUERY_GENE_ROUTING";
const GENE_FIRST_ROUTING_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2_500);

/// How a free-text variant query's gene-symbol first token is recognized.
///
/// `mygene` is the supported default because no offline gene list ships with
/// BioMCP and MyGene's unique entrez-backed official-symbol confirmation is
/// the lookup this routing uses; unlike the `discover` path's symbol/alias
/// lookup, aliases never route (ticket 2022). `off` restores the
/// whole-phrase condition search for operators who must not spend a MyGene
/// call on routing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum VariantQueryGeneRouting {
    #[default]
    Mygene,
    Off,
}

impl VariantQueryGeneRouting {
    fn from_env() -> Self {
        Self::from_env_value(
            std::env::var(VARIANT_QUERY_GENE_ROUTING_ENV)
                .ok()
                .as_deref(),
        )
    }

    pub(super) fn from_env_value(value: Option<&str>) -> Self {
        let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
            return Self::Mygene;
        };
        match value.to_ascii_lowercase().as_str() {
            "mygene" => Self::Mygene,
            "off" => Self::Off,
            _ => {
                tracing::warn!("Unknown {VARIANT_QUERY_GENE_ROUTING_ENV}={value:?}, using mygene");
                Self::Mygene
            }
        }
    }
}

/// Confirm a free-text first token is a known gene symbol (ticket 1301).
/// Ticket 2022 narrows the oracle to official symbols: MyGene's unique
/// entrez-backed resolution must return the query itself as the official
/// symbol, so aliases such as HCC (HYCC1), MODY (HNF4A), or HHT (ACVRL1)
/// no longer route. Refusal, ambiguity, timeout, and MyGene outages all
/// return `None`.
pub(super) async fn confirm_gene_first_candidate(gene: &str) -> Option<String> {
    if VariantQueryGeneRouting::from_env() == VariantQueryGeneRouting::Off {
        return None;
    }
    match tokio::time::timeout(
        GENE_FIRST_ROUTING_TIMEOUT,
        crate::entities::gene::resolve_unique_official_symbol(gene),
    )
    .await
    {
        Ok(Ok(Some(official))) => Some(official.symbol),
        _ => None,
    }
}

pub(super) fn resolve_variant_query(
    gene_flag: Option<String>,
    hgvsp_flag: Option<String>,
    consequence_flag: Option<String>,
    condition_flag: Option<String>,
    positional_tokens: Vec<String>,
) -> Result<VariantSearchPlan, crate::error::BioMcpError> {
    let gene_flag = normalize_cli_query(gene_flag);
    let hgvsp_flag = normalize_cli_query(hgvsp_flag);
    let consequence_flag = consequence_flag.map(|value| value.trim().to_string());
    let condition_flag = normalize_cli_query(condition_flag);

    let positional = positional_tokens
        .iter()
        .map(|token| token.trim())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let positional = normalize_cli_query(Some(positional));

    let Some(query) = positional else {
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: gene_flag,
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    };

    let token_count = query.split_whitespace().count();
    if token_count <= 1 {
        if let Ok(crate::entities::variant::VariantIdFormat::RsId(rsid)) =
            crate::entities::variant::parse_variant_id(&query)
        {
            if gene_flag.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "Use either positional QUERY or --gene, not both".into(),
                ));
            }
            return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
                rsid: Some(rsid),
                hgvsp: hgvsp_flag,
                consequence: consequence_flag,
                condition: condition_flag,
                ..Default::default()
            }));
        }

        if let Some(gene) = gene_flag.clone() {
            if let Some(protein_alias) =
                crate::entities::variant::parse_variant_protein_alias(&query)
            {
                if hgvsp_flag.is_some() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Positional residue alias conflicts with --hgvsp".into(),
                    ));
                }
                return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
                    gene: Some(gene),
                    protein_alias: Some(protein_alias),
                    consequence: consequence_flag,
                    condition: condition_flag,
                    ..Default::default()
                }));
            }
            if let crate::entities::variant::VariantInputKind::Shorthand(
                crate::entities::variant::VariantShorthand::ProteinChangeOnly { .. },
            ) = crate::entities::variant::classify_variant_input(&query)
            {
                if hgvsp_flag.is_some() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Positional protein change conflicts with --hgvsp".into(),
                    ));
                }
                return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
                    gene: Some(gene),
                    hgvsp: Some(query.clone()),
                    consequence: consequence_flag,
                    condition: condition_flag,
                    ..Default::default()
                }));
            }
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Use either positional QUERY or --gene, not both".into(),
            ));
        }

        if let Some(guidance) = crate::entities::variant::variant_guidance(&query) {
            return Ok(VariantSearchPlan::Guidance(guidance));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(query),
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, change)) = parse_simple_gene_change(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE CHANGE\" conflicts with --gene".into(),
            ));
        }
        if hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE CHANGE\" conflicts with --hgvsp".into(),
            ));
        }
        let supplied_change = query
            .split_whitespace()
            .nth(1)
            .unwrap_or(&change)
            .to_string();
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: Some(supplied_change),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let crate::entities::variant::VariantInputKind::Shorthand(
        crate::entities::variant::VariantShorthand::GeneResidueAlias {
            gene,
            position,
            residue,
            ..
        },
    ) = crate::entities::variant::classify_variant_input(&query)
    {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional residue alias conflicts with --gene".into(),
            ));
        }
        if hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional residue alias conflicts with --hgvsp".into(),
            ));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            protein_alias: Some(crate::entities::variant::VariantProteinAlias {
                position,
                residue,
            }),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, hgvsc)) = parse_gene_c_hgvs(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE c.HGVS\" conflicts with --gene".into(),
            ));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: hgvsp_flag,
            hgvsc: Some(hgvsc),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, consequence)) = parse_exon_deletion_phrase(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional exon-deletion query conflicts with --gene".into(),
            ));
        }
        if consequence_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional exon-deletion query conflicts with --consequence".into(),
            ));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: hgvsp_flag,
            consequence: Some(consequence),
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if condition_flag.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Use either positional QUERY or --condition, not both".into(),
        ));
    }
    if gene_flag.is_none()
        && let Some((gene, condition)) = split_gene_first_candidate(&query)
    {
        // A leading protein change belongs to the hgvsp filter, not the
        // condition (ticket 2022): 'BRAF V600E melanoma' must not become
        // condition='V600E melanoma'.
        let (protein_change, condition) = match split_leading_protein_change(&condition) {
            Some((change, rest)) => (Some(change), rest),
            None => (None, condition),
        };
        if protein_change.is_some() && hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional protein change conflicts with --hgvsp".into(),
            ));
        }
        return Ok(VariantSearchPlan::GeneFirstCandidate {
            gene,
            protein_change,
            condition,
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
        });
    }
    Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
        gene: gene_flag,
        hgvsp: hgvsp_flag,
        consequence: consequence_flag,
        condition: Some(query),
        ..Default::default()
    }))
}

pub(super) fn trim_protein_change_prefix(value: &str) -> &str {
    value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.")
}

pub(super) fn normalize_search_hgvsp(value: &str) -> String {
    let normalized = crate::entities::variant::normalize_protein_substitution(value)
        .unwrap_or_else(|| trim_protein_change_prefix(value).to_string());
    normalized
        .strip_suffix('*')
        .map(|prefix| format!("{prefix}X"))
        .unwrap_or(normalized)
}
