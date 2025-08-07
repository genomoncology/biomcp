"""
Helper functions for OpenFDA device events to reduce complexity.
"""

from collections import Counter
from typing import Any

from .utils import clean_text, truncate_text


def analyze_device_problems(
    results: list[dict[str, Any]],
) -> tuple[list, list, list]:
    """Analyze problems, devices, and manufacturers from results."""
    all_problems = []
    all_device_names = []
    all_manufacturers = []

    for result in results:
        devices = result.get("device", [])
        for dev in devices:
            # Collect device names
            if "brand_name" in dev:
                all_device_names.append(dev["brand_name"])
            elif "generic_name" in dev:
                all_device_names.append(dev["generic_name"])

            # Collect manufacturers
            if "manufacturer_d_name" in dev:
                all_manufacturers.append(dev["manufacturer_d_name"])

            # Collect problems
            if "device_problem_text" in dev:
                problems = dev["device_problem_text"]
                if isinstance(problems, str):
                    all_problems.append(problems)
                elif isinstance(problems, list):
                    all_problems.extend(problems)

    return all_problems, all_device_names, all_manufacturers


def format_top_problems(all_problems: list, results: list) -> list[str]:
    """Format top reported device problems."""
    output = []

    if len(results) > 1 and all_problems:
        problem_counts = Counter(all_problems)
        top_problems = problem_counts.most_common(5)

        output.append("### Top Reported Problems:")
        for prob, count in top_problems:
            percentage = (count / len(results)) * 100
            output.append(f"- **{prob}**: {count} reports ({percentage:.1f}%)")
        output.append("")

    return output


def format_device_distribution(
    all_device_names: list, results: list
) -> list[str]:
    """Format device distribution for problem searches."""
    output = []

    if len(results) > 1 and all_device_names:
        device_counts = Counter(all_device_names)
        top_devices = device_counts.most_common(5)

        output.append("### Devices with This Problem:")
        for dev_name, count in top_devices:
            output.append(f"- **{dev_name}**: {count} reports")
        output.append("")

    return output


def format_device_report_summary(
    result: dict[str, Any], report_num: int
) -> list[str]:
    """Format a single device event report summary."""
    output = [f"#### Report {report_num}"]

    # Event type
    event_type_map = {
        "D": "Death",
        "IN": "Injury",
        "IL": "Illness",
        "M": "Malfunction",
        "O": "Other",
    }
    event_type_code = result.get("event_type") or "Unknown"
    event_type = event_type_map.get(event_type_code, "Unknown")
    output.append(f"**Event Type**: {event_type}")

    # Date
    if date_received := result.get("date_received"):
        output.append(f"**Date Received**: {date_received}")

    # Device information
    devices = result.get("device", [])
    for j, dev in enumerate(devices, 1):
        output.extend(_format_device_info(dev, j, len(devices)))

    # Event description
    if event_desc := result.get("event_description"):
        output.append("\n**Event Description**:")
        cleaned_desc = clean_text(event_desc)
        output.append(truncate_text(cleaned_desc, 500))

    # Patient impact
    output.extend(_format_patient_impact(result.get("patient", [])))

    # MDR report number
    if mdr_key := result.get("mdr_report_key"):
        output.append(f"\n*MDR Report #: {mdr_key}*")

    output.append("")
    return output


def _format_device_info(
    dev: dict, device_num: int, total_devices: int
) -> list[str]:
    """Format individual device information."""
    output = []

    if total_devices > 1:
        output.append(f"\n**Device {device_num}:**")

    # Basic device info
    output.extend(_format_device_basic_info(dev))

    # Problem
    if "device_problem_text" in dev:
        problems = dev["device_problem_text"]
        if isinstance(problems, str):
            problems = [problems]
        if problems:
            output.append(f"- **Problem**: {', '.join(problems[:3])}")

    # OpenFDA info
    output.extend(_format_device_class_info(dev.get("openfda", {})))

    return output


def _format_device_basic_info(dev: dict) -> list[str]:
    """Format basic device information."""
    output = []

    # Device name
    dev_name = dev.get("brand_name") or dev.get("generic_name") or "Unknown"
    output.append(f"- **Device**: {dev_name}")

    # Manufacturer
    if "manufacturer_d_name" in dev:
        output.append(f"- **Manufacturer**: {dev['manufacturer_d_name']}")

    # Model/Catalog
    if "model_number" in dev:
        output.append(f"- **Model**: {dev['model_number']}")
    if "catalog_number" in dev:
        output.append(f"- **Catalog #**: {dev['catalog_number']}")

    return output


def _format_device_class_info(openfda: dict) -> list[str]:
    """Format device class and specialty information."""
    output = []

    if "device_class" in openfda:
        dev_class = openfda["device_class"]
        class_map = {"1": "Class I", "2": "Class II", "3": "Class III"}
        output.append(
            f"- **FDA Class**: {class_map.get(dev_class, dev_class)}"
        )

    if "medical_specialty_description" in openfda:
        specialties = openfda["medical_specialty_description"]
        if specialties:
            output.append(f"- **Medical Specialty**: {specialties[0]}")

    return output


def _format_patient_impact(patient_list: list) -> list[str]:
    """Format patient impact information."""
    output = []

    if patient_list:
        patient_info = patient_list[0]
        outcomes = []

        if patient_info.get("date_of_death"):
            outcomes.append("Death")
        if patient_info.get("life_threatening") == "Y":
            outcomes.append("Life-threatening")
        if patient_info.get("disability") == "Y":
            outcomes.append("Disability")

        if outcomes:
            output.append(f"\n**Patient Impact**: {', '.join(outcomes)}")

    return output


