from pathlib import Path
import re

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/release.yml"


def _text() -> str:
    return WORKFLOW.read_text(encoding="utf-8")


def test_stage_and_protected_promotion_are_the_only_callable_modes() -> None:
    workflow = yaml.safe_load(_text())
    dispatch = workflow[True]["workflow_dispatch"]["inputs"]
    assert set(dispatch) == {
        "mode",
        "source_sha",
        "stage_run_id",
        "windows_desktop_smoke",
        "updater_transition",
    }
    assert dispatch["mode"]["options"] == ["stage", "promote"]
    assert workflow["permissions"] == {"contents": "read"}
    jobs = workflow["jobs"]
    assert jobs["candidate-gates"]["if"] == "inputs.mode == 'stage'"
    assert jobs["promotion-preflight"]["environment"] == "biomcp-release-promotion"
    assert jobs["advance-mutable-pointers"]["needs"] == "reconcile-public-release"


def test_stage_binds_checkout_build_and_manifest_to_input_sha() -> None:
    text = _text()
    assert text.count("ref: ${{ inputs.source_sha }}") >= 3
    assert 'git merge-base --is-ancestor "$SOURCE_SHA" origin/main' in text
    assert "release/candidate.py init" in text
    assert "release/candidate.py register" in text
    assert "release/candidate.py finalize" in text
    assert "github.sha" not in text


def test_stage_runs_and_records_the_all_feature_candidate_gate() -> None:
    text = _text()
    assert "make full-feature-check" in text
    assert "for gate in lint test full-feature-check spec" in text


def test_baseline_build_is_once_in_pinned_manylinux_and_never_publishes() -> None:
    text = _text()
    assert "manylinux_2_28_x86_64@sha256:" in text
    assert "manylinux_2_28_aarch64@sha256:" in text
    assert "release/build_target.py" in text
    workflow = yaml.safe_load(text)
    stage_jobs = [
        value
        for value in workflow["jobs"].values()
        if value.get("if") == "inputs.mode == 'stage'"
    ]
    stage_text = yaml.safe_dump(stage_jobs)
    for forbidden in ("packages: write", "contents: write", "git push", "uv publish"):
        assert forbidden not in stage_text


def _action_references(value: object) -> list[str]:
    if isinstance(value, dict):
        references = [str(value["uses"])] if "uses" in value else []
        return references + [
            reference
            for child in value.values()
            for reference in _action_references(child)
        ]
    if isinstance(value, list):
        return [reference for child in value for reference in _action_references(child)]
    return []


def _is_immutable_action_reference(reference: str) -> bool:
    return re.fullmatch(r"[^@\s]+@[0-9a-f]{40}", reference) is not None


def test_action_pin_shape_rejects_symbolic_and_shortened_revisions() -> None:
    assert not _is_immutable_action_reference("actions/checkout@v4")
    assert not _is_immutable_action_reference(f"astral-sh/setup-uv@{'a' * 39}")
    assert _is_immutable_action_reference(f"actions/checkout@{'a' * 40}")


def test_every_workflow_action_and_release_container_is_immutably_pinned() -> None:
    workflow_paths = sorted(
        path
        for path in (ROOT / ".github/workflows").iterdir()
        if path.suffix in {".yml", ".yaml"}
    )
    for path in workflow_paths:
        for action in _action_references(
            yaml.safe_load(path.read_text(encoding="utf-8"))
        ):
            assert _is_immutable_action_reference(action), (path.name, action)
    for line in _text().splitlines():
        stripped = line.strip()
        if "container:" in stripped or stripped.startswith("MANYLINUX_"):
            assert "${{ matrix.container }}" in stripped or "@sha256:" in stripped


def test_candidate_gate_installs_the_complete_pinned_canonical_toolset() -> None:
    workflow = yaml.safe_load(_text())
    candidate = workflow["jobs"]["candidate-gates"]
    install = next(
        step["run"]
        for step in candidate["steps"]
        if step.get("name") == "Install canonical gate tools"
    )
    for command in (
        '"bubblewrap=$BUBBLEWRAP_VERSION"',
        'cargo install cargo-nextest --version "$CARGO_NEXTEST_VERSION" --locked',
        'cargo install cargo-deny --version "$CARGO_DENY_VERSION" --locked',
        'uv tool install "ruff==$RUFF_VERSION"',
        'uv tool install "mustmatch==$MUSTMATCH_VERSION"',
    ):
        assert command in install


