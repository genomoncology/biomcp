"""Registry for tracking all external HTTP endpoints used by BioMCP."""

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


class EndpointCategory(str, Enum):
    """Categories of external endpoints."""

    BIOMEDICAL_LITERATURE = "biomedical_literature"
    CLINICAL_TRIALS = "clinical_trials"
    VARIANT_DATABASES = "variant_databases"
    CANCER_GENOMICS = "cancer_genomics"
    HEALTH_MONITORING = "health_monitoring"


class DataType(str, Enum):
    """Types of data accessed from endpoints."""

    RESEARCH_ARTICLES = "research_articles"
    CLINICAL_TRIAL_DATA = "clinical_trial_data"
    GENETIC_VARIANTS = "genetic_variants"
    CANCER_MUTATIONS = "cancer_mutations"
    GENE_ANNOTATIONS = "gene_annotations"
    SERVICE_STATUS = "service_status"


@dataclass
class EndpointInfo:
    """Information about an external endpoint."""

    url: str
    category: EndpointCategory
    data_types: list[DataType] = field(default_factory=list)
    description: str = ""
    compliance_notes: str = ""
    rate_limit: str | None = None
    authentication: str | None = None

    @property
    def domain(self) -> str:
        """Extract domain from URL."""
        parsed = urlparse(self.url)
        return parsed.netloc


