use super::*;
use crate::entities::variant::{
    GenomeBuild, TreatmentImplication, VariantNormalizationAggregate, VariantNormalizationResponse,
    VariantNormalizationServiceResult, VariantNormalizationStatus,
};

#[test]
fn markdown_render_variant_entity() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr7:g.55259515T>G",
        "gene": "EGFR",
        "hgvs_p": "p.L858R",
        "legacy_name": "EGFR L858R",
        "significance": "Pathogenic"
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &[]).expect("rendered markdown");
    assert!(markdown.contains("EGFR"));
    assert!(markdown.contains("p.L858R"));
    assert!(markdown.contains("Legacy Name: EGFR L858R"));
}

#[test]
fn variant_markdown_names_the_competing_build_identity() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr10:g.87933119A>C",
        "gene": "PTEN",
        "rsid": "rs759485888",
        "genome_build": "GRCh38",
        "build_ambiguous": true,
        "build_candidates": [
            {"genome_build": "GRCh37", "id": "chr10:g.87933119A>C", "rsid": "rs1212585646"}
        ]
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &[]).expect("rendered markdown");
    assert!(markdown.contains("different GRCh37 record"));
    assert!(markdown.contains("rsID: rs1212585646"));
}

#[test]
fn variant_markdown_default_card_renders_cached_civic_actionability_pointer() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr1:g.100A>T",
        "gene": "TST",
        "hgvs_p": "p.A1V",
        "civic": {
            "cached_evidence": [
                {
                    "id": 11,
                    "name": "cached predictive evidence",
                    "molecular_profile": "TST A1V",
                    "evidence_type": "Predictive",
                    "evidence_level": "B",
                    "significance": "Sensitivity",
                    "disease": "Example carcinoma",
                    "therapies": ["exampletinib"],
                    "status": "accepted"
                },
                {
                    "id": 12,
                    "name": "cached prognostic evidence",
                    "molecular_profile": "TST A1V",
                    "evidence_type": "Prognostic",
                    "evidence_level": "C",
                    "significance": "Poor outcome",
                    "status": "accepted"
                }
            ]
        }
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &[]).expect("rendered markdown");
    assert!(markdown.contains(
        "Therapeutic evidence: 1 CIViC predictive item(s) / 0 assertion(s) — see `get variant \"chr1:g.100A>T\" civic`"
    ));
    assert!(!markdown.contains("## CIViC"));
}

#[test]
fn variant_markdown_default_card_renders_bare_civic_pointer_without_cached_evidence() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr1:g.101A>T",
        "gene": "TST",
        "hgvs_p": "p.A2V"
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &[]).expect("rendered markdown");
    assert!(markdown.contains("Therapeutic evidence: see `get variant \"chr1:g.101A>T\" civic`"));
    assert!(!markdown.contains("CIViC predictive item(s)"));
}

#[test]
fn variant_markdown_civic_section_renders_currency_caveat_and_cross_checks() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr1:g.100A>T",
        "gene": "TST",
        "hgvs_p": "p.A1V"
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &["civic".to_string()]).expect("rendered markdown");
    assert!(markdown.contains(
        "Caveat: CIViC evidence may lag current standard of care — cross-check the literature and therapy layers."
    ));
    assert!(markdown.contains("See also: `variant articles \"chr1:g.100A>T\"` / `gene drugs TST`"));
}

#[test]
fn variant_markdown_next_commands_quote_variant_ids_with_spaces() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "BRAF V600E",
        "gene": "BRAF",
        "hgvs_p": "p.Val600Glu"
    }))
    .expect("variant should deserialize");

    let default_markdown = variant_markdown(&variant, &[]).expect("rendered markdown");
    assert!(
        default_markdown.contains("Therapeutic evidence: see `get variant \"BRAF V600E\" civic`")
    );

    let civic_markdown =
        variant_markdown(&variant, &["civic".to_string()]).expect("rendered markdown");
    assert!(
        civic_markdown.contains("See also: `variant articles \"BRAF V600E\"` / `gene drugs BRAF`")
    );
}

