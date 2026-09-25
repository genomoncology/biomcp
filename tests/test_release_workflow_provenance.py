from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"

EXPECTED_NEEDS = {
    "version-check": [],
    "create-draft": ["version-check"],
    "build": ["version-check", "create-draft"],
    "pypi-build": ["version-check"],
    "wheel-smoke": ["pypi-build"],
    "docs-live": ["version-check"],
    "pypi-publish": ["pypi-build", "wheel-smoke", "docs-live"],
    "homebrew-tap": ["build", "docs-live", "wheel-smoke"],
    "container-publish": [
        "build",
        "docs-live",
        "version-check",
        "wheel-smoke",
        "create-draft",
    ],
    "publish-release": [
        "build",
        "pypi-publish",
        "homebrew-tap",
        "container-publish",
        "docs-live",
    ],
}


def _job_block(workflow: str, job: str) -> str:
    match = re.search(
        rf"^  {re.escape(job)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
        re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing workflow job {job}"
    return match.group(1)


def _needs(block: str) -> list[str]:
    match = re.search(r"^    needs: \[([^]]*)\]$", block, re.MULTILINE)
    if match is None:
        return []
    return [part.strip() for part in match.group(1).split(",") if part.strip()]


def _assert_release_contract(workflow: str) -> None:
    assert "push:\n    tags: ['v*']" in workflow
    assert "permissions: {}" in workflow
    assert "group: release-${{ inputs.tag || github.ref_name }}" in workflow
    assert "github.event.release.tag_name" not in workflow
    for job, needs in EXPECTED_NEEDS.items():
        assert _needs(_job_block(workflow, job)) == needs, job

    expected_ifs = {
        "create-draft": "if: github.event_name == 'push'",
        "build": "if: github.event_name == 'push'",
        "pypi-build": "if: github.event_name == 'push'",
        "wheel-smoke": "if: github.event_name == 'push'",
        "pypi-publish": "if: github.event_name == 'push'",
        "homebrew-tap": "if: github.event_name == 'push'",
        "publish-release": "if: github.event_name == 'push'",
    }
    for job, condition in expected_ifs.items():
        assert condition in _job_block(workflow, job), job
    assert "\n    if:" not in _job_block(workflow, "version-check")
    assert "\n    if:" not in _job_block(workflow, "docs-live")

    container = _job_block(workflow, "container-publish")
    for clause in (
        "!cancelled()",
        "github.event_name == 'push' && success()",
        "inputs.container_only == true",
        "needs.version-check.result == 'success'",
        "needs.docs-live.result == 'success'",
        "needs.create-draft.result == 'skipped'",
        "needs.build.result == 'skipped'",
        "needs['wheel-smoke'].result == 'skipped'",
    ):
        assert clause in container
    assert "always()" not in workflow

    version = _job_block(workflow, "version-check")
    assert "fetch-depth: 0" in version
    assert "check-release-versions.py" in version
    assert "check-changelog-coverage.py" in version
    assert "continue-on-error" not in version
    assert "|| true" not in version
    assert "if: false" not in version

    permissions = {
        "version-check": "contents: read",
        "create-draft": "contents: write",
        "build": "contents: write",
        "pypi-build": "contents: read",
        "docs-live": "contents: read",
        "homebrew-tap": "contents: read",
        "pypi-publish": "id-token: write",
        "container-publish": "packages: write",
        "publish-release": "contents: write",
    }
    for job, grant in permissions.items():
        assert grant in _job_block(workflow, job), job
    assert "permissions:" not in _job_block(workflow, "wheel-smoke")

    draft = _job_block(workflow, "create-draft")
    assert 'gh release create "$TAG" --draft --verify-tag' in draft
    build = _job_block(workflow, "build")
    assert 'gh release upload "$TAG"' in build and "--clobber" in build
    assert "skip-existing" not in workflow and "skip_existing" not in workflow

    smoke = _job_block(workflow, "wheel-smoke")
    for artifact in (
        "wheel-x86_64-unknown-linux-gnu",
        "wheel-aarch64-unknown-linux-gnu",
        "wheel-aarch64-apple-darwin",
        "wheel-x86_64-apple-darwin",
        "wheel-x86_64-pc-windows-msvc",
    ):
        assert artifact in smoke
    assert "Scripts/biomcp.exe" in smoke and "bin/biomcp" in smoke
    assert '"$status" -eq 101' in smoke and '"$status" -ge 128' in smoke
    assert "require_exit 0 drug interactions apixaban" in smoke
    assert "Drug not found in FAERS" in smoke
    assert "skill asset" in smoke

    final = _job_block(workflow, "publish-release")
    assert 'gh release edit "$TAG" --draft=false' in final
    assert "gh release view" in final
    assert "docker buildx imagetools create --tag" in final
    assert "gh release view" not in container
    assert "imagetools create --tag" not in container


def _assert_linux_floor_contract(workflow: str) -> None:
    """Pin the glibc 2.28 floor plumbing for Linux artifacts.

    Both Linux artifact families (tarballs and wheels) build inside
    manylinux_2_28 containers with checksum-pinned rustup, the wheel
    platform tag is asserted before the scan runs, and the runtime
    floor smoke exercises a deep offline command.
    """
    build = _job_block(workflow, "build")
    pypi = _job_block(workflow, "pypi-build")
    smoke = _job_block(workflow, "wheel-smoke")

    # Both Linux tarball legs build inside the manylinux containers.
    assert (
        "- os: ubuntu-24.04\n            target: x86_64-unknown-linux-gnu\n"
        "            artifact: biomcp-linux-x86_64.tar.gz\n"
        "            container: quay.io/pypa/manylinux_2_28_x86_64\n" in build
    )
    assert (
        "- os: ubuntu-24.04-arm\n            target: aarch64-unknown-linux-gnu\n"
        "            artifact: biomcp-linux-arm64.tar.gz\n"
        "            container: quay.io/pypa/manylinux_2_28_aarch64\n" in build
    )
    assert "cross: true" not in build
    assert "Install cross-compilation toolchain" not in build
    assert "Build and package inside the manylinux 2_28 container" in build
    # The pre-tar binary carries the same floor as the wheel scan.
    assert (
        "scripts/check-wheel-glibc-floor.py --elf target/${{ matrix.target }}/release/biomcp 2.28"
        in build
    )
    # Host toolchain steps stay off the container legs.
    assert build.count("if: runner.os != 'Linux'") >= 2
    assert "- name: Package (macOS)\n        if: runner.os == 'macOS'" in build
    assert "Package (Unix)" not in build

    # rustup is pinned by version and checksum in both container jobs;
    # matrix.target is the rustup dist triple, and the matrix asserts
    # below pin both Linux legs (x86_64 and aarch64 on their native
    # containers), so the templated URL covers both archives.
    rustup_base = (
        "https://static.rust-lang.org/rustup/archive/1.28.2/${{ matrix.target }}"
    )
    for block in (build, pypi):
        assert f"{rustup_base}/rustup-init\n" in block
        assert f"{rustup_base}/rustup-init.sha256\n" in block
        assert "sha256sum -c rustup-init.sha256" in block
    assert (
        "- os: ubuntu-24.04\n            target: x86_64-unknown-linux-gnu\n"
        "            container: quay.io/pypa/manylinux_2_28_x86_64\n" in pypi
    )
    assert (
        "- os: ubuntu-24.04-arm\n            target: aarch64-unknown-linux-gnu\n"
        "            container: quay.io/pypa/manylinux_2_28_aarch64\n" in pypi
    )
    # No remote script is piped into a shell anywhere in the release flow.
    assert "sh.rustup.rs" not in workflow
    assert "| sh -s" not in workflow

    # The wheel platform tag is pinned and asserted before the scan.
    assert "--compatibility manylinux_2_28" in pypi
    assert "wheel must end in $tag.whl: $wheel" in pypi

    # The runtime floor smoke runs a deep offline command, not only
    # the version banner, and the JSON guard is pinned exactly so a
    # quoting typo cannot ship.
    assert "cache stats --json" in smoke
    assert 'case "$stats" in' in smoke
    assert '"{"*) ;;' in smoke


def test_release_workflow_contract() -> None:
    _assert_release_contract(RELEASE_WORKFLOW.read_text(encoding="utf-8"))


def test_linux_floor_contract() -> None:
    _assert_linux_floor_contract(RELEASE_WORKFLOW.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "job,edge", [(job, edge) for job, edges in EXPECTED_NEEDS.items() for edge in edges]
)
def test_removing_each_needs_edge_breaks_the_contract(
    tmp_path: Path, job: str, edge: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    block = _job_block(workflow, job)
    mutated = (
        block.replace(edge + ", ", "", 1)
        .replace(", " + edge, "", 1)
        .replace("[" + edge + "]", "[]", 1)
    )
    assert mutated != block
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(block, mutated, 1), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_release_contract(scratch.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "needle,replacement",
    [
        (
            'run: python3 scripts/check-release-versions.py --tag "$TAG"',
            'run: python3 scripts/check-release-versions.py --tag "$TAG" || true',
        ),
        ("    steps:\n", "    continue-on-error: true\n    steps:\n"),
        (
            "      - name: Require the tag and committed versions to agree\n",
            "      - name: Require the tag and committed versions to agree\n        if: false\n",
        ),
    ],
)
def test_neutering_a_gate_breaks_the_contract(
    tmp_path: Path, needle: str, replacement: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    assert needle in workflow
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(needle, replacement, 1), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_release_contract(scratch.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "clause",
    [
        "success()",
        "inputs.container_only == true",
        "needs.version-check.result == 'success'",
        "needs.docs-live.result == 'success'",
        "needs.create-draft.result == 'skipped'",
        "needs.build.result == 'skipped'",
        "needs['wheel-smoke'].result == 'skipped'",
    ],
)
def test_removing_a_container_condition_clause_breaks_the_contract(
    tmp_path: Path, clause: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(clause, "true", 1), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_release_contract(scratch.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "needle,replacement",
    [
        (
            "            container: quay.io/pypa/manylinux_2_28_x86_64\n",
            "            container: quay.io/pypa/manylinux_2_28_x86_64-alpine\n",
        ),
        (
            "- os: ubuntu-24.04-arm\n            target: aarch64-unknown-linux-gnu",
            "- os: ubuntu-24.04\n            target: aarch64-unknown-linux-gnu",
        ),
        (
            "https://static.rust-lang.org/rustup/archive/1.28.2/${{ matrix.target }}/rustup-init\n",
            "https://static.rust-lang.org/rustup/archive/1.27.1/${{ matrix.target }}/rustup-init\n",
        ),
        (
            "            sha256sum -c rustup-init.sha256\n",
            "            true\n",
        ),
        (
            "--compatibility manylinux_2_28",
            "",
        ),
        (
            '                *) echo "wheel must end in $tag.whl: $wheel" >&2; exit 1 ;;',
            "                :",
        ),
        (
            "scripts/check-wheel-glibc-floor.py --elf target/${{ matrix.target }}/release/biomcp 2.28",
            "true",
        ),
        (
            '          stats="$(/opt/python/cp312-cp312/bin/biomcp cache stats --json)"',
            '          stats="$(/opt/python/cp312-cp312/bin/biomcp --version)"',
        ),
    ],
)
def test_neutering_the_linux_floor_breaks_the_contract(
    tmp_path: Path, needle: str, replacement: str
) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    assert needle in workflow, needle
    scratch = tmp_path / "release.yml"
    scratch.write_text(workflow.replace(needle, replacement), encoding="utf-8")
    with pytest.raises(AssertionError):
        _assert_linux_floor_contract(scratch.read_text(encoding="utf-8"))


def test_restoring_the_unpinned_rustup_pipe_breaks_the_contract(tmp_path: Path) -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    pinned = (
        '            curl --proto "=https" --tlsv1.2 -sSfO '
        "https://static.rust-lang.org/rustup/archive/1.28.2/${{ matrix.target }}/rustup-init\n"
        '            curl --proto "=https" --tlsv1.2 -sSfO '
        "https://static.rust-lang.org/rustup/archive/1.28.2/${{ matrix.target }}/rustup-init.sha256\n"
        "            sha256sum -c rustup-init.sha256\n"
        "            chmod +x rustup-init\n"
        "            ./rustup-init -y --default-toolchain 1.93.1 --profile minimal"
    )
    assert pinned in workflow
    scratch = tmp_path / "release.yml"
    scratch.write_text(
        workflow.replace(
            pinned,
            '            curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs '
            "| sh -s -- -y --default-toolchain 1.93.1 --profile minimal",
        ),
        encoding="utf-8",
    )
    with pytest.raises(AssertionError):
        _assert_linux_floor_contract(scratch.read_text(encoding="utf-8"))


def test_actions_are_pinned_and_pypi_uses_trusted_publishing() -> None:
    workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    assert not re.findall(r"uses: [^\s]+@(?![0-9a-f]{40}\b)[^\s]+", workflow)
    pypi = _job_block(workflow, "pypi-publish")
    assert "environment: pypi" in pypi and "id-token: write" in pypi
    assert "uv publish --trusted-publishing always dist/*" in pypi


def test_no_other_workflow_exposes_release_publication() -> None:
    routes = (
        "gh release create",
        "uv publish",
        "skopeo copy",
        "git push",
        "docker push",
        "imagetools create",
    )
    for path in WORKFLOWS.glob("*.yml"):
        if path != RELEASE_WORKFLOW:
            text = path.read_text(encoding="utf-8")
            assert not any(route in text for route in routes), path.name
