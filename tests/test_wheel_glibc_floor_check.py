"""Unit tests for scripts/check-wheel-glibc-floor.py."""

import zipfile
from pathlib import Path

import pytest

import importlib.util

ROOT = Path(__file__).resolve().parents[1]
_SPEC = importlib.util.spec_from_file_location(
    "check_wheel_glibc_floor", ROOT / "scripts" / "check-wheel-glibc-floor.py"
)
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)
main = _MODULE.main
wheel_floor = _MODULE.wheel_floor


def build_wheel(path: Path, members: dict[str, bytes]) -> Path:
    with zipfile.ZipFile(path, "w") as archive:
        for name, data in members.items():
            archive.writestr(name, data)
    return path


def test_wheel_floor_reports_the_highest_glibc_reference(tmp_path: Path) -> None:
    wheel = build_wheel(
        tmp_path / "w.whl",
        {
            "biomcp_cli/biomcp": b"prefix GLIBC_2.17 GLIBC_2.28 suffix",
            "metadata.json": b"GLIBC_9.99 ignored: not an ELF member",
        },
    )
    assert wheel_floor(wheel) == 2.28


def test_wheel_floor_zero_when_no_markers(tmp_path: Path) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"biomcp_cli/biomcp": b"clean"})
    assert wheel_floor(wheel) == 0.0


def test_main_fails_over_the_floor_and_names_the_wheel(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    wheel = build_wheel(
        tmp_path / "wheel-a.whl",
        {"biomcp_cli/biomcp": b"GLIBC_2.39"},
    )
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.28"]) == 1
    assert "wheel-a.whl" in capsys.readouterr().out


def test_main_passes_at_or_below_the_default_floor(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    wheel = build_wheel(
        tmp_path / "wheel-b.whl",
        {"biomcp_cli/libhelper.so": b"GLIBC_2.2.5 GLIBC_2.17"},
    )
    assert main(["check-wheel-glibc-floor.py", str(wheel)]) == 0
    assert "OK wheel-b.whl" in capsys.readouterr().out


def test_wheel_floor_fails_when_no_member_matches(tmp_path: Path) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"metadata.json": b"GLIBC_2.39 ignored"})
    with pytest.raises(SystemExit, match="no ELF member"):
        wheel_floor(wheel)


def test_main_rejects_bad_usage(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["check-wheel-glibc-floor.py"]) == 2
    assert capsys.readouterr().err.strip()
