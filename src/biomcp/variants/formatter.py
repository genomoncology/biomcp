"""Formatter for optimizing variant output for LLM consumption.

This module provides utilities to consolidate multi-allelic variants
and reduce token usage while preserving all critical information.
"""

import logging

logger = logging.getLogger(__name__)


def consolidate_multi_allelic_variants(variants: list[dict]) -> dict:
    """Consolidate variant(s) into compact LLM-friendly format.

    Takes one or more variant records and creates a consolidated structure
    that eliminates repetition while preserving all critical information.
    Works for both single variants and multi-allelic variants.

    Args:
        variants: List of variant dictionaries from MyVariant.info

    Returns:
        Consolidated dictionary with summary, variants, predictions, clinical data
    """
    if not variants:
        return {}

    logger.info(f"Consolidating {len(variants)} variant(s) for compact output")

    # Extract shared information from first variant
    first = variants[0]
    consolidated = {
        "summary": _extract_summary(variants),
        "alleles": _extract_alleles(variants),
        "predictions": _consolidate_predictions(variants),
        "clinical_significance": _consolidate_clinical(variants),
        "population_frequencies": _consolidate_frequencies(variants),
        "databases": _consolidate_databases(variants),
        "external_annotations": _consolidate_external(variants),
    }

    # Add shared metadata
    if "url" in first:
        consolidated["urls"] = first["url"]

    return consolidated


def _extract_summary(variants: list[dict]) -> dict:
    """Extract shared genomic context from variants."""
    first = variants[0]
    summary = {}

    # Gene information
    gene_info = {}
    if "dbsnp" in first and "gene" in first["dbsnp"]:
        gene = first["dbsnp"]["gene"]
        gene_info = {
            "symbol": gene.get("symbol"),
            "id": gene.get("geneid"),
            "name": gene.get("name"),
        }
    elif "dbnsfp" in first and "genename" in first["dbnsfp"]:
        gene_info["symbol"] = first["dbnsfp"]["genename"]

    if gene_info:
        summary["gene"] = gene_info

    # Genomic position
    if "hg19" in first:
        hg19 = first["hg19"]
        summary["position"] = {
            "chromosome": first.get("chrom"),
            "hg19_start": hg19.get("start"),
            "hg19_end": hg19.get("end"),
        }

    # rsID
    if "dbsnp" in first:
        summary["rsid"] = first["dbsnp"].get("rsid")

    # Protein position (from first variant)
    if "dbnsfp" in first and "aa" in first["dbnsfp"]:
        aa_info = first["dbnsfp"]["aa"]
        if "pos" in aa_info:
            summary["protein_position"] = aa_info["pos"]

    return summary


def _deduplicate_list(items: list) -> list:
    """Deduplicate list while preserving order."""
    seen = set()
    unique = []
    for item in items:
        if item not in seen:
            unique.append(item)
            seen.add(item)
    return unique


def _extract_hgvs_notations(variant: dict) -> list[str]:
    """Extract HGVS notations from variant."""
    hgvs: list[str] = []
    if "dbnsfp" not in variant:
        return hgvs

    dbnsfp = variant["dbnsfp"]

    # Extract protein HGVS
    if "hgvsp" in dbnsfp:
        hgvsp = dbnsfp["hgvsp"]
        if isinstance(hgvsp, list):
            hgvs.extend([h for h in hgvsp if h])
        elif hgvsp:
            hgvs.append(hgvsp)

    # Extract coding HGVS
    if "hgvsc" in dbnsfp:
        hgvsc = dbnsfp["hgvsc"]
        if isinstance(hgvsc, list):
            hgvs.extend([h for h in hgvsc if h])
        elif hgvsc:
            hgvs.append(hgvsc)

    return hgvs


def _extract_alleles(variants: list[dict]) -> list[dict]:
    """Extract allele-specific information for each variant."""
    alleles = []

    for variant in variants:
        allele = {"id": variant.get("_id")}

        # VCF info
        if "vcf" in variant:
            vcf = variant["vcf"]
            allele["ref"] = vcf.get("ref")
            allele["alt"] = vcf.get("alt")

        # Protein change
        if "dbnsfp" in variant and "aa" in variant["dbnsfp"]:
            aa = variant["dbnsfp"]["aa"]
            allele["aa_change"] = (
                f"{aa.get('ref')}{aa.get('pos')}{aa.get('alt')}"
            )

        # HGVS notations
        hgvs = _extract_hgvs_notations(variant)
        if hgvs:
            allele["hgvs"] = _deduplicate_list(hgvs)[:3]

        alleles.append(allele)

    return alleles


def _consolidate_predictions(variants: list[dict]) -> dict:
    """Consolidate prediction scores across variants.

    Focuses on most commonly used pathogenicity predictors.
    """
    predictions = {}

    # Key predictors to extract
    predictors = {
        "cadd": ("phred", "CADD"),
        "revel": ("score", "REVEL"),
        "sift": ("val", "SIFT"),
        "polyphen": ("val", "PolyPhen2"),
        "alphamissense": ("score", "AlphaMissense"),
        "primateai": ("score", "PrimateAI"),
        "clinpred": ("score", "ClinPred"),
    }

    for pred_key, (score_key, display_name) in predictors.items():
        scores = []
        for variant in variants:
            if "cadd" in variant and pred_key == "cadd":
                score = variant["cadd"].get(score_key)
            elif "dbnsfp" in variant and pred_key in variant["dbnsfp"]:
                pred_data = variant["dbnsfp"][pred_key]
                score = pred_data.get(score_key)
            else:
                score = None

            scores.append(score)

        # Only include if at least one variant has a score
        if any(s is not None for s in scores):
            predictions[display_name] = scores

    return predictions


