from __future__ import annotations

from importlib.machinery import SourceFileLoader
from importlib.util import module_from_spec, spec_from_loader
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

import pytest


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "tools"
MANIFEST = TOOLS / "biodata-1.0-focused.toml"
RUNNER = TOOLS / "check-biodata-1.0"
RETIRED_WORKFLOW = ROOT / ".github/workflows/biodata-1.0.yml"
sys.path.insert(0, str(TOOLS))

from biodata_focused_selection import (  # noqa: E402
    FocusedSelection,
    SelectionError,
    load_selection,
    nextest_filter,
    nextest_names,
    validate_python_collection,
    validate_rust_discovery,
    validate_rust_execution,
)


def _load_runner() -> object:
    loader = SourceFileLoader("biodata_focused_runner", str(RUNNER))
    spec = spec_from_loader(loader.name, loader)
    if spec is None:
        raise RuntimeError("cannot load focused runner")
    module = module_from_spec(spec)
    sys.modules[loader.name] = module
    loader.exec_module(module)
    return module


RUNNER_MODULE = _load_runner()


def _current_process_has_verified_offline_isolation() -> bool:
    probe = subprocess.run(
        [str(TOOLS / "check-offline-network"), "true"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return probe.returncode == 0


def _manifest_with(replacements: tuple[tuple[str, str], ...]) -> str:
    text = MANIFEST.read_text(encoding="utf-8")
    for old, new in replacements:
        assert old in text
        text = text.replace(old, new, 1)
    return text


def _load_mutation(tmp_path: Path, text: str) -> FocusedSelection:
    path = tmp_path / "focused.toml"
    path.write_text(text, encoding="utf-8")
    return load_selection(path)


def test_manifest_is_the_only_focused_selection_source() -> None:
    selection = load_selection(MANIFEST)
    assert len(selection.rust) == 572
    assert len(selection.python) == 15
    runner = RUNNER.read_text(encoding="utf-8")
    assert runner.count("biodata-1.0-focused.toml") == 1
    assert not RETIRED_WORKFLOW.exists()
    assert RUNNER.stat().st_mode & 0o111


@pytest.mark.parametrize(
    "replacement",
    (
        (
                '  "cli::system::batch::tests::trial_batch_json_keeps_shared_projection_metadata",',
                '  "cli::trial::dispatch::site_directory_tests::location_page_filters_sites_and_site_contacts_but_keeps_central_first",',
        ),
        (
                '  "cli::system::batch::tests::trial_batch_json_keeps_shared_projection_metadata",',
            '  " ",',
        ),
        (
            '  "tests/test_biodata_boundary.py",',
            '  "tests/test_live_provider.py",',
        ),
        (
            '  "tests/test_biodata_boundary.py",',
            '  "tests/test_credentials.py",',
        ),
        (
                '  "cli::system::batch::tests::trial_batch_json_keeps_shared_projection_metadata",',
            '  "sources::tests::reads_credential_from_environment",',
        ),
        (
            '  "tests/test_biodata_boundary.py",',
            '  "tests/",',
        ),
        (
            '  "tests/test_biodata_boundary.py",',
            '  "tests/test_publication.py",',
        ),
        (
            '  "tests/test_biodata_boundary.py",',
            '  "tests/test_deployment.py",',
        ),
        (
            '  "tests/test_biodata_boundary.py",',
            '  "tests/test_release.py",',
        ),
    ),
)
def test_duplicate_empty_and_forbidden_manifest_mutations_fail(
    tmp_path: Path, replacement: tuple[str, str]
) -> None:
    with pytest.raises(SelectionError):
        _load_mutation(tmp_path, _manifest_with((replacement,)))


def test_renamed_or_zero_match_rust_selector_fails() -> None:
    selection = load_selection(MANIFEST)
    validate_rust_discovery(selection, selection.rust)
    renamed = FocusedSelection((selection.rust[0] + "_renamed", *selection.rust[1:]), selection.python)
    with pytest.raises(SelectionError, match="matched 0"):
        validate_rust_discovery(renamed, selection.rust)
    with pytest.raises(SelectionError, match="matched 0"):
        validate_rust_discovery(selection, selection.rust[1:])


def test_python_collection_validates_in_the_execution_process() -> None:
    selection = FocusedSelection(
        rust=("module::test_one",),
        python=("tests/test_one.py", "tests/test_two.py::test_exact"),
    )
    nodeids = ("tests/test_one.py::test_a", "tests/test_two.py::test_exact")
    validate_python_collection(selection, nodeids)
    with pytest.raises(SelectionError, match="matched no tests"):
        validate_python_collection(selection, nodeids[:1])
    with pytest.raises(SelectionError, match="more than once"):
        validate_python_collection(selection, nodeids + (nodeids[1],))


def test_runner_uses_one_discovery_one_nextest_run_and_one_pytest(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    selection = load_selection(MANIFEST)
    discovery = json.dumps(
        {"rust-suites": {"library": {"testcases": {name: {} for name in selection.rust}}}}
    )
    calls: list[tuple[list[str], dict[str, str]]] = []

    def fake_run(
        arguments: list[str], environment: dict[str, str]
    ) -> subprocess.CompletedProcess[str]:
        calls.append((arguments, environment))
        stdout = discovery if arguments[:3] == ["cargo", "nextest", "list"] else ""
        if arguments[:3] == ["cargo", "nextest", "run"]:
            stdout = "\n".join(json.dumps({"type": "test", "event": "ok", "name": "binary$"+name}) for name in selection.rust)
        return subprocess.CompletedProcess(arguments, 0, stdout=stdout, stderr="")

    with tempfile.TemporaryDirectory(dir=ROOT) as directory:
        binary = Path(directory) / "biomcp"
        binary.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        binary.chmod(0o755)
        monkeypatch.setenv("BIOMCP_BIN", str(binary))
        monkeypatch.setattr(RUNNER_MODULE, "_run", fake_run)
        RUNNER_MODULE.execute({"NCI_API_KEY": "must-be-removed"})

    commands = [arguments for arguments, _ in calls]
    assert sum(command[:3] == ["cargo", "nextest", "list"] for command in commands) == 1
    assert sum(command[:3] == ["cargo", "nextest", "run"] for command in commands) == 1
    assert sum("pytest" in command for command in commands) == 1
    rust_run = next(command for command in commands if command[:3] == ["cargo", "nextest", "run"])
    for command in commands:
        if command[:2] == ["cargo", "nextest"]:
            assert "--lib" in command
            assert command[command.index("--test") + 1] == "json_error_contract"
    assert rust_run.count("--filterset") == 1
    assert nextest_filter(selection) in rust_run
    pytest_run = next(command for command in commands if "pytest" in command)
    assert pytest_run[-len(selection.python) :] == list(selection.python)
    assert all("NCI_API_KEY" not in environment for _, environment in calls)


def test_runner_requires_the_prebuilt_worktree_local_binary(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    external = tmp_path / "biomcp"
    external.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    external.chmod(0o755)
    monkeypatch.setenv("BIOMCP_BIN", str(external))
    with pytest.raises(SelectionError, match="worktree-local"):
        RUNNER_MODULE._worktree_binary()


def test_forged_offline_marker_fails_in_the_normal_namespace() -> None:
    if _current_process_has_verified_offline_isolation():
        pytest.skip("host-only forged-marker check cannot run inside offline isolation")
    with tempfile.TemporaryDirectory(dir=ROOT) as directory:
        binary = Path(directory) / "biomcp"
        binary.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        binary.chmod(0o755)
        environment = os.environ.copy()
        environment.update(BIOMCP_BIN=str(binary), BIOMCP_OFFLINE_NETWORK="1")
        result = subprocess.run(
            [str(RUNNER), "--already-isolated"],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
        )
    assert result.returncode != 0
    assert "offline privilege isolation failed" in result.stdout + result.stderr


def test_runner_contains_no_broad_or_external_command() -> None:
    runner = RUNNER.read_text(encoding="utf-8").casefold()
    for command in (
        "make lint",
        "make test",
        "make spec",
        "make full-feature-check",
        "make release-gate",
        "curl ",
        "wget ",
        "cargo publish",
        "uv publish",
        "git push",
        "deploy",
    ):
        assert command not in runner


def test_mixed_manifest_top_level_names_and_discovery_negatives(tmp_path: Path) -> None:
    selection = _load_mutation(tmp_path, 'version = 1\n[selection]\nrust = ["module::bounded", "gene_identity_error_table"]\npython = ["tests/test_biodata_boundary.py"]\n')
    validate_rust_discovery(selection, selection.rust)
    for names in [("module::bounded",), (*selection.rust, "gene_identity_error_table")]:
        with pytest.raises(SelectionError, match="match exactly once"):
            validate_rust_discovery(selection, names)
    for forbidden in ["arbitrary_top_level", "sources::mygene::tests::live::live_get", "live_cli_smoke_get_gene_braf_returns_gene_information", "provider_smoke", "credential"]:
        with pytest.raises(SelectionError):
            _load_mutation(tmp_path, 'version = 1\n[selection]\nrust = ["'+forbidden+'"]\npython = ["tests/test_biodata_boundary.py"]\n')


def test_execution_requires_exact_successful_terminal_selections() -> None:
    selection = FocusedSelection(("module::bounded", "gene_identity_error_table"), ("tests/test_biodata_boundary.py",))
    passed = [{"type": "test", "event": "ok", "name": "binary$"+name} for name in selection.rust]
    encode = lambda events: "\n".join(json.dumps(event) for event in events)
    validate_rust_execution(selection, encode(passed))
    for events in [passed[:1], passed+passed[:1], passed+[{"type":"test","event":"ok","name":"other"}], [passed[0], {"type":"test","event":"ignored","name":selection.rust[1]}]]:
        with pytest.raises(SelectionError, match="each selected test exactly once"):
            validate_rust_execution(selection, encode(events))


def test_ignored_discovery_cannot_satisfy_a_selected_name() -> None:
    selection = FocusedSelection(("module::bounded", "gene_identity_error_table"), ("tests/test_biodata_boundary.py",))
    discovery = json.dumps({"rust-suites": {"library": {"testcases": {
        "module::bounded": {"ignored": False}, "gene_identity_error_table": {"ignored": True}
    }}}})
    with pytest.raises(SelectionError, match="matched 0"):
        validate_rust_discovery(selection, nextest_names(discovery))
