"""Reusable request-contract inventory library for BioMCP ticket 371.

The library is intentionally static/lightweight: it extracts repo-local
contracts, source-test seams, spec dependency hints, and recent March preflight
evidence without rerunning expensive live gates.
"""

from __future__ import annotations

import json
import re
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

DEFAULT_MARCH_RUNTIME = Path("/home/ian/workspace/.march-runtime/runs/biomcp/370-add-transcript-hgvs-normalization-proxies")

DEPENDENCY_PATTERNS = {
    "ols4_discover": re.compile(r"OLS4|ols4|discover |discover\\\"|ERBB1|Arnold Chiari|MEF2", re.I),
    "disease_crosswalk": re.compile(r"crosswalk|Disease|disease|MONDO|MyDisease|phenotypes|diagnostics", re.I),
    "article_sources": re.compile(r"PubTator|Europe PMC|PubMed|Semantic Scholar|LitSense2|article|PMID", re.I),
    "variant_sources": re.compile(r"MyVariant|ClinVar|gnomAD|mutalyzer|variantvalidator|BRAF V600E|HGVS|rs113488022", re.I),
    "fixture_backed": re.compile(r"setup-|fixture|BIOMCP_.*_BASE|127\\.0\\.0\\.1|localhost", re.I),
    "live_public_api": re.compile(r"tools/biomcp-ci (search|get|discover|variant normalize)", re.I),
    "render_json_envelope": re.compile(r"--json|_meta|next_commands|jq -e|markdown|table", re.I),
    "help_list_contract": re.compile(r"--help|list |skill|suggest", re.I),
}

PROOF_PATTERNS = {
    "cli_parsing_routing": re.compile(r"--help|list |search |get |variant normalize|suggest|discover|Query:", re.I),
    "command_to_request_plan": re.compile(r"debug-plan|planner=|source_status|Resolved via|routing|filters", re.I),
    "source_request_construction": re.compile(r"BIOMCP_.*_BASE|setup-|fixture|source status|PubMed|MyVariant|OLS4|Semantic Scholar", re.I),
    "fixture_response_mapping": re.compile(r"fixture|Saved to:|fallback body text|source_status|degrades|unavailable", re.I),
    "render_json_envelope": re.compile(r"mustmatch like|mustmatch '/|jq -e|_meta|next_commands|table|# ", re.I),
    "cross_entity_workflow": re.compile(r"next_commands|follow-up|trials|articles|diagnostics|pivots|workflow", re.I),
    "live_smoke_canary": re.compile(r"tools/biomcp-ci (search|get|discover|variant normalize)", re.I),
}

STRUCT_RE = re.compile(r"(?:pub\([^)]*\)\s+|pub\s+)?struct\s+(\w+)")
ENUM_RE = re.compile(r"(?:pub\([^)]*\)\s+|pub\s+)?enum\s+(\w+)")
PLAN_FN_RE = re.compile(r"fn\s+(\w*plan\w*)", re.I)
DIRECT_ENTITY_CALL_RE = re.compile(r"crate::entities::[a-z_]+::")
DIRECT_SOURCE_CALL_RE = re.compile(r"crate::sources::[a-z_]+::")
FAILED_SECTION_RE = re.compile(r"FAILED\s+((?:spec/)[^\n]+?)(?:\s+-|\n)")
PASSED_TIMINGS_RE = re.compile(r"=+\s*(\d+) passed in ([0-9.]+)s")


@dataclass(frozen=True)
class InventoryConfig:
    """Filesystem inputs and output location for a ticket-371 inventory run."""

    repo_root: Path
    results_dir: Path
    march_runtime: Path = DEFAULT_MARCH_RUNTIME
    script_path: Path | None = None


@dataclass
class SpecSection:
    path: str
    heading: str
    line: int
    tags: list[str]
    proof_types: list[str]
    has_fixture_setup: bool
    uses_live_cli_wrapper: bool


def default_config(script_path: Path | None = None) -> InventoryConfig:
    """Return the default BioMCP worktree configuration.

    `script_path` should be supplied by thin wrappers when they want the JSON
    `script` field to identify that wrapper rather than this library module.
    """

    if script_path is not None:
        repo_root = script_path.resolve().parents[4]
    else:
        repo_root = Path(__file__).resolve().parents[5]
    return InventoryConfig(
        repo_root=repo_root,
        results_dir=repo_root / "architecture/experiments/reset-biomcp-test-strategy-around-request-contracts/results",
        script_path=script_path,
    )


def representative_specs(root: Path) -> list[Path]:
    return [
        root / "spec/entity/disease.md",
        root / "spec/surface/discover.md",
        root / "spec/entity/article.md",
        root / "spec/entity/variant.md",
    ]


def source_files(root: Path) -> list[Path]:
    return [
        root / "src/sources/ols4.rs",
        root / "src/sources/mydisease.rs",
        root / "src/sources/myvariant.rs",
        root / "src/sources/pubmed.rs",
        root / "src/sources/europepmc.rs",
        root / "src/sources/semantic_scholar.rs",
    ]


