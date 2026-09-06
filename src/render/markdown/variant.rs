//! Variant, phenotype, and GWAS markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn variant_markdown(
    variant: &Variant,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("variant.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_prediction_section = !section_only || include_all || has_requested("predict");
    let show_predictions_section = include_all || has_requested("predictions");
    let show_clinvar_section = !section_only || include_all || has_requested("clinvar");
    let show_population_section = !section_only
        || include_all
        || has_requested("population")
        || has_requested("population-details");
    let show_population_details = has_requested("population-details");
    let exome_highest_ancestry = variant
        .population
        .as_ref()
        .and_then(|population| population.exome.as_ref())
        .and_then(highest_ancestry_frequency);
    let genome_highest_ancestry = variant
        .population
        .as_ref()
        .and_then(|population| population.genome.as_ref())
        .and_then(highest_ancestry_frequency);
    let show_conservation_section = include_all || has_requested("conservation");
    let show_cosmic_section = include_all || has_requested("cosmic");
    let show_cgi_section = include_all || has_requested("cgi");
    let show_civic_section = include_all || has_requested("civic");
    let show_cancerhotspots_section = include_all && variant.cancerhotspots.is_some();
    let show_cbioportal_section = include_all || has_requested("cbioportal");
    let show_gwas_section = include_all || has_requested("gwas");
    let variant_label = if !variant.gene.trim().is_empty() && variant.hgvs_p.is_some() {
        format!(
            "{} {}",
            variant.gene.trim(),
            variant.hgvs_p.as_deref().unwrap_or_default().trim()
        )
    } else if !variant.gene.trim().is_empty() {
        variant.gene.trim().to_string()
    } else {
        variant.id.trim().to_string()
    };
    let prediction = variant.prediction.as_ref();
    let (expr_i, splice_i, chrom_i) = prediction
        .map(prediction_interpretations)
        .unwrap_or((None, None, None));
    let civic_actionability_pointer = civic_actionability_pointer(variant);
    let follow_up_id = preferred_variant_follow_up_id(variant);
    let variant_command_arg = quote_arg(follow_up_id);
    let gene_command_arg = quote_arg(&variant.gene);
    let genome_build_provider_default = variant
        .genome_build_provenance
        .as_deref()
        .is_some_and(|value| value.contains("provider default"));
    let exome_filters = variant
        .population
        .as_ref()
        .and_then(|population| population.exome.as_ref())
        .map(expanded_gnomad_filters)
        .unwrap_or_default();
    let genome_filters = variant
        .population
        .as_ref()
        .and_then(|population| population.genome.as_ref())
        .map(expanded_gnomad_filters)
        .unwrap_or_default();
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&variant_label, requested_sections),
        id => &variant.id,
        genome_build => &variant.genome_build,
        genome_build_provider_default => genome_build_provider_default,
        build_ambiguous => &variant.build_ambiguous,
        build_candidates => &variant.build_candidates,
        variant_command_arg => variant_command_arg,
        gene => &variant.gene,
        gene_command_arg => gene_command_arg,
        hgvs_p => &variant.hgvs_p,
        legacy_name => &variant.legacy_name,
        hgvs_c => &variant.hgvs_c,
        transcript => &variant.transcript,
        consequence => &variant.consequence,
        rsid => &variant.rsid,
        cosmic_id => &variant.cosmic_id,
        significance => &variant.significance,
        clinvar_id => &variant.clinvar_id,
        clinvar_review_status => &variant.clinvar_review_status,
        clinvar_review_stars => &variant.clinvar_review_stars,
        conditions => &variant.conditions,
        clinvar => &variant.clinvar,
        clinvar_conditions => &variant.clinvar_conditions,
        clinvar_condition_reports => &variant.clinvar_condition_reports,
        top_disease => &variant.top_disease,
        population => &variant.population,
        exome_filters => exome_filters,
        genome_filters => genome_filters,
        exome_highest_ancestry => exome_highest_ancestry,
        genome_highest_ancestry => genome_highest_ancestry,
        cadd_score => &variant.cadd_score,
        sift_pred => &variant.sift_pred,
        polyphen_pred => &variant.polyphen_pred,
        conservation => &variant.conservation,
        expanded_predictions => &variant.expanded_predictions,
        cosmic_context => &variant.cosmic_context,
        cgi_associations => &variant.cgi_associations,
        civic => &variant.civic,
        civic_actionability_pointer => civic_actionability_pointer,
        cancerhotspots => &variant.cancerhotspots,
        cancer_frequencies => &variant.cancer_frequencies,
        cancer_frequency_source => &variant.cancer_frequency_source,
        gwas => &variant.gwas,
        gwas_unavailable_reason => &variant.gwas_unavailable_reason,
        prediction => prediction,
        expression_interpretation => expr_i,
        splice_interpretation => splice_i,
        chromatin_interpretation => chrom_i,
        show_prediction_section => show_prediction_section,
        show_predictions_section => show_predictions_section,
        show_clinvar_section => show_clinvar_section,
        show_population_section => show_population_section,
        show_population_details => show_population_details,
        show_conservation_section => show_conservation_section,
        show_cosmic_section => show_cosmic_section,
        show_cgi_section => show_cgi_section,
        show_civic_section => show_civic_section,
        show_cancerhotspots_section => show_cancerhotspots_section,
        show_cbioportal_section => show_cbioportal_section,
        show_gwas_section => show_gwas_section,
        sections_block => format_sections_block("variant", follow_up_id, sections_variant(variant, requested_sections)),
        related_block => format_related_block(related_variant(variant)),
        source_states => section_render_contexts("variant", follow_up_id, &variant.section_outcomes),
    })?;
    Ok(append_evidence_urls(body, variant_evidence_urls(variant)))
}

