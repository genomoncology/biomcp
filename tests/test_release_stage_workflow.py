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
    assert "manylinux_2_28_aarch64@sha256:" in text
    assert "release/build_target.py" in text
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
            assert "${{ matrix.container }}" in stripped or "@sha256:" in stripped


def test_five_platform_jobs_are_explicit_and_signing_is_protected() -> None:
    text = _text()
    for value in (
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "macos-15-intel",
        "macos-15",
        "windows-2022",
    ):
        assert value in text
    assert "environment: biomcp-release-signing" in text
    assert "BIOMCP_SIGNING_POLICY_SHA256: ${{ secrets.BIOMCP_SIGNING_POLICY_SHA256 }}" in text


def test_container_consumes_both_registered_linux_archives_without_push() -> None:
    text = _text()
    assert "container-artifact:" in text
    assert "pattern: 'linux-*-${{ github.run_id }}'" in text
    assert "--platform linux/amd64,linux/arm64" in text
    assert "--output type=oci,dest=dist/oci/biomcp.oci.tar" in text
    assert "release/container.py" in text
    assert "push: true" not in text
    assert "docker push" not in text


def test_homebrew_formula_is_generated_once_and_smoked_offline_on_both_macs() -> None:
    text = _text()
    assert "homebrew-formula:" in text
    assert "release/homebrew.py" in text
    assert "homebrew-smoke:" in text
    assert "HOMEBREW_NO_INSTALL_FROM_API: 1" in text
    assert "macos-15-intel" in text and "macos-15" in text
    assert "spctl --assess --type execute" in text
    assert "homebrew-biomcp" not in text
    assert "HOMEBREW_TAP_TOKEN" not in text
