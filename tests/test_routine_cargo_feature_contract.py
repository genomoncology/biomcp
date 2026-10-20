from __future__ import annotations

from pathlib import Path
import json
import os
import subprocess
import sys
import re
import shutil
import pytest

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module


REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _target(makefile: str, name: str) -> str:
    match = re.search(rf"(?ms)^{re.escape(name)}:.*?(?=^[A-Za-z][^\n]*:|\Z)", makefile)
    assert match is not None, f"missing Make target {name}"
    return match.group(0)


def test_routine_feature_probe_runs_without_a_prepared_checkout(tmp_path: Path) -> None:
    shutil.copy2(REPO_ROOT / "Makefile", tmp_path / "Makefile")
    spec_dir = tmp_path / "spec" / "surface"
    spec_dir.mkdir(parents=True)
    section = _read("spec/surface/build-profile.md").split(
        "## Routine gates select the smaller feature set", 1
    )[1]
    block = section.split("```bash\n", 1)[1].split("```", 1)[0]
    # Execute the spec's first command, leaving its mustmatch assertions below
    # at this boundary so the fixture needs neither product builds nor tools.
    command = block.split("| mustmatch like", 1)[0]
    routine_tmp = tmp_path / ".cache" / "routine-test-tmp"
    env = {key: value for key, value in os.environ.items()
           if key not in ("MAKEFLAGS", "MAKEOVERRIDES")}
    env["TMPDIR"] = str(routine_tmp)
    result = subprocess.run(
        ["bash", "-eu", "-c", command], cwd=spec_dir, env=env,
        capture_output=True, text=True,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert "cargo nextest archive --locked --no-default-features" in result.stdout
    assert "cargo nextest run --archive-file" in result.stdout
    assert str(routine_tmp) not in result.stdout
    assert not routine_tmp.exists()


@pytest.mark.parametrize("target", ["spec-pr", "spec-contracts"])
def test_alternate_spec_targets_sync_python_before_the_runner(
    tmp_path: Path, target: str,
) -> None:
    shutil.copy2(REPO_ROOT / "Makefile", tmp_path / "Makefile")
    env = {key: value for key, value in os.environ.items()
           if key not in ("MAKEFLAGS", "MAKEOVERRIDES")}
    result = subprocess.run(
        ["make", "--dry-run", target], cwd=tmp_path, env=env,
        capture_output=True, text=True,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    sync = "uv sync --extra dev --no-install-project"
    runner = f"bash scripts/run-specs.sh {target}"
    assert sync in result.stdout, result.stdout
    assert result.stdout.index(sync) < result.stdout.index(runner), result.stdout


def test_routine_lint_test_and_spec_share_the_declared_feature_graph() -> None:
    makefile = _read("Makefile")
    lint = _read("bin/lint")
    runner = _read("scripts/run-specs.sh")
    preparer = _read("scripts/prepare-spec-artifacts.py")

    assert "ROUTINE_CARGO_FEATURES ?= --no-default-features" in makefile
    assert "export ROUTINE_CARGO_FEATURES" in makefile
    assert 'ROUTINE_CARGO_FEATURES="$(ROUTINE_CARGO_FEATURES)" ./bin/lint' in (
        _target(makefile, "lint")
    )
    assert "nextest archive --locked $(ROUTINE_CARGO_FEATURES)" in _target(
        makefile, "prepare-test"
    )
    # Both local lanes consume the prepared archive; the Rust lane has
    # an explicit worker budget so concurrent jobs share the machine.
    assert "nextest run --archive-file" in _target(makefile, "test-rust")
    assert "$(NEXTEST_JOBS_ARG)" in _target(makefile, "test-rust")
    assert "--cargo-feature-arg" in runner
    assert '"${routine_cargo_features[@]}"' in runner
    assert "args.cargo_feature_arg" in preparer
    assert '"--no-default-features"' not in preparer
    assert 'cargo clippy --all-targets "${routine_cargo_features[@]}" -- -D warnings' in lint


def test_release_specs_use_the_optimized_full_feature_binary() -> None:
    release = _target(_read("Makefile"), "release-gate")

    assert "ROUTINE_CARGO_FEATURES=" not in release
    assert (
        '$(MAKE) spec SPEC_PROFILE=release SPEC_BIN="$(CURDIR)/target/release/biomcp"'
        in release
    )


def test_ci_and_developer_docs_name_small_and_full_feature_lanes() -> None:
    workflow = _read(".github/workflows/ci.yml")
    docs = "\n".join(
        _read(path)
        for path in (
            "AGENTS.md",
            "CONTRIBUTING.md",
            "RUN.md",
            "architecture/technical/overview.md",
        )
    )

    assert "run: make lint" in workflow
    assert "run: make test-rust" in workflow
    assert "run: make test-python" in workflow
    assert "make full-feature-check" in workflow
    assert "Routine gates use `--no-default-features`" in docs
    assert "`make full-feature-check`" in docs
    assert "AlphaGenome" in docs


def test_contributor_docs_record_the_supported_rust_test_lane_decision() -> None:
    docs = _read("CONTRIBUTING.md")
    test_section = re.search(
        r"(?ims)^## [^\n]*test[^\n]*\n(?P<body>.*?)(?=^## |\Z)", docs
    )
    assert test_section is not None, "CONTRIBUTING.md needs a discoverable test section"
    body = re.sub(r"\s+", " ", test_section.group("body").replace("`", "")).lower()

    requirements = {
        "make test is the supported lane": re.search(
            r"(?:supported (?:rust )?(?:test )?lane (?:is )?make test|"
            r"make test (?:is|remains) the supported)",
            body,
        ),
        "the lane is offline no-default-features nextest": all(
            term in body for term in ("offline", "--no-default-features", "nextest")
        ),
        "direct bare cargo test is unsupported": re.search(
            r"(?:direct|bare).{0,40}cargo test.{0,80}unsupported|"
            r"unsupported.{0,80}(?:direct|bare).{0,40}cargo test",
            body,
        ),
        "the known order-dependent failure is recognizable": all(
            term in body
            for term in (
                "selected_fixture_origin_allows_only_exact_ip_loopback",
                "order-dependent",
            )
        ),
        "the dated ruling records declined reconciliation": all(
            term in body for term in ("2026-08-23", "reconciliation", "declined")
        ),
    }

    missing = [name for name, present in requirements.items() if not present]
    assert not missing, f"missing supported test-lane documentation: {missing}"


@pytest.mark.parametrize(
    ("target", "expected"),
    [
        ("landing-gate", ["lint", "test", "spec"]),
        ("main-gate", ["lint", "test", "spec", "repo-audit", "full-feature-check", "installed-wheel-smoke"]),
    ],
)
@pytest.mark.parametrize("failed_stage", [None, "test", "repo-audit", "installed-wheel-smoke"])
def test_landing_commands_run_ordered_checks_and_stop_on_failure(
    tmp_path: Path, target: str, expected: list[str], failed_stage: str | None
) -> None:
    if failed_stage and failed_stage not in expected:
        pytest.skip("this check runs after merging")
    # Override existing leaves, not the gate recipes: exercise Make's real
    # ordering and failure propagation without compiling the product again.
    wrapper = tmp_path / "gate.mk"
    wrapper.write_text(
        f"include {REPO_ROOT / 'Makefile'}\n"
        + "\n".join(
            f"{stage}:\n\t@echo gate-stage:{stage}\n"
            + ("\t@exit 23\n" if stage == failed_stage else "")
            for stage in ("lint", "test", "spec", "repo-audit", "full-feature-check", "installed-wheel-smoke")
        ),
        encoding="utf-8",
    )
    result = subprocess.run(
        [
            "make", "--no-print-directory", "-s", "-j4", "-f", str(wrapper),
            f"MAKE=make --no-print-directory -s -f {wrapper}", target,
        ],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    stages = [
        line.removeprefix("gate-stage:")
        for line in result.stdout.splitlines()
        if line.startswith("gate-stage:")
    ]
    if failed_stage:
        assert result.returncode != 0
        assert stages == expected[: expected.index(failed_stage) + 1]
    else:
        assert result.returncode == 0, result.stdout + result.stderr
        assert stages == expected
    config = json.loads(_read("sdlc/pm.json"))
    assert config["ticketsGate"] == "make landing-gate"


@pytest.mark.parametrize("failed_lane", ["", "rust", "python"])
def test_make_test_prepares_once_and_waits_for_both_lanes(
    tmp_path: Path, failed_lane: str
) -> None:
    """Exercise Make's orchestration without rebuilding product artifacts."""
    worker = tmp_path / "lane.py"
    worker.write_text(
        "import os, pathlib, sys, time\n"
        "root = pathlib.Path(sys.argv[1]); lane = sys.argv[2]\n"
        "(root / (lane + '.started')).touch()\n"
        "peer = 'python' if lane == 'rust' else 'rust'\n"
        "deadline = time.monotonic() + 3\n"
        "while not (root / (peer + '.started')).exists():\n"
        "    if time.monotonic() > deadline: raise SystemExit('lanes did not overlap')\n"
        "    time.sleep(.01)\n"
        "(root / (lane + '.finished')).touch()\n"
        "print(lane + ' output')\n"
        "raise SystemExit(7 if os.environ['FAILED_LANE'] == lane else 0)\n"
    )
    overlay = tmp_path / "lanes.mk"
    overlay.write_text(
        f'prepare-test:\n\t@echo prepare >> "{tmp_path / "prepared"}"\n'
        f"test-rust:\n\t@{sys.executable} {worker} {tmp_path} rust\n"
        f"test-python:\n\t@{sys.executable} {worker} {tmp_path} python\n"
    )
    command = [
        "make",
        "--no-print-directory",
        "-f",
        str(REPO_ROOT / "Makefile"),
        "-f",
        str(overlay),
    ]
    result = subprocess.run(
        [
            *command,
            "test",
            "ROUTINE_TEST_TMPDIR=" + str(tmp_path),
            "MAKE=" + " ".join(command),
        ],
        cwd=REPO_ROOT,
        env={**os.environ, "FAILED_LANE": failed_lane},
        capture_output=True,
        text=True,
        timeout=15,
    )
    assert (result.returncode == 0) == (not failed_lane), result.stdout + result.stderr
    assert (tmp_path / "prepared").read_text().splitlines() == ["prepare"]
    assert all(
        (tmp_path / (lane + ".finished")).exists() for lane in ("rust", "python")
    )
    assert not any(path.is_dir() for path in tmp_path.iterdir())
    assert "=== Rust lane (exit " in result.stdout
    assert "=== Python lane (exit " in result.stdout
    assert "rust output" in result.stdout and "python output" in result.stdout


@pytest.mark.parametrize(
    ("target", "profile", "directory"),
    [("main-gate", "dev", "debug"), ("release-gate", "release", "release")],
)
def test_full_feature_gates_build_and_smoke_the_selected_profile(
    tmp_path: Path, target: str, profile: str, directory: str
) -> None:
    """Run the gate recipes; replace only external commands and unrelated leaves."""
    recorder = tmp_path / "record.py"
    recorder.write_text(
        "import json, sys\n"
        "print('invocation:' + json.dumps(sys.argv[1:]))\n",
        encoding="utf-8",
    )
    tools = tmp_path / "tools"
    tools.mkdir()
    for name in ("check-png-artifact", "run-offline"):
        command = tools / name
        command.write_text(
            f"#!{sys.executable}\n"
            "import json, sys\n"
            f"print('invocation:' + json.dumps([{name!r}, *sys.argv[1:]]))\n",
            encoding="utf-8",
        )
        command.chmod(0o755)
    wrapper = tmp_path / "gate.mk"
    wrapper.write_text(
        f"include {REPO_ROOT / 'Makefile'}\n"
        "landing-gate lint test test-long prepare-test prepare-routine-test-tmp spec repo-audit sync-python-dev installed-wheel-smoke:\n\t@:\n",
        encoding="utf-8",
    )
    result = subprocess.run(
        [
            "make", "--no-print-directory", "-s", "-f", str(wrapper), target,
            f"MAKE=make --no-print-directory -s -f {wrapper}",
            f"CARGO_WITH_IDENTITY={sys.executable} {recorder} cargo",
        ],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
        timeout=15,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    calls = [
        json.loads(line.removeprefix("invocation:"))
        for line in result.stdout.splitlines()
        if line.startswith("invocation:")
    ]
    assert [call for call in calls if call[0] == "cargo"] == [
        ["cargo", "clippy", "--locked", "--all-targets", "--all-features", "--", "-D", "warnings"],
        ["cargo", "test", "--locked", "--all-features", "--lib", "sources::alphagenome::tests"],
        ["cargo", "build", "--locked", "--profile", profile, "--all-features", "--bin", "biomcp"],
    ]
    assert [call for call in calls if call[0] == "check-png-artifact"] == [
        ["check-png-artifact", f"target/{directory}/biomcp"]
    ]
