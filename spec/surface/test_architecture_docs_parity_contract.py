from __future__ import annotations

import json
from pathlib import Path
import re

REPO_ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP_ENTITY_SET = {"gene", "variant", "article"}
ENV_VAR_RE = re.compile(r"`([A-Z][A-Z0-9_]*(?:API_KEY|TOKEN))`")


def _read(relative_path: str) -> str:
    return (REPO_ROOT / relative_path).read_text(encoding="utf-8")


def _section(text: str, heading: str) -> str:
    start = text.index(heading)
    rest = text[start + len(heading) :]
    match = re.search(r"\n## ", rest)
    if match is None:
        return rest
    return rest[: match.start()]


def _entity_spec_names() -> set[str]:
    return {path.stem for path in (REPO_ROOT / "spec" / "entity").glob("*.md")}


def test_cli_decomposition_doc_does_not_claim_absorbed_allowlist_work_is_pending() -> None:
    allowlist = json.loads(_read("tools/cli-line-cap-allowlist.json"))
    assert allowlist["entries"] == []

    cli_decomposition = _read("architecture/technical/cli-decomposition-2026.md")
    stale_pending_markers = [
        "The remaining over-cap files are:",
        "Ticket 334 is the absorb-back path",
        "Ticket 334 global line-cap ratchet allowlist",
    ]

    stale_markers_present = [
        marker for marker in stale_pending_markers if marker in cli_decomposition
    ]
    assert not stale_markers_present, (
        "architecture/technical/cli-decomposition-2026.md must not describe "
        "absorbed line-cap allowlist work as pending when the allowlist is empty; "
        f"stale markers still present: {stale_markers_present}"
    )


def test_functional_docs_describe_current_spec_v2_entity_corpus() -> None:
    entity_specs = _entity_spec_names()
    assert BOOTSTRAP_ENTITY_SET < entity_specs
    assert "diagnostic" in entity_specs

    functional_docs = {
        "architecture/functional/diagnostic.md": _read(
            "architecture/functional/diagnostic.md"
        ),
        "architecture/functional/clinical-features-port.md": _read(
            "architecture/functional/clinical-features-port.md"
        ),
    }

    stale_subset_pattern = re.compile(
        r"spec-v2\s+canar(?:y|ies).*limited\s+to\s+gene,\s*variant,\s+and\s+article",
        re.IGNORECASE | re.DOTALL,
    )
    stale_docs = [
        path for path, text in functional_docs.items() if stale_subset_pattern.search(text)
    ]
    assert not stale_docs, (
        "functional architecture docs must describe the current spec/entity corpus, "
        f"not the old gene/variant/article-only bootstrap subset: {stale_docs}"
    )

    diagnostic_doc = functional_docs["architecture/functional/diagnostic.md"]
    assert "spec/entity/diagnostic.md" in diagnostic_doc, (
        "architecture/functional/diagnostic.md must point readers at the shipped "
        "diagnostic executable spec when spec/entity/diagnostic.md exists"
    )


def test_staging_demo_optional_runtime_keys_cover_overview_api_key_table() -> None:
    overview_api_keys = set(
        ENV_VAR_RE.findall(_section(_read("architecture/technical/overview.md"), "## API Keys"))
    )
    staging_optional_keys = set(
        ENV_VAR_RE.findall(
            _section(
                _read("architecture/technical/staging-demo.md"),
                "## Credentials and Environment Variables",
            )
        )
    )

    assert overview_api_keys, "overview API-key table should expose runtime key names"
    assert staging_optional_keys, "staging-demo optional-key section should expose key names"

    missing_from_staging = sorted(overview_api_keys - staging_optional_keys)
    assert not missing_from_staging, (
        "architecture/technical/staging-demo.md optional runtime keys must cover "
        "architecture/technical/overview.md API-key table; missing: "
        f"{missing_from_staging}"
    )
