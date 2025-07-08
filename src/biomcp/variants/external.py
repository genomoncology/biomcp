"""External data sources for enhanced variant annotations."""

import asyncio
import json
import logging
import re
from typing import Any
from urllib.parse import quote

from pydantic import BaseModel, Field

from .. import http_client

# Import CBioPortalVariantData from the new module
from .cbio_external_client import CBioPortalVariantData

logger = logging.getLogger(__name__)

# TCGA/GDC API endpoints
GDC_BASE = "https://api.gdc.cancer.gov"
GDC_SSMS_ENDPOINT = f"{GDC_BASE}/ssms"  # Simple Somatic Mutations

# 1000 Genomes API endpoints
ENSEMBL_REST_BASE = "https://rest.ensembl.org"
ENSEMBL_VARIATION_ENDPOINT = f"{ENSEMBL_REST_BASE}/variation/human"

# Import constants


class TCGAVariantData(BaseModel):
    """TCGA/GDC variant annotation data."""

    cosmic_id: str | None = None
    tumor_types: list[str] = Field(default_factory=list)
    mutation_frequency: float | None = None
    mutation_count: int | None = None
    affected_cases: int | None = None
    consequence_type: str | None = None
    clinical_significance: str | None = None


class ThousandGenomesData(BaseModel):
    """1000 Genomes variant annotation data."""

    global_maf: float | None = Field(
        None, description="Global minor allele frequency"
    )
    afr_maf: float | None = Field(None, description="African population MAF")
    amr_maf: float | None = Field(None, description="American population MAF")
    eas_maf: float | None = Field(
        None, description="East Asian population MAF"
    )
    eur_maf: float | None = Field(None, description="European population MAF")
    sas_maf: float | None = Field(
        None, description="South Asian population MAF"
    )
    ancestral_allele: str | None = None
    most_severe_consequence: str | None = None


# CBioPortalVariantData is now imported from cbio_external_client.py


class EnhancedVariantAnnotation(BaseModel):
    """Enhanced variant annotation combining multiple sources."""

    variant_id: str
    tcga: TCGAVariantData | None = None
    thousand_genomes: ThousandGenomesData | None = None
    cbioportal: CBioPortalVariantData | None = None
    error_sources: list[str] = Field(default_factory=list)


