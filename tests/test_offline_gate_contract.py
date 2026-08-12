from __future__ import annotations

import os
from pathlib import Path
import platform
import shutil
import subprocess

import pytest


ROOT = Path(__file__).resolve().parents[1]
MAKEFILE = (ROOT / "Makefile").read_text(encoding="utf-8")
WORKFLOW = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")


def test_routine_gates_prepare_before_entering_offline_namespace() -> None:
    assert "test: prepare-test" in MAKEFILE
    assert "spec: prepare-spec" in MAKEFILE
    assert "tools/run-offline" in MAKEFILE
    assert "nextest archive --locked $(ROUTINE_CARGO_FEATURES)" in MAKEFILE
    assert 'nextest run --archive-file "$(ROUTINE_TEST_ARCHIVE)"' in MAKEFILE
    assert "scripts/run-specs.sh prepare-spec" in MAKEFILE


def test_live_verify_lane_is_not_network_isolated() -> None:
    verify = MAKEFILE.split("verify:\n", 1)[1].split("\nrelease-live-smoke:", 1)[0]
    assert "tools/run-offline" not in verify


def test_authoritative_linux_job_installs_pinned_bubblewrap() -> None:
    canonical = WORKFLOW.split("  canonical-gates:\n", 1)[1].split(
        "\n  full-features:", 1
    )[0]
    assert "BUBBLEWRAP_VERSION:" in WORKFLOW
    assert '"bubblewrap=$BUBBLEWRAP_VERSION"' in canonical
    assert "make test" in canonical
    assert "make spec" in canonical


@pytest.mark.skipif(platform.system() != "Linux", reason="Linux is authoritative")
def test_offline_runner_proves_network_boundary_and_local_transports() -> None:
    if shutil.which("bwrap") is None:
        pytest.fail("Linux routine gates require bubblewrap")

    completed = subprocess.run(
        [
            str(ROOT / "tools/run-offline"),
            "--",
            "python3",
            "-c",
            "print('isolated command ran')",
        ],
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env={"PATH": os.environ["PATH"]},
        timeout=20,
    )

    assert "offline network isolation: bubblewrap --unshare-net" in completed.stdout
    assert (
        "offline network controls: public DNS blocked; direct public TCP blocked"
        in completed.stdout
    )
    assert (
        "offline network controls: loopback TCP and Unix sockets available"
        in completed.stdout
    )
    assert "isolated command ran" in completed.stdout
