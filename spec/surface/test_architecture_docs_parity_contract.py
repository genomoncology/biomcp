from __future__ import annotations

import json
from pathlib import Path
import re

REPO_ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP_ENTITY_SET = {"gene", "variant", "article"}
ABSORBED_CLI_RESIDUAL_PATHS = {
    "src/cli/drug/tests.rs",
    "src/cli/trial/tests.rs",
    "src/cli/cache.rs",
    "src/cli/article/session.rs",
    "src/cli/article/dispatch.rs",
    "src/cli/variant/dispatch.rs",
}
ENV_VAR_RE = re.compile(r"`([A-Z][A-Z0-9_]*(?:API_KEY|TOKEN))`")
COMPLETED_STATE_RE = re.compile(
    r"\b(absorbed|complete|completed|former|historical|no longer|under the cap)\b",
    re.IGNORECASE,
)
CURRENT_RESIDUAL_RE = re.compile(
    r"\b(residual|remaining|over-cap|allowlist|follow-up|future work|split axis)\b",
    re.IGNORECASE,
)


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


def _paragraphs(text: str) -> list[str]:
    return [block.strip() for block in re.split(r"\n\s*\n", text) if block.strip()]


def _optional_runtime_key_bullets(staging_demo: str) -> set[str]:
    credentials = _section(staging_demo, "## Credentials and Environment Variables")
    marker = "Optional runtime keys:"
    assert marker in credentials, "staging-demo must keep an Optional runtime keys list"
    optional_block = credentials.split(marker, 1)[1]

    bullet_lines = []
    for line in optional_block.splitlines():
        stripped = line.strip()
        if not stripped:
            if bullet_lines:
                break
            continue
        if stripped.startswith("- "):
            bullet_lines.append(stripped)
            continue
        if bullet_lines:
            break

    return set(ENV_VAR_RE.findall("\n".join(bullet_lines)))


def test_cli_decomposition_doc_does_not_claim_absorbed_allowlist_work_is_pending():
    allowlist = json.loads(_read("tools/cli-line-cap-allowlist.json"))
    assert allowlist["entries"] == []

    cli_decomposition = _read("architecture/technical/cli-decomposition-2026.md")
    current_residual_path_claims = set()
    current_ticket_334_claims = []
    for paragraph in _paragraphs(cli_decomposition):
        describes_current_residual_work = CURRENT_RESIDUAL_RE.search(
            paragraph
        ) and not COMPLETED_STATE_RE.search(paragraph)
        if not describes_current_residual_work:
            continue

        for path in sorted(ABSORBED_CLI_RESIDUAL_PATHS):
            if path in paragraph:
                current_residual_path_claims.add(path)
        if "Ticket 334" in paragraph:
            current_ticket_334_claims.append(re.sub(r"\s+", " ", paragraph))

    assert not current_residual_path_claims, (
        "architecture/technical/cli-decomposition-2026.md must not keep the "
        "absorbed ticket-347 CLI files in a current residual-over-cap plan when "
        "the allowlist is empty; current residual path claims: "
        f"{sorted(current_residual_path_claims)}"
    )
    assert not current_ticket_334_claims, (
        "architecture/technical/cli-decomposition-2026.md must describe ticket-334 "
        "line-cap work as absorbed/completed or historical when the allowlist is "
        f"empty; current ticket-334 claims: {current_ticket_334_claims}"
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
        path
        for path, text in functional_docs.items()
        if stale_subset_pattern.search(text)
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
        ENV_VAR_RE.findall(
            _section(_read("architecture/technical/overview.md"), "## API Keys")
        )
    )
    staging_optional_keys = _optional_runtime_key_bullets(
        _read("architecture/technical/staging-demo.md")
    )

    assert overview_api_keys, "overview API-key table should expose runtime key names"
    assert staging_optional_keys, (
        "staging-demo optional runtime key bullets should expose key names"
    )

    missing_from_staging = sorted(overview_api_keys - staging_optional_keys)
    assert not missing_from_staging, (
        "architecture/technical/staging-demo.md optional runtime keys must cover "
        "architecture/technical/overview.md API-key table; missing: "
        f"{missing_from_staging}"
    )