def test_candidate_gate_loads_scoped_apparmor_before_compilation() -> None:
    text = _text()
    workflow = yaml.safe_load(text)
    candidate = workflow["jobs"]["candidate-gates"]
    install = next(
        step["run"]
        for step in candidate["steps"]
        if step.get("name") == "Install canonical gate tools"
    )
    assert "APPARMOR_VERSION: 4.0.1really4.0.1-0ubuntu0.24.04.7" in text
    expected = (
        '"apparmor=$APPARMOR_VERSION"',
        '"apparmor-profiles=$APPARMOR_VERSION"',
        "/usr/share/apparmor/extra-profiles/bwrap-userns-restrict",
        "/etc/apparmor.d/bwrap-userns-restrict",
        "apparmor_parser -r /etc/apparmor.d/bwrap-userns-restrict",
        "sysctl -n kernel.apparmor_restrict_unprivileged_userns",
        "tools/run-offline -- true",
    )
    for contract in expected:
        assert contract in install

    assert install.index("tools/run-offline -- true") < install.index(
        "cargo install cargo-nextest"
    )
    gate_index = next(
        index
        for index, step in enumerate(candidate["steps"])
        if step.get("name") == "Canonical candidate gates"
    )
    install_index = next(
        index
        for index, step in enumerate(candidate["steps"])
        if step.get("name") == "Install canonical gate tools"
    )
    assert install_index < gate_index


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
    assert (
        "BIOMCP_SIGNING_POLICY_SHA256: ${{ secrets.BIOMCP_SIGNING_POLICY_SHA256 }}"
        in text
    )


def test_all_five_private_wheels_install_and_smoke_both_commands_before_sealing() -> (
    None
):
    workflow = yaml.safe_load(_text())
    jobs = workflow["jobs"]
    expected_counts = {"linux-artifacts": 2, "signed-artifacts": 3}
    for job_name, target_count in expected_counts.items():
        job = jobs[job_name]
        assert len(job["strategy"]["matrix"]["include"]) == target_count
        names = [step.get("name") for step in job["steps"]]
        smoke_index = names.index("Install and smoke both private wheel commands")
        upload_index = next(
            index
            for index, step in enumerate(job["steps"])
            if str(step.get("uses", "")).startswith("actions/upload-artifact@")
        )
        assert smoke_index < upload_index
        smoke = job["steps"][smoke_index]["run"]
        assert "pip install --no-index" in smoke
        assert "PYTHON_VERSION=" in smoke
        assert '"biomcp_cli-$PYTHON_VERSION-"*' in smoke
        assert 'release/smoke.py --bin "$wheel_bin/biomcp"' in smoke
        assert 'release/smoke.py --bin "$wheel_bin/biomcp-cli"' in smoke
    assert {"linux-artifacts", "signed-artifacts"} <= set(
        jobs["seal-candidate"]["needs"]
    )


def test_manual_promotion_inputs_are_validated_once_before_publication() -> None:
    workflow = yaml.safe_load(_text())
    jobs = workflow["jobs"]
    preflight = yaml.safe_dump(jobs["promotion-preflight"])
    publish = yaml.safe_dump(jobs["publish-versioned"])
    reconcile = yaml.safe_dump(jobs["reconcile-public-release"])
    assert "--windows-desktop-smoke" in preflight
    assert "--updater-transition" in preflight
    assert "--public-releases" in preflight
    assert "gh api --paginate" in preflight
    assert "inputs.windows_desktop_smoke" in preflight
    assert "inputs.updater_transition" in preflight
    assert "inputs.windows_desktop_smoke" not in publish
    assert "inputs.updater_transition" not in publish
    assert "inputs.windows_desktop_smoke" not in reconcile
    assert "inputs.updater_transition" not in reconcile
    assert "promotion-inventory.json" in publish
    assert "promotion-inventory.json" in reconcile


def test_development_candidate_guard_runs_before_public_release_lookup() -> None:
    workflow = yaml.safe_load(_text())
    preflight = next(
        step["run"]
        for step in workflow["jobs"]["promotion-preflight"]["steps"]
        if step.get("name") == "Resolve and verify every private candidate byte"
    )
    assert preflight.index("release/promotion.py require-release") < preflight.index(
        "gh api --paginate"
    )


def test_target_builds_thread_distinct_rust_and_python_versions() -> None:
    text = _text()
    assert text.count('--python-version "$PYTHON_VERSION"') == 2
    assert text.count("PYTHON_VERSION=") >= 4


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
    workflow = yaml.safe_load(text)
    stage_text = yaml.safe_dump(workflow["jobs"]["homebrew-smoke"])
    assert "homebrew-biomcp" not in stage_text
    assert "HOMEBREW_TAP_TOKEN" not in stage_text


def test_mcpb_is_derived_signed_once_and_smoked_on_three_platform_runners() -> None:
    text = _text()
    assert "mcpb-artifact:" in text
    assert "lipo -create" in text
    assert "--target macos-universal" in text
    assert "mcpb pack" in text
    assert "release/mcpb_sign.py" in text
    assert text.count("mcpb verify") >= 2
    assert "mcpb-smoke:" in text
    assert "signtool verify /pa /all /tw" in text
    assert "macos-15-intel" in text and "windows-2022" in text
