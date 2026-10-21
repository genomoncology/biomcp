"""Reviewed release inventory: ordered steps, body pins and earlier checks.

Update a workflow step and its disposition here together. The admission test
owns structural drift; other release contracts wait until this inventory agrees.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import pytest
import yaml

INVENTORY_PATH = "tests/release_workflow_inventory.py"
NATIVE_WINDOWS_CACHE_STEP = "Check cold cache statistics in native Windows PowerShell"


@dataclass(frozen=True)
class Step:
    name: str
    counterpart: tuple[str, str] | None = None
    sha256: str | None = None
    local_command: str | None = None


# All release jobs occur once. Steps stay in runner order. Hashes retain the
# existing reviewed canonical bodies; counterparts retain platform boundaries.
RELEASE_JOBS = {
    'version-check': {
        "needs": [],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683', sha256='20b8f03bdcde7c6e9713aa4c8aefe129de93e2dcc00e902f2b22cf8589b63a9f'),
            Step('Require the tag and committed versions to agree', counterpart=('scripts/check-release-metadata.py', '"scripts/check-release-versions.py"'), sha256='65da2401fd66e25808ab8c3abc9758b8671fef888ffd0eb13457044764c5bf5d', local_command='scripts/check-release-versions.py'),
            Step('Require the changelog to cover merged tickets', counterpart=('scripts/check-release-metadata.py', '"scripts/check-changelog-coverage.py"'), sha256='1f76a37b3d73ab514debb49e722c4e1be630ffec9d136fadb6c2c0ad70a1b78b', local_command='scripts/check-changelog-coverage.py'),
        ],
    },
    'qualification': {
        "needs": ['version-check'],
        "steps": [
        ],
    },
    'release-gate': {
        "needs": ['version-check', 'qualification'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('Read declared Rust toolchain', counterpart=('tools/rust-toolchain-version', 'rust-toolchain.toml')),
            Step('dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c'),
            Step('actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065'),
            Step('astral-sh/setup-uv@38f3f104447c67c051c4a08e39b64a148898af3a'),
            Step('Prepare the offline sandbox', sha256='965fc709b77784b3c7604ffcd85236ec692e7daae8979098703946af18ccf4a2'),
            Step('Fetch the cargo registry for the offline marker set', counterpart=('tests/test_release_workflow_provenance.py', 'def test_the_release_gate_marker_job_and_make_run_the_same_selection('), sha256='bd8a3eea8fbfa7f0592cc9160f727c40bf2cac4f30ca475277ea9ce01a8b7403'),
            Step('Run the release-gate marker set', counterpart=('Makefile', 'release-gate-markers:'), sha256='477790ec0a17570da21b61265e5ac11c81b6b047fe889e57570aa4f334f266bb', local_command='make release-gate-markers'),
        ],
    },
    'create-draft': {
        "needs": ['version-check', 'qualification'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('Render the release notes from the changelog', counterpart=('release/release_notes.py', 'def main('), local_command='release/release_notes.py'),
            Step('Create the draft release', counterpart=('tests/test_release_draft_notes.py', 'def test_')),
        ],
    },
    'build': {
        "needs": ['version-check', 'release-gate'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065'),
            Step('Read declared Rust toolchain', counterpart=('tools/rust-toolchain-version', 'rust-toolchain.toml')),
            Step('dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c'),
            Step('arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b'),
            Step('Build release binary', counterpart=('tests/test_release_workflow_provenance.py', 'def test_release_builders_use_admitted_version_and_actual_revision(')),
            Step('Build and package inside the manylinux 2_28 container', counterpart=('tests/test_release_workflow_provenance.py', 'def test_linux_floor_contract(')),
            Step('Package (macOS)', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('Package (Windows)', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('Retain archive and checksum'),
        ],
    },
    'publish-assets': {
        "needs": ['build', 'create-draft'],
        "steps": [
            Step('actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093'),
            Step('Upload release assets', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
        ],
    },
    'pypi-build': {
        "needs": ['version-check', 'release-gate'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683', sha256='17795dceef610a9d0dbdf4d93fa227a7c453b77ebd72a7cc057d592d711bff57'),
            Step('actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065', sha256='004630710366bff7851b00b676ce5044490a91b7220ce97a1f1e48fa16679424'),
            Step('Read declared Rust toolchain', counterpart=('tools/rust-toolchain-version', 'rust-toolchain.toml')),
            Step('dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c', sha256='7cf8d8f0412743a3fe26312d709d9101fe171f1ac7df797b9ef52b6abb3a7e21'),
            Step('arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b', sha256='4e605dbe89d4dcacdf70d170853634c9f4c518e1a41f5d2c5e2bd27cf4c89e7c'),
            Step('Install pinned maturin', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract('), sha256='996e8a6336fff6d3d0b3a74230bb8b24021e34e7d06390412ac88bd2308173e0'),
            Step('Build wheels', counterpart=('tests/test_release_workflow_provenance.py', 'def test_release_builders_use_admitted_version_and_actual_revision('), sha256='cbb986dad5ba25475c9731e0d012cfff7c3cefdbb280bd480e9024e51e6c3337'),
            Step('Build wheels inside the manylinux 2_28 container', counterpart=('tests/test_release_workflow_provenance.py', 'def test_linux_floor_contract('), sha256='8bbf07cb07509fe7c6981b115283ac00e78ca20c5130cfec70509d8cb16d08b6'),
            Step('actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02', sha256='b9b6cca0e685928f03d3f444bac8c0825b2ee3510663d6503c24525218de1e23'),
        ],
    },
    'wheel-smoke': {
        "needs": ['version-check', 'pypi-build'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683', sha256='17795dceef610a9d0dbdf4d93fa227a7c453b77ebd72a7cc057d592d711bff57'),
            Step('actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065', sha256='53e861877de3017c6e648667b07f052d923c6c50c065f7b9b60d72589c0887e0'),
            Step('actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093', sha256='7da61a5393486e202557314e916ce6b47390f55bd62478135e9c7911337da88e'),
            Step('Install the wheel into a clean venv', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract('), sha256='b71c6070feccc452c661c5d04a95d0e016d1e0af74da26175ca57d5244c821bc'),
            Step(NATIVE_WINDOWS_CACHE_STEP, counterpart=('tests/test_installed_wheel_smoke.py', 'def test_main_and_release_use_the_shared_installed_wheel_smoke('), sha256='799830265b580448486a7da39c8fe8414e6a9e35aae1234233f064172e012e42'),
            Step('Run the wheel inside the manylinux 2_28 container', counterpart=('scripts/check-linux-wheel.sh', 'docker run'), sha256='68652b2b9183303c3c7e8a10a889029f8bed17057c66fac871ef97bb75f00bea'),
            Step('Smoke the installed wheel on every shipped platform', counterpart=('scripts/installed-wheel-smoke.sh', '--version'), sha256='ed392815d8efbff2696f38b5fea48c680b371605f9925129ffe956de5333d127', local_command='scripts/installed-wheel-smoke.sh'),
        ],
    },
    'docs-publish': {
        "needs": ['version-check', 'qualification'],
        "steps": [
        ],
    },
    'docs-live': {
        "needs": ['version-check', 'qualification', 'docs-publish'],
        "steps": [
            Step('Check out the gate helper', sha256='ff29379fee6ad185b41206f3cd927d55f6c99711dafe8c11de48998ec8e64373'),
            Step('Resolve the tag commit', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract('), sha256='b9e7731ec89c65f683eb9045aa898fcb7091e0c718f75c1f6ca29bc49d615255'),
            Step('Require the live documentation revision to equal or descend from the tag', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract('), sha256='d8e88f95d2e890e14d314242bcce698767ddd834b4ccbab56469d5fd684eb54d'),
        ],
    },
    'pypi-publish': {
        "needs": ['pypi-build', 'wheel-smoke', 'docs-live', 'build', 'container-publish'],
        "steps": [
            Step('actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093'),
            Step('Install pinned uv'),
            Step('Publish to PyPI with trusted publishing (production only)', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pypi_publish_guard_runs_the_right_publisher_under_each_repository(')),
        ],
    },
    'homebrew-tap': {
        "needs": ['build', 'docs-live', 'wheel-smoke', 'container-publish', 'pypi-publish', 'publish-release'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('Download checksums from the release', counterpart=('tests/test_release_workflow_provenance.py', 'def test_homebrew_guard_renders_and_checks_both_checksums_then_guards_the_push(')),
            Step('Update Homebrew tap (production only)', counterpart=('tests/test_release_workflow_provenance.py', 'def test_homebrew_guard_renders_and_checks_both_checksums_then_guards_the_push(')),
        ],
    },
    'container-publish': {
        "needs": ['build', 'docs-live', 'version-check', 'wheel-smoke', 'create-draft', 'publish-assets'],
        "steps": [
            Step('Check out the packaging ref'),
            Step('Resolve the image version and source revision', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('Stage the released Linux executables', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('Set up QEMU'),
            Step('Set up Buildx'),
            Step('Log in to GHCR'),
            Step('Build and push the versioned image'),
            Step('Smoke the linux/amd64 image from the registry', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('Smoke the linux/arm64 image from the registry', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
        ],
    },
    'publish-release': {
        "needs": ['build', 'pypi-publish', 'container-publish', 'docs-live'],
        "steps": [
            Step('Check out the exact revision'),
            Step('docker/setup-buildx-action@e468171a9de216ec08956ac3ada2f0791b6bd435'),
            Step('docker/login-action@9780b0c442fbb1117ed29e0efdff1e18412f7567'),
            Step('Move latest after both platform smokes pass', counterpart=('tests/test_release_workflow_provenance.py', 'def test_the_latest_step_fails_when_gh_fails(')),
            Step('Publish the GitHub release after every publisher succeeds', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
        ],
    },
    'rehearsal-docs': {
        "needs": ['version-check', 'qualification'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065'),
            Step('astral-sh/setup-uv@38f3f104447c67c051c4a08e39b64a148898af3a'),
            Step('Validate the candidate documentation without deployment', counterpart=('tests/test_release_workflow_provenance.py', 'def test_rehearsal_builds_have_no_publication_permissions_or_credentials(')),
            Step('actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02'),
        ],
    },
    'rehearsal-container': {
        "needs": ['version-check', 'build', 'wheel-smoke'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093'),
            Step('Stage the checked Linux executables', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('docker/setup-qemu-action@e77e8065d9f7ec6abdd9838668cd7b43924dd64d'),
            Step('docker/setup-buildx-action@e468171a9de216ec08956ac3ada2f0791b6bd435'),
            Step('Build the production Dockerfile without a registry push'),
            Step('Smoke the local platform image and retain it', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02'),
        ],
    },
    'rehearsal-payloads': {
        "needs": ['version-check', 'build', 'pypi-build'],
        "steps": [
            Step('actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683'),
            Step('actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093'),
            Step('actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093'),
            Step('astral-sh/setup-uv@38f3f104447c67c051c4a08e39b64a148898af3a'),
            Step('Validate release publication payloads without credentials', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
            Step('actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02'),
        ],
    },
    'rehearsal-complete': {
        "needs": ['qualification', 'release-gate', 'build', 'pypi-build', 'wheel-smoke', 'rehearsal-docs', 'rehearsal-container', 'rehearsal-payloads'],
        "steps": [
            Step('Record the limits of a successful rehearsal', counterpart=('tests/test_release_workflow_provenance.py', 'def test_pipeline_jobs_contract(')),
        ],
    },
}

EXPECTED_NEEDS = {job: spec["needs"] for job, spec in RELEASE_JOBS.items()}
COUNTERPARTS = {
    job: {step.name: step.counterpart for step in spec["steps"] if step.counterpart}
    for job, spec in RELEASE_JOBS.items()
}
LOCAL_COMMANDS = {
    (job, step.name): step.local_command
    for job, spec in RELEASE_JOBS.items() for step in spec["steps"] if step.local_command
}
PINNED_STEPS = {
    (job, step.name): step.sha256
    for job, spec in RELEASE_JOBS.items() for step in spec["steps"] if step.sha256
}


def step_id(step: dict) -> str:
    return step.get("name") or step.get("uses") or "<unnamed>"


def assert_step_inventory(parsed: dict) -> None:
    assert set(parsed["jobs"]) == set(RELEASE_JOBS), (
        f"release job set changed; update {INVENTORY_PATH} with the workflow"
    )
    for job, spec in RELEASE_JOBS.items():
        expected = [step.name for step in spec["steps"]]
        actual = [step_id(step) for step in parsed["jobs"][job].get("steps", [])]
        assert actual == expected, (
            f"{job}: the pinned job's step list must match exactly.\n"
            f"  expected: {expected}\n  actual:   {actual}\n"
            f"Update {INVENTORY_PATH} with the workflow's deliberate step change."
        )


@pytest.fixture(scope="module")
def release_inventory_state(request: pytest.FixtureRequest) -> AssertionError | None:
    root = Path(request.module.__file__).resolve().parents[1]
    workflow = yaml.safe_load((root / ".github/workflows/release.yml").read_text())
    try:
        assert_step_inventory(workflow)
    except AssertionError as error:
        return error
    return None


@pytest.fixture(autouse=True)
def release_inventory_admission(
    request: pytest.FixtureRequest, release_inventory_state: AssertionError | None,
) -> None:
    """One selected contract owns structural drift; semantic mutations still run."""
    if release_inventory_state is None:
        return
    selected = [item for item in request.session.items if item.path.name in {
        "test_release_check_counterparts.py", "test_release_workflow_provenance.py",
    }]
    owners = ("test_release_commands_have_local_or_platform_counterparts",
              "test_pipeline_jobs_contract")
    owner = next((item for name in owners for item in selected
                  if item.nodeid.endswith("::" + name)), selected[0])
    if request.node.nodeid == owner.nodeid:
        # These two entry points assert inventory themselves. A narrower selected
        # semantic test has no such entry point, so its setup must fail closed.
        if owner.name not in owners:
            raise release_inventory_state
        return
    pytest.skip(f"release inventory admission failed; update {INVENTORY_PATH}")