class TCGAClient:
    """Client for TCGA/GDC API."""

    async def get_variant_data(
        self, variant_id: str
    ) -> TCGAVariantData | None:
        """Fetch variant data from TCGA/GDC.

        Args:
            variant_id: Can be gene AA change (e.g., "BRAF V600E") or genomic coordinates
        """
        try:
            # Determine the search field based on variant_id format
            # If it looks like "GENE AA_CHANGE" format, use gene_aa_change field
            if " " in variant_id and not variant_id.startswith("chr"):
                search_field = "gene_aa_change"
                search_value = variant_id
            else:
                # Otherwise try genomic_dna_change
                search_field = "genomic_dna_change"
                search_value = variant_id

            # First, search for the variant
            params = {
                "filters": json.dumps({
                    "op": "in",
                    "content": {
                        "field": search_field,
                        "value": [search_value],
                    },
                }),
                "fields": "cosmic_id,genomic_dna_change,gene_aa_change,ssm_id",
                "format": "json",
                "size": "5",  # Get a few in case of multiple matches
            }

            response, error = await http_client.request_api(
                url=GDC_SSMS_ENDPOINT,
                method="GET",
                request=params,
                domain="gdc",
            )

            if error or not response:
                return None

            data = response.get("data", {})
            hits = data.get("hits", [])

            if not hits:
                return None

            # Get the first hit
            hit = hits[0]
            ssm_id = hit.get("ssm_id")
            cosmic_id = hit.get("cosmic_id")

            # For gene_aa_change searches, verify we have the right variant
            if search_field == "gene_aa_change":
                gene_aa_changes = hit.get("gene_aa_change", [])
                if (
                    isinstance(gene_aa_changes, list)
                    and search_value not in gene_aa_changes
                ):
                    # This SSM has multiple AA changes, but not the one we're looking for
                    return None

            if not ssm_id:
                return None

            # Now query SSM occurrences to get project information
            occ_params = {
                "filters": json.dumps({
                    "op": "in",
                    "content": {"field": "ssm.ssm_id", "value": [ssm_id]},
                }),
                "fields": "case.project.project_id",
                "format": "json",
                "size": "2000",  # Get more occurrences
            }

            occ_response, occ_error = await http_client.request_api(
                url="https://api.gdc.cancer.gov/ssm_occurrences",
                method="GET",
                request=occ_params,
                domain="gdc",
            )

            if occ_error or not occ_response:
                # Return basic info without occurrence data
                cosmic_id_str = (
                    cosmic_id[0]
                    if isinstance(cosmic_id, list) and cosmic_id
                    else cosmic_id
                )
                return TCGAVariantData(
                    cosmic_id=cosmic_id_str,
                    tumor_types=[],
                    affected_cases=0,
                    consequence_type="missense_variant",  # Most COSMIC variants are missense
                )

            # Process occurrence data
            occ_data = occ_response.get("data", {})
            occ_hits = occ_data.get("hits", [])

            # Count by project
            project_counts = {}
            for occ in occ_hits:
                case = occ.get("case", {})
                project = case.get("project", {})
                if project_id := project.get("project_id"):
                    project_counts[project_id] = (
                        project_counts.get(project_id, 0) + 1
                    )

            # Extract tumor types
            tumor_types = []
            total_cases = 0
            for project_id, count in project_counts.items():
                # Extract tumor type from project ID
                # TCGA format: "TCGA-LUAD" -> "LUAD"
                # Other formats: "MMRF-COMMPASS" -> "MMRF-COMMPASS", "CPTAC-3" -> "CPTAC-3"
                if project_id.startswith("TCGA-") and "-" in project_id:
                    tumor_type = project_id.split("-")[-1]
                    tumor_types.append(tumor_type)
                else:
                    # For non-TCGA projects, use the full project ID
                    tumor_types.append(project_id)
                total_cases += count

            # Handle cosmic_id as list
            cosmic_id_str = (
                cosmic_id[0]
                if isinstance(cosmic_id, list) and cosmic_id
                else cosmic_id
            )

            return TCGAVariantData(
                cosmic_id=cosmic_id_str,
                tumor_types=tumor_types,
                affected_cases=total_cases,
                consequence_type="missense_variant",  # Default for now
            )

        except (KeyError, ValueError, TypeError, IndexError) as e:
            # Log the error for debugging while gracefully handling API response issues
            # KeyError: Missing expected fields in API response
            # ValueError: Invalid data format or conversion issues
            # TypeError: Unexpected data types in response
            # IndexError: Array access issues with response data
            logger.warning(
                f"Failed to fetch TCGA variant data for {variant_id}: {type(e).__name__}: {e}"
            )
            return None


