"""Check section credits through the CLI using the existing recorded replies.

The registry supplies the sections and allowed providers. Replay gaps remain
unavailable outcomes, not invented successful provider responses. The exact
protein regression separately requires a successful recorded response.
"""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path

import pytest
from golden.replay_env import REPO_ROOT, replay_environment

pytestmark = [pytest.mark.needs_binary]

# Inputs only: section names and provider labels come from the runtime registry.
IDENTIFIERS = {
    "gene": "BRAF",
    "article": "42875213",
    "cell_line": "CVCL_2119",
    "pathway": "R-HSA-5673001",
    "protein": "P15056",
    "drug": "pembrolizumab",
    "disease": "MONDO:0007947",
    "variant": "BRAF V600E",
    "pgx": "CYP2D6",
    "diagnostic": "ITPW02232- TC40",
}


def _binary() -> Path:
    return Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))


def _registry():
    spec = importlib.util.spec_from_file_location(
        "section_credit_registry", REPO_ROOT / "tools/check-quality-ratchet.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    states, selectors, errors = module._source_state_registry_rows(REPO_ROOT)
    assert states and not errors, errors
    return states, selectors


def test_protein_structures_json_credits_the_recorded_uniprot_response() -> None:
    with replay_environment() as replay:
        run = replay.run(_binary(), ("get", "protein", "P15056", "structures", "--json"))
    assert run.request_log_lines == [{
        "key": "GET /uniprot/uniprotkb/P15056.json", "result": "hit", "status": 200,
    }]
    assert run.process.returncode == 0, run.process.stderr
    card = json.loads(run.process.stdout)
    outcome = card["section_outcomes"]["structures"]
    assert outcome["outcome"] == "data"
    assert outcome["sources"] == ["UniProt"]
    assert card["structures"]
    projected = next(row for row in card["_meta"]["section_sources"] if row["key"] == "structures")
    assert projected["sources"] == outcome["sources"]


def test_every_entity_section_credits_only_registered_providers() -> None:
    """Walk every row, collecting all credit errors and naming replay gaps."""
    states, selectors = _registry()
    failures = []
    credited = []
    gaps = []
    with replay_environment() as replay:
        for (entity, key), (_label, providers, _aggregation) in states.items():
            if entity == "variant_structure":
                argv = ("variant", "structure", "BRAF V600E", "--json")
            elif entity == "adverse_event":
                argv = ("search", "adverse-event", "--drug", "pembrolizumab", "--source", key, "--json")
            else:
                section = next((selector for (owner, selector), (kind, canonical) in selectors.items()
                                if owner == entity and kind == "canonical" and canonical == key), None)
                argv = ("get", entity.replace("_", "-"), IDENTIFIERS[entity])
                if section is not None:
                    argv += (section,)
                argv += ("--json",)
            run = replay.run(_binary(), argv)
            identity = f"{entity}/{key}"
            if "panicked" in run.process.stderr:
                failures.append(f"{identity}: {run.process.stderr.strip()}")
                continue
            if run.process.returncode:
                gaps.append(f"{identity}: command unavailable on recorded inputs")
                continue
            card = json.loads(run.process.stdout)
            outcomes = card.get("section_outcomes", card.get("lookup_outcomes", {}))
            outcome = outcomes.get(key)
            if outcome is None:
                gaps.append(f"{identity}: no section outcome on recorded inputs")
                continue
            sources = outcome["sources"]
            unknown = set(sources) - set(providers)
            if unknown:
                failures.append(f"{identity}: unregistered credits {sorted(unknown)}")
            if sources:
                credited.append(identity)
            else:
                gaps.append(f"{identity}: {outcome['outcome']}")
    print(f"Successful provider credits checked ({len(credited)}): {', '.join(credited)}")
    print(f"Recorded-input coverage gaps ({len(gaps)}): {'; '.join(gaps)}")
    assert credited, "the recorded replies must exercise successful provider credits"
    assert not failures, "\n".join(failures)
