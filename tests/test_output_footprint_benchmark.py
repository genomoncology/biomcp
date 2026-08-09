from __future__ import annotations

import importlib.util
import os
import sys
from pathlib import Path
from types import ModuleType

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
RUNNER = REPO_ROOT / "benchmarks/output-footprint/run.py"


def _load_runner() -> ModuleType:
    spec = importlib.util.spec_from_file_location("output_footprint", RUNNER)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_compact_ratchet_rejects_first_byte_above_each_ceiling() -> None:
    runner = _load_runner()
    at_ceiling = dict(runner.COMPACT_BYTE_CEILINGS)

    assert runner._compact_ratchet(at_ceiling)["passed"] is True
    for case_id, ceiling in runner.COMPACT_BYTE_CEILINGS.items():
        over_ceiling = at_ceiling | {case_id: ceiling + 1}
        ratchet = runner._compact_ratchet(over_ceiling)
        assert ratchet["passed"] is False
        assert ratchet["regressions"] == [
            {"id": case_id, "output_bytes": ceiling + 1, "byte_ceiling": ceiling}
        ]


def test_tokenizer_vocabulary_is_available_offline(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    runner = _load_runner()

    def reject_network(*args: object, **kwargs: object) -> None:
        raise AssertionError("tokenizer attempted a network request")

    monkeypatch.setattr("requests.get", reject_network)
    runner.tiktoken.registry.ENCODINGS.pop(runner.TOKENIZER, None)

    assert runner._encoding().name == runner.TOKENIZER


def test_benchmark_env_points_every_provider_base_at_the_replay_server() -> None:
    """A provider base the corpus reaches but does not pin is a live call.

    Semantic Scholar was that gap. Article rows are enriched from it, so two
    runs of the same committed corpus disagreed whenever the live API answered
    one and timed out the other -- 31 and 1 are the real citation counts for
    PMIDs 123 and 456. The proxy variables here do not stop it: the client
    does not read them.
    """
    runner = _load_runner()
    env = runner._benchmark_env("http://127.0.0.1:9999")

    bases = {key: value for key, value in env.items() if key.endswith("_BASE")}
    assert "BIOMCP_S2_BASE" in bases
    assert all(value.startswith("http://127.0.0.1:9999/") for value in bases.values())


def test_offline_corpus_is_deterministic_and_reports_real_token_counts() -> None:
    runner = _load_runner()
    binary = Path(os.environ["BIOMCP_BIN"])

    first = runner.collect(binary)
    second = runner.collect(binary)

    assert first == second
    assert first["tokenizer"] == "cl100k_base"
    assert [row["id"] for row in first["commands"]] == [
        "article_search_compact",
        "article_search_full",
        "variant_search",
        "gene_get_sections",
        "trial_search",
    ]
    assert all(row["output_bytes"] > 0 for row in first["commands"])
    assert all(row["token_estimate"] > 0 for row in first["commands"])
    assert first["headline"]["compact_bytes"] < first["headline"]["full_bytes"]
    assert first["headline"]["compact_tokens"] < first["headline"]["full_tokens"]
    assert first["ratchet"]["passed"] is True