#[test]
fn rsid_indel_card_only_prints_variant_ids_accepted_by_the_parser() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr19:g.11106928AAG[1]",
        "rsid": "rs876657378",
        "gene": "SMARCA4"
    }))
    .expect("rsID-resolved indel should deserialize");

    let markdown = variant_markdown(&variant, &[]).expect("rendered indel card");
    let mut checked_routes = std::collections::BTreeSet::new();

    let mut check_command = |command: &str| {
        let argv = shlex::split(command)
            .unwrap_or_else(|| panic!("rendered command is not shell-safe: {command}"));
        let (route, id) = match argv.as_slice() {
            [biomcp, get, variant, id, ..]
                if biomcp == "biomcp" && get == "get" && variant == "variant" =>
            {
                ("get", id)
            }
            [biomcp, variant, subcommand, id, ..]
                if biomcp == "biomcp"
                    && variant == "variant"
                    && matches!(subcommand.as_str(), "trials" | "articles" | "oncokb") =>
            {
                (subcommand.as_str(), id)
            }
            _ => return,
        };
        checked_routes.insert(route.to_string());
        assert!(
            crate::entities::variant::parse_variant_id(id).is_ok(),
            "card printed an unreadable variant ID in `{command}`"
        );
    };

    for line in markdown.lines() {
        if let Some(start) = line.find("biomcp ") {
            let command = line[start..]
                .split_once("   -")
                .map_or(&line[start..], |(command, _)| command);
            check_command(command.trim());
        }
        for inline in line.split('`').skip(1).step_by(2) {
            if inline.starts_with("get variant ") || inline.starts_with("variant ") {
                check_command(&format!("biomcp {inline}"));
            }
        }
    }

    for route in ["get", "trials", "articles"] {
        assert!(
            checked_routes.contains(route),
            "indel card did not exercise the {route} follow-up producer"
        );
    }
}

#[test]
fn variant_markdown_renders_compact_clinvar_and_population_fields() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr7:g.140453136A>T",
        "genome_build": "GRCh38",
        "gene": "BRAF",
        "population": {
            "status": "data",
            "dataset": "gnomad_r4",
            "release": "gnomAD v4",
            "exome": {
                "allele_frequency": 0.0001,
                "ac": 2,
                "an": 20000,
                "homozygote_count": 0,
                "hemizygote_count": 0,
                "filters": ["RF"],
                "faf95": {"popmax": 0.0002, "popmax_population": "nfe"},
                "populations": [
                    {
                        "id": "nfe", "allele_frequency": 0.0002,
                        "ac": 2, "an": 10000, "homozygote_count": 0,
                        "hemizygote_count": 0
                    },
                    {
                        "id": "ac_zero", "allele_frequency": 0.0,
                        "ac": 0, "an": 100, "homozygote_count": 0,
                        "hemizygote_count": 0
                    },
                    {
                        "id": "no_observations", "allele_frequency": null,
                        "ac": 0, "an": 0, "homozygote_count": 0,
                        "hemizygote_count": 0
                    }
                ]
            },
            "genome": {
                "allele_frequency": 0.00015,
                "ac": 3,
                "an": 20000,
                "homozygote_count": 0,
                "hemizygote_count": 0,
                "filters": [],
                "faf95": {"popmax": 0.0003, "popmax_population": "afr"},
                "populations": [{
                    "id": "afr", "allele_frequency": 0.0004,
                    "ac": 2, "an": 5000, "homozygote_count": 0,
                    "hemizygote_count": 0
                }]
            },
            "faf_caveat": "gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."
        },
        "top_disease": {"condition": "Melanoma", "reports": 2},
        "clinvar_conditions": [{"condition": "Melanoma", "reports": 2}]
    }))
    .expect("variant should deserialize");

    let compact = variant_markdown(&variant, &["population".to_string()])
        .expect("rendered compact population markdown");
    assert!(
        compact.contains("## Population (direct gnomAD v4)"),
        "{compact}"
    );
    assert!(
        compact.contains("Exome overall frequency: 0.0001"),
        "{compact}"
    );
    assert!(
        compact.contains("Exome highest ancestry frequency: nfe (0.0002)"),
        "{compact}"
    );
    assert!(compact.contains("gnomAD v4 exome grpmax FAF95: 0.0002 (nfe)"));
    assert!(compact.contains("RF (random forest quality filter)"));
    assert!(
        compact.contains("Genome overall frequency: 0.00015"),
        "{compact}"
    );
    assert!(
        compact.contains("Genome highest ancestry frequency: afr (0.0004)"),
        "{compact}"
    );
    assert!(compact.contains("gnomAD v4 genome grpmax FAF95: 0.0003 (afr)"));
    assert!(compact.contains("PASS (no filter flags reported)"));
    assert!(compact.contains(
        "gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."
    ));
    assert!(!compact.contains("| gnomAD v4 | nfe |"), "{compact}");

    let detailed = variant_markdown(
        &variant,
        &["all".to_string(), "population-details".to_string()],
    )
    .expect("rendered detailed population markdown");
    assert!(detailed.contains("Top disease (ClinVar): Melanoma (2 reports)"));
    assert!(
        detailed.contains("| gnomAD v4 | Overall | 0.0001"),
        "{detailed}"
    );
    assert!(
        detailed.contains("| gnomAD v4 | nfe | 0.0002"),
        "{detailed}"
    );
    assert!(
        detailed.contains("| gnomAD v4 | ac_zero | 0 | 0 | 100"),
        "{detailed}"
    );
    assert!(
        detailed.contains("| gnomAD v4 | no_observations | - | 0 | 0"),
        "{detailed}"
    );
}

