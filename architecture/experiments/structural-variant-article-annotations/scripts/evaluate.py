#!/usr/bin/env -S uv run --no-project
"""Run four small structural-event extraction approaches and score exact spans."""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "fixtures" / "corpus.json"
ONTOLOGY = ROOT / "fixtures" / "ontology_terms.json"
PUBTATOR = ROOT / "work" / "pubtator.json"
DEFAULT_OUTPUT = ROOT / "measurements.json"

# Generic syntax/context rules only. No PMID, disease, or event-to-gene mappings.
PARSER_PATTERNS: list[tuple[str, str]] = [
    (r"\bt\(\d{1,2};\d{1,2}\)(?:\([pq][\d.]+;[pq][\d.]+\))?", "translocation"),
    (r"\bdel\([^)]+\)", "deletion"),
    (r"\binv\([^)]+\)", "inversion"),
    (r"\bgain\([^)]+\)", "gain"),
    (r"\bamp\([^)]+\)", "amplification"),
    (r"\br\(\d{1,2}\)c\b", "complex_event"),
    (r"\biAMP\d+\b", "amplification"),
    (r"\b[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*::[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*\b", "free_text_structural_variant"),
    (r"\b[Ss]tructural [Vv]ariants?\b", "free_text_structural_variant"),
    (r"\bstructural genome variations?\b", "free_text_structural_variant"),
    (r"\bchromosomal rearrangements?\b", "free_text_structural_variant"),
    (r"\bprimary rearrangement\b", "free_text_structural_variant"),
    (r"\bcopy-number abnormalities\b", "free_text_structural_variant"),
    (r"\bCopy-Number(?= and Structural Variants)", "free_text_structural_variant"),
    (r"\b(?:CNA|CNAs|SV|SVs)\b", "free_text_structural_variant"),
    (r"\b(?:complex (?:genomic )?rearrangements?|chromoplexy|chromothripsis|templated insertions?)\b", "complex_event"),
    (r"\bring chromosome \d{1,2}\b", "complex_event"),
    (r"\b[Pp]ericentric inversion (?:of|involving) (?:the )?(?:der\(\d{1,2}\)|(?:derivative )?chromosome \d{1,2})", "inversion"),
    (r"\b(?:codeletion|\d+[pq][\d.]* deletion|regions of deletion)\b", "deletion"),
    (r"\b(?:chromosomal gains|focal gains|whole-arm gains|duplication of \d+[pq]?|of gain)\b", "gain"),
    (r"\bintrachromosomal amplification of chromosome \d{1,2}\b", "amplification"),
    (r"\b(?:high )?hyperdiploid(?: karyotype)?\b|\bhigh hyperdiploidy\b", "ploidy_state"),
    (r"\buniparental disomy\b", "free_text_structural_variant"),
    (r"\b(?:complex )?genomic rearrangements?\b", "complex_event"),
    (r"(?<=chromosomal gains and )losses\b", "deletion"),
]


def prediction(start: int, end: int, text: str, event_type: str, source: str) -> dict[str, Any]:
    return {"start": start, "end": end, "text": text[start:end], "event_type": event_type, "source": source}


