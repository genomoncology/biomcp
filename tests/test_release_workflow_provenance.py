from __future__ import annotations

import re
from pathlib import Path

import pytest
import yaml

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
    # PyPI last (2026-09-28): every build and smoke job first.
    "pypi-publish": ["pypi-build", "wheel-smoke", "docs-live", "build", "container-publish"],
    # Ticket 1266: the release is public before the tap lands.
    "homebrew-tap": [
        "build",
        "docs-live",
        "wheel-smoke",
        "container-publish",
        "pypi-publish",
        "publish-release",
    ],
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
        # The text read takes the FIRST needs line; the YAML parser
        # keeps the LAST. A duplicated needs line diverges here, and
        # each job block may carry at most one.
        block = _job_block(workflow, job)
        try:
            parsed_needs = yaml.safe_load(workflow)["jobs"][job].get("needs", [])
        except yaml.YAMLError as error:
            raise AssertionError(
                f"the release workflow no longer parses as YAML: {error}"
            ) from error
        assert list(parsed_needs) == needs, (
            f"{job}: needs as the runner parses them must be exactly "
            f"{needs}; a duplicated needs line shows up here"
        )
        assert len(re.findall(r"^    needs: ", block, re.MULTILINE)) <= 1, (
            f"{job}: duplicated needs line"
        )

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
    for escape in ("|| true", "|| :", "; exit 0", "if: false", "if: ${{ false }}"):
        assert escape not in version, escape

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
    # The latest decision is a version comparison against published
    # releases (2026-09-30 blocker fix): the draft-skipping
    # "gh release view" call is gone entirely.
    assert "gh release view" not in final
    assert "gh release list" in final and "--exclude-drafts" in final
    assert "scripts/should-move-latest.sh" in final
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


# ---------------------------------------------------------------------------
# Data-driven contract over the parsed workflow for the pipeline jobs the
# issue's mutations target: pypi-build, wheel-smoke, and docs-live. The
# version-check no-escape assertions above stay text-based; these cover
# every step of the three jobs plus the trigger block, so a new step in
# any of them is guarded without a new hand-written assertion.
# ---------------------------------------------------------------------------

PIPELINE_JOBS = ("pypi-build", "wheel-smoke", "docs-live")

# A closed allowlist: any other step-level condition in the three
# pipeline jobs is a way to switch a check off, whatever syntax it
# uses (false, ${{ false }}, ${{ false && true }}, ${{true}}, ...).
# Only the conditions actually used by steps in the guarded jobs. A
# new legitimate condition requires an explicit, reviewable edit here.
ALLOWED_STEP_IFS = {
    "runner.os == 'Linux'",
    "runner.os != 'Linux'",
}

# Exit-code swallows are not banned by phrase: every step of the
# pinned jobs below is hash-pinned, so `|| echo skip`,
# `set +o errexit`, `trap -- 'exit 0' EXIT`, `|| exit $((0))`, or any
# other spelling changes the step's text and fails the hash. The
# patterns are gone on purpose — structure closed the class.
#
# Hash pinning: each load-bearing step is serialized from the parsed
# workflow (so YAML folding cannot hide a change), its `run` body is
# normalized to trivial whitespace (per-line trailing space stripped,
# blank runs collapsed to one blank line), and the SHA-256 of the
# canonical JSON is recorded below. ANY edit to a pinned step — run
# text, `if:`, `shell:`, `with:`, `continue-on-error`, `env:` — fails
# with the step's name and the new hash; deliberate changes update
# the hash here in the same commit.
    # Steps of the three pipeline jobs plus version-check's gate
    # steps, pinned 2026-09-27 (ticket 1258). Update a hash only in
    # the same commit as the deliberate workflow edit it records.
    # The two pypi-build build steps were re-pinned 2026-10-08
    # (ticket 2024): both now route through tools/with-build-identity
    # so the shipped wheels report their commit.
PINNED_STEPS: dict[tuple[str, str], str] = {
    ("pypi-build", "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683"): "435261111ae8c13c6efd4c0122d30af2df68e5cbc57a7e92ccd1568897ae4024",
    ("pypi-build", "actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065"): "004630710366bff7851b00b676ce5044490a91b7220ce97a1f1e48fa16679424",
    ("pypi-build", "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c"): "e5cf6427d1b42e2b776f2589adc36320874f109751f02494b68ee9068c79b0d5",
    ("pypi-build", "arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b"): "4e605dbe89d4dcacdf70d170853634c9f4c518e1a41f5d2c5e2bd27cf4c89e7c",
    ("pypi-build", "Install pinned maturin"): "996e8a6336fff6d3d0b3a74230bb8b24021e34e7d06390412ac88bd2308173e0",
    ("pypi-build", "Build wheels"): "cbb986dad5ba25475c9731e0d012cfff7c3cefdbb280bd480e9024e51e6c3337",
    ("pypi-build", "Build wheels inside the manylinux 2_28 container"): "d04d98be4ff1c1caee89a3337461678427197f11d91b9a36c4b8006413578b9b",
    ("pypi-build", "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"): "b9b6cca0e685928f03d3f444bac8c0825b2ee3510663d6503c24525218de1e23",
    ("wheel-smoke", "actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065"): "53e861877de3017c6e648667b07f052d923c6c50c065f7b9b60d72589c0887e0",
    ("wheel-smoke", "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093"): "7da61a5393486e202557314e916ce6b47390f55bd62478135e9c7911337da88e",
    ("wheel-smoke", "Install the wheel into a clean venv"): "362a4ba663ff744574ec1f229c6ecd32e1f0f308ae95f0000f79efbf089bf6f9",
    ("wheel-smoke", "Run the wheel inside the manylinux 2_28 container"): "d5495643fdb712f52fbaa636b5198909e890806d92a309d073dfc1bb328cffc4",
    ("wheel-smoke", "Smoke the installed wheel on every shipped platform"): "3177e159d9592abf0e9635ff7afb9f31bcb9906a9b8308f838a993500a7e3c6e",
    ("docs-live", "Check out the gate helper"): "afce43fafcab696d9cef03f29b0c43b6c9849baf126b749e198bdb9d83555430",
    ("docs-live", "Resolve the tag commit"): "b52e25a4026bed9172a0eff4b90f6a706ec984875307d7e89f9d450878a96e44",
    ("docs-live", "Require the live documentation revision to equal or descend from the tag"): "d8e88f95d2e890e14d314242bcce698767ddd834b4ccbab56469d5fd684eb54d",
    ("version-check", "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683"): "3ffb87228f0c77aecb87c34ee83d661a80bff50e397e911e917de5891db2a5fb",
    ("version-check", "Require the tag and committed versions to agree"): "8bdcc74fd872a7a77d8341bd5049e3a3845a89825f02875f4258e5d665243b8b",
    ("version-check", "Require the changelog to cover merged tickets"): "8dcf47fdb31ff44771a511058fa05d8e17d1a551744dcc1e8631fa7f8979118e",
}