def format_device_detail_header(
    result: dict[str, Any], mdr_report_key: str
) -> list[str]:
    """Format device event detail header."""
    output = [f"## Device Event Report: {mdr_report_key}\n"]
    output.append("### Event Overview")

    event_type_map = {
        "D": "Death",
        "IN": "Injury",
        "IL": "Illness",
        "M": "Malfunction",
        "O": "Other",
    }
    event_type_code = result.get("event_type") or "Unknown"
    event_type = event_type_map.get(event_type_code, "Unknown")
    output.append(f"**Event Type**: {event_type}")

    if date_received := result.get("date_received"):
        output.append(f"**Date Received**: {date_received}")

    if date_of_event := result.get("date_of_event"):
        output.append(f"**Date of Event**: {date_of_event}")

    # Report source
    source_map = {
        "P": "Physician",
        "O": "Other health professional",
        "U": "User facility",
        "C": "Distributor",
        "M": "Manufacturer",
    }
    source_type = result.get("source_type")
    if isinstance(source_type, list):
        # Handle case where source_type is a list
        sources: list[str] = []
        for st in source_type:
            if st:
                mapped = source_map.get(st)
                sources.append(mapped if mapped else st)
            else:
                sources.append("Unknown")
        output.append(f"**Report Source**: {', '.join(sources)}")
    elif source_type:
        source = source_map.get(source_type, source_type)
        output.append(f"**Report Source**: {source}")
    else:
        output.append("**Report Source**: Unknown")

    output.append("")
    return output


def format_detailed_device_info(devices: list[dict[str, Any]]) -> list[str]:
    """Format detailed device information."""
    output = ["### Device Information"]

    for i, dev in enumerate(devices, 1):
        if len(devices) > 1:
            output.append(f"\n#### Device {i}")

        # Basic info
        dev_name = (
            dev.get("brand_name") or dev.get("generic_name") or "Unknown"
        )
        output.append(f"**Device Name**: {dev_name}")

        for field, label in [
            ("manufacturer_d_name", "Manufacturer"),
            ("model_number", "Model Number"),
            ("catalog_number", "Catalog Number"),
            ("lot_number", "Lot Number"),
            ("date_received", "Device Received Date"),
            ("expiration_date_of_device", "Expiration Date"),
        ]:
            if value := dev.get(field):
                output.append(f"**{label}**: {value}")

        # Problems
        if "device_problem_text" in dev:
            problems = dev["device_problem_text"]
            if isinstance(problems, str):
                problems = [problems]
            output.append(f"**Device Problems**: {', '.join(problems)}")

        # OpenFDA data
        output.extend(_format_device_openfda(dev.get("openfda", {})))

        # Evaluation
        if "device_evaluated_by_manufacturer" in dev:
            evaluated = (
                "Yes"
                if dev["device_evaluated_by_manufacturer"] == "Y"
                else "No"
            )
            output.append(f"**Evaluated by Manufacturer**: {evaluated}")

    output.append("")
    return output


def _format_device_openfda(openfda: dict) -> list[str]:
    """Format OpenFDA device data."""
    output = []

    if "device_class" in openfda:
        dev_class = openfda["device_class"]
        class_map = {"1": "Class I", "2": "Class II", "3": "Class III"}
        output.append(
            f"**FDA Device Class**: {class_map.get(dev_class, dev_class)}"
        )

    if specialties := openfda.get("medical_specialty_description"):
        if isinstance(specialties, list):
            output.append(f"**Medical Specialty**: {', '.join(specialties)}")
        else:
            output.append(f"**Medical Specialty**: {specialties}")

    if "product_code" in openfda:
        output.append(f"**Product Code**: {openfda['product_code']}")

    return output


def format_patient_details(patient_list: list) -> list[str]:
    """Format detailed patient information."""
    output: list[str] = []

    if not patient_list:
        return output

    output.append("### Patient Information")
    patient_info = patient_list[0]

    # Demographics
    output.extend(_format_patient_demographics(patient_info))

    # Outcomes
    outcomes = _collect_patient_outcomes(patient_info)
    if outcomes:
        output.append(f"**Outcomes**: {', '.join(outcomes)}")

    output.append("")
    return output


def _format_patient_demographics(patient_info: dict) -> list[str]:
    """Format patient demographic information."""
    output = []

    if "patient_age" in patient_info:
        output.append(f"**Age**: {patient_info['patient_age']} years")

    if "patient_sex" in patient_info:
        sex_map = {"M": "Male", "F": "Female", "U": "Unknown"}
        sex = sex_map.get(patient_info["patient_sex"], "Unknown")
        output.append(f"**Sex**: {sex}")

    return output


def _collect_patient_outcomes(patient_info: dict) -> list[str]:
    """Collect patient outcome information."""
    outcomes = []

    if date_of_death := patient_info.get("date_of_death"):
        outcomes.append(f"Death ({date_of_death})")
    if patient_info.get("life_threatening") == "Y":
        outcomes.append("Life-threatening")
    if patient_info.get("disability") == "Y":
        outcomes.append("Disability")
    if patient_info.get("hospitalization") == "Y":
        outcomes.append("Hospitalization")
    if patient_info.get("congenital_anomaly") == "Y":
        outcomes.append("Congenital anomaly")
    if patient_info.get("required_intervention") == "Y":
        outcomes.append("Required intervention")

    return outcomes
