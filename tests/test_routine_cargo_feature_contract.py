from __future__ import annotations

from pathlib import Path
import json
import subprocess
import re
import pytest

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module


REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _target(makefile: str, name: str) -> str:
    match = re.search(rf"(?ms)^{re.escape(name)}:.*?(?=^[A-Za-z][^\n]*:|\Z)", makefile)
    assert match is not None, f"missing Make target {name}"
    return match.group(0)


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
    # The lane split cut `make test` in two so the CI lanes can
    # run the Rust half alone; the archive run is the rust lane's recipe.
    assert "nextest run --archive-file" in _target(makefile, "test-rust")
    assert "--cargo-feature-arg" in runner
    assert '"${routine_cargo_features[@]}"' in runner
    assert "args.cargo_feature_arg" in preparer
    assert '"--no-default-features"' not in preparer
    assert 'cargo clippy --all-targets "${routine_cargo_features[@]}" -- -D warnings' in lint


def test_release_gate_runs_a_named_all_feature_check() -> None:
    makefile = _read("Makefile")
    full = _target(makefile, "full-feature-check")
    release = _target(makefile, "release-gate")

    assert "$(CARGO_WITH_IDENTITY) clippy --locked --all-targets --all-features" in full
    assert "$(CARGO_WITH_IDENTITY) test --locked --all-features --lib" in full
    assert "sources::alphagenome::tests" in full
    assert (
        "$(CARGO_WITH_IDENTITY) build --release --locked --all-features --bin biomcp"
        in full
    )
    assert "$(MAKE) full-feature-check" in release
    assert "ROUTINE_CARGO_FEATURES=" not in release
    assert (
        '$(MAKE) spec SPEC_PROFILE=release SPEC_BIN="$(CURDIR)/target/release/biomcp"'
        in (release)
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
        ("main-gate", ["lint", "test", "spec", "full-feature-check"]),
    ],
)
@pytest.mark.parametrize("failed_stage", [None, "test"])
def test_landing_commands_run_ordered_checks_and_stop_on_failure(
    tmp_path: Path, target: str, expected: list[str], failed_stage: str | None
) -> None:
    # Override existing leaves, not the gate recipes: exercise Make's real
    # ordering and failure propagation without compiling the product again.
    wrapper = tmp_path / "gate.mk"
    wrapper.write_text(
        f"include {REPO_ROOT / 'Makefile'}\n"
        + "\n".join(
            f"{stage}:\n\t@echo gate-stage:{stage}\n"
            + ("\t@exit 23\n" if stage == failed_stage else "")
            for stage in ("lint", "test", "spec", "full-feature-check")
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