# The ENTIRE step list of each pinned job, in order. The hash pins
# catch edits to a pinned step; this catches a step ADDED before or
# between them — the BASH_ENV-writer attack, where a new first step
# poisons the environment and every pinned step still hashes clean.
PINNED_JOB_STEP_LISTS: dict[str, list[str]] = {
    "pypi-build": [
        "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683",
        "actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065",
        "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
        "arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b",
        "Install pinned maturin",
        "Build wheels",
        "Build wheels inside the manylinux 2_28 container",
        "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
    ],
    "wheel-smoke": [
        "actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065",
        "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
        "Install the wheel into a clean venv",
        "Run the wheel inside the manylinux 2_28 container",
        "Smoke the installed wheel on every shipped platform",
    ],
    "docs-live": [
        "Check out the gate helper",
        "Resolve the tag commit",
        "Require the live documentation revision to equal or descend from the tag",
    ],
    "version-check": [
        "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683",
        "Require the tag and committed versions to agree",
        "Require the changelog to cover merged tickets",
    ],
    # 2026-09-28 round three: EVERY release job is pinned, not only
    # the four above — pypi-publish uploads to PyPI, so a new first
    # step there (a BASH_ENV writer via $GITHUB_ENV, a swapped uv)
    # must fail even when no hash exists to drift.
    "pypi-publish": [
        "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
        "Install pinned uv",
        "Publish to PyPI with trusted publishing",
    ],
    "homebrew-tap": [
        "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683",
        "Download checksums from the release",
        "Update Homebrew tap",
    ],
    "build": [
        "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683",
        # Ticket 2024: the interpreter for tools/with-build-identity,
        # which the host build steps run under so the shipped
        # binaries report their commit.
        "actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065",
        "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
        "arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b",
        "Build release binary",
        "Build and package inside the manylinux 2_28 container",
        "Package (macOS)",
        "Package (Windows)",
        "Upload release assets",
    ],
    "container-publish": [
        "Check out the packaging ref",
        "Resolve the image version and source revision",
        "Stage the released Linux executables",
        "Set up QEMU",
        "Set up Buildx",
        "Log in to GHCR",
        "Build and push the versioned image",
        "Smoke the linux/amd64 image from the registry",
        "Smoke the linux/arm64 image from the registry",
    ],
    "publish-release": [
        # The repository must be checked out before any repository
        # script runs (2026-09-30 third go-request review).
        "Check out the exact revision",
        # Docker work precedes the public flip (2026-09-30 review).
        "docker/setup-buildx-action@e468171a9de216ec08956ac3ada2f0791b6bd435",
        "docker/login-action@9780b0c442fbb1117ed29e0efdff1e18412f7567",
        "Move latest after both platform smokes pass",
        "Publish the GitHub release after every publisher succeeds",
    ],
    "create-draft": [
        "Create the draft release",
    ],
}


def _step_id(step: dict) -> str:
    return step.get("name") or step.get("uses") or "<unnamed>"


def _assert_pinned_step_lists(parsed: dict) -> None:
    """Each pinned job's step list matches exactly: ids, order, count."""
    for job, expected in PINNED_JOB_STEP_LISTS.items():
        actual = [_step_id(step) for step in _pipeline_steps(parsed, job)]
        assert actual == expected, (
            f"{job}: the pinned job's step list must match exactly.\n"
            f"  expected: {expected}\n"
            f"  actual:   {actual}\n"
            f"A new or reordered step in a pinned job fails here even "
            f"when every pinned step still hashes clean."
        )


def _assert_no_bash_env(parsed: dict) -> None:
    """BASH_ENV is banned in every env block and as a workflow key.

    A BASH_ENV pointing at `trap 'exit 0' EXIT` reinterprets every
    `bash -e` run in the job; hash pins cannot see it because the
    step text never changes.
    """
    def check_env(env: object, where: str) -> None:
        if isinstance(env, dict):
            for name in env:
                # BASH_ENV reinterprets every bash -e run; BASH_FUNC_
                # names are imported as shell functions by bash.
                assert "BASH_ENV" not in name, (
                    f"{where}: BASH_ENV is forbidden in every env block"
                )
                assert not name.startswith("BASH_FUNC_"), (
                    f"{where}: {name} is a bash function-import variable "
                    f"(BASH_FUNC_name%% is executed by every bash run)"
                )
        elif isinstance(env, str) and env:
            # Expression-form env blocks parse as strings, not dicts;
            # a fromJSON payload can still carry the same names.
            assert "BASH_ENV" not in env, (
                f"{where}: expression env names BASH_ENV: {env[:60]}"
            )
            assert "BASH_FUNC_" not in env, (
                f"{where}: expression env names a BASH_FUNC_ variable"
            )

    check_env(parsed.get("env"), "workflow-level env")
    assert "BASH_ENV" not in parsed, "workflow-level BASH_ENV key is forbidden"
    # Workflow-level PATH or ENV rewrites the environment of every
    # job in the file — a hijacked bin dir or options file reaches
    # even the pinned steps. Job and step PATH stay legal (toolchain
    # paths are set there legitimately).
    workflow_env = parsed.get("env") or {}
    if isinstance(workflow_env, dict):
        for name in workflow_env:
            assert name not in ("PATH", "ENV"), (
                f"workflow-level env {name} rewrites every job's "
                f"environment; set it per job or per step"
            )
    for job, spec in parsed["jobs"].items():
        check_env(spec.get("env"), f"{job} job env")
        for index, step in enumerate(spec.get("steps", [])):
            check_env(step.get("env"), f"{job} step {index} env")
            for key, value in (step.get("with") or {}).items():
                if isinstance(value, str):
                    assert "BASH_ENV" not in value, (
                        f"{job} step {index} with.{key} names BASH_ENV"
                    )


