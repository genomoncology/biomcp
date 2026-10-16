from __future__ import annotations

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