fn highest_ancestry_frequency(
    population: &crate::sources::gnomad::GnomadSequencingPopulation,
) -> Option<&crate::sources::gnomad::GnomadAncestryPopulation> {
    population
        .populations
        .iter()
        .filter(|row| row.allele_frequency.is_some_and(f64::is_finite))
        .max_by(|left, right| {
            let frequency_order = left
                .allele_frequency
                .expect("filtered finite ancestry frequency")
                .total_cmp(
                    &right
                        .allele_frequency
                        .expect("filtered finite ancestry frequency"),
                );
            frequency_order
                .then_with(|| left.an.cmp(&right.an))
                .then_with(|| right.id.as_str().cmp(left.id.as_str()))
        })
}

fn expanded_gnomad_filters(
    population: &crate::sources::gnomad::GnomadSequencingPopulation,
) -> Vec<String> {
    population
        .filters
        .iter()
        .map(|flag| {
            let meaning = match flag.as_str() {
                "AC0" => "allele count is zero after filtering",
                "InbreedingCoeff" => "inbreeding coefficient filter",
                "RF" => "random forest quality filter",
                "AS_VQSR" => "allele-specific variant quality score recalibration filter",
                "EXCESS_HET" => "excess heterozygosity filter",
                "LCR" => "low-complexity region",
                "SEGDUP" => "segmental duplication",
                "monoallelic" => "site is monoallelic after filtering",
                _ => return flag.clone(),
            };
            format!("{flag} ({meaning})")
        })
        .collect()
}

fn civic_actionability_pointer(variant: &Variant) -> String {
    let command = format!(
        "get variant {} civic",
        quote_arg(preferred_variant_follow_up_id(variant))
    );
    let Some(civic) = variant.civic.as_ref() else {
        return format!("Therapeutic evidence: see `{command}`");
    };

    if civic.cached_evidence.is_empty() {
        return format!("Therapeutic evidence: see `{command}`");
    }

    let predictive_items = civic
        .cached_evidence
        .iter()
        .filter(|row| row.evidence_type.trim().eq_ignore_ascii_case("predictive"))
        .count();
    let assertions = civic
        .graphql
        .as_ref()
        .map_or(0, |context| context.assertion_total_count);
    format!(
        "Therapeutic evidence: {predictive_items} CIViC predictive item(s) / {assertions} assertion(s) — see `{command}`"
    )
}