def _normalize_step_text(text: str) -> str:
    lines = [line.rstrip() for line in text.splitlines()]
    out: list[str] = []
    blank = 0
    for line in lines:
        if line == "":
            blank += 1
            if blank <= 1:
                out.append(line)
        else:
            blank = 0
            out.append(line)
    return "\n".join(out).strip("\n")


def _canonical_step(step: dict) -> str:
    import copy
    import json as _json

    normalized = copy.deepcopy(step)
    if isinstance(normalized.get("run"), str):
        normalized["run"] = _normalize_step_text(normalized["run"])
    return _json.dumps(normalized, sort_keys=True, ensure_ascii=False, indent=1)


def _step_sha(step: dict) -> str:
    import hashlib

    return hashlib.sha256(_canonical_step(step).encode("utf-8")).hexdigest()

EXPECTED_MATRICES = {
    "pypi-build": [
        {
            "os": "ubuntu-24.04",
            "target": "x86_64-unknown-linux-gnu",
            "container": "quay.io/pypa/manylinux_2_28_x86_64",
        },
        {
            "os": "ubuntu-24.04-arm",
            "target": "aarch64-unknown-linux-gnu",
            "container": "quay.io/pypa/manylinux_2_28_aarch64",
        },
        {"os": "macos-14", "target": "aarch64-apple-darwin"},
        {"os": "macos-latest", "target": "x86_64-apple-darwin"},
        {"os": "windows-latest", "target": "x86_64-pc-windows-msvc"},
    ],
    "wheel-smoke": [
        {
            "os": "ubuntu-24.04",
            "artifact": "wheel-x86_64-unknown-linux-gnu",
            "executable": "bin/biomcp",
            "python": "bin/python",
            "container": "quay.io/pypa/manylinux_2_28_x86_64",
        },
        {
            "os": "ubuntu-24.04-arm",
            "artifact": "wheel-aarch64-unknown-linux-gnu",
            "executable": "bin/biomcp",
            "python": "bin/python",
            "container": "quay.io/pypa/manylinux_2_28_aarch64",
        },
        {
            "os": "macos-14",
            "artifact": "wheel-aarch64-apple-darwin",
            "executable": "bin/biomcp",
            "python": "bin/python",
        },
        {
            "os": "macos-15-intel",
            "artifact": "wheel-x86_64-apple-darwin",
            "executable": "bin/biomcp",
            "python": "bin/python",
        },
        {
            "os": "windows-latest",
            "artifact": "wheel-x86_64-pc-windows-msvc",
            "executable": "Scripts/biomcp.exe",
            "python": "Scripts/python.exe",
        },
    ],
}

# The only step in the release flow allowed to tolerate failure: the
# optional protoc installer, whose generated code is committed.
CONTINUE_ON_ERROR_ALLOW = ("arduino/setup-protoc",)


def _load_release_pipeline() -> dict:
    return yaml.safe_load(RELEASE_WORKFLOW.read_text(encoding="utf-8"))


def _pipeline_steps(parsed: dict, job: str) -> list[dict]:
    return parsed["jobs"][job]["steps"]


def _step_by_name(parsed: dict, job: str, needle: str) -> dict:
    for step in _pipeline_steps(parsed, job):
        if needle in (step.get("name") or step.get("uses") or ""):
            return step
    raise AssertionError(f"no step matching {needle!r} in {job}")


def _assert_step_hashes(parsed: dict) -> None:
    """Every pinned step matches its recorded SHA-256 exactly."""
    for (job, needle), expected in PINNED_STEPS.items():
        step = _step_by_name(parsed, job, needle)
        actual = _step_sha(step)
        assert actual == expected, (
            f"{job}/{needle}: step text drifted from its pinned hash.\n"
            f"  expected sha256: {expected}\n"
            f"  actual   sha256: {actual}\n"
            f"If the edit is deliberate, update PINNED_STEPS in this file "
            f"in the same commit (the workflow comment names the pins)."
        )


def _assert_no_defaults_shell(parsed: dict) -> None:
    """`shell:` may exist only inside a step.

    A job- or workflow-level `defaults.run.shell: bash {0}` would
    swallow step failures for every step at once, so no `defaults`
    key may exist anywhere in the release workflow.
    """
    assert "defaults" not in parsed, (
        "workflow-level defaults are forbidden (a defaults.run.shell "
        "would apply to every step at once)"
    )
    for job, spec in parsed["jobs"].items():
        assert "defaults" not in spec, f"{job}: job-level defaults are forbidden"


