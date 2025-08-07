"""
Helper functions for OpenFDA adverse events to reduce complexity.
"""

from collections import Counter
from typing import Any

from .utils import (
    extract_drug_names,
    extract_reactions,
    format_count,
    format_drug_list,
)


def format_search_summary(
    drug: str | None, reaction: str | None, serious: bool | None, total: int
) -> list[str]:
    """Format the search summary section."""
    output = []

    # Add search criteria
    search_desc = []
    if drug:
        search_desc.append(f"**Drug**: {drug}")
    if reaction:
        search_desc.append(f"**Reaction**: {reaction}")
    if serious is not None:
        search_desc.append(f"**Serious Events**: {'Yes' if serious else 'No'}")

    if search_desc:
        output.append(" | ".join(search_desc))
    output.append(
        f"**Total Reports Found**: {format_count(total, 'report')}\n"
    )

    return output


def format_top_reactions(results: list[dict[str, Any]]) -> list[str]:
    """Format top reported reactions from search results."""
    output = []
    all_reactions = []

    for result in results:
        all_reactions.extend(extract_reactions(result))

    if all_reactions:
        reaction_counts = Counter(all_reactions)
        top_reactions = reaction_counts.most_common(10)

        output.append("### Top Reported Reactions:")
        for rxn, count in top_reactions:
            percentage = (count / len(results)) * 100
            output.append(f"- **{rxn}**: {count} reports ({percentage:.1f}%)")
        output.append("")

    return output


def format_report_summary(
    result: dict[str, Any], report_num: int
) -> list[str]:
    """Format a single report summary."""
    output = [f"#### Report {report_num}"]

    # Extract key information
    drugs = extract_drug_names(result)
    reactions = extract_reactions(result)

    # Patient info
    patient = result.get("patient", {})
    age = patient.get("patientonsetage")
    sex_map = {0: "Unknown", 1: "Male", 2: "Female"}
    sex = sex_map.get(patient.get("patientsex"), "Unknown")

    # Serious outcomes
    serious_flag = result.get("serious", "0")
    outcomes = []
    for code in [
        "seriousnessdeath",
        "seriousnesslifethreatening",
        "seriousnesshospitalization",
        "seriousnessdisabling",
    ]:
        if result.get(code) == "1":
            outcomes.append(code.replace("seriousness", "").title())

    # Format output
    output.append(f"- **Drugs**: {format_drug_list(drugs)}")
    output.append(f"- **Reactions**: {', '.join(reactions[:5])}")
    if age:
        output.append(f"- **Patient**: {age} years, {sex}")
    if serious_flag == "1" and outcomes:
        output.append(f"- **Serious Outcome**: {', '.join(outcomes)}")

    # Dates
    receive_date = result.get("receivedate", "")
    if receive_date:
        output.append(
            f"- **Report Date**: {receive_date[:4]}-{receive_date[4:6]}-{receive_date[6:]}"
        )

    output.append("")
    return output


def format_drug_details(drugs: list[dict[str, Any]]) -> list[str]:
    """Format drug information details."""
    from .utils import clean_text

    output = ["### Drug Information"]

    for i, drug in enumerate(drugs, 1):
        output.append(
            f"\n#### Drug {i}: {drug.get('medicinalproduct', 'Unknown')}"
        )

        if "drugindication" in drug:
            output.append(f"- **Indication**: {drug['drugindication']}")

        if "drugdosagetext" in drug:
            dosage = clean_text(drug["drugdosagetext"])
            output.append(f"- **Dosage**: {dosage}")

        if "drugadministrationroute" in drug:
            output.append(f"- **Route**: {drug['drugadministrationroute']}")

        # Drug action taken
        action_map = {
            1: "Drug withdrawn",
            2: "Dose reduced",
            3: "Dose increased",
            4: "Dose not changed",
            5: "Unknown",
            6: "Not applicable",
        }
        action_code = drug.get("actiondrug")
        action = (
            action_map.get(action_code, "Unknown")
            if action_code is not None
            else "Unknown"
        )
        output.append(f"- **Action Taken**: {action}")

    output.append("")
    return output


def format_reaction_details(reactions: list[dict[str, Any]]) -> list[str]:
    """Format adverse reaction details."""
    output = ["### Adverse Reactions"]

    for reaction in reactions:
        rxn_name = reaction.get("reactionmeddrapt", "Unknown")
        outcome_map = {
            1: "Recovered/Resolved",
            2: "Recovering/Resolving",
            3: "Not recovered/Not resolved",
            4: "Recovered/Resolved with sequelae",
            5: "Fatal",
            6: "Unknown",
        }
        outcome_code = reaction.get("reactionoutcome")
        outcome = (
            outcome_map.get(outcome_code, "Unknown")
            if outcome_code is not None
            else "Unknown"
        )
        output.append(f"- **{rxn_name}**: {outcome}")

    output.append("")
    return output


def format_report_metadata(result: dict[str, Any]) -> list[str]:
    """Format report metadata information."""
    output = ["### Report Information"]

    receive_date = result.get("receivedate", "")
    if receive_date:
        formatted_date = (
            f"{receive_date[:4]}-{receive_date[4:6]}-{receive_date[6:]}"
        )
        output.append(f"- **Report Date**: {formatted_date}")

    report_type_map = {
        1: "Spontaneous",
        2: "Report from study",
        3: "Other",
        4: "Not available to sender",
    }
    report_type_code = result.get("reporttype")
    report_type = (
        report_type_map.get(report_type_code, "Unknown")
        if report_type_code is not None
        else "Unknown"
    )
    output.append(f"- **Report Type**: {report_type}")

    # Seriousness
    if result.get("serious") == "1":
        outcomes = []
        if result.get("seriousnessdeath") == "1":
            outcomes.append("Death")
        if result.get("seriousnesslifethreatening") == "1":
            outcomes.append("Life-threatening")
        if result.get("seriousnesshospitalization") == "1":
            outcomes.append("Hospitalization")
        if result.get("seriousnessdisabling") == "1":
            outcomes.append("Disability")
        if result.get("seriousnesscongenitalanomali") == "1":
            outcomes.append("Congenital anomaly")
        if result.get("seriousnessother") == "1":
            outcomes.append("Other serious")

        if outcomes:
            output.append(f"- **Serious Outcomes**: {', '.join(outcomes)}")

    return output