#[test]
fn variant_population_markdown_labels_residual_group_but_json_keeps_raw_id() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "rs1426654",
        "gene": "SLC24A5",
        "population": {
            "status": "data",
            "dataset": "gnomad_r4",
            "release": "gnomAD v4",
            "exome": {
                "allele_frequency": 0.001,
                "ac": 2,
                "an": 2000,
                "homozygote_count": 0,
                "hemizygote_count": 0,
                "filters": [],
                "faf95": null,
                "populations": [
                    {
                        "id": "afr", "allele_frequency": 0.001,
                        "ac": 1, "an": 1000, "homozygote_count": 0,
                        "hemizygote_count": 0
                    },
                    {
                        "id": "remaining", "allele_frequency": 0.002,
                        "ac": 2, "an": 1000, "homozygote_count": 0,
                        "hemizygote_count": 0
                    }
                ]
            },
            "genome": null,
            "faf_caveat": "gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."
        }
    }))
    .expect("variant should deserialize");

    let detailed = variant_markdown(&variant, &["population-details".to_string()])
        .expect("rendered detailed population markdown");
    assert!(
        detailed.contains(
            "| gnomAD v4 | Other / not assigned (gnomAD residual) | 0.002 | 2 | 1000 | 0 | 0 |"
        ),
        "{detailed}"
    );
    assert!(
        detailed.contains("| gnomAD v4 | afr | 0.001 | 1 | 1000 | 0 | 0 |"),
        "{detailed}"
    );
    assert!(
        !detailed.contains("| gnomAD v4 | remaining |"),
        "{detailed}"
    );

    let compact = variant_markdown(&variant, &["population".to_string()])
        .expect("rendered compact population markdown");
    assert!(
        compact.contains(
            "Exome highest ancestry frequency: Other / not assigned (gnomAD residual) (0.002)"
        ),
        "{compact}"
    );
    assert!(!compact.contains("frequency: remaining"), "{compact}");

    let json = serde_json::to_value(&variant).expect("variant should serialize");
    assert_eq!(
        json["population"]["exome"]["populations"][1]["id"],
        "remaining"
    );
}