def _assert_pipeline_contract(parsed: dict) -> None:
    _assert_pinned_step_lists(parsed)
    _assert_no_bash_env(parsed)
    # The trigger block: tag pushes only, never a release publication
    # event (the YAML 1.1 `on:` key parses as boolean True).
    triggers = parsed[True]
    assert triggers["push"] == {"tags": ["v*"]}, "trigger block must be tag pushes only"
    assert "release" not in triggers, "trigger block must be tag pushes only"
    assert "workflow_dispatch" in triggers, (
        "trigger block must keep the manual dispatch"
    )

    for job in PIPELINE_JOBS:
        job_spec = parsed["jobs"][job]
        assert "continue-on-error" not in job_spec, (
            f"{job}: job-level continue-on-error is forbidden"
        )
        steps = _pipeline_steps(parsed, job)
        assert steps, f"{job}: job must have steps"
        for step in steps:
            uses = step.get("uses", "")
            # Only the optional protoc installer may tolerate failure.
            # Forbidding the key outright catches every spelling of
            # true: the boolean, "true", "${{ true }}", and numbers.
            if "continue-on-error" in step:
                assert any(allow in uses for allow in CONTINUE_ON_ERROR_ALLOW), (
                    f"{job}: continue-on-error is forbidden outside the protoc installer"
                )
            step_if = step.get("if")
            if step_if is not None:
                assert step_if in ALLOWED_STEP_IFS, (
                    f"{job}: step if {step_if!r} is not in the allowlist"
                )
            shell = step.get("shell")
            assert shell in (None, "bash"), (
                f"{job}: custom shell {shell!r} is forbidden; declare shell: bash"
            )

    # Matrix entries are pinned exactly, so a swapped runner, a new
    # leg, or a dropped leg (including the ARM wheel build and smoke)
    # breaks the contract.
    for job, expected in EXPECTED_MATRICES.items():
        matrix = parsed["jobs"][job]["strategy"]["matrix"]["include"]
        assert matrix == expected, f"{job}: matrix entries must be pinned exactly"
    assert parsed["jobs"]["docs-live"].get("strategy") is None
    assert parsed["jobs"]["docs-live"]["runs-on"] == "ubuntu-24.04"

    # The floor check runs in both wheel legs: the two Linux matrix
    # entries above carry their containers, and the container build
    # step scans the artifact that ships.
    wheel_build = _step_by_name(
        parsed, "pypi-build", "Build wheels inside the manylinux 2_28 container"
    )
    assert "check-wheel-glibc-floor.py" in wheel_build["run"], (
        "pypi-build: the wheel build step must run the floor check"
    )

    smoke = _step_by_name(parsed, "wheel-smoke", "Smoke the installed wheel")
    # The panic guard must return failure, not just print.
    assert re.search(
        r'echo "panic or crash \(exit \$status\) from: biomcp \$\*" >&2\s*\n\s*return 1',
        smoke["run"],
    ), "wheel-smoke: the panic guard must return failure"
    # The not-found exit check runs after the adverse-events probe.
    run = smoke["run"]
    probe = run.index("drug adverse-events qwertyzzznonexistent999")
    assert '[ "$status" -eq 0 ]' in run[probe:], (
        "wheel-smoke: the not-found exit check must follow the adverse-events probe"
    )
    assert 'grep -q "Drug not found in FAERS"' in run[probe:], (
        "wheel-smoke: the not-found exit check must follow the adverse-events probe"
    )

    # The runtime floor smoke stays on the Linux legs only.
    floor_smoke = _step_by_name(
        parsed, "wheel-smoke", "Run the wheel inside the manylinux 2_28 container"
    )
    assert floor_smoke["if"] == "runner.os == 'Linux'", (
        "wheel-smoke: the runtime floor smoke must stay on the Linux legs"
    )

    # docs-live: the tag lookup must assert the SHA it resolved, and
    # the revision timeout branch must fail the step.
    resolve = _step_by_name(parsed, "docs-live", "Resolve the tag commit")
    assert 'test -n "$TAG_SHA"' in resolve["run"], (
        "docs-live: the tag lookup must assert the resolved SHA"
    )
    revision = _step_by_name(
        parsed, "docs-live", "Require the live documentation revision"
    )
    assert re.search(
        r"did not reach \$\{TAG_SHA\}[^\"\n]*\" >&2\s*\n\s*exit 1\b",
        revision["run"],
    ), "docs-live: the revision timeout branch must end in exit 1"


def test_pipeline_jobs_contract() -> None:
    parsed = _load_release_pipeline()
    _assert_pipeline_contract(parsed)
    _assert_publish_gating(parsed)
    _assert_no_defaults_shell(parsed)
    _assert_step_hashes(parsed)
    _assert_release_locked_builds(parsed)
    _assert_build_identity_routing(parsed)


def _assert_build_identity_routing(parsed: dict) -> None:
    """Ticket 2024: released binaries must report their commit. The
    0.9.1 wheels and tarballs shipped "git unknown, build unknown"
    because the release workflow called cargo and maturin directly.
    Host build steps run under tools/with-build-identity; the
    manylinux container legs resolve the identity on the host (the
    container has no git checkout of its own) and the container
    build inherits it through --env-file.
    """
    for job in ("build", "pypi-build"):
        for index, step in enumerate(parsed["jobs"][job]["steps"]):
            run = str(step.get("run", ""))
            where = f"{job} step {index}"
            if "docker run" in run:
                assert "tools/with-build-identity env" in run, (
                    f"{where}: the container build must resolve the identity "
                    "through tools/with-build-identity on the host checkout"
                )
                assert "grep '^BIOMCP_BUILD_'" in run, (
                    f"{where}: the identity hand-off must select the "
                    "BIOMCP_BUILD_ variables"
                )
                assert "--env-file" in run, (
                    f"{where}: the container build must inherit the identity "
                    "through --env-file; docker starts the command so "
                    "actionlint keeps checking the container script"
                )
                continue
            for line in run.splitlines():
                if re.search(r"\b(?:cargo|maturin) build\b", line):
                    assert "with-build-identity" in line, (
                        f"{where}: {line.strip()[:60]} must run under "
                        "tools/with-build-identity or the shipped binary "
                        'prints "git unknown, build unknown"'
                    )


def test_build_identity_routing_assertion_catches_a_direct_build() -> None:
    """Red proof: dropping the with-build-identity routing from a
    host build step (the 0.9.1 shape) fails the routing contract."""
    import copy

    parsed = _load_release_pipeline()
    planted = copy.deepcopy(parsed)
    step = _step_by_name(planted, "pypi-build", "Build wheels")
    step["run"] = step["run"].replace("tools/with-build-identity ", "")
    assert "maturin build" in step["run"] and "with-build-identity" not in step["run"]
    with pytest.raises(AssertionError, match="with-build-identity"):
        _assert_build_identity_routing(planted)