def dedupe(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    unique = {(row["start"], row["end"], row["event_type"]): row for row in rows}
    return sorted(unique.values(), key=lambda row: (row["start"], row["end"], row["event_type"]))


def deterministic(text: str) -> list[dict[str, Any]]:
    rows = []
    for expression, event_type in PARSER_PATTERNS:
        for match in re.finditer(expression, text):
            rows.append(prediction(match.start(), match.end(), text, event_type, "deterministic"))
    return dedupe(rows)


def ontology_matcher(terms: list[dict[str, Any]]) -> Callable[[str], list[dict[str, Any]]]:
    def match(text: str) -> list[dict[str, Any]]:
        rows = []
        for term in terms:
            for lexeme in term["lexemes"]:
                expression = rf"(?<!\w){re.escape(lexeme)}(?!\w)"
                for found in re.finditer(expression, text, re.IGNORECASE):
                    row = prediction(found.start(), found.end(), text, term["event_type"], f"ontology:{term['id']}")
                    rows.append(row)
        return dedupe(rows)
    return match


def pubtator_predictions(payload: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    output: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for document in payload.get("PubTator3", []):
        pmid = str(document.get("pmid") or document.get("id") or "")
        for passage in document.get("passages", []):
            for annotation in passage.get("annotations", []):
                kind = str(annotation.get("infons", {}).get("type", "")).lower()
                if "mutation" not in kind and "variant" not in kind:
                    continue
                # Current BioMCP exposes mention text but not locations. Event typing would
                # require a new layer; retain an unknown event class for an honest baseline.
                for location in annotation.get("locations", []):
                    start = int(location.get("offset", -1))
                    length = int(location.get("length", 0))
                    if start >= 0 and length > 0:
                        output[pmid].append({
                            "start": start, "end": start + length,
                            "text": annotation.get("text", ""),
                            "event_type": "free_text_structural_variant", "source": "pubtator",
                        })
    return {pmid: dedupe(rows) for pmid, rows in output.items()}


def key(row: dict[str, Any]) -> tuple[int, int, str]:
    return row["start"], row["end"], row["event_type"]


def score(name: str, documents: list[dict[str, Any]], predictions: dict[str, list[dict[str, Any]]]) -> dict[str, Any]:
    types = sorted({event["event_type"] for doc in documents for event in doc["gold_events"]})
    counts = {kind: {"tp": 0, "fp": 0, "fn": 0} for kind in types}
    examples = {"false_positives": [], "false_negatives": []}
    covered, positive_documents, control_fp = 0, 0, 0
    for document in documents:
        pmid = document["pmid"]
        gold = {key(event): event for event in document["gold_events"]}
        predicted = {key(row): row for row in predictions.get(pmid, [])}
        true_keys = gold.keys() & predicted.keys()
        if gold:
            positive_documents += 1
            covered += bool(true_keys)
        else:
            control_fp += len(predicted)
        for event_key in true_keys:
            counts[event_key[2]]["tp"] += 1
        for event_key, row in predicted.items():
            if event_key not in gold:
                counts.setdefault(event_key[2], {"tp": 0, "fp": 0, "fn": 0})["fp"] += 1
                if len(examples["false_positives"]) < 8:
                    examples["false_positives"].append({"pmid": pmid, "text": row["text"], "event_type": row["event_type"]})
        for event_key, row in gold.items():
            if event_key not in predicted:
                counts[event_key[2]]["fn"] += 1
                if len(examples["false_negatives"]) < 8:
                    examples["false_negatives"].append({"pmid": pmid, "text": row["text"], "event_type": row["event_type"]})

    def metrics(values: dict[str, int]) -> dict[str, Any]:
        tp, fp, fn = values["tp"], values["fp"], values["fn"]
        precision = tp / (tp + fp) if tp + fp else 0.0
        recall = tp / (tp + fn) if tp + fn else 0.0
        f1 = 2 * precision * recall / (precision + recall) if precision + recall else 0.0
        return {**values, "precision": round(precision, 4), "recall": round(recall, 4), "f1": round(f1, 4)}

    per_type = {kind: metrics(values) for kind, values in sorted(counts.items())}
    totals = {field: sum(values[field] for values in counts.values()) for field in ("tp", "fp", "fn")}
    micro = metrics(totals)
    macro = {
        field: round(sum(per_type[kind][field] for kind in types) / len(types), 4)
        for field in ("precision", "recall", "f1")
    }
    return {
        "approach": name, "per_event_type": per_type, "micro": micro, "macro": macro,
        "document_coverage": {"covered": covered, "positive_documents": positive_documents, "rate": round(covered / positive_documents, 4)},
        "control_false_positives": control_fp, "examples": examples,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    corpus = json.loads(CORPUS.read_text())
    documents = corpus["documents"]
    terms = json.loads(ONTOLOGY.read_text())["terms"]
    ontology = ontology_matcher(terms)
    pubtator_payload = json.loads(PUBTATOR.read_text())
    pubtator = pubtator_predictions(pubtator_payload)
    parser_rows = {doc["pmid"]: deterministic(doc["text"]) for doc in documents}
    ontology_rows = {doc["pmid"]: ontology(doc["text"]) for doc in documents}
    hybrid_rows = {
        doc["pmid"]: dedupe(parser_rows[doc["pmid"]] + ontology_rows[doc["pmid"]])
        for doc in documents
    }
    results = {
        "experiment": "structural-variant-article-annotations",
        "corpus": {
            "documents": len(documents),
            "positive_documents": sum(bool(doc["gold_events"]) for doc in documents),
            "control_documents": sum(not doc["gold_events"] for doc in documents),
            "gold_events": sum(len(doc["gold_events"]) for doc in documents),
            "gold_gene_relationships": sum(len(doc["gold_gene_relationships"]) for doc in documents),
        },
        "match_rule": "exact unicode-codepoint start, end, and event_type",
        "source_snapshots": {
            "measured_on": "2026-07-14",
            "pubtator_documents_available": len(pubtator_payload.get("PubTator3", [])),
            "pubtator_documents_unavailable": pubtator_payload.get("unavailable", []),
            "ontology": "EMBL-EBI OLS4 terms retrieved 2026-07-14",
        },
        "approaches": [
            score("pubtator_current", documents, pubtator),
            score("deterministic_parser", documents, parser_rows),
            score("ontology_lexical", documents, ontology_rows),
            score("hybrid_union", documents, hybrid_rows),
        ],
    }
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    for result in results["approaches"]:
        micro = result["micro"]
        coverage = result["document_coverage"]["rate"]
        print(f"{result['approach']}: P={micro['precision']:.4f} R={micro['recall']:.4f} F1={micro['f1']:.4f} coverage={coverage:.4f} control_fp={result['control_false_positives']}")


if __name__ == "__main__":
    main()