#[test]
fn variant_population_markdown_keeps_missing_status_compact() {
    let unresolved: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr7:g.140453136A>T",
        "genome_build": "GRCh37",
        "gene": "BRAF",
        "population": {
            "status": "inapplicable",
            "dataset": "gnomad_r4",
            "release": "gnomAD v4",
            "message": "Direct gnomAD v4 population data requires a trustworthy GRCh38 coordinate; tried dbSNP.",
            "exome": null,
            "genome": null,
            "faf_caveat": "gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."
        }
    }))
    .expect("unresolved variant should deserialize");

    let unresolved_markdown = variant_markdown(&unresolved, &["population".to_string()]).unwrap();
    assert!(unresolved_markdown.starts_with("# BRAF - population"));
    assert!(unresolved_markdown.contains("requires a trustworthy GRCh38 coordinate"));
    assert!(unresolved_markdown.contains("tried dbSNP"));
    assert!(!unresolved_markdown.contains("### Exomes"));
    assert!(!unresolved_markdown.contains("## ClinVar"));

    let resolved: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr11:g.5248232T>A",
        "genome_build": "GRCh37",
        "gene": "HBB",
        "population": {
            "status": "data",
            "dataset": "gnomad_r4",
            "release": "gnomAD v4",
            "resolved_coordinate": {
                "id": "chr11:g.5227002T>A",
                "genome_build": "GRCh38",
                "source": "dbSNP"
            },
            "exome": {
                "allele_frequency": 0.001,
                "ac": 2335,
                "an": 1458356,
                "homozygote_count": 31,
                "hemizygote_count": 0,
                "filters": [],
                "faf95": {"popmax": 0.05474387, "popmax_population": "afr"},
                "populations": []
            },
            "genome": null,
            "faf_caveat": "gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."
        }
    }))
    .expect("resolved variant should deserialize");

    let resolved_markdown = variant_markdown(&resolved, &["population".to_string()]).unwrap();
    assert!(resolved_markdown.contains("Resolved GRCh38 coordinate: chr11:g.5227002T>A (dbSNP)"));
    assert!(resolved_markdown.contains("Exomes"));
}

#[test]
fn variant_markdown_renders_cancerhotspots_recurrence_when_present() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr7:g.140453136A>T",
        "gene": "BRAF",
        "hgvs_p": "p.V600E",
        "cancerhotspots": {
            "source": "cancerhotspots.org",
            "position_count": 897,
            "same_aa_count": 833,
            "matched_transcript": "ENST00000288602"
        }
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &["all".to_string()]).expect("rendered markdown");
    assert!(markdown.contains("## Cancerhotspots.org Recurrence"));
    assert!(markdown.contains("Source: cancerhotspots.org"));
    assert!(markdown.contains("Matched transcript: ENST00000288602"));
    assert!(markdown.contains("Position count: 897"));
    assert!(markdown.contains("Same amino-acid count: 833"));
}

#[test]
fn variant_markdown_renders_gwas_unavailable_message() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "rs7903146",
        "gene": "TCF7L2",
        "rsid": "rs7903146",
        "gwas": [],
        "gwas_unavailable_reason": "GWAS association data temporarily unavailable."
    }))
    .expect("variant should deserialize");

    let markdown = variant_markdown(&variant, &["gwas".to_string()]).expect("rendered markdown");
    assert!(markdown.contains("GWAS association data temporarily unavailable."));
    assert!(!markdown.contains("No GWAS associations found for this variant."));
}