def _assert_release_locked_builds(parsed: dict) -> None:
    """Ticket 1266: every build step in the release pipeline names
    --release and --locked directly, not only through a hash pin a
    repin could clear. The 0.9.0 debug-profile wheel shipped because
    no assertion stated the profile (#287).
    """
    offenders: list[str] = []
    for job_id in ("pypi-build", "build"):
        steps = parsed["jobs"][job_id]["steps"]
        for index, step in enumerate(steps):
            run = step.get("run", "")
            # Per command, not per step (2026-09-29 second review):
            # a step mixing an unlocked build with a locked one
            # passed the substring check.
            build_commands = [
                line
                for line in run.splitlines()
                if "maturin build" in line or "cargo build" in line
            ]
            if not build_commands:
                continue
            offenders.extend(
                f"{job_id} step {index}: {line.strip()[:70]}"
                for line in build_commands
                if "--release" not in line or "--locked" not in line
            )
    assert not offenders, (
        "every wheel and tarball build must pass --release --locked "
        f"(GitHub #287): {offenders}"
    )


def test_release_locked_build_assertion_catches_a_dropped_flag(tmp_path: Path) -> None:
    """Red proof: planting a workflow copy with --release dropped
    from one build step fails the assertion."""
    import copy

    parsed = _load_release_pipeline()
    planted = copy.deepcopy(parsed)
    steps = planted["jobs"]["pypi-build"]["steps"]
    for step in steps:
        run = step.get("run", "")
        if "maturin build" in run:
            step["run"] = run.replace(" --release", "")
            break
    else:
        pytest.fail("pypi-build carries no maturin build step to plant")
    with pytest.raises(AssertionError, match="--release"):
        _assert_release_locked_builds(planted)


# Every job's condition, asserted exactly. A suffix like
# `&& !cancelled()` would re-run a publish after an upstream failure
# and must break the contract.
EXPECTED_JOB_IFS = {
    "create-draft": "github.event_name == 'push'",
    "build": "github.event_name == 'push'",
    "pypi-build": "github.event_name == 'push'",
    "wheel-smoke": "github.event_name == 'push'",
    "pypi-publish": "github.event_name == 'push'",
    "homebrew-tap": "github.event_name == 'push'",
    "publish-release": "github.event_name == 'push'",
    "container-publish": (
        "!cancelled() && ((github.event_name == 'push' && success()) || "
        "(inputs.container_only == true && needs.version-check.result == 'success' && "
        "needs.docs-live.result == 'success' && needs.create-draft.result == 'skipped' && "
        "needs.build.result == 'skipped' && needs['wheel-smoke'].result == 'skipped'))"
    ),
}


def _assert_publish_gating(parsed: dict) -> None:
    for job in ("version-check", "docs-live"):
        assert "if" not in parsed["jobs"][job], f"{job}: must run unconditionally"
    for job, expected in EXPECTED_JOB_IFS.items():
        actual = parsed["jobs"][job].get("if")
        assert actual == expected, (
            f"{job}: job if must be exactly {expected!r}, got {actual!r}"
        )
    # `!cancelled()` re-runs a job after an upstream failure. Only the
    # container rebuild path may use it; a publish path never may.
    for job, spec in parsed["jobs"].items():
        if job == "container-publish":
            continue
        assert "!cancelled()" not in (spec.get("if") or ""), (
            f"{job}: !cancelled() is forbidden"
        )
        for step in spec.get("steps", []):
            assert "!cancelled()" not in (step.get("if") or ""), (
                f"{job}: !cancelled() is forbidden in a step if"
            )


def _set_job_flag(job: str, key: str, value):
    def mutate(parsed: dict) -> None:
        parsed["jobs"][job][key] = value

    return mutate


def _mutated_pipeline(mutate) -> dict:
    parsed = _load_release_pipeline()
    mutate(parsed)
    return parsed


def _set_step_flag(job: str, needle: str, key: str, value):
    def mutate(parsed: dict) -> None:
        _step_by_name(parsed, job, needle)[key] = value

    return mutate


def _mutate_run(job: str, needle: str, transform):
    def mutate(parsed: dict) -> None:
        step = _step_by_name(parsed, job, needle)
        step["run"] = transform(step["run"])

    return mutate


def _drop_matrix_entry(job: str, target: str):
    def mutate(parsed: dict) -> None:
        include = parsed["jobs"][job]["strategy"]["matrix"]["include"]
        kept = [entry for entry in include if entry.get("target") != target]
        assert len(kept) < len(include), target
        parsed["jobs"][job]["strategy"]["matrix"]["include"] = kept

    return mutate


def _retarget_matrix_os(job: str, key: str, value: str, os_name: str):
    def mutate(parsed: dict) -> None:
        include = parsed["jobs"][job]["strategy"]["matrix"]["include"]
        for entry in include:
            if entry.get(key) == value:
                entry["os"] = os_name
                return
        raise AssertionError(value)

    return mutate


HASH_DRIFT = "step text drifted from its pinned hash"