class ThousandGenomesClient:
    """Client for 1000 Genomes data via Ensembl REST API."""

    def _extract_population_frequencies(
        self, populations: list[dict]
    ) -> dict[str, Any]:
        """Extract population frequencies from Ensembl response."""
        # Note: Multiple entries per population (one per allele), we want the alternate allele frequency
        # The reference allele will have higher frequency for rare variants
        pop_data: dict[str, float] = {}

        for pop in populations:
            pop_name = pop.get("population", "")
            frequency = pop.get("frequency", 0)

            # Map 1000 Genomes population codes - taking the minor allele frequency
            if pop_name == "1000GENOMES:phase_3:ALL":
                if "global_maf" not in pop_data or frequency < pop_data.get(
                    "global_maf", 1
                ):
                    pop_data["global_maf"] = frequency
            elif pop_name == "1000GENOMES:phase_3:AFR":
                if "afr_maf" not in pop_data or frequency < pop_data.get(
                    "afr_maf", 1
                ):
                    pop_data["afr_maf"] = frequency
            elif pop_name == "1000GENOMES:phase_3:AMR":
                if "amr_maf" not in pop_data or frequency < pop_data.get(
                    "amr_maf", 1
                ):
                    pop_data["amr_maf"] = frequency
            elif pop_name == "1000GENOMES:phase_3:EAS":
                if "eas_maf" not in pop_data or frequency < pop_data.get(
                    "eas_maf", 1
                ):
                    pop_data["eas_maf"] = frequency
            elif pop_name == "1000GENOMES:phase_3:EUR":
                if "eur_maf" not in pop_data or frequency < pop_data.get(
                    "eur_maf", 1
                ):
                    pop_data["eur_maf"] = frequency
            elif pop_name == "1000GENOMES:phase_3:SAS" and (
                "sas_maf" not in pop_data
                or frequency < pop_data.get("sas_maf", 1)
            ):
                pop_data["sas_maf"] = frequency

        return pop_data

    async def get_variant_data(
        self, variant_id: str
    ) -> ThousandGenomesData | None:
        """Fetch variant data from 1000 Genomes via Ensembl."""
        try:
            # Try to get rsID or use the variant ID directly
            encoded_id = quote(variant_id, safe="")
            url = f"{ENSEMBL_VARIATION_ENDPOINT}/{encoded_id}"

            # Request with pops=1 to get population data
            params = {"content-type": "application/json", "pops": "1"}

            response, error = await http_client.request_api(
                url=url,
                method="GET",
                request=params,
                domain="ensembl",
            )

            if error or not response:
                return None

            # Extract population frequencies
            populations = response.get("populations", [])
            pop_data = self._extract_population_frequencies(populations)

            # Get most severe consequence
            consequence = None
            if mappings := response.get("mappings", []):
                # Extract consequences from transcript consequences
                all_consequences = []
                for mapping in mappings:
                    if transcript_consequences := mapping.get(
                        "transcript_consequences", []
                    ):
                        for tc in transcript_consequences:
                            if consequence_terms := tc.get(
                                "consequence_terms", []
                            ):
                                all_consequences.extend(consequence_terms)

                if all_consequences:
                    # Take the first unique consequence
                    seen = set()
                    unique_consequences = []
                    for c in all_consequences:
                        if c not in seen:
                            seen.add(c)
                            unique_consequences.append(c)
                    consequence = (
                        unique_consequences[0] if unique_consequences else None
                    )

            # Only return data if we found population frequencies
            if pop_data:
                return ThousandGenomesData(
                    **pop_data,
                    ancestral_allele=response.get("ancestral_allele"),
                    most_severe_consequence=consequence,
                )
            else:
                # No population data found
                return None

        except (KeyError, ValueError, TypeError, AttributeError) as e:
            # Log the error for debugging while gracefully handling API response issues
            # KeyError: Missing expected fields in API response
            # ValueError: Invalid data format or conversion issues
            # TypeError: Unexpected data types in response
            # AttributeError: Missing attributes on response objects
            logger.warning(
                f"Failed to fetch 1000 Genomes data for {variant_id}: {type(e).__name__}: {e}"
            )
            return None