#[test]
fn variant_search_markdown_renders_legacy_name_column_and_fallback() {
    let results = vec![
        VariantSearchResult {
            id: "chr6:g.118880200T>G".to_string(),
            genome_build: GenomeBuild::Grch37,
            genome_build_provenance: "test".into(),
            gene: "PLN".to_string(),
            hgvs_p: Some("p.L39X".to_string()),
            hgvs_c: None,
            transcript: None,
            legacy_name: Some("PLN L39stop".to_string()),
            significance: Some("Pathogenic".to_string()),
            clinvar_stars: Some(2),
            gnomad_af: None,
            revel: Some(0.935),
            gerp: Some(5.12),
            source_identity: None,
            matched_alias: None,
        },
        VariantSearchResult {
            id: "chr6:g.118880100A>G".to_string(),
            genome_build: GenomeBuild::Grch37,
            genome_build_provenance: "test".into(),
            gene: "PLN".to_string(),
            hgvs_p: Some("p.K3R".to_string()),
            hgvs_c: None,
            transcript: None,
            legacy_name: None,
            significance: None,
            clinvar_stars: None,
            gnomad_af: None,
            revel: None,
            gerp: None,
            source_identity: None,
            matched_alias: None,
        },
    ];

    let markdown =
        variant_search_markdown("gene=PLN, hgvsp=L39X", &results).expect("rendered markdown");
    assert!(markdown.contains(
        "| ID | Build | Gene | Transcript | Coding | Protein | Legacy Name | Significance |"
    ));
    assert!(
        markdown.contains("| chr6:g.118880200T>G | GRCh37 | PLN | - | - | p.L39X | PLN L39stop |")
    );
    assert!(markdown.contains("| chr6:g.118880100A>G | GRCh37 | PLN | - | - | p.K3R | - |"));
}

#[test]
fn variant_search_markdown_renders_related_commands_from_context() {
    let results = vec![
        VariantSearchResult {
            id: "rs199473688".to_string(),
            genome_build: GenomeBuild::Grch37,
            genome_build_provenance: "test".into(),
            gene: "SCN5A".to_string(),
            hgvs_p: Some("p.Arg282His".to_string()),
            hgvs_c: None,
            transcript: None,
            legacy_name: None,
            significance: Some("Pathogenic".to_string()),
            clinvar_stars: Some(2),
            gnomad_af: None,
            revel: Some(0.91),
            gerp: Some(5.7),
            source_identity: None,
            matched_alias: None,
        },
        VariantSearchResult {
            id: "rs7626962".to_string(),
            genome_build: GenomeBuild::Grch37,
            genome_build_provenance: "test".into(),
            gene: "SCN5A".to_string(),
            hgvs_p: Some("p.Gly514Cys".to_string()),
            hgvs_c: None,
            transcript: None,
            legacy_name: None,
            significance: Some("Likely pathogenic".to_string()),
            clinvar_stars: Some(1),
            gnomad_af: None,
            revel: Some(0.88),
            gerp: Some(5.1),
            source_identity: None,
            matched_alias: None,
        },
    ];

    let markdown = variant_search_markdown_with_context(
        "gene=SCN5A, condition=Brugada",
        &results,
        "",
        Some("SCN5A"),
        Some("Brugada"),
        &Default::default(),
        &[],
    )
    .expect("rendered markdown");

    assert!(markdown.contains("See also:"));
    assert!(markdown.contains("biomcp get variant rs199473688"));
    assert!(markdown.contains("biomcp get gene SCN5A"));
    assert!(markdown.contains("biomcp search disease --query Brugada"));
}

#[test]
fn phenotype_search_markdown_renders_top_disease_follow_up() {
    let results = vec![
        crate::entities::disease::PhenotypeSearchResult {
            disease_id: "MONDO:0100135".to_string(),
            disease_name: "Dravet syndrome".to_string(),
            score: 15.036,
        },
        crate::entities::disease::PhenotypeSearchResult {
            disease_id: "MONDO:0000032".to_string(),
            disease_name: "febrile seizures, familial".to_string(),
            score: 15.036,
        },
    ];

    let markdown = phenotype_search_markdown_with_footer(
        "HP:0002373 HP:0001250",
        &results,
        "Showing 1-2 of 2 results.",
    )
    .expect("rendered markdown");

    assert!(markdown.contains("See also:"));
    assert!(markdown.contains("biomcp get disease \"Dravet syndrome\" genes phenotypes"));
    assert_eq!(
        related_command_description("biomcp get disease \"Dravet syndrome\" genes phenotypes"),
        Some("open the top phenotype-match disease with genes and phenotypes")
    );
}