class EndpointRegistry:
    """Registry for tracking all external endpoints."""

    def __init__(self):
        self._endpoints: dict[str, EndpointInfo] = {}
        self._initialize_known_endpoints()

    def _initialize_known_endpoints(self):
        """Initialize registry with known endpoints."""
        # PubMed/PubTator3
        self.register(
            "pubtator3_search",
            EndpointInfo(
                url="https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/",
                category=EndpointCategory.BIOMEDICAL_LITERATURE,
                data_types=[DataType.RESEARCH_ARTICLES],
                description="PubTator3 API for searching biomedical literature with entity annotations",
                compliance_notes="Public NIH/NCBI service, no PII transmitted",
                rate_limit="20 requests/second",
            ),
        )

        self.register(
            "pubtator3_export",
            EndpointInfo(
                url="https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson",
                category=EndpointCategory.BIOMEDICAL_LITERATURE,
                data_types=[DataType.RESEARCH_ARTICLES],
                description="PubTator3 API for fetching full article annotations in BioC-JSON format",
                compliance_notes="Public NIH/NCBI service, no PII transmitted",
                rate_limit="20 requests/second",
            ),
        )

        self.register(
            "pubtator3_autocomplete",
            EndpointInfo(
                url="https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/",
                category=EndpointCategory.BIOMEDICAL_LITERATURE,
                data_types=[DataType.GENE_ANNOTATIONS],
                description="PubTator3 API for entity name autocomplete suggestions",
                compliance_notes="Public NIH/NCBI service, no PII transmitted",
                rate_limit="20 requests/second",
            ),
        )

        # ClinicalTrials.gov
        self.register(
            "clinicaltrials_search",
            EndpointInfo(
                url="https://clinicaltrials.gov/api/v2/studies",
                category=EndpointCategory.CLINICAL_TRIALS,
                data_types=[DataType.CLINICAL_TRIAL_DATA],
                description="ClinicalTrials.gov API v2 for searching clinical trials",
                compliance_notes="Public NIH service, may contain trial participant criteria",
                rate_limit="10 requests/second",
            ),
        )

        # MyVariant.info
        self.register(
            "myvariant_query",
            EndpointInfo(
                url="https://myvariant.info/v1/query",
                category=EndpointCategory.VARIANT_DATABASES,
                data_types=[DataType.GENETIC_VARIANTS],
                description="MyVariant.info API for querying genetic variants",
                compliance_notes="Public service aggregating variant databases, no patient data",
                rate_limit="1000 requests/hour (anonymous)",
            ),
        )

        self.register(
            "myvariant_variant",
            EndpointInfo(
                url="https://myvariant.info/v1/variant",
                category=EndpointCategory.VARIANT_DATABASES,
                data_types=[DataType.GENETIC_VARIANTS],
                description="MyVariant.info API for fetching specific variant details",
                compliance_notes="Public service aggregating variant databases, no patient data",
                rate_limit="1000 requests/hour (anonymous)",
            ),
        )

        # Preprint servers
        self.register(
            "biorxiv_api",
            EndpointInfo(
                url="https://api.biorxiv.org/details/biorxiv",
                category=EndpointCategory.BIOMEDICAL_LITERATURE,
                data_types=[DataType.RESEARCH_ARTICLES],
                description="bioRxiv API for searching biology preprints",
                compliance_notes="Public preprint server, no PII transmitted",
                rate_limit="Not specified",
            ),
        )

        self.register(
            "medrxiv_api",
            EndpointInfo(
                url="https://api.biorxiv.org/details/medrxiv",
                category=EndpointCategory.BIOMEDICAL_LITERATURE,
                data_types=[DataType.RESEARCH_ARTICLES],
                description="medRxiv API for searching medical preprints",
                compliance_notes="Public preprint server, no PII transmitted",
                rate_limit="Not specified",
            ),
        )

        self.register(
            "europe_pmc",
            EndpointInfo(
                url="https://www.ebi.ac.uk/europepmc/webservices/rest/search",
                category=EndpointCategory.BIOMEDICAL_LITERATURE,
                data_types=[DataType.RESEARCH_ARTICLES],
                description="Europe PMC REST API for searching biomedical literature",
                compliance_notes="Public EMBL-EBI service, no PII transmitted",
                rate_limit="Not specified",
            ),
        )

        # External variant sources
        self.register(
            "gdc_ssms",
            EndpointInfo(
                url="https://api.gdc.cancer.gov/ssms",
                category=EndpointCategory.VARIANT_DATABASES,
                data_types=[DataType.CANCER_MUTATIONS],
                description="NCI GDC API for somatic mutations",
                compliance_notes="Public NCI service, aggregate cancer genomics data",
                rate_limit="Not specified",
            ),
        )

        self.register(
            "gdc_ssm_occurrences",
            EndpointInfo(
                url="https://api.gdc.cancer.gov/ssm_occurrences",
                category=EndpointCategory.VARIANT_DATABASES,
                data_types=[DataType.CANCER_MUTATIONS],
                description="NCI GDC API for mutation occurrences in cancer samples",
                compliance_notes="Public NCI service, aggregate cancer genomics data",
                rate_limit="Not specified",
            ),
        )

        self.register(
            "ensembl_variation",
            EndpointInfo(
                url="https://rest.ensembl.org/variation/human",
                category=EndpointCategory.VARIANT_DATABASES,
                data_types=[DataType.GENETIC_VARIANTS],
                description="Ensembl REST API for human genetic variation data",
                compliance_notes="Public EMBL-EBI service, population genetics data",
                rate_limit="15 requests/second",
            ),
        )

        self.register(
            "cbioportal_api",
            EndpointInfo(
                url="https://www.cbioportal.org/api",
                category=EndpointCategory.CANCER_GENOMICS,
                data_types=[
                    DataType.CANCER_MUTATIONS,
                    DataType.CLINICAL_TRIAL_DATA,
                ],
                description="cBioPortal API for cancer genomics data",
                compliance_notes="Public MSKCC/Dana-Farber service, aggregate cancer genomics",
                rate_limit="5 requests/second",
                authentication="Optional API token for increased rate limits",
            ),
        )

        # Specific cBioPortal endpoints
        self.register(
            "cbioportal_genes",
            EndpointInfo(
                url="https://www.cbioportal.org/api/genes",
                category=EndpointCategory.CANCER_GENOMICS,
                data_types=[DataType.GENE_ANNOTATIONS],
                description="cBioPortal API for gene information",
                compliance_notes="Public MSKCC/Dana-Farber service, gene metadata",
                rate_limit="5 requests/second",
            ),
        )

        self.register(
            "cbioportal_cancer_types",
            EndpointInfo(
                url="https://www.cbioportal.org/api/cancer-types",
                category=EndpointCategory.CANCER_GENOMICS,
                data_types=[DataType.CANCER_MUTATIONS],
                description="cBioPortal API for cancer type hierarchy",
                compliance_notes="Public MSKCC/Dana-Farber service, cancer type metadata",
                rate_limit="5 requests/second",
            ),
        )

        self.register(
            "cbioportal_molecular_profiles",
            EndpointInfo(
                url="https://www.cbioportal.org/api/molecular-profiles",
                category=EndpointCategory.CANCER_GENOMICS,
                data_types=[DataType.CANCER_MUTATIONS],
                description="cBioPortal API for molecular profiles",
                compliance_notes="Public MSKCC/Dana-Farber service, study metadata",
                rate_limit="5 requests/second",
            ),
        )

        self.register(
            "cbioportal_mutations",
            EndpointInfo(
                url="https://www.cbioportal.org/api/mutations",
                category=EndpointCategory.CANCER_GENOMICS,
                data_types=[DataType.CANCER_MUTATIONS],
                description="cBioPortal API for mutation data",
                compliance_notes="Public MSKCC/Dana-Farber service, aggregate mutation data",
                rate_limit="5 requests/second",
            ),
        )

        self.register(
            "cbioportal_studies",
            EndpointInfo(
                url="https://www.cbioportal.org/api/studies",
                category=EndpointCategory.CANCER_GENOMICS,
                data_types=[
                    DataType.CLINICAL_TRIAL_DATA,
                    DataType.CANCER_MUTATIONS,
                ],
                description="cBioPortal API for cancer studies",
                compliance_notes="Public MSKCC/Dana-Farber service, study metadata",
                rate_limit="5 requests/second",
            ),
        )

    def register(self, key: str, endpoint: EndpointInfo):
        """Register an endpoint for tracking.

        Args:
            key: Unique identifier for the endpoint
            endpoint: Endpoint metadata including URL, description, and compliance notes
        """
        self._endpoints[key] = endpoint

    def get_all_endpoints(self) -> dict[str, EndpointInfo]:
        """Get all registered endpoints.

        Returns:
            Dictionary mapping endpoint keys to their metadata
        """
        return self._endpoints.copy()

    def get_endpoints_by_category(
        self, category: EndpointCategory
    ) -> dict[str, EndpointInfo]:
        """Get endpoints filtered by category.

        Args:
            category: The category to filter by

        Returns:
            Dictionary of endpoints belonging to the specified category
        """
        return {
            key: info
            for key, info in self._endpoints.items()
            if info.category == category
        }

    def get_unique_domains(self) -> set[str]:
        """Get all unique domains accessed by BioMCP.

        Returns:
            Set of unique domain names (e.g., 'api.ncbi.nlm.nih.gov')
        """
        return {info.domain for info in self._endpoints.values()}

    def generate_markdown_report(self) -> str:
        """Generate markdown documentation of all endpoints."""
        lines = [
            "# Third-Party Endpoints Used by BioMCP",
            "",
            "_This file is auto-generated from the endpoint registry._",
            "",
            "## Overview",
            "",
            f"BioMCP connects to {len(self.get_unique_domains())} external domains across {len(self._endpoints)} endpoints.",
            "",
            "## Endpoints by Category",
            "",
        ]

        # Group by category
        for category in EndpointCategory:
            endpoints = self.get_endpoints_by_category(category)
            if not endpoints:
                continue

            lines.append(f"### {category.value.replace('_', ' ').title()}")
            lines.append("")

            for key, info in sorted(endpoints.items()):
                lines.append(f"#### {key}")
                lines.append("")
                lines.append(f"- **URL**: `{info.url}`")
                lines.append(f"- **Description**: {info.description}")
                lines.append(
                    f"- **Data Types**: {', '.join(dt.value for dt in info.data_types)}"
                )
                lines.append(
                    f"- **Rate Limit**: {info.rate_limit or 'Not specified'}"
                )

                if info.authentication:
                    lines.append(
                        f"- **Authentication**: {info.authentication}"
                    )

                if info.compliance_notes:
                    lines.append(
                        f"- **Compliance Notes**: {info.compliance_notes}"
                    )

                lines.append("")

        # Add summary section
        lines.extend([
            "## Domain Summary",
            "",
            "| Domain               | Category              | Endpoints |",
            "| -------------------- | --------------------- | --------- |",
        ])

        domain_stats: dict[str, dict[str, Any]] = {}
        for info in self._endpoints.values():
            domain = info.domain
            if domain not in domain_stats:
                domain_stats[domain] = {
                    "category": info.category.value,
                    "count": 0,
                }
            domain_stats[domain]["count"] = (
                int(domain_stats[domain]["count"]) + 1
            )

        for domain, stats in sorted(domain_stats.items()):
            lines.append(
                f"| {domain} | {stats['category']} | {stats['count']} |"
            )

        lines.extend([
            "",
            "## Compliance and Privacy",
            "",
            "All endpoints accessed by BioMCP:",
            "",
            "- Use publicly available APIs",
            "- Do not transmit personally identifiable information (PII)",
            "- Access only aggregate or de-identified data",
            "- Comply with respective terms of service",
            "",
            "## Network Control",
            "",
            "For air-gapped or restricted environments, BioMCP supports:",
            "",
            "- Offline mode via `BIOMCP_OFFLINE=true` environment variable",
            "- Custom proxy configuration via standard HTTP(S)\\_PROXY variables",
            "- SSL certificate pinning for enhanced security",
            "",
        ])

        return "\n".join(lines)

    def save_markdown_report(self, output_path: Path | None = None):
        """Save markdown report to file."""
        if output_path is None:
            output_path = (
                Path(__file__).parent.parent.parent
                / "THIRD_PARTY_ENDPOINTS.md"
            )

        output_path.write_text(self.generate_markdown_report())
        return output_path


# Global registry instance
_registry = EndpointRegistry()


def get_registry() -> EndpointRegistry:
    """Get the global endpoint registry."""
    return _registry