fn prediction_interpretations(
    pred: &VariantPrediction,
) -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
) {
    let expr = pred.expression_lfc.map(|v| {
        if v > 0.2 {
            "Increased expression"
        } else if v < -0.2 {
            "Decreased expression"
        } else {
            "Minimal change"
        }
    });

    let splice = pred.splice_score.map(|v| {
        if v.abs() > 0.5 {
            "Higher splice impact"
        } else {
            "Low splice impact"
        }
    });

    let chrom = pred.chromatin_score.map(|v| {
        if v.abs() > 0.5 {
            "Altered accessibility"
        } else {
            "Low chromatin impact"
        }
    });

    (expr, splice, chrom)
}

// dead-code reason: variant::variant_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn variant_search_markdown(
    query: &str,
    results: &[VariantSearchResult],
) -> Result<String, BioMcpError> {
    variant_search_markdown_with_footer(query, results, "")
}

pub fn variant_search_markdown_with_footer(
    query: &str,
    results: &[VariantSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    variant_search_markdown_with_context(
        query,
        results,
        pagination_footer,
        None,
        None,
        &Default::default(),
        &[],
    )
}

pub fn variant_search_markdown_with_context(
    query: &str,
    results: &[VariantSearchResult],
    pagination_footer: &str,
    gene_filter: Option<&str>,
    condition_filter: Option<&str>,
    filter_evaluation: &crate::entities::variant::VariantFilterEvaluation,
    diagnostics: &[crate::entities::variant::SearchDiagnostic],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("variant_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        filter_evaluation => filter_evaluation,
        diagnostics => diagnostics,
        related_block => format_related_block(related_variant_search_results(
            results,
            gene_filter,
            condition_filter,
        )),
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

// dead-code reason: variant::phenotype_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn phenotype_search_markdown(
    query: &str,
    results: &[PhenotypeSearchResult],
) -> Result<String, BioMcpError> {
    let next_commands = super::search_next_commands_phenotype(results, None);
    phenotype_search_markdown_with_footer(query, &[], results, "", &next_commands)
}

pub fn phenotype_search_markdown_with_footer(
    query: &str,
    resolved_query: &[crate::entities::disease::ResolvedPhenotypeQuery],
    results: &[PhenotypeSearchResult],
    pagination_footer: &str,
    next_commands: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("phenotype_search.md.j2")?;
    let has_disease_follow_up = next_commands
        .iter()
        .any(|command| command.starts_with("biomcp get disease "));
    let body = tmpl.render(context! {
        query => query,
        resolved_query => resolved_query,
        count => results.len(),
        results => results,
        related_block => format_related_block(next_commands.to_vec()),
        has_disease_follow_up => has_disease_follow_up,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

// dead-code reason: variant::gwas_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn gwas_search_markdown(
    query: &str,
    results: &[VariantGwasAssociation],
) -> Result<String, BioMcpError> {
    gwas_search_markdown_with_footer(query, results, "")
}

pub fn gwas_search_markdown_with_footer(
    query: &str,
    results: &[VariantGwasAssociation],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gwas_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn variant_normalization_markdown(result: &VariantNormalizationResponse) -> String {
    let mut out = String::new();
    out.push_str("# Variant normalization\n\n");
    out.push_str(&format!("Input: {}\n\n", result.input));

    for service in &result.services {
        match service {
            crate::entities::variant::VariantNormalizationAggregate::Legacy(service) => {
                out.push_str(&format!("## {}\n\n", service.service));
                out.push_str(&format!("Status: {}\n", service.status.as_str()));
                if let Some(value) = service.input_description.as_deref() {
                    out.push_str(&format!("Input description: {value}\n"));
                }
                if let Some(value) = service.normalized_description.as_deref() {
                    out.push_str(&format!("Normalized description: {value}\n"));
                }
                if let Some(value) = service.corrected_description.as_deref() {
                    out.push_str(&format!("Corrected description: {value}\n"));
                }
                if let Some(value) = service.transcript_description.as_deref() {
                    out.push_str(&format!("Transcript description: {value}\n"));
                }
                if !service.genomic_descriptions.is_empty() {
                    out.push_str("Genomic descriptions:\n");
                    for value in &service.genomic_descriptions {
                        let provenance = value
                            .provenance
                            .as_deref()
                            .filter(|text| text.contains("provider default"))
                            .map(|_| ", provider default")
                            .unwrap_or_default();
                        out.push_str(&format!(
                            "- Genomic coordinate ({}{provenance}): {}\n",
                            value.genome_build, value.coordinate
                        ));
                    }
                }
                if let Some(protein) = &service.protein {
                    out.push_str(&format!("Protein: {protein}\n"));
                }
                if !service.warnings.is_empty() {
                    out.push_str("Warnings:\n");
                    for warning in &service.warnings {
                        out.push_str(&format!("- {warning}\n"));
                    }
                }
                if let Some(message) = service.message.as_deref() {
                    out.push_str(&format!("Message: {message}\n"));
                }
            }
            crate::entities::variant::VariantNormalizationAggregate::Car(car) => {
                out.push_str(&format!(
                    "## {}\n\nStatus: {:?}\nCAid: {}\n",
                    car.service,
                    car.item.status,
                    car.item.caid.as_deref().unwrap_or("-")
                ));
            }
        }
        out.push('\n');
    }

    out
}

pub fn variant_structure_markdown(result: &VariantStructureResult) -> String {
    let mut out = String::new();
    let structure_retry = section_recovery_commands(
        "variant_structure",
        &result.variant,
        &result.lookup_outcomes,
    )
    .into_iter()
    .next();
    let domains_recoverable = result
        .lookup_outcomes
        .get("domains")
        .is_some_and(|outcome| {
            matches!(
                outcome.outcome(),
                SectionOutcomeState::Degraded | SectionOutcomeState::Unavailable
            )
        });
    out.push_str(&format!("# Variant structure: {}\n\n", result.variant));
    out.push_str(&format!("Gene: {}\n", result.gene));
    if let Some(position) = result.residue.position {
        out.push_str(&format!("Residue: {position}\n"));
    }
    out.push_str(&format!(
        "Position confidence: {}\n\n",
        result.residue.position_confidence
    ));

    out.push_str("## Protein\n\n");
    out.push_str(&format!("Accession: {}\n", result.protein.accession));
    if let Some(entry) = result.protein.entry.as_deref() {
        out.push_str(&format!("Entry: {entry}\n"));
    }
    if let Some(length) = result.protein.length {
        out.push_str(&format!("Length: {length}\n"));
    }
    out.push('\n');

    out.push_str("## Domains (InterPro)\n\n");
    let domains_outcome = result.lookup_outcomes.get("domains");
    match domains_outcome.map(|outcome| outcome.outcome()) {
        Some(SectionOutcomeState::Data) => {
            for domain in &result.domains {
                let name = domain.name.as_deref().unwrap_or("InterPro domain");
                out.push_str(&format!(
                    "- {} ({}) {}-{}\n",
                    name, domain.accession, domain.start, domain.end
                ));
            }
            out.push('\n');
        }
        Some(SectionOutcomeState::Empty) => {
            out.push_str("No overlapping InterPro domains found for the selected residue.\n\n");
        }
        Some(
            SectionOutcomeState::Inapplicable
            | SectionOutcomeState::Degraded
            | SectionOutcomeState::Unavailable,
        ) => {
            if let Some(message) = domains_outcome.and_then(|outcome| outcome.message()) {
                out.push_str(message);
                out.push('\n');
            }
            if let Some(command) = structure_retry.as_deref() {
                out.push_str(&format!("Retry: {}\n", markdown_code_span(command)));
            }
            out.push('\n');
        }
        Some(SectionOutcomeState::NotRequested) | None => {}
    }

    out.push_str("## Structures (PDB / AlphaFold)\n\n");
    if result.structures.pdb.is_empty() {
        out.push_str("No UniProt PDB cross-references returned.\n");
    } else {
        for row in result.structures.pdb.iter().take(10) {
            let covered = row
                .residue_covered
                .map(|value| {
                    if value {
                        "covers residue"
                    } else {
                        "does not cover residue"
                    }
                })
                .unwrap_or("coverage unknown");
            out.push_str(&format!("- PDB {} ({covered})\n", row.id));
        }
    }
    if let Some(alphafold) = result.structures.alphafold.as_ref() {
        out.push_str(&format!("- AlphaFold: {}\n", alphafold.url));
    }
    out.push('\n');

    out.push_str("## Cancerhotspots\n\n");
    let hotspots_outcome = result.lookup_outcomes.get("cancerhotspots");
    match hotspots_outcome.map(|outcome| outcome.outcome()) {
        Some(SectionOutcomeState::Data | SectionOutcomeState::Empty) => {
            if let Some(recurrence) = result.cancerhotspots.as_ref() {
                out.push_str(&format!("Source: {}\n", recurrence.source));
                if let Some(count) = recurrence.position_count {
                    out.push_str(&format!("Position count: {count}\n"));
                }
                if let Some(count) = recurrence.same_aa_count {
                    out.push_str(&format!("Same amino-acid count: {count}\n"));
                }
                if hotspots_outcome
                    .is_some_and(|outcome| outcome.outcome() == SectionOutcomeState::Empty)
                {
                    out.push_str("No Cancer Hotspots recurrence match was found.\n");
                }
            }
            out.push('\n');
        }
        Some(
            SectionOutcomeState::Inapplicable
            | SectionOutcomeState::Degraded
            | SectionOutcomeState::Unavailable,
        ) => {
            if let Some(message) = hotspots_outcome.and_then(|outcome| outcome.message()) {
                out.push_str(message);
                out.push('\n');
            }
            if !domains_recoverable && let Some(command) = structure_retry.as_deref() {
                out.push_str(&format!("Retry: {}\n", markdown_code_span(command)));
            }
            out.push('\n');
        }
        Some(SectionOutcomeState::NotRequested) | None => {}
    }

    if !result.warnings.is_empty() {
        out.push_str("## Warnings\n\n");
        for warning in &result.warnings {
            out.push_str(&format!("- {warning}\n"));
        }
        out.push('\n');
    }

    let mut next_commands = result.meta.next_commands.clone();
    if let Some(command) = structure_retry.as_deref() {
        next_commands.retain(|candidate| candidate != command);
    }
    out.push_str(&format_related_block(next_commands));
    out
}

pub fn variant_oncokb_markdown(result: &VariantOncoKbResult) -> String {
    let mut out = String::new();
    out.push_str("# OncoKB\n\n");
    out.push_str(&format!("Gene: {}\n", result.gene.trim()));
    out.push_str(&format!("Alteration: {}\n", result.alteration.trim()));
    if let Some(level) = result
        .level
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push_str(&format!("Level: {level}\n"));
    }
    if let Some(oncogenic) = result
        .oncogenic
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push_str(&format!("Oncogenic: {oncogenic}\n"));
    }
    if let Some(effect) = result
        .effect
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push_str(&format!("Effect: {effect}\n"));
    }
    out.push('\n');

    if result.therapies.is_empty() {
        out.push_str("No therapy implications returned by OncoKB.\n");
    } else {
        out.push_str("## Therapies\n\n");
        out.push_str("| Drug | Level | Cancer Type | Note |\n");
        out.push_str("|------|-------|-------------|------|\n");
        for row in &result.therapies {
            let drugs = if row.drugs.is_empty() {
                "unspecified".to_string()
            } else {
                row.drugs.join(" + ")
            };
            let cancer = row.cancer_type.as_deref().unwrap_or("-");
            let note = row.note.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {drugs} | {} | {cancer} | {note} |\n",
                row.level
            ));
        }
    }

    if !result.gene.trim().is_empty() && !result.alteration.trim().is_empty() {
        out.push_str(&format!(
            "\n[OncoKB](https://www.oncokb.org/gene/{}/{})\n",
            result.gene.trim(),
            result.alteration.trim()
        ));
    }

    out
}