#[test]
fn ticket_406_coordinate_outputs_carry_genome_build_context() {
    let response = VariantNormalizationResponse {
        input: "NM_000248.3:c.135del".to_string(),
        services: vec![
            crate::entities::variant::VariantNormalizationAggregate::Legacy(
                VariantNormalizationServiceResult {
                    service: "variantvalidator".to_string(),
                    status: VariantNormalizationStatus::Success,
                    input_description: Some("NM_000248.3:c.135del".to_string()),
                    normalized_description: Some("NM_000248.3:c.135del".to_string()),
                    corrected_description: None,
                    transcript_description: Some("NM_000248.3:c.135del".to_string()),
                    protein: None,
                    genomic_descriptions: vec![crate::entities::GenomicCoordinate {
                        coordinate: "NC_000023.11:g.32389644del".into(),
                        genome_build: "GRCh38".into(),
                        source: "test".into(),
                        provenance: None,
                    }],
                    warnings: Vec::new(),
                    message: None,
                },
            ),
        ],
    };

    let markdown = variant_normalization_markdown(&response);

    assert!(
        markdown.contains("GRCh") && markdown.contains("NC_000023.11:g.32389644del"),
        "variant genomic descriptions must include explicit genome-build context with the position, got {markdown:?}"
    );
}

#[test]
fn ticket_589_variant_structure_failures_do_not_render_as_checked_absence_or_source_credit() {
    let fixture = |failed_key: &str| {
        let (domains_outcome, hotspots_outcome, recurrence) = if failed_key == "domains" {
            (
                serde_json::json!({
                    "outcome": "unavailable",
                    "sources": [],
                    "message": "InterPro domain data is temporarily unavailable."
                }),
                serde_json::json!({"outcome": "empty", "sources": ["cancerhotspots.org"]}),
                serde_json::json!({
                    "source": "cancerhotspots.org",
                    "position_count": null,
                    "same_aa_count": null,
                    "matched_transcript": null
                }),
            )
        } else {
            (
                serde_json::json!({"outcome": "empty", "sources": ["InterPro"]}),
                serde_json::json!({
                    "outcome": "unavailable",
                    "sources": [],
                    "message": "Cancer Hotspots recurrence is temporarily unavailable."
                }),
                serde_json::Value::Null,
            )
        };
        serde_json::from_value::<VariantStructureResult>(serde_json::json!({
            "variant": "BRAF V600E",
            "gene": "BRAF",
            "input_kind": "gene_protein_change",
            "residue": {
                "requested_change": "V600E",
                "position": 600,
                "reference_aa": "V",
                "alternate_aa": "E",
                "source": "MyVariant.info/dbNSFP",
                "matched_hgvsp": ["p.V600E"],
                "other_source_positions": [],
                "position_confidence": "requested_hgvsp_exact_match"
            },
            "protein": {
                "accession": "P15056",
                "entry": "BRAF_HUMAN",
                "length": 766,
                "source": "UniProt"
            },
            "domains": [],
            "structures": {"pdb": [], "alphafold": null},
            "cancerhotspots": recurrence,
            "lookup_outcomes": {
                "domains": domains_outcome,
                "cancerhotspots": hotspots_outcome
            },
            "warnings": [],
            "_meta": {"next_commands": []}
        }))
        .expect("valid structure failure fixture")
    };

    let interpro = fixture("domains");
    let interpro_json = serde_json::to_value(&interpro).expect("structure serializes");
    assert_eq!(
        interpro_json["lookup_outcomes"]["domains"]["outcome"],
        "unavailable"
    );
    assert_eq!(
        interpro_json["lookup_outcomes"]["domains"]["sources"],
        serde_json::json!([])
    );
    let interpro_markdown = variant_structure_markdown(&interpro);
    assert!(
        interpro_markdown
            .to_ascii_lowercase()
            .contains("unavailable")
    );
    assert!(!interpro_markdown.contains("No overlapping InterPro domains found"));

    let hotspots = fixture("cancerhotspots");
    let hotspots_json = serde_json::to_value(&hotspots).expect("structure serializes");
    assert_eq!(
        hotspots_json["lookup_outcomes"]["cancerhotspots"]["outcome"],
        "unavailable"
    );
    assert_eq!(
        hotspots_json["lookup_outcomes"]["cancerhotspots"]["sources"],
        serde_json::json!([])
    );
    assert!(hotspots_json["cancerhotspots"].is_null());
    assert!(!hotspots_json.to_string().contains("cancerhotspots.org"));
    let hotspots_markdown = variant_structure_markdown(&hotspots);
    assert!(
        hotspots_markdown
            .to_ascii_lowercase()
            .contains("unavailable")
    );
    assert!(!hotspots_markdown.contains("Source: cancerhotspots.org"));
    assert!(!hotspots_markdown.contains("No Cancer Hotspots recurrence match was found"));
}

