"""The stress lane contract: a pinned CPU set, forced workers, the flaky set."""

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
    """Ticket 2075: the lane runs the prepared dev build — the CI job
    downloads the build job's archive instead of rebuilding it — so
    the recipe must guard that the archive exists and never rebuild
    it, while both test invocations stay pinned to the CPU set."""
    recipe = stress_recipe()
    assert "$(MAKE) prepare-test" not in recipe, (
        "the lane must not rebuild the archive; it runs the prepared one"
    )
    assert "ROUTINE_TEST_ARCHIVE" in recipe, (
        "the recipe must fail loudly when the prepared archive is missing"
    )
    pinned_count = recipe.count("taskset -c 0 tools/run-offline")
    assert pinned_count >= 2, "both the cargo and pytest invocations must be pinned"
    guard_line = next(
        line for line in recipe.splitlines() if "ROUTINE_TEST_ARCHIVE" in line
    )
    assert "taskset" not in guard_line, "the guard must not be pinned to the CPU set"
    sync_line = next(
        line for line in recipe.splitlines() if "sync-python-dev" in line
    )
    assert "taskset" not in sync_line, "the python sync must not be pinned"


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
    # The fixture-server case is the replay server behind the golden suite
    # (the disease-survival fixture lifecycle test is gone with its fixture).
    assert "tests/golden" in pytest_line


def test_the_rust_lane_names_the_flaky_set() -> None:
    recipe = stress_recipe()
    cargo_line = next(line for line in recipe.splitlines() if "nextest run" in line)
    for name in (
        "subprocess_lease_defers_old_generation_cleanup_until_reader_exits",
        "subprocess_lease_child_exits_on_parent_end_of_input",
        "cancelling_stalled_headers_and_streamed_body_drops_request_and_store_work",
        "cancelled_publication_settles_and_the_previous_generation_survives",
    ):
        assert name in cargo_line, f"{name} must be in the stress lane"


def test_the_repeat_count_defaults_to_three() -> None:
    recipe = stress_recipe()
    assert 'repeat="$${BIOMCP_STRESS_REPEAT:-3}"' in recipe


def test_the_lane_exercises_single_cpu_pinning() -> None:
    """The lease readiness handshake must work with one available CPU."""
    recipe = stress_recipe()
    pins = re.findall(r"taskset\s+(?:-c|--cpus)[ =]\s*(\S+)", recipe)
    assert len(pins) == 2, "both test invocations must exercise CPU affinity"
    assert all(pin.isdecimal() for pin in pins), (
        "the stress lane must pin each test invocation to exactly one CPU"
    )