def plan_files(root: Path) -> list[Path]:
    return [
        root / "src/cli/search_all/plan.rs",
        root / "src/entities/article/planner.rs",
        root / "src/cli/debug_plan.rs",
        root / "src/cli/article/dispatch.rs",
        root / "src/cli/disease/dispatch.rs",
        root / "src/cli/variant/dispatch.rs",
    ]


def rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def extract_sections(path: Path, root: Path) -> list[SpecSection]:
    text = read(path)
    lines = text.splitlines()
    headings: list[tuple[int, str]] = []
    for i, line in enumerate(lines, start=1):
        if line.startswith("## ") and not line.startswith("### "):
            headings.append((i, line[3:].strip()))
    sections: list[SpecSection] = []
    for idx, (line_no, heading) in enumerate(headings):
        end = headings[idx + 1][0] - 1 if idx + 1 < len(headings) else len(lines)
        body = "\n".join(lines[line_no - 1 : end])
        tags = [name for name, pat in DEPENDENCY_PATTERNS.items() if pat.search(body)]
        proof_types = [name for name, pat in PROOF_PATTERNS.items() if pat.search(body)]
        sections.append(
            SpecSection(
                path=rel(path, root),
                heading=heading,
                line=line_no,
                tags=tags,
                proof_types=proof_types,
                has_fixture_setup="fixture_backed" in tags,
                uses_live_cli_wrapper="live_public_api" in tags,
            )
        )
    return sections


def validation_profiles(config: InventoryConfig) -> dict[str, Any]:
    del config
    return {}


def makefile_targets(config: InventoryConfig) -> dict[str, str]:
    targets: dict[str, list[str]] = defaultdict(list)
    current = None
    for line in read(config.repo_root / "Makefile").splitlines():
        m = re.match(r"^([A-Za-z0-9_.-]+):(?:\s|$)", line)
        if m and not line.startswith("."):
            current = m.group(1)
            continue
        if current and line.startswith("\t"):
            targets[current].append(line.strip())
    return {k: " && ".join(v) for k, v in targets.items() if k in {"check", "release-gate", "spec", "spec-pr", "test-contracts", "focused"}}


def source_contract_inventory(config: InventoryConfig) -> list[dict[str, Any]]:
    rows = []
    for path in source_files(config.repo_root):
        text = read(path)
        rows.append(
            {
                "path": rel(path, config.repo_root),
                "new_for_test": text.count("new_for_test"),
                "wiremock_tests": text.count("wiremock"),
                "mock_given": text.count("Mock::given"),
                "query_param_assertions": text.count("query_param"),
                "header_assertions": text.count("header("),
                "body_assertions": text.count("body_string_contains"),
                "env_base": text.count("env_base"),
                "apply_cache_mode": text.count("apply_cache_mode"),
                "tokio_tests": text.count("#[tokio::test]"),
                "plain_tests": text.count("#[test]"),
            }
        )
    return rows


def plan_seam_inventory(config: InventoryConfig) -> list[dict[str, Any]]:
    rows = []
    for path in plan_files(config.repo_root):
        text = read(path)
        rows.append(
            {
                "path": rel(path, config.repo_root),
                "structs": STRUCT_RE.findall(text),
                "enums": ENUM_RE.findall(text),
                "functions_with_plan_name": sorted(set(PLAN_FN_RE.findall(text))),
                "filter_struct_mentions": text.count("Filters") + text.count("SearchFilters"),
                "debug_plan_mentions": text.count("DebugPlan"),
                "direct_entity_calls": len(DIRECT_ENTITY_CALL_RE.findall(text)),
                "direct_source_calls": len(DIRECT_SOURCE_CALL_RE.findall(text)),
            }
        )
    return rows


def march_preflight_evidence(config: InventoryConfig) -> list[dict[str, Any]]:
    rows = []
    for path in sorted(config.march_runtime.glob("*/preflight-check.json")):
        data = json.loads(read(path))
        output = data.get("output", "")
        failed = [
            line.split(" FAILED", 1)[0].strip()
            for line in output.splitlines()
            if " FAILED" in line and line.strip().startswith("spec/")
        ]
        failed.extend(
            match.strip()
            for match in FAILED_SECTION_RE.findall(output)
            if match.strip() not in failed
        )
        passed_timings = PASSED_TIMINGS_RE.findall(output)
        rows.append(
            {
                "path": str(path),
                "profile": data.get("profile"),
                "command": data.get("command"),
                "ok": data.get("ok"),
                "classification": data.get("classification"),
                "timestamp": data.get("timestamp"),
                "failed_sections": failed,
                "passed_timings": [
                    {"passed": int(count), "seconds": float(seconds)}
                    for count, seconds in passed_timings
                ],
                "output_mentions": {
                    "ols4_timeout": "ols4" in output.lower() and "timed out" in output.lower(),
                    "synonym_rescue_failure": "Synonym Rescue" in output and "FAILED" in output,
                    "mef2_failure": "MEF2 relational query" in output and "FAILED" in output,
                },
                "attempts": data.get("attempts", []),
            }
        )
    return rows