#[test]
fn variant_oncokb_markdown_shows_truncation_note() {
    let result = VariantOncoKbResult {
        gene: "EGFR".to_string(),
        alteration: "L858R".to_string(),
        oncogenic: Some("Oncogenic".to_string()),
        level: Some("Level 1".to_string()),
        effect: Some("Gain-of-function".to_string()),
        therapies: vec![
            TreatmentImplication {
                level: "Level 1".to_string(),
                drugs: vec!["osimertinib".to_string()],
                cancer_type: Some("Lung adenocarcinoma".to_string()),
                note: None,
            },
            TreatmentImplication {
                level: "Level 2".to_string(),
                drugs: vec!["afatinib".to_string()],
                cancer_type: Some("Lung adenocarcinoma".to_string()),
                note: Some("(and 2 more)".to_string()),
            },
        ],
    };

    let markdown = variant_oncokb_markdown(&result);
    assert!(markdown.contains("| Drug | Level | Cancer Type | Note |"));
    assert!(markdown.contains("(and 2 more)"));
}

#[test]
fn normalization_markdown_keeps_legacy_collection_labels() {
    let result = VariantNormalizationResponse {
        input: "NM_000546.6:c.215C>G".into(),
        services: vec![VariantNormalizationAggregate::Legacy(
            VariantNormalizationServiceResult {
                service: "mutalyzer".into(),
                status: VariantNormalizationStatus::Success,
                input_description: None,
                normalized_description: None,
                corrected_description: None,
                transcript_description: None,
                protein: None,
                genomic_descriptions: vec![crate::entities::GenomicCoordinate {
                    coordinate: "NC_000017.11:g.7674220C>G".into(),
                    genome_build: "GRCh38".into(),
                    source: "test".into(),
                    provenance: None,
                }],
                warnings: vec!["provider note".into()],
                message: None,
            },
        )],
    };

    let markdown = variant_normalization_markdown(&result);
    assert!(markdown.contains(
        "Genomic descriptions:\n- Genomic coordinate (GRCh38): NC_000017.11:g.7674220C>G"
    ));
    assert!(markdown.contains("Warnings:\n- provider note"));
}

#[test]
fn gwas_search_markdown_renders_result_row() {
    let markdown = gwas_search_markdown(
        "EGFR",
        &[crate::entities::variant::VariantGwasAssociation {
            rsid: "rs121434568".to_string(),
            trait_name: Some("Lung adenocarcinoma".to_string()),
            p_value: crate::entities::variant::GwasPValue::from_numeric(5.0e-8),
            effect_size: Some(1.23),
            effect_type: Some("OR".to_string()),
            confidence_interval: None,
            risk_allele_frequency: Some(0.12),
            risk_allele: None,
            mapped_genes: vec!["EGFR".to_string()],
            study_accession: Some("GCST000001".to_string()),
            pmid: Some("12345678".to_string()),
            author: None,
            sample_description: None,
        }],
    )
    .expect("gwas markdown");

    assert!(markdown.contains("# GWAS Search: EGFR"));
    assert!(markdown.contains("| rs121434568 | Lung adenocarcinoma |"));
    assert!(markdown.contains("| OR 1.230 |") || markdown.contains("OR 1.230"));
}
