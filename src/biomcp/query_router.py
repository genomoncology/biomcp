"""Query router for unified search in BioMCP."""

import asyncio
from dataclasses import dataclass
from typing import Any

from biomcp.articles.search import PubmedRequest
from biomcp.articles.unified import search_articles_unified
from biomcp.query_parser import ParsedQuery
from biomcp.trials.search import TrialQuery, search_trials
from biomcp.variants.search import VariantQuery, search_variants


@dataclass
class RoutingPlan:
    """Plan for routing a query to appropriate tools."""

    tools_to_call: list[str]
    field_mappings: dict[str, dict[str, Any]]
    coordination_strategy: str = "parallel"


class QueryRouter:
    """Routes unified queries to appropriate domain-specific tools."""

    def route(self, parsed_query: ParsedQuery) -> RoutingPlan:
        """Determine which tools to call based on query fields."""
        tools_to_call = []
        field_mappings = {}

        # Check which domains are referenced
        domains_referenced = self._get_referenced_domains(parsed_query)

        # Build field mappings for each domain
        domain_mappers = {
            "articles": ("article_searcher", self._map_article_fields),
            "trials": ("trial_searcher", self._map_trial_fields),
            "variants": ("variant_searcher", self._map_variant_fields),
            "genes": ("gene_searcher", self._map_gene_fields),
            "drugs": ("drug_searcher", self._map_drug_fields),
            "diseases": ("disease_searcher", self._map_disease_fields),
        }

        for domain, (tool_name, mapper_func) in domain_mappers.items():
            if domain in domains_referenced:
                tools_to_call.append(tool_name)
                field_mappings[tool_name] = mapper_func(parsed_query)

        return RoutingPlan(
            tools_to_call=tools_to_call,
            field_mappings=field_mappings,
            coordination_strategy="parallel",
        )

    def _get_referenced_domains(self, parsed_query: ParsedQuery) -> set[str]:
        """Get all domains referenced in the query."""
        domains_referenced = set()

        # Check domain-specific fields
        for domain, fields in parsed_query.domain_specific_fields.items():
            if fields:
                domains_referenced.add(domain)

        # Check cross-domain fields (these trigger multiple searches)
        if parsed_query.cross_domain_fields:
            cross_domain_mappings = {
                "gene": ["articles", "variants", "genes", "trials"],
                "disease": ["articles", "trials", "diseases"],
                "variant": ["articles", "variants"],
                "chemical": ["articles", "trials", "drugs"],
                "drug": ["articles", "trials", "drugs"],
            }

            for field, domains in cross_domain_mappings.items():
                if field in parsed_query.cross_domain_fields:
                    domains_referenced.update(domains)

        return domains_referenced

    def _map_article_fields(self, parsed_query: ParsedQuery) -> dict[str, Any]:
        """Map query fields to article searcher parameters."""
        mapping: dict[str, Any] = {}

        # Map cross-domain fields
        if "gene" in parsed_query.cross_domain_fields:
            mapping["genes"] = [parsed_query.cross_domain_fields["gene"]]
        if "disease" in parsed_query.cross_domain_fields:
            mapping["diseases"] = [parsed_query.cross_domain_fields["disease"]]
        if "variant" in parsed_query.cross_domain_fields:
            mapping["variants"] = [parsed_query.cross_domain_fields["variant"]]

        # Map article-specific fields
        article_fields = parsed_query.domain_specific_fields.get(
            "articles", {}
        )
        if "title" in article_fields:
            mapping["keywords"] = [article_fields["title"]]
        if "author" in article_fields:
            mapping["keywords"] = mapping.get("keywords", []) + [
                article_fields["author"]
            ]
        if "journal" in article_fields:
            mapping["keywords"] = mapping.get("keywords", []) + [
                article_fields["journal"]
            ]

        # Extract mutation patterns from raw query
        import re

        raw_query = parsed_query.raw_query
        # Look for mutation patterns like F57Y, F57*, V600E
        mutation_patterns = re.findall(r"\b[A-Z]\d+[A-Z*]\b", raw_query)
        if mutation_patterns:
            if "keywords" not in mapping:
                mapping["keywords"] = []
            mapping["keywords"].extend(mutation_patterns)

        return mapping

    def _map_trial_fields(self, parsed_query: ParsedQuery) -> dict[str, Any]:
        """Map query fields to trial searcher parameters."""
        mapping: dict[str, Any] = {}

        # Map cross-domain fields
        if "disease" in parsed_query.cross_domain_fields:
            mapping["conditions"] = [
                parsed_query.cross_domain_fields["disease"]
            ]

        # Gene searches in trials might look for targeted therapies
        if "gene" in parsed_query.cross_domain_fields:
            gene = parsed_query.cross_domain_fields["gene"]
            # Search for gene-targeted interventions
            mapping["keywords"] = [gene]

        # Map trial-specific fields
        trial_fields = parsed_query.domain_specific_fields.get("trials", {})
        if "condition" in trial_fields:
            mapping["conditions"] = [trial_fields["condition"]]
        if "intervention" in trial_fields:
            mapping["interventions"] = [trial_fields["intervention"]]
        if "phase" in trial_fields:
            mapping["phase"] = f"PHASE{trial_fields['phase']}"
        if "status" in trial_fields:
            mapping["recruiting_status"] = trial_fields["status"].upper()

        return mapping

    def _map_variant_fields(self, parsed_query: ParsedQuery) -> dict[str, Any]:
        """Map query fields to variant searcher parameters."""
        mapping: dict[str, Any] = {}

        # Map cross-domain fields
        if "gene" in parsed_query.cross_domain_fields:
            mapping["gene"] = parsed_query.cross_domain_fields["gene"]
        if "variant" in parsed_query.cross_domain_fields:
            variant = parsed_query.cross_domain_fields["variant"]
            # Check if it's an rsID or protein change
            if variant.startswith("rs"):
                mapping["rsid"] = variant
            else:
                mapping["hgvsp"] = variant

        # Map variant-specific fields
        variant_fields = parsed_query.domain_specific_fields.get(
            "variants", {}
        )
        if "rsid" in variant_fields:
            mapping["rsid"] = variant_fields["rsid"]
        if "gene" in variant_fields:
            mapping["gene"] = variant_fields["gene"]
        if "significance" in variant_fields:
            mapping["significance"] = variant_fields["significance"]
        if "frequency" in variant_fields:
            # Parse frequency operators
            freq = variant_fields["frequency"]
            if freq.startswith("<"):
                mapping["max_frequency"] = float(freq[1:])
            elif freq.startswith(">"):
                mapping["min_frequency"] = float(freq[1:])

        return mapping

    def _map_gene_fields(self, parsed_query: ParsedQuery) -> dict[str, Any]:
        """Map query fields to gene searcher parameters."""
        mapping: dict[str, Any] = {}

        # Map cross-domain fields
        if "gene" in parsed_query.cross_domain_fields:
            mapping["query"] = parsed_query.cross_domain_fields["gene"]

        # Map gene-specific fields
        gene_fields = parsed_query.domain_specific_fields.get("genes", {})
        if "symbol" in gene_fields:
            mapping["query"] = gene_fields["symbol"]
        elif "name" in gene_fields:
            mapping["query"] = gene_fields["name"]
        elif "type" in gene_fields:
            mapping["type_of_gene"] = gene_fields["type"]

        return mapping

    def _map_drug_fields(self, parsed_query: ParsedQuery) -> dict[str, Any]:
        """Map query fields to drug searcher parameters."""
        mapping: dict[str, Any] = {}

        # Map cross-domain fields
        if "chemical" in parsed_query.cross_domain_fields:
            mapping["query"] = parsed_query.cross_domain_fields["chemical"]
        elif "drug" in parsed_query.cross_domain_fields:
            mapping["query"] = parsed_query.cross_domain_fields["drug"]

        # Map drug-specific fields
        drug_fields = parsed_query.domain_specific_fields.get("drugs", {})
        if "name" in drug_fields:
            mapping["query"] = drug_fields["name"]
        elif "tradename" in drug_fields:
            mapping["query"] = drug_fields["tradename"]
        elif "indication" in drug_fields:
            mapping["indication"] = drug_fields["indication"]

        return mapping

    def _map_disease_fields(self, parsed_query: ParsedQuery) -> dict[str, Any]:
        """Map query fields to disease searcher parameters."""
        mapping: dict[str, Any] = {}

        # Map cross-domain fields
        if "disease" in parsed_query.cross_domain_fields:
            mapping["query"] = parsed_query.cross_domain_fields["disease"]

        # Map disease-specific fields
        disease_fields = parsed_query.domain_specific_fields.get(
            "diseases", {}
        )
        if "name" in disease_fields:
            mapping["query"] = disease_fields["name"]
        elif "mondo" in disease_fields:
            mapping["query"] = disease_fields["mondo"]
        elif "synonym" in disease_fields:
            mapping["query"] = disease_fields["synonym"]

        return mapping


