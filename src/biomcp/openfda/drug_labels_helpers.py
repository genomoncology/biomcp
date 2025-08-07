"""
Helper functions for OpenFDA drug labels to reduce complexity.
"""

from typing import Any

from .input_validation import sanitize_input
from .utils import clean_text, extract_drug_names, truncate_text


def build_label_search_query(
    name: str | None,
    indication: str | None,
    boxed_warning: bool,
    section: str | None,
) -> str:
    """Build the search query for drug labels."""
    search_parts = []

    if name:
        # Sanitize input to prevent injection
        name = sanitize_input(name, max_length=100)

    if name:
        name_query = (
            f'(openfda.brand_name:"{name}" OR '
            f'openfda.generic_name:"{name}" OR '
            f'openfda.substance_name:"{name}")'
        )
        search_parts.append(name_query)

    if indication:
        # Sanitize indication input
        indication = sanitize_input(indication, max_length=200)
        if indication:
            search_parts.append(f'indications_and_usage:"{indication}"')

    if boxed_warning:
        search_parts.append("_exists_:boxed_warning")

    if section:
        # Map common section names to FDA fields
        section_map = {
            "indications": "indications_and_usage",
            "dosage": "dosage_and_administration",
            "contraindications": "contraindications",
            "warnings": "warnings_and_precautions",
            "adverse": "adverse_reactions",
            "interactions": "drug_interactions",
            "pregnancy": "pregnancy",
            "pediatric": "pediatric_use",
            "geriatric": "geriatric_use",
            "overdose": "overdosage",
        }
        field_name = section_map.get(section.lower(), section)
        search_parts.append(f"_exists_:{field_name}")

    return " AND ".join(search_parts)


def format_label_summary(result: dict[str, Any], index: int) -> list[str]:
    """Format a single drug label summary."""
    output = []

    # Extract drug names
    drug_names = extract_drug_names(result)
    primary_name = drug_names[0] if drug_names else "Unknown Drug"

    output.append(f"#### {index}. {primary_name}")

    # Get OpenFDA data
    openfda = result.get("openfda", {})

    # Show all names if multiple
    if len(drug_names) > 1:
        output.append(f"**Also known as**: {', '.join(drug_names[1:])}")

    # Basic info
    output.extend(_format_label_basic_info(openfda))

    # Boxed warning
    if "boxed_warning" in result:
        warning_text = clean_text(" ".join(result["boxed_warning"]))
        output.append(
            f"\n⚠️ **BOXED WARNING**: {truncate_text(warning_text, 200)}"
        )

    # Key sections
    output.extend(_format_label_key_sections(result))

    # Set ID for retrieval
    if "set_id" in result:
        output.append(f"\n*Label ID: {result['set_id']}*")

    output.append("")
    return output


def _format_label_basic_info(openfda: dict) -> list[str]:
    """Format basic label information from OpenFDA data."""
    output = []

    # Application number
    if app_numbers := openfda.get("application_number", []):
        output.append(f"**FDA Application**: {app_numbers[0]}")

    # Manufacturer
    if manufacturers := openfda.get("manufacturer_name", []):
        output.append(f"**Manufacturer**: {manufacturers[0]}")

    # Route
    if routes := openfda.get("route", []):
        output.append(f"**Route**: {', '.join(routes)}")

    return output


def _format_label_key_sections(result: dict) -> list[str]:
    """Format key label sections."""
    output = []

    # Indications
    if "indications_and_usage" in result:
        indications_text = clean_text(
            " ".join(result["indications_and_usage"])
        )
        output.append(
            f"\n**Indications**: {truncate_text(indications_text, 300)}"
        )

    # Contraindications
    if "contraindications" in result:
        contra_text = clean_text(" ".join(result["contraindications"]))
        output.append(
            f"\n**Contraindications**: {truncate_text(contra_text, 200)}"
        )

    return output


def format_label_header(result: dict[str, Any], set_id: str) -> list[str]:
    """Format the header for detailed drug label."""
    output = []

    drug_names = extract_drug_names(result)
    primary_name = drug_names[0] if drug_names else "Unknown Drug"

    output.append(f"## FDA Drug Label: {primary_name}\n")

    # Basic information
    openfda = result.get("openfda", {})

    if len(drug_names) > 1:
        output.append(f"**Other Names**: {', '.join(drug_names[1:])}")

    output.extend(_format_detailed_metadata(openfda))
    output.append(f"**Label ID**: {set_id}\n")

    return output


def _format_detailed_metadata(openfda: dict) -> list[str]:
    """Format detailed metadata from OpenFDA."""
    output = []

    # FDA application numbers
    if app_numbers := openfda.get("application_number", []):
        output.append(f"**FDA Application**: {', '.join(app_numbers)}")

    # Manufacturers
    if manufacturers := openfda.get("manufacturer_name", []):
        output.append(f"**Manufacturer**: {', '.join(manufacturers)}")

    # Routes of administration
    if routes := openfda.get("route", []):
        output.append(f"**Route of Administration**: {', '.join(routes)}")

    # Pharmacologic class
    if pharm_classes := openfda.get("pharm_class_epc", []):
        output.append(f"**Pharmacologic Class**: {', '.join(pharm_classes)}")

    return output


def format_label_section(
    result: dict[str, Any], section: str, section_titles: dict[str, str]
) -> list[str]:
    """Format a single label section."""
    output: list[str] = []

    if section not in result:
        return output

    title = section_titles.get(section, section.upper().replace("_", " "))
    output.append(f"### {title}\n")

    section_text = result[section]
    if isinstance(section_text, list):
        section_text = " ".join(section_text)

    cleaned_text = clean_text(section_text)

    # For very long sections, provide a truncated version
    if len(cleaned_text) > 3000:
        output.append(truncate_text(cleaned_text, 3000))
        output.append("\n*[Section truncated for brevity]*")
    else:
        output.append(cleaned_text)

    output.append("")
    return output


def get_default_sections() -> list[str]:
    """Get the default sections to display."""
    return [
        "indications_and_usage",
        "dosage_and_administration",
        "contraindications",
        "warnings_and_precautions",
        "adverse_reactions",
        "drug_interactions",
        "use_in_specific_populations",
        "clinical_pharmacology",
        "clinical_studies",
    ]


def get_section_titles() -> dict[str, str]:
    """Get the mapping of section names to display titles."""
    return {
        "indications_and_usage": "INDICATIONS AND USAGE",
        "dosage_and_administration": "DOSAGE AND ADMINISTRATION",
        "contraindications": "CONTRAINDICATIONS",
        "warnings_and_precautions": "WARNINGS AND PRECAUTIONS",
        "adverse_reactions": "ADVERSE REACTIONS",
        "drug_interactions": "DRUG INTERACTIONS",
        "use_in_specific_populations": "USE IN SPECIFIC POPULATIONS",
        "clinical_pharmacology": "CLINICAL PHARMACOLOGY",
        "clinical_studies": "CLINICAL STUDIES",
        "how_supplied": "HOW SUPPLIED",
        "storage_and_handling": "STORAGE AND HANDLING",
        "patient_counseling_information": "PATIENT COUNSELING INFORMATION",
        "pregnancy": "PREGNANCY",
        "nursing_mothers": "NURSING MOTHERS",
        "pediatric_use": "PEDIATRIC USE",
        "geriatric_use": "GERIATRIC USE",
        "overdosage": "OVERDOSAGE",
    }