PIPELINE_MUTATIONS = {
    # --- Leak spellings the old phrase bans never named (ticket
    # --- 1258): each edits a pinned step, so the hash fails.
    "or_echo_skip_on_the_venv_smoke": (
        _mutate_run(
            "wheel-smoke",
            "Smoke the installed wheel",
            lambda run: run.replace("set -euo pipefail", "set -euo pipefail || echo skip", 1)
            if "set -euo pipefail" in run
            else run + "\ntrue || echo skip\n",
        ),
        HASH_DRIFT,
    ),
    "set_plus_o_errexit_in_the_venv_smoke": (
        _mutate_run(
            "wheel-smoke", "Smoke the installed wheel", lambda run: "set +o errexit\n" + run
        ),
        HASH_DRIFT,
    ),
    "trap_dash_dash_exit_zero_in_the_venv_smoke": (
        _mutate_run(
            "wheel-smoke",
            "Smoke the installed wheel",
            lambda run: run + "\ntrap -- 'exit 0' EXIT\n",
        ),
        HASH_DRIFT,
    ),
    "arithmetic_exit_zero_on_the_floor_check": (
        _mutate_run(
            "pypi-build",
            "Build wheels inside the manylinux 2_28 container",
            lambda run: run.replace(
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28",
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28 || exit $((0))",
            ),
        ),
        HASH_DRIFT,
    ),
    "one_character_edit_in_a_pinned_step": (
        # A single character of pinned prose fails the hash: the pin
        # covers everything, not a list of known escapes. (The step's
        # exit-1 branch has its own content assertion, so the mutated
        # character is prose, not the exit code.)
        _mutate_run(
            "docs-live",
            "Require the live documentation revision",
            lambda run: run.replace("revision", "revixion", 1),
        ),
        HASH_DRIFT,
    ),
    "runner_os_linux_if_on_the_main_smoke_step": (
        # Would turn off the macOS and Windows smoke legs entirely.
        _set_step_flag(
            "wheel-smoke", "Smoke the installed wheel", "if", "runner.os == 'Linux'"
        ),
        HASH_DRIFT,
    ),
    "workflow_level_defaults_shell_bash_brace_zero": (
        lambda parsed: parsed.update(
            {"defaults": {"run": {"shell": "bash {0}"}}}
        ),
        "workflow-level defaults are forbidden",
    ),
    "job_level_defaults_shell_bash_brace_zero": (
        lambda parsed: parsed["jobs"]["wheel-smoke"].update(
            {"defaults": {"run": {"shell": "bash {0}"}}}
        ),
        "job-level defaults are forbidden",
    ),
    "continue_on_error_on_venv_smoke": (
        _set_step_flag(
            "wheel-smoke", "Smoke the installed wheel", "continue-on-error", True
        ),
        "continue-on-error is forbidden outside the protoc installer",
    ),
    "continue_on_error_template_true_on_venv_smoke": (
        _set_step_flag(
            "wheel-smoke",
            "Smoke the installed wheel",
            "continue-on-error",
            "${{ true }}",
        ),
        "continue-on-error is forbidden outside the protoc installer",
    ),
    "continue_on_error_quoted_true_on_wheel_smoke_job": (
        _set_job_flag("wheel-smoke", "continue-on-error", "true"),
        "job-level continue-on-error is forbidden",
    ),
    "continue_on_error_on_docs_live": (
        _set_step_flag(
            "docs-live",
            "Require the live documentation revision",
            "continue-on-error",
            True,
        ),
        "continue-on-error is forbidden outside the protoc installer",
    ),
    "continue_on_error_on_a_non_protoc_uses_step": (
        _set_step_flag(
            "wheel-smoke", "actions/download-artifact", "continue-on-error", True
        ),
        "continue-on-error is forbidden outside the protoc installer",
    ),
    "or_true_on_the_floor_check": (
        _mutate_run(
            "pypi-build",
            "Build wheels inside the manylinux 2_28 container",
            lambda run: run.replace(
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28",
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28 || true",
            ),
        ),
        "step text drifted from its pinned hash",
    ),
    "or_exit_zero_on_the_floor_check": (
        _mutate_run(
            "pypi-build",
            "Build wheels inside the manylinux 2_28 container",
            lambda run: run.replace(
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28",
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28||exit 0",
            ),
        ),
        "step text drifted from its pinned hash",
    ),
    "set_plus_e_in_the_venv_smoke": (
        _mutate_run(
            "wheel-smoke",
            "Smoke the installed wheel",
            lambda run: "set +e\n" + run,
        ),
        "step text drifted from its pinned hash",
    ),
    "trap_exit_zero_in_the_venv_smoke": (
        _mutate_run(
            "wheel-smoke",
            "Smoke the installed wheel",
            lambda run: run + "\ntrap 'exit 0' EXIT\n",
        ),
        "step text drifted from its pinned hash",
    ),
    "tag_lookup_or_exit_zero": (
        _mutate_run(
            "docs-live",
            "Resolve the tag commit",
            lambda run: run.replace(
                'test -n "$TAG_SHA"', 'test -n "$TAG_SHA" || exit 0'
            ),
        ),
        "step text drifted from its pinned hash",
    ),
    "removing_the_floor_check": (
        _mutate_run(
            "pypi-build",
            "Build wheels inside the manylinux 2_28 container",
            lambda run: run.replace(
                "check-wheel-glibc-floor.py target/wheels/*.whl 2.28", ":"
            ),
        ),
        "must run the floor check",
    ),
    "arm_wheel_build_dropped": (
        _drop_matrix_entry("pypi-build", "aarch64-unknown-linux-gnu"),
        "matrix entries must be pinned exactly",
    ),
    "arm_smoke_on_an_x86_runner": (
        _retarget_matrix_os(
            "wheel-smoke", "artifact", "wheel-aarch64-unknown-linux-gnu", "ubuntu-24.04"
        ),
        "matrix entries must be pinned exactly",
    ),
    "if_false_on_the_runtime_floor_smoke": (
        _set_step_flag(
            "wheel-smoke",
            "Run the wheel inside the manylinux 2_28 container",
            "if",
            False,
        ),
        "is not in the allowlist",
    ),
    "if_template_false_on_the_runtime_floor_smoke": (
        _set_step_flag(
            "wheel-smoke",
            "Run the wheel inside the manylinux 2_28 container",
            "if",
            "${{ false }}",
        ),
        "is not in the allowlist",
    ),
    "if_template_false_and_true_on_docs_live_revision": (
        _set_step_flag(
            "docs-live",
            "Require the live documentation revision",
            "if",
            "${{ false && true }}",
        ),
        "is not in the allowlist",
    ),
    "custom_shell_template_on_the_venv_smoke": (
        _set_step_flag("wheel-smoke", "Smoke the installed wheel", "shell", "bash {0}"),
        "custom shell",
    ),
    "docs_live_timeout_exit_zero": (
        _mutate_run(
            "docs-live",
            "Require the live documentation revision",
            lambda run: run.replace("exit 1", "exit 0"),
        ),
        "revision timeout branch must end in exit 1",
    ),
    "not_cancelled_on_pypi_publish": (
        _set_job_flag(
            "pypi-publish", "if", "github.event_name == 'push' && !cancelled()"
        ),
        "job if must be exactly",
    ),
    "deleting_the_return_1_after_the_panic_message": (
        _mutate_run(
            "wheel-smoke",
            "Smoke the installed wheel",
            lambda run: re.sub(
                r'(echo "panic or crash \(exit \$status\) from: biomcp \$\*" >&2)\s*\n\s*return 1',
                r"\1\n              :",
                run,
            ),
        ),
        "panic guard must return failure",
    ),
    "deleting_the_not_found_exit_check": (
        _mutate_run(
            "wheel-smoke",
            "Smoke the installed wheel",
            lambda run: run.replace('grep -q "Drug not found in FAERS"', ":"),
        ),
        "not-found exit check must follow",
    ),
    "a_new_first_step_slipped_into_a_pinned_job": (
        lambda parsed: parsed["jobs"]["pypi-build"]["steps"].insert(
            0,
            {
                "name": "Prepare environment",
                "run": "echo \"trap 'exit 0' EXIT\" > /tmp/poison.sh && "
                "echo \"BASH_ENV=/tmp/poison.sh\" >> \"$GITHUB_ENV\"",
            },
        ),
        "the pinned job's step list must match exactly",
    ),
    "bash_env_at_job_level": (
        lambda parsed: parsed["jobs"]["wheel-smoke"].update(
            {"env": {"BASH_ENV": "/tmp/poison.sh"}}
        ),
        "BASH_ENV is forbidden",
    ),
    "bash_env_at_workflow_level": (
        lambda parsed: parsed.update({"env": {"BASH_ENV": "/tmp/poison.sh"}}),
        "BASH_ENV is forbidden",
    ),
    "bash_env_at_step_level": (
        lambda parsed: _pipeline_steps(parsed, "docs-live")[0].update(
            {"env": {"BASH_ENV": "/tmp/poison.sh"}}
        ),
        "BASH_ENV is forbidden",
    ),
    "bash_env_named_in_a_with_block": (
        lambda parsed: _step_by_name(
            parsed, "version-check", "actions/checkout"
        ).update({"with": {"something": "BASH_ENV=/tmp/poison.sh"}}),
        "names BASH_ENV",
    ),
    "restoring_a_release_published_trigger": (
        lambda parsed: parsed[True].update({"release": {"types": ["published"]}}),
        "trigger block must be tag pushes only",
    ),
    "no_space_template_true_on_a_smoke_step_if": (
        lambda parsed: _step_by_name(
            parsed, "wheel-smoke", "Smoke the installed wheel"
        ).update({"if": "${{true}}"}),
        "is not in the allowlist",
    ),
    "double_quoted_trap_exit_zero": (
        lambda parsed: _step_by_name(
            parsed, "wheel-smoke", "Smoke the installed wheel"
        ).update(
            {
                "run": 'trap "exit 0" EXIT\n'
                + _step_by_name(parsed, "wheel-smoke", "Smoke the installed wheel")[
                    "run"
                ]
            }
        ),
        "step text drifted from its pinned hash",
    ),
    # --- Round three (ticket 1269): expression envs, hijack
    # --- variable names, workflow-level PATH/ENV, and a new first
    # --- step in pypi-publish itself.
    "expression_env_carrying_bash_env": (
        lambda parsed: parsed["jobs"]["wheel-smoke"].update(
            {"env": 'FAKE_EXPRESSION{ "BASH_ENV": "/tmp/poison.sh" }'}
        ),
        "expression env names BASH_ENV",
    ),
    "bash_func_import_at_job_level": (
        lambda parsed: parsed["jobs"]["wheel-smoke"].update(
            {"env": {"BASH_FUNC_python%%": "() { exit 0; }"}}
        ),
        "is a bash function-import variable",
    ),
    "bash_func_import_at_workflow_level": (
        lambda parsed: parsed.update(
            {"env": {"BASH_FUNC_python%%": "() { exit 0; }"}}
        ),
        "is a bash function-import variable",
    ),
    "path_at_workflow_level": (
        lambda parsed: parsed.update(
            {"env": {**parsed.get("env", {}), "PATH": "/hijack/bin:/usr/bin"}}
        ),
        "rewrites every job's",
    ),
    "env_options_at_workflow_level": (
        lambda parsed: parsed.update(
            {"env": {**parsed.get("env", {}), "ENV": "/hijack/options"}}
        ),
        "rewrites every job's",
    ),
    "a_new_first_step_slipped_into_pypi_publish": (
        lambda parsed: parsed["jobs"]["pypi-publish"]["steps"].insert(
            0,
            {
                "name": "Prepare environment",
                "run": 'echo "BASH_ENV=/tmp/poison.sh" >> "$GITHUB_ENV"',
            },
        ),
        "the pinned job's step list must match exactly",
    ),
}


