from __future__ import annotations

import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE_WORKFLOW = WORKFLOWS / "release.yml"


def _job_block(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"^  {re.escape(job_name)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing workflow job {job_name}"
    return match.group(1)


def test_container_only_dispatch_gates_the_release_jobs() -> None:
    release = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    container_only_gate = (
        "if: github.event_name == 'release' || inputs.container_only != true"
    )

    assert "container_only:" in release
    assert container_only_gate in _job_block(release, "build")
    assert container_only_gate in _job_block(release, "pypi-build")
    assert container_only_gate in _job_block(release, "wheel-smoke")


def test_no_reference_to_the_archived_release_asset_action() -> None:
    release = RELEASE_WORKFLOW.read_text(encoding="utf-8")

    assert "actions/upload-release-asset" not in release
    assert "github.event.release.upload_url" not in release


def test_release_upload_step_uses_gh_and_runs_only_on_a_release() -> None:
    build = _job_block(RELEASE_WORKFLOW.read_text(encoding="utf-8"), "build")
    upload_steps = [
        step for step in build.split("\n      - ") if "gh release upload" in step
    ]

    assert "TAG: ${{ github.event.release.tag_name || inputs.tag }}" in build
    assert len(upload_steps) == 1
    upload = upload_steps[0]
    assert "if: github.event_name == 'release'" in upload
    # The Windows matrix leg defaults to PowerShell, which does not expand $TAG.
    assert "shell: bash" in upload
    assert "GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}" in upload
    assert 'gh release upload "$TAG"' in upload
    assert '"${{ matrix.artifact }}"' in upload
    assert '"${{ matrix.artifact }}.sha256"' in upload
    assert "--clobber" in upload


def test_homebrew_tap_resolves_the_tag_once_for_every_use() -> None:
    homebrew_tap = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "homebrew-tap"
    )

    assert "needs: [build, docs-live]" in homebrew_tap
    assert "GITHUB_REF_NAME" not in homebrew_tap
    assert homebrew_tap.count("github.event.release.tag_name || inputs.tag") == 1
    assert "TAG: ${{ github.event.release.tag_name || inputs.tag }}" in homebrew_tap
    assert 'gh release download "$TAG"' in homebrew_tap
    assert 'VERSION="${TAG#v}"' in homebrew_tap
    assert 'git commit -m "Update biomcp formula for ${TAG}"' in homebrew_tap


def test_pypi_wheels_build_in_the_release_profile() -> None:
    pypi_build = _job_block(RELEASE_WORKFLOW.read_text(encoding="utf-8"), "pypi-build")
    maturin_steps = [
        step
        for step in pypi_build.split("\n      - ")
        if "uses: PyO3/maturin-action@v1" in step
    ]

    assert len(maturin_steps) == 1
    assert "args: --release --locked" in maturin_steps[0]


def test_wheel_smoke_runs_the_built_wheel_and_gates_pypi_publish() -> None:
    release = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    wheel_smoke = _job_block(release, "wheel-smoke")
    pypi_publish = _job_block(release, "pypi-publish")

    assert "needs: [pypi-build]" in wheel_smoke
    assert "runs-on: ubuntu-24.04" in wheel_smoke
    assert "uses: actions/download-artifact@v4" in wheel_smoke
    assert "name: wheel-x86_64-unknown-linux-gnu" in wheel_smoke
    assert 'BIOMCP="$RUNNER_TEMP/wheel-venv/bin/biomcp"' in wheel_smoke
    assert "uses: PyO3/maturin-action@v1" not in wheel_smoke
    assert "needs: [pypi-build, wheel-smoke, docs-live]" in pypi_publish


def test_wheel_smoke_fails_on_a_stack_overflow_and_runs_the_deep_paths() -> None:
    wheel_smoke = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "wheel-smoke"
    )

    assert 'grep -q "has overflowed its stack"' in wheel_smoke
    assert '"$status" -eq 134' in wheel_smoke
    assert '"$status" -ge 128' in wheel_smoke
    assert wheel_smoke.count("return 1") == 3
    for command in (
        "run_smoke search trial --condition diabetes --limit 1",
        'run_smoke search trial --criteria "anti-PD-1 therapy" --limit 3',
        "run_smoke drug interactions apixaban",
        "run_smoke drug trials imatinib",
    ):
        assert command in wheel_smoke


def test_container_publish_platforms_and_latest_guard() -> None:
    container_publish = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "container-publish"
    )

    assert "platforms: linux/amd64,linux/arm64" in container_publish
    assert "gh release view" in container_publish
    assert '"$LATEST_RELEASE" != "$TAG"' in container_publish
    assert "docker buildx imagetools create --tag" in container_publish


def test_container_publish_checks_out_the_packaging_ref() -> None:
    container_publish = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "container-publish"
    )
    checkout_steps = [
        step
        for step in container_publish.split("\n      - ")
        if "uses: actions/checkout@v4" in step
    ]

    assert len(checkout_steps) == 1
    assert "ref:" not in checkout_steps[0]


def test_container_publish_resolves_the_revision_from_the_tag() -> None:
    container_publish = _job_block(
        RELEASE_WORKFLOW.read_text(encoding="utf-8"), "container-publish"
    )

    assert "SOURCE_SHA=" in container_publish
    assert "repos/${GITHUB_REPOSITORY}/commits/${TAG}" in container_publish
    assert "git rev-parse HEAD" not in container_publish


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
        if path == RELEASE_WORKFLOW:
            continue
        text = path.read_text(encoding="utf-8")
        assert not any(route in text for route in routes), path.name


def test_docs_live_reads_the_pointer_on_both_triggers() -> None:
    docs_live = _job_block(RELEASE_WORKFLOW.read_text(encoding="utf-8"), "docs-live")

    container_only_gate = (
        "if: github.event_name == 'release' || inputs.container_only != true"
    )
    assert container_only_gate not in docs_live
    assert "permissions:\n      contents: read" in docs_live
    assert "TAG: ${{ github.event.release.tag_name || inputs.tag }}" in docs_live
    assert (
        "LIVE_REVISION_URL: https://biomcp.org/__biomcp_revision__/latest.txt"
        in docs_live
    )
    assert 'gh api "repos/${GITHUB_REPOSITORY}/commits/${TAG}" --jq .sha' in docs_live
    assert "Cache-Control: no-cache" in docs_live
    assert "Pragma: no-cache" in docs_live
    assert "check-docs-live-revision.py" in docs_live
    assert "--tag-sha \"$TAG_SHA\"" in docs_live
    assert '--live-revision "$live_revision"' in docs_live
    assert 'RETRY_WINDOW_SECONDS: "600"' in docs_live
    assert 'RETRY_INTERVAL_SECONDS: "30"' in docs_live


def test_docs_live_gates_every_publisher() -> None:
    release = RELEASE_WORKFLOW.read_text(encoding="utf-8")
    container_publish = _job_block(release, "container-publish")

    assert "needs: [pypi-build, wheel-smoke, docs-live]" in _job_block(
        release, "pypi-publish"
    )
    assert "needs: [build, docs-live]" in _job_block(release, "homebrew-tap")
    assert "needs: [build, docs-live]" in container_publish
    assert "needs.docs-live.result == 'success'" in container_publish
