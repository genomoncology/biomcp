from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/release.yml"


def _text() -> str:
    return WORKFLOW.read_text(encoding="utf-8")


def test_stage_is_the_only_callable_release_mode_before_promotion() -> None:
    workflow = yaml.safe_load(_text())
    dispatch = workflow[True]["workflow_dispatch"]["inputs"]
    assert set(dispatch) == {"source_sha"}
    assert "promote" not in _text().lower()
    assert workflow["permissions"] == {"contents": "read"}


def test_stage_binds_checkout_build_and_manifest_to_input_sha() -> None:
    text = _text()
    assert text.count("ref: ${{ inputs.source_sha }}") >= 3
    assert "git merge-base --is-ancestor \"$SOURCE_SHA\" origin/main" in text
    assert "release/candidate.py init" in text
    assert "release/candidate.py register" in text
    assert "release/candidate.py finalize" in text
    assert "github.sha" not in text


def test_baseline_build_is_once_in_pinned_manylinux_and_never_publishes() -> None:
    text = _text()
    assert "manylinux_2_28_x86_64@sha256:" in text
    assert text.count("cargo build --release") == 1
    assert "--bin biomcp --bin biomcp-cli" in text
    forbidden = (
        "softprops/action-gh-release",
        "pypa/gh-action-pypi-publish",
        "docker push",
        "packages: write",
        "contents: write",
        "git push",
        "HOMEBREW_TAP_TOKEN",
    )
    assert not any(item in text for item in forbidden)


def test_every_action_and_container_is_commit_or_digest_pinned() -> None:
    for line in _text().splitlines():
        stripped = line.strip()
        if stripped.startswith("uses:"):
            reference = stripped.rsplit("@", 1)[1]
            assert len(reference) == 40 and all(char in "0123456789abcdef" for char in reference)
        if "container:" in stripped or stripped.startswith("MANYLINUX_"):
            assert "@sha256:" in stripped
