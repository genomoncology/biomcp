"""Query parser for unified search language in BioMCP."""

from dataclasses import dataclass
from enum import Enum
from typing import Any


class Operator(str, Enum):
    """Query operators."""

    EQ = ":"
    GT = ">"
    LT = "<"
    GTE = ">="
    LTE = "<="
    RANGE = ".."
    AND = "AND"
    OR = "OR"
    NOT = "NOT"


class FieldType(str, Enum):
    """Field data types."""

    STRING = "string"
    NUMBER = "number"
    DATE = "date"
    ENUM = "enum"
    BOOLEAN = "boolean"


@dataclass
class FieldDefinition:
    """Definition of a searchable field."""

    name: str
    domain: str  # "trials", "articles", "variants", "cross"
    type: FieldType
    operators: list[str]
    example_values: list[str]
    description: str
    underlying_api_field: str
    aliases: list[str] | None = None


@dataclass
class QueryTerm:
    """Parsed query term."""

    field: str
    operator: Operator
    value: Any
    domain: str | None = None
    is_negated: bool = False


@dataclass
class ParsedQuery:
    """Parsed query structure."""

    terms: list[QueryTerm]
    cross_domain_fields: dict[str, Any]
    domain_specific_fields: dict[str, dict[str, Any]]
    raw_query: str