def summarize(sections: list[SpecSection], sources: list[dict[str, Any]], plans: list[dict[str, Any]], preflights: list[dict[str, Any]]) -> dict[str, Any]:
    tag_counts = Counter(tag for sec in sections for tag in sec.tags)
    proof_counts = Counter(tag for sec in sections for tag in sec.proof_types)
    live_sections = [asdict(sec) for sec in sections if sec.uses_live_cli_wrapper and not sec.has_fixture_setup]
    fixture_sections = [asdict(sec) for sec in sections if sec.has_fixture_setup]
    source_totals = Counter()
    for row in sources:
        for key, value in row.items():
            if isinstance(value, int):
                source_totals[key] += value
    return {
        "representative_spec_sections": len(sections),
        "tag_counts": dict(tag_counts),
        "proof_type_counts": dict(proof_counts),
        "live_nonfixture_sections": len(live_sections),
        "fixture_backed_sections": len(fixture_sections),
        "source_contract_totals": dict(source_totals),
        "plan_files_with_plan_functions": [row["path"] for row in plans if row["functions_with_plan_name"]],
        "march_preflight_runs": len(preflights),
        "march_preflight_failures": sum(1 for row in preflights if not row["ok"]),
        "march_failed_sections": [failure for row in preflights for failure in row["failed_sections"]],
    }


def proposed_profiles(summary: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        {
            "name": "kickoff",
            "command_shape": "cargo check --all-targets + deterministic contract inventory subset",
            "live_network": False,
            "proves": ["compilation", "CLI command/filter/request-plan contracts", "source request construction for touched area"],
            "evidence": "Current preflight cargo check passed in 44.45s cold; no live upstream needed.",
        },
        {
            "name": "focused",
            "command_shape": "cargo test --lib for touched modules + clippy; optionally targeted wiremock source tests",
            "live_network": False,
            "proves": ["unit behavior", "source request/status mapping", "renderer envelope for touched code"],
            "evidence": f"Representative source files already expose {summary['source_contract_totals'].get('mock_given', 0)} wiremock Mock::given contracts.",
        },
        {
            "name": "spec-only",
            "command_shape": "fixture-backed executable specs and static surface contracts only",
            "live_network": False,
            "proves": ["CLI help/list", "fixture-backed response/rendering", "JSON envelope shape"],
            "evidence": f"Representative specs have {summary['fixture_backed_sections']} fixture-backed sections but {summary['live_nonfixture_sections']} live non-fixture sections to split/move.",
        },
        {
            "name": "verify/full-blocking",
            "command_shape": "make check + deterministic spec-only; no broad live canaries by default",
            "live_network": "minimal/controlled",
            "proves": ["repo health", "contract completeness", "quality ratchets"],
            "evidence": "Ticket 370 succeeded only after unrelated live/spec issues were resolved; keep broad checks out of kickoff.",
        },
        {
            "name": "release-live-smoke",
            "command_shape": "small serialized opt-in live smoke for OLS4/discover, disease crosswalk, article source status, variant normalization",
            "live_network": True,
            "proves": ["public upstream availability", "cache/keyless operator path", "release confidence"],
            "evidence": "Known failures are OLS4/discover/disease crosswalk; keep as explicit release/operator signal.",
        },
    ]


def build_inventory(config: InventoryConfig) -> dict[str, Any]:
    """Build the complete request-contract inventory payload without writing it."""

    sections = [sec for path in representative_specs(config.repo_root) for sec in extract_sections(path, config.repo_root)]
    sources = source_contract_inventory(config)
    plans = plan_seam_inventory(config)
    preflights = march_preflight_evidence(config)
    profiles = validation_profiles(config)
    make_targets = makefile_targets(config)
    summary = summarize(sections, sources, plans, preflights)
    script_path = config.script_path or Path(__file__)

    return {
        "script": rel(script_path, config.repo_root),
        "repo_root": str(config.repo_root),
        "approaches": {
            "status_quo_live_partitioning": {
                "profiles": profiles,
                "make_targets": make_targets,
                "march_preflight_evidence": preflights,
            },
            "contract_first_existing_seams": {
                "representative_sections": [asdict(sec) for sec in sections],
                "source_contract_inventory": sources,
                "plan_seam_inventory": plans,
            },
            "minimal_request_plan_seam": {
                "candidate_existing_seams": plans,
                "recommended_boundaries": {
                    "cli": "clap args -> RequestCommand / typed filters; no network",
                    "entity": "RequestCommand -> source request plan(s) and renderer model orchestration",
                    "source": "SourceRequestPlan -> HTTP method/url/query/headers/body/cache/auth + fixture response mapping",
                    "render": "entity model -> markdown/JSON envelope, no upstream calls",
                },
            },
        },
        "summary": summary,
        "proposed_profiles": proposed_profiles(summary),
    }


def write_inventory(payload: dict[str, Any], results_dir: Path) -> Path:
    """Write the inventory JSON using the stable checked-in output format."""

    results_dir.mkdir(parents=True, exist_ok=True)
    out = results_dir / "test_strategy_inventory.json"
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out


def main(config: InventoryConfig | None = None) -> Path:
    """Run the inventory and return the generated JSON path."""

    active_config = config or default_config()
    return write_inventory(build_inventory(active_config), active_config.results_dir)
