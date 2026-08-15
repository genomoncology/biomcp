from __future__ import annotations

import json
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
    assert "test:\n\t$(MAKE) prepare-test" in MAKEFILE
    assert "spec:\n\t$(MAKE) prepare-spec" in MAKEFILE
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


def test_offline_runner_maps_root_inside_an_isolated_user_namespace() -> None:
    runner = (ROOT / "tools/run-offline").read_text(encoding="utf-8")
    assert "--unshare-user" in runner
    assert "--uid 0" in runner
    assert "--gid 0" in runner
    assert "--unshare-net" in runner
    assert "BIOMCP_OFFLINE_OWNERSHIP_SENTINEL" in runner
    assert "BIOMCP_OFFLINE_OWNERSHIP_TOKEN" in runner


@pytest.mark.skipif(platform.system() != "Linux", reason="Linux is authoritative")
def test_offline_runner_proves_outer_namespace_identity_and_local_transports(
    tmp_path: Path,
) -> None:
    if shutil.which("bwrap") is None:
        pytest.fail("Linux routine gates require bubblewrap")

    already_enclosed = os.environ.get("BIOMCP_OFFLINE_NETWORK") == "1"
    child_env = {"PATH": os.environ["PATH"], "TMPDIR": str(tmp_path)}
    child_env.pop("BIOMCP_OFFLINE_NETWORK", None)
    child = """
import json
import os
from pathlib import Path

status = dict(
    line.split(":", 1)
    for line in Path("/proc/self/status").read_text(encoding="utf-8").splitlines()
    if ":" in line
)
print(json.dumps({
    "uid": os.getuid(),
    "gid": os.getgid(),
    "capabilities": {
        key: status[key].strip()
        for key in ("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb")
    },
    "no_new_privileges": status["NoNewPrivs"].strip(),
}))
"""
    completed = subprocess.run(
        [
            str(ROOT / "tools/run-offline"),
            "--",
            "python3",
            "-c",
            child,
        ],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=child_env,
        timeout=20,
    )
    if (
        completed.returncode != 0
        and already_enclosed
        and "No permissions to create new namespace" in completed.stdout
    ):
        pytest.skip("the enclosing AppArmor profile forbids a nested user namespace")
    completed.check_returncode()

    assert (
        "offline network isolation: bubblewrap isolated user and network namespaces"
        in completed.stdout
    )
    assert (
        "offline network controls: public DNS blocked; direct public TCP blocked"
        in completed.stdout
    )
    assert (
        "offline network controls: loopback TCP and Unix sockets available"
        in completed.stdout
    )
    assert "offline privilege controls: uid/gid 0; capabilities zero; NoNewPrivs 1" in completed.stdout
    assert f"offline ownership mapping: host uid/gid {os.getuid()}/{os.getgid()} verified" in completed.stdout
    assert "reusing the verified enclosing namespace" not in completed.stdout
    report = json.loads(next(line for line in completed.stdout.splitlines() if line.startswith("{")))
    assert report["uid"] == 0
    assert report["gid"] == 0
    assert set(report["capabilities"].values()) == {"0000000000000000"}
    assert report["no_new_privileges"] == "1"
    assert list(tmp_path.iterdir()) == []


@pytest.mark.skipif(
    os.environ.get("BIOMCP_OFFLINE_NETWORK") == "1",
    reason="the outer-wrapper success test covers host-side ownership",
)
def test_outer_runner_preserves_child_failure_and_cleans_ownership_sentinel(
    tmp_path: Path,
) -> None:
    completed = subprocess.run(
        [
            str(ROOT / "tools/run-offline"),
            "--",
            "python3",
            "-c",
            "raise SystemExit(37)",
        ],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env={"PATH": os.environ["PATH"], "TMPDIR": str(tmp_path)},
        timeout=20,
    )
    assert completed.returncode == 37
    assert "offline ownership mapping:" in completed.stdout
    assert list(tmp_path.iterdir()) == []


@pytest.mark.skipif(
    os.environ.get("BIOMCP_OFFLINE_NETWORK") == "1",
    reason="requires an unisolated caller",
)
def test_marker_cannot_bypass_outer_privilege_isolation() -> None:
    completed = subprocess.run(
        [str(ROOT / "tools/run-offline"), "--", "true"],
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env={"PATH": os.environ["PATH"], "BIOMCP_OFFLINE_NETWORK": "1"},
        timeout=20,
    )
    assert completed.returncode != 0
    assert "offline privilege isolation failed: namespace uid/gid are not 0" in completed.stdout


@pytest.mark.skipif(
    os.environ.get("BIOMCP_OFFLINE_NETWORK") != "1",
    reason="requires the canonical enclosing offline wrapper",
)
def test_enclosed_runner_revalidates_privilege_and_network_state() -> None:
    completed = subprocess.run(
        [str(ROOT / "tools/run-offline"), "--", "python3", "-c", "print('reused')"],
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env={"PATH": os.environ["PATH"], "BIOMCP_OFFLINE_NETWORK": "1"},
        timeout=20,
    )
    assert "reusing the verified enclosing namespace" in completed.stdout
    assert "offline privilege controls: uid/gid 0; capabilities zero; NoNewPrivs 1" in completed.stdout
    assert "offline network controls: public DNS blocked; direct public TCP blocked" in completed.stdout
    assert "offline network controls: loopback TCP and Unix sockets available" in completed.stdout
    assert completed.stdout.rstrip().endswith("reused")