@pytest.mark.parametrize("name", sorted(PIPELINE_MUTATIONS))
def test_pipeline_mutations_break_the_contract(name: str) -> None:
    mutate, expected_message = PIPELINE_MUTATIONS[name]
    mutated = _mutated_pipeline(mutate)
    with pytest.raises(AssertionError, match=re.escape(expected_message)):
        _assert_pipeline_contract(mutated)
        _assert_publish_gating(mutated)
        _assert_no_defaults_shell(mutated)
        _assert_step_hashes(mutated)


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
        (
            'run: python3 scripts/check-release-versions.py --tag "$TAG"',
            'run: python3 scripts/check-release-versions.py --tag "$TAG" || :',
        ),
        (
            'run: python3 scripts/check-changelog-coverage.py --tag "$TAG"',
            'run: python3 scripts/check-changelog-coverage.py --tag "$TAG"; exit 0',
        ),
        (
            "      - name: Require the changelog to cover merged tickets\n",
            "      - name: Require the changelog to cover merged tickets\n        if: ${{ false }}\n",
        ),
        ("    steps:\n", "    continue-on-error: true\n    steps:\n"),
        (
            "      - name: Require the tag and committed versions to agree\n",
            "      - name: Require the tag and committed versions to agree\n        if: false\n",
        ),
        (
            "    needs: [pypi-build, wheel-smoke, docs-live, build, container-publish]\n",
            "    needs: [pypi-build, wheel-smoke, docs-live, build, container-publish]\n"
            "    needs: [docs-live]\n",
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

def test_rust_embed_carries_debug_embed_github_287() -> None:
    """GitHub #287: the 0.9.0 wheels shipped a debug-profile binary
    whose rust_embed read assets from the build machine at runtime.
    The feature makes debug builds embed like release builds; this
    pin fails if it leaves Cargo.toml silently.
    """
    manifest = (REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r"rust-embed\s*=\s*\{[^}]*\}", manifest)
    assert match is not None, "rust-embed entry not found as a table"
    assert 'features = ["debug-embed"]' in match.group(0), (
        f"rust-embed must keep debug-embed (GitHub #287): {match.group(0)}"
    )


def test_every_job_calling_gh_names_its_repository() -> None:
    """2026-09-30 fourth go-request review: create-draft called gh
    with no checkout and no GH_REPO, so gh could not discover the
    repository and the release stopped at its second job. Any job
    whose steps call gh must set GH_REPO or check out the repository.
    """
    parsed = yaml.safe_load(
        (REPO_ROOT / ".github" / "workflows" / "release.yml").read_text(encoding="utf-8")
    )
    for job_id, job in parsed["jobs"].items():
        calls_gh = any(
            # 2026-10-01 fifth go-request review: gh also appears
            # mid-line (if ! gh ..., $(gh ...), | gh, && gh), so
            # match gh as its own word anywhere in the run text,
            # not only at the start of a line.
            re.search(r"(^|[^a-zA-Z0-9_.-])gh\s+[a-z]", str(step.get("run", "")), re.M)
            for step in job.get("steps", [])
        )
        if not calls_gh:
            continue
        has_repo = job.get("env", {}).get("GH_REPO") == "${{ github.repository }}" or any(
            step.get("env", {}).get("GH_REPO")
            for step in job.get("steps", [])
            if isinstance(step.get("env"), dict)
        )
        has_checkout = any(
            str(step.get("uses", "")).startswith("actions/checkout@")
            for step in job.get("steps", [])
        )
        assert has_repo or has_checkout, (
            f"{job_id} calls gh with neither GH_REPO nor a checkout; gh "
            "cannot know the repository without a git directory"
        )


def test_every_job_running_a_repository_script_checks_out_first() -> None:
    """2026-09-30 third go-request review: publish-release ran
    scripts/should-move-latest.sh with no checkout step, so the script
    was missing on the runner and the step passed while doing nothing.
    Any job that runs a scripts/ or tools/ path must check out the
    repository first.
    """
    parsed = yaml.safe_load(
        (REPO_ROOT / ".github" / "workflows" / "release.yml").read_text(encoding="utf-8")
    )
    for job_id, job in parsed["jobs"].items():
        steps = job.get("steps", [])
        has_checkout = any(
            str(step.get("uses", "")).startswith("actions/checkout@")
            for step in steps
        )
        for index, step in enumerate(steps):
            run = str(step.get("run", ""))
            if "scripts/" in run or " tools/" in run or run.startswith("tools/"):
                assert has_checkout, (
                    f"{job_id} step {index} runs repository code ({run.splitlines()[0][:60]}) "
                    "without an actions/checkout step"
                )
                checkout_at = next(
                    i
                    for i, s in enumerate(steps)
                    if str(s.get("uses", "")).startswith("actions/checkout@")
                )
                assert checkout_at < index, (
                    f"{job_id}: the checkout must precede the step running repository code"
                )


def test_the_latest_step_fails_when_gh_fails() -> None:
    """The guard at release.yml reads `if ! gh release list ... exit 1`;
    restoring a silent `|| true` would read a failed query as an empty
    list and move latest (fourth go-request review).
    """
    text = (REPO_ROOT / ".github" / "workflows" / "release.yml").read_text(encoding="utf-8")
    assert "if ! gh release list" in text, "the failure guard must exist"
    guarded = text[text.index("if ! gh release list") : text.index("mapfile -t PUBLISHED")]
    assert "|| true" not in guarded, (
        "a silent fallback inside the guarded block defeats the guard"
    )


def test_jobs_downloading_the_draft_release_can_write_contents() -> None:
    """2026-10-01 fifth go-request review: container-publish ran
    `gh release download` on the still-draft release with
    contents: read. GitHub lists drafts only to callers with push
    access, so the token saw "release not found" and the release
    stopped before PyPI. Every job that downloads the release
    before publish-release makes it public needs contents: write.
    """
    parsed = yaml.safe_load(
        (REPO_ROOT / ".github" / "workflows" / "release.yml").read_text(encoding="utf-8")
    )
    checked = []
    for job_id, job in parsed["jobs"].items():
        downloads = any(
            "gh release download" in str(step.get("run", ""))
            for step in job.get("steps", [])
        )
        if not downloads:
            continue
        waits_for_public = "publish-release" in (job.get("needs") or [])
        if waits_for_public:
            # The release is public by then; read access is enough.
            continue
        checked.append(job_id)
        assert job.get("permissions", {}).get("contents") == "write", (
            f"{job_id} downloads the draft release but lacks contents: write; "
            "GitHub hides drafts from read-only tokens"
        )
    assert "container-publish" in checked, (
        "container-publish must stay covered by the draft-visibility check"
    )


def test_the_decision_case_accepts_exactly_zero_three_and_catchall() -> None:
    """2026-10-01 fifth go-request review: widening the stay arm to
    `3|*)` would make every script error read as a legitimate stay
    and no test noticed. The case must keep three separate arms.
    """
    text = (REPO_ROOT / ".github" / "workflows" / "release.yml").read_text(encoding="utf-8")
    block = text[text.index('case "$decision" in') : text.index("esac", text.index('case "$decision" in'))]
    arms = re.findall(r"(?m)^\s*([0-9]|\*|[0-9]\|\*)\)", block)
    assert arms == ["0", "3", "*"], (
        f"the decision case must have exactly 0), 3) and *), found {arms}"
    )
    assert '3|*)' not in block, "a widened stay arm would swallow script errors"