def _process_clinvar_data(variants: list[dict]) -> dict | None:
    """Process ClinVar data from variants."""
    clinvar_data = []
    all_conditions = set()
    all_significances = set()

    for variant in variants:
        if "clinvar" not in variant:
            continue

        cv = variant["clinvar"]
        clinvar_data.append(cv)

        # Collect conditions and significances
        if "rcv" in cv:
            rcvs = cv["rcv"] if isinstance(cv["rcv"], list) else [cv["rcv"]]
            for rcv in rcvs:
                if "clinical_significance" in rcv:
                    all_significances.add(rcv["clinical_significance"])
                if "conditions" in rcv:
                    cond = rcv["conditions"]
                    if isinstance(cond, dict) and "name" in cond:
                        all_conditions.add(cond["name"])

    if not clinvar_data:
        return None

    return {
        "variant_count": len(clinvar_data),
        "clinical_significances": sorted(all_significances),
        "associated_conditions": sorted(all_conditions)[:10],
    }


def _process_cosmic_data(variants: list[dict]) -> dict | None:
    """Process COSMIC data from variants."""
    cosmic_data = []
    tumor_sites = set()

    for variant in variants:
        if "cosmic" not in variant:
            continue

        cosmic = variant["cosmic"]
        cosmic_list = cosmic if isinstance(cosmic, list) else [cosmic]
        cosmic_data.extend(cosmic_list)
        for c in cosmic_list:
            if "tumor_site" in c:
                tumor_sites.add(c["tumor_site"])

    if not cosmic_data:
        return None

    return {
        "total_entries": len(cosmic_data),
        "tumor_sites": sorted(tumor_sites),
    }


def _consolidate_clinical(variants: list[dict]) -> dict:
    """Consolidate clinical significance data."""
    clinical = {}

    # Process ClinVar data
    clinvar = _process_clinvar_data(variants)
    if clinvar:
        clinical["clinvar"] = clinvar

    # Process COSMIC data
    cosmic = _process_cosmic_data(variants)
    if cosmic:
        clinical["cosmic"] = cosmic

    # CIViC data
    civic_count = sum(1 for v in variants if "civic" in v)
    if civic_count > 0:
        clinical["civic"] = {"variant_count": civic_count}

    return clinical


def _consolidate_frequencies(variants: list[dict]) -> dict:
    """Consolidate population frequency data."""
    frequencies: dict[str, dict] = {}

    for i, variant in enumerate(variants):
        variant_key = f"allele_{i + 1}"

        # gnomAD exome
        if "gnomad_exome" in variant:
            gnomad = variant["gnomad_exome"]
            if "af" in gnomad:
                if "gnomad_exome" not in frequencies:
                    frequencies["gnomad_exome"] = {}
                frequencies["gnomad_exome"][variant_key] = gnomad["af"]["af"]

        # ExAC
        if "exac" in variant:
            exac = variant["exac"]
            if "af" in exac:
                if "exac" not in frequencies:
                    frequencies["exac"] = {}
                frequencies["exac"][variant_key] = exac["af"]

    return frequencies


def _consolidate_databases(variants: list[dict]) -> dict:
    """Consolidate database cross-references."""
    databases = {}

    # Collect unique database IDs
    cosmic_ids = set()
    clinvar_ids = set()

    for variant in variants:
        if "cosmic" in variant:
            cosmic = variant["cosmic"]
            cosmic_list = cosmic if isinstance(cosmic, list) else [cosmic]
            for c in cosmic_list:
                if "cosmic_id" in c:
                    cosmic_ids.add(c["cosmic_id"])

        if "clinvar" in variant and "variant_id" in variant["clinvar"]:
            clinvar_ids.add(variant["clinvar"]["variant_id"])

    if cosmic_ids:
        databases["cosmic_ids"] = sorted(cosmic_ids)[:5]  # Top 5
    if clinvar_ids:
        databases["clinvar_ids"] = sorted(clinvar_ids)

    return databases


def _consolidate_external(variants: list[dict]) -> dict:
    """Consolidate external annotations (cBioPortal, OncoKB, TCGA)."""
    external: dict[str, list] = {}

    for variant in variants:
        # cBioPortal
        if "cbioportal" in variant:
            cbio = variant["cbioportal"]
            if "cbioportal" not in external:
                external["cbioportal"] = []
            external["cbioportal"].append({
                "variant": variant.get("_id"),
                "total_cases": cbio.get("total_cases"),
                "hotspot_samples": cbio.get("hotspot_samples"),
                "cancer_types": list(cbio.get("cancer_types", {}).keys())[:5],
            })

        # OncoKB
        if "oncokb" in variant:
            oncokb = variant["oncokb"]
            if "oncokb" not in external:
                external["oncokb"] = []
            external["oncokb"].append({
                "variant": variant.get("_id"),
                "oncogenic": oncokb.get("oncogenic"),
                "mutation_effect": oncokb.get("mutation_effect"),
                "is_hotspot": oncokb.get("is_hotspot"),
            })

        # TCGA
        if "tcga" in variant:
            tcga = variant["tcga"]
            if "tcga" not in external:
                external["tcga"] = []
            external["tcga"].append({
                "variant": variant.get("_id"),
                "case_count": tcga.get("case_count"),
            })

    return external