async def execute_routing_plan(
    plan: RoutingPlan, output_json: bool = True
) -> dict[str, Any]:
    """Execute a routing plan by calling the appropriate tools."""
    tasks = []
    task_names = []

    for tool_name in plan.tools_to_call:
        params = plan.field_mappings[tool_name]

        if tool_name == "article_searcher":
            request = PubmedRequest(**params)
            tasks.append(
                search_articles_unified(
                    request,
                    include_pubmed=True,
                    include_preprints=False,
                    output_json=output_json,
                )
            )
            task_names.append("articles")

        elif tool_name == "trial_searcher":
            query = TrialQuery(**params)
            tasks.append(search_trials(query, output_json=output_json))
            task_names.append("trials")

        elif tool_name == "variant_searcher":
            variant_query = VariantQuery(**params)
            tasks.append(
                search_variants(variant_query, output_json=output_json)
            )
            task_names.append("variants")

        elif tool_name == "gene_searcher":
            # For gene search, we'll use the BioThingsClient directly
            from biomcp.integrations.biothings_client import BioThingsClient

            client = BioThingsClient()
            query_str = params.get("query", "")
            tasks.append(_search_genes(client, query_str, output_json))
            task_names.append("genes")

        elif tool_name == "drug_searcher":
            # For drug search, we'll use the BioThingsClient directly
            from biomcp.integrations.biothings_client import BioThingsClient

            client = BioThingsClient()
            query_str = params.get("query", "")
            tasks.append(_search_drugs(client, query_str, output_json))
            task_names.append("drugs")

        elif tool_name == "disease_searcher":
            # For disease search, we'll use the BioThingsClient directly
            from biomcp.integrations.biothings_client import BioThingsClient

            client = BioThingsClient()
            query_str = params.get("query", "")
            tasks.append(_search_diseases(client, query_str, output_json))
            task_names.append("diseases")

    # Execute all searches in parallel
    results = await asyncio.gather(*tasks, return_exceptions=True)

    # Package results
    output: dict[str, Any] = {}
    for name, result in zip(task_names, results, strict=False):
        if isinstance(result, Exception):
            output[name] = {"error": str(result)}
        else:
            output[name] = result

    return output