class ExternalVariantAggregator:
    """Aggregates variant data from multiple external sources."""

    def __init__(self):
        self.tcga_client = TCGAClient()
        self.thousand_genomes_client = ThousandGenomesClient()
        # Import here to avoid circular imports
        from .cbio_external_client import CBioPortalExternalClient

        self.cbioportal_client = CBioPortalExternalClient()

    def _extract_gene_aa_change(
        self, variant_data: dict[str, Any]
    ) -> str | None:
        """Extract gene and AA change in format like 'BRAF V600A' from variant data."""
        logger.info("_extract_gene_aa_change called")
        try:
            # First try to get gene name from CADD data
            gene_name = None
            if (cadd := variant_data.get("cadd")) and (
                gene := cadd.get("gene")
            ):
                gene_name = gene.get("genename")

            # If not found in CADD, try other sources
            if not gene_name:
                # Try docm
                if docm := variant_data.get("docm"):
                    gene_name = docm.get("gene") or docm.get("genename")

                # Try dbnsfp
                if not gene_name and (dbnsfp := variant_data.get("dbnsfp")):
                    gene_name = dbnsfp.get("genename")

            if not gene_name:
                return None

            # Now try to get the protein change
            aa_change = None

            # Try to get from docm first (it has clean p.V600A format)
            if (docm := variant_data.get("docm")) and (
                aa := docm.get("aa_change")
            ):
                # Convert p.V600A to V600A
                aa_change = aa.replace("p.", "")

            # Try hgvsp if not found
            if (
                not aa_change
                and (hgvsp_list := variant_data.get("hgvsp"))
                and isinstance(hgvsp_list, list)
                and hgvsp_list
            ):
                # Take the first one and clean it
                hgvsp = hgvsp_list[0]
                # Remove p. prefix
                aa_change = hgvsp.replace("p.", "")
                # Handle formats like Val600Ala -> V600A
                if "Val" in aa_change or "Ala" in aa_change:
                    # Try to extract the short form
                    match = re.search(r"[A-Z]\d+[A-Z]", aa_change)
                    if match:
                        aa_change = match.group()

            # Try CADD data
            if (
                not aa_change
                and (cadd := variant_data.get("cadd"))
                and (gene_info := cadd.get("gene"))
                and (prot := gene_info.get("prot"))
            ):
                protpos = prot.get("protpos")
                if protpos and cadd.get("oaa") and cadd.get("naa"):
                    aa_change = f"{cadd['oaa']}{protpos}{cadd['naa']}"

            if gene_name and aa_change:
                result = f"{gene_name} {aa_change}"
                logger.info(f"Extracted gene/AA change: {result}")
                return result

            logger.warning(
                f"Failed to extract gene/AA change: gene_name={gene_name}, aa_change={aa_change}"
            )
            return None
        except (
            KeyError,
            ValueError,
            TypeError,
            AttributeError,
            re.error,
        ) as e:
            # Log the error for debugging while gracefully handling data extraction issues
            # KeyError: Missing expected fields in variant data
            # ValueError: Invalid data format or conversion issues
            # TypeError: Unexpected data types in variant data
            # AttributeError: Missing attributes on data objects
            # re.error: Regular expression matching errors
            logger.warning(
                f"Failed to extract gene/AA change from variant data: {type(e).__name__}: {e}"
            )
            return None

    async def get_enhanced_annotations(
        self,
        variant_id: str,
        include_tcga: bool = True,
        include_1000g: bool = True,
        include_cbioportal: bool = True,
        variant_data: dict[str, Any] | None = None,
    ) -> EnhancedVariantAnnotation:
        """Fetch and aggregate variant annotations from external sources.

        Args:
            variant_id: The variant identifier (rsID or HGVS)
            include_tcga: Whether to include TCGA data
            include_1000g: Whether to include 1000 Genomes data
            include_cbioportal: Whether to include cBioPortal data
            variant_data: Optional variant data from MyVariant.info to extract gene/protein info
        """
        logger.info(
            f"get_enhanced_annotations called for {variant_id}, include_cbioportal={include_cbioportal}"
        )
        tasks: list[Any] = []
        task_names = []

        # Extract gene/AA change once for sources that need it
        gene_aa_change = None
        if variant_data:
            logger.info(
                f"Extracting gene/AA from variant_data keys: {list(variant_data.keys())}"
            )
            gene_aa_change = self._extract_gene_aa_change(variant_data)
        else:
            logger.warning("No variant_data provided for gene/AA extraction")

        if include_tcga:
            # Try to extract gene and protein change from variant data for TCGA
            tcga_id = gene_aa_change if gene_aa_change else variant_id
            tasks.append(self.tcga_client.get_variant_data(tcga_id))
            task_names.append("tcga")

        if include_1000g:
            tasks.append(
                self.thousand_genomes_client.get_variant_data(variant_id)
            )
            task_names.append("thousand_genomes")

        if include_cbioportal and gene_aa_change:
            # cBioPortal requires gene/AA format
            logger.info(
                f"Adding cBioPortal task with gene_aa_change: {gene_aa_change}"
            )
            tasks.append(
                self.cbioportal_client.get_variant_data(gene_aa_change)
            )
            task_names.append("cbioportal")
        elif include_cbioportal and not gene_aa_change:
            logger.warning(
                "Skipping cBioPortal: no gene/AA change could be extracted"
            )

        # Run all queries in parallel
        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Build the enhanced annotation
        annotation = EnhancedVariantAnnotation(variant_id=variant_id)

        for _i, (result, name) in enumerate(
            zip(results, task_names, strict=False)
        ):
            if isinstance(result, Exception):
                annotation.error_sources.append(name)
            elif result is not None:
                setattr(annotation, name, result)
            else:
                # No data found for this source
                pass

        return annotation