class QueryParser:
    """Parser for unified search queries."""

    def __init__(self):
        self.field_registry = self._build_field_registry()

    def _build_field_registry(self) -> dict[str, FieldDefinition]:
        """Build the field registry with all searchable fields."""
        registry = {}

        # Cross-domain fields
        cross_domain_fields = [
            FieldDefinition(
                name="gene",
                domain="cross",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["BRAF", "TP53", "EGFR"],
                description="Gene symbol",
                underlying_api_field="gene",
            ),
            FieldDefinition(
                name="variant",
                domain="cross",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["V600E", "L858R", "rs113488022"],
                description="Variant notation or rsID",
                underlying_api_field="variant",
            ),
            FieldDefinition(
                name="disease",
                domain="cross",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["melanoma", "lung cancer", "diabetes"],
                description="Disease or condition",
                underlying_api_field="disease",
            ),
        ]

        # Trial-specific fields
        trial_fields = [
            FieldDefinition(
                name="trials.condition",
                domain="trials",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["melanoma", "lung cancer"],
                description="Clinical trial condition",
                underlying_api_field="conditions",
            ),
            FieldDefinition(
                name="trials.intervention",
                domain="trials",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["osimertinib", "pembrolizumab"],
                description="Trial intervention",
                underlying_api_field="interventions",
            ),
            FieldDefinition(
                name="trials.phase",
                domain="trials",
                type=FieldType.ENUM,
                operators=[Operator.EQ],
                example_values=["1", "2", "3", "4"],
                description="Trial phase",
                underlying_api_field="phase",
            ),
            FieldDefinition(
                name="trials.status",
                domain="trials",
                type=FieldType.ENUM,
                operators=[Operator.EQ],
                example_values=["recruiting", "active", "completed"],
                description="Trial recruitment status",
                underlying_api_field="recruiting_status",
            ),
        ]

        # Article-specific fields
        article_fields = [
            FieldDefinition(
                name="articles.title",
                domain="articles",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["EGFR mutations", "cancer therapy"],
                description="Article title",
                underlying_api_field="title",
            ),
            FieldDefinition(
                name="articles.author",
                domain="articles",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["Smith J", "Johnson A"],
                description="Article author",
                underlying_api_field="author",
            ),
            FieldDefinition(
                name="articles.journal",
                domain="articles",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["Nature", "Science", "Cell"],
                description="Journal name",
                underlying_api_field="journal",
            ),
            FieldDefinition(
                name="articles.date",
                domain="articles",
                type=FieldType.DATE,
                operators=[Operator.GT, Operator.LT, Operator.RANGE],
                example_values=[">2023-01-01", "2023-01-01..2024-01-01"],
                description="Publication date",
                underlying_api_field="date",
            ),
        ]

        # Variant-specific fields
        variant_fields = [
            FieldDefinition(
                name="variants.rsid",
                domain="variants",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["rs113488022", "rs121913529"],
                description="dbSNP rsID",
                underlying_api_field="rsid",
            ),
            FieldDefinition(
                name="variants.gene",
                domain="variants",
                type=FieldType.STRING,
                operators=[Operator.EQ],
                example_values=["BRAF", "TP53"],
                description="Gene containing variant",
                underlying_api_field="gene",
            ),
            FieldDefinition(
                name="variants.significance",
                domain="variants",
                type=FieldType.ENUM,
                operators=[Operator.EQ],
                example_values=["pathogenic", "benign", "uncertain"],
                description="Clinical significance",
                underlying_api_field="significance",
            ),
            FieldDefinition(
                name="variants.frequency",
                domain="variants",
                type=FieldType.NUMBER,
                operators=[Operator.LT, Operator.GT],
                example_values=["<0.01", ">0.05"],
                description="Population allele frequency",
                underlying_api_field="frequency",
            ),
        ]

        # Build registry
        for field_list in [
            cross_domain_fields,
            trial_fields,
            article_fields,
            variant_fields,
        ]:
            for field in field_list:
                registry[field.name] = field

        return registry

    def parse(self, query: str) -> ParsedQuery:
        """Parse a unified search query."""
        # Simple tokenization - in production, use a proper parser
        terms = self._tokenize(query)
        parsed_terms = []

        cross_domain = {}
        domain_specific: dict[str, dict[str, Any]] = {
            "trials": {},
            "articles": {},
            "variants": {},
        }

        for term in terms:
            if ":" in term:
                field, value = term.split(":", 1)

                # Check if it's a known field
                if field in self.field_registry:
                    field_def = self.field_registry[field]
                    parsed_term = QueryTerm(
                        field=field,
                        operator=Operator.EQ,
                        value=value.strip('"'),
                        domain=field_def.domain,
                    )
                    parsed_terms.append(parsed_term)

                    # Categorize the term
                    if field_def.domain == "cross":
                        cross_domain[field] = value.strip('"')
                    else:
                        domain = (
                            field.split(".")[0]
                            if "." in field
                            else field_def.domain
                        )
                        if domain not in domain_specific:
                            domain_specific[domain] = {}
                        field_name = (
                            field.split(".")[-1] if "." in field else field
                        )
                        domain_specific[domain][field_name] = value.strip('"')

        return ParsedQuery(
            terms=parsed_terms,
            cross_domain_fields=cross_domain,
            domain_specific_fields=domain_specific,
            raw_query=query,
        )

    def _tokenize(self, query: str) -> list[str]:
        """Simple tokenizer for query strings."""
        # This is a simplified tokenizer - in production, use a proper lexer
        # For now, split on AND/OR/NOT while preserving field:value pairs
        tokens = []
        current_token = ""
        in_quotes = False

        for char in query:
            if char == '"':
                in_quotes = not in_quotes
                current_token += char
            elif char == " " and not in_quotes:
                if current_token:
                    tokens.append(current_token)
                    current_token = ""
            else:
                current_token += char

        if current_token:
            tokens.append(current_token)

        # Filter out boolean operators for now
        return [t for t in tokens if t not in ["AND", "OR", "NOT"]]

    def get_schema(self) -> dict[str, Any]:
        """Get the complete field schema for discovery."""
        schema: dict[str, Any] = {
            "domains": ["trials", "articles", "variants"],
            "cross_domain_fields": {},
            "domain_fields": {"trials": {}, "articles": {}, "variants": {}},
            "operators": [op.value for op in Operator],
            "examples": [
                "gene:BRAF AND trials.condition:melanoma",
                "articles.date:>2023 AND disease:cancer",
                "variants.significance:pathogenic AND gene:TP53",
            ],
        }

        for field_name, field_def in self.field_registry.items():
            field_info = {
                "type": field_def.type.value,
                "operators": field_def.operators,
                "examples": field_def.example_values,
                "description": field_def.description,
            }

            if field_def.domain == "cross":
                schema["cross_domain_fields"][field_name] = field_info
            else:
                domain = field_name.split(".")[0]
                field_short_name = field_name.split(".")[-1]
                schema["domain_fields"][domain][field_short_name] = field_info

        return schema