async def _search_genes(client, query: str, output_json: bool) -> Any:
    """Search for genes using BioThingsClient."""
    results = await client._query_gene(query)
    if not results:
        return [] if output_json else "No genes found"

    # Fetch full details for each result
    detailed_results = []
    for result in results[:10]:  # Limit to 10 results
        gene_id = result.get("_id")
        if gene_id:
            full_gene = await client._get_gene_by_id(gene_id)
            if full_gene:
                detailed_results.append(full_gene.model_dump(by_alias=True))

    if output_json:
        import json

        return json.dumps(detailed_results)
    else:
        return detailed_results


async def _search_drugs(client, query: str, output_json: bool) -> Any:
    """Search for drugs using BioThingsClient."""
    results = await client._query_drug(query)
    if not results:
        return [] if output_json else "No drugs found"

    # Fetch full details for each result
    detailed_results = []
    for result in results[:10]:  # Limit to 10 results
        drug_id = result.get("_id")
        if drug_id:
            full_drug = await client._get_drug_by_id(drug_id)
            if full_drug:
                detailed_results.append(full_drug.model_dump(by_alias=True))

    if output_json:
        import json

        return json.dumps(detailed_results)
    else:
        return detailed_results


async def _search_diseases(client, query: str, output_json: bool) -> Any:
    """Search for diseases using BioThingsClient."""
    results = await client._query_disease(query)
    if not results:
        return [] if output_json else "No diseases found"

    # Fetch full details for each result
    detailed_results = []
    for result in results[:10]:  # Limit to 10 results
        disease_id = result.get("_id")
        if disease_id:
            full_disease = await client._get_disease_by_id(disease_id)
            if full_disease:
                detailed_results.append(full_disease.model_dump(by_alias=True))

    if output_json:
        import json

        return json.dumps(detailed_results)
    else:
        return detailed_results