def format_enhanced_annotations(
    annotation: EnhancedVariantAnnotation,
) -> dict[str, Any]:
    """Format enhanced annotations for display."""
    formatted: dict[str, Any] = {
        "variant_id": annotation.variant_id,
        "external_annotations": {},
    }

    external_annot = formatted["external_annotations"]

    if annotation.tcga:
        external_annot["tcga"] = {
            "tumor_types": annotation.tcga.tumor_types,
            "affected_cases": annotation.tcga.affected_cases,
            "cosmic_id": annotation.tcga.cosmic_id,
            "consequence": annotation.tcga.consequence_type,
        }

    if annotation.thousand_genomes:
        external_annot["1000_genomes"] = {
            "global_maf": annotation.thousand_genomes.global_maf,
            "population_frequencies": {
                "african": annotation.thousand_genomes.afr_maf,
                "american": annotation.thousand_genomes.amr_maf,
                "east_asian": annotation.thousand_genomes.eas_maf,
                "european": annotation.thousand_genomes.eur_maf,
                "south_asian": annotation.thousand_genomes.sas_maf,
            },
            "ancestral_allele": annotation.thousand_genomes.ancestral_allele,
            "consequence": annotation.thousand_genomes.most_severe_consequence,
        }

    if annotation.cbioportal:
        cbio_data: dict[str, Any] = {
            "studies": annotation.cbioportal.studies,
            "total_cases": annotation.cbioportal.total_cases,
        }

        # Add cancer type distribution if available
        if annotation.cbioportal.cancer_type_distribution:
            cbio_data["cancer_types"] = (
                annotation.cbioportal.cancer_type_distribution
            )

        # Add mutation type distribution if available
        if annotation.cbioportal.mutation_types:
            cbio_data["mutation_types"] = annotation.cbioportal.mutation_types

        # Add hotspot count if > 0
        if annotation.cbioportal.hotspot_count > 0:
            cbio_data["hotspot_samples"] = annotation.cbioportal.hotspot_count

        # Add mean VAF if available
        if annotation.cbioportal.mean_vaf is not None:
            cbio_data["mean_vaf"] = annotation.cbioportal.mean_vaf

        # Add sample type distribution if available
        if annotation.cbioportal.sample_types:
            cbio_data["sample_types"] = annotation.cbioportal.sample_types

        external_annot["cbioportal"] = cbio_data

    if annotation.error_sources:
        external_annot["errors"] = annotation.error_sources

    return formatted
