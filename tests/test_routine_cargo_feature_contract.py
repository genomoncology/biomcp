from __future__ import annotations

from pathlib import Path
import re


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
    assert "nextest run $(ROUTINE_CARGO_FEATURES)" in _target(makefile, "test")
    assert "--cargo-feature-arg" in runner
    assert '"${routine_cargo_features[@]}"' in runner
    assert "args.cargo_feature_arg" in preparer
    assert '"--no-default-features"' not in preparer
    assert 'cargo clippy "${routine_cargo_features[@]}" -- -D warnings' in lint


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

    assert "cargo clippy --no-default-features -- -D warnings" in workflow
    assert "cargo test --no-default-features" in workflow
    assert "make full-feature-check" in workflow
    assert "Routine gates use `--no-default-features`" in docs
    assert "`make full-feature-check`" in docs
    assert "AlphaGenome" in docs
