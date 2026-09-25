"""The stress lane contract: one CPU, forced workers, the flaky set."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAKEFILE = (ROOT / "Makefile").read_text(encoding="utf-8")


def stress_recipe() -> str:
    """The stress recipe with make continuations joined into logical lines."""
    recipe = MAKEFILE.split("stress:", 1)[1].split("\n\n", 1)[0]
    return recipe.replace("\\\n", " ")


def test_the_stress_target_exists() -> None:
    assert re.search(r"^stress:", MAKEFILE, re.MULTILINE), "make stress must exist"
    assert (
        ".PHONY: build test lint" in MAKEFILE
        and " stress" in MAKEFILE.split(".PHONY: build test lint")[1].splitlines()[0]
    )


def test_the_build_is_not_pinned_but_the_tests_are() -> None:
    recipe = stress_recipe()
    assert "$(MAKE) prepare-test" in recipe, "the archive must be prepared first"
    pinned_count = recipe.count("taskset -c 0,1 tools/run-offline")
    assert pinned_count >= 2, "both the cargo and pytest invocations must be pinned"
    prepare_line = next(line for line in recipe.splitlines() if "prepare-test" in line)
    assert "taskset" not in prepare_line, "the build must not be pinned to the CPU set"


def test_the_rust_lane_pins_worker_parallelism() -> None:
    recipe = stress_recipe()
    cargo_line = next(line for line in recipe.splitlines() if "nextest run" in line)
    assert "-j 4" in cargo_line, "nextest must not auto-serialize on the pinned set"


def test_the_python_lane_pins_worker_parallelism() -> None:
    recipe = stress_recipe()
    pytest_line = next(line for line in recipe.splitlines() if "pytest" in line)
    assert "-n 4" in pytest_line, (
        "pytest-xdist must not auto-serialize on the pinned set"
    )
    assert "test_disease_survival_fixture_lifecycle" in pytest_line


def test_the_rust_lane_names_the_flaky_set() -> None:
    recipe = stress_recipe()
    cargo_line = next(line for line in recipe.splitlines() if "nextest run" in line)
    for name in (
        "subprocess_lease_defers_old_generation_cleanup_until_reader_exits",
        "subprocess_lease_child_exits_on_parent_end_of_input",
        "cancelling_stalled_headers_and_streamed_body_drops_request_and_store_work",
        "cancelling_active_publication_joins_cleanup_and_releases_locks",
    ):
        assert name in cargo_line, f"{name} must be in the stress lane"


def test_the_repeat_count_defaults_to_three() -> None:
    recipe = stress_recipe()
    assert 'repeat="$${BIOMCP_STRESS_REPEAT:-3}"' in recipe


def test_the_lane_avoids_single_cpu_pinning() -> None:
    """One-CPU pinning deadlocks the pipe handshake child; see
    sdlc/issues/2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md."""
    recipe = stress_recipe()
    assert "taskset -c 0 " not in recipe, "never pin the lane to exactly one CPU"
