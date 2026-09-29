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
elf_floor = _MODULE.elf_floor


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
    assert wheel_floor(wheel) == (2, 28)


def test_wheel_floor_zero_when_no_markers(tmp_path: Path) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"biomcp_cli/biomcp": b"clean"})
    assert wheel_floor(wheel) == (0, 0)


def test_main_fails_over_the_floor_and_names_the_wheel(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    wheel = build_wheel(
        tmp_path / "wheel-a.whl",
        {"biomcp_cli/biomcp": b"GLIBC_2.39"},
    )
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.28"]) == 1
    assert "wheel-a.whl" in capsys.readouterr().out


def test_main_passes_at_or_below_the_default_floor(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
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


# The floor compares integer (major, minor) pairs, the way the dynamic
# loader orders them. A decimal comparison reads GLIBC_2_9 as 2.9 and
# fails a 2.28 floor, and reads GLIBC_2_30 as 2.3 and passes it.


@pytest.mark.parametrize(
    "markers",
    [
        b"GLIBC_2.2.5 GLIBC_2.3 GLIBC_2.3.4 GLIBC_2.4 GLIBC_2.9 GLIBC_2.17",
        b"GLIBC_2.3",
        b"GLIBC_2.9",
        b"GLIBC_2.28",
    ],
)
def test_versions_at_or_below_the_floor_pass(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], markers: bytes
) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"biomcp_cli/biomcp": markers})
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.28"]) == 0
    assert "OK w.whl" in capsys.readouterr().out


@pytest.mark.parametrize(
    "markers,expected", [(b"GLIBC_2.29", "2.29"), (b"GLIBC_2.30", "2.30")]
)
def test_versions_above_the_floor_fail_with_their_exact_number(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], markers: bytes, expected: str
) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"biomcp_cli/biomcp": markers})
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.28"]) == 1
    out = capsys.readouterr().out
    assert f"highest glibc reference {expected} exceeds floor 2.28" in out


def test_the_real_release_marker_set_passes_the_floor(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    wheel = build_wheel(
        tmp_path / "real.whl",
        {
            "biomcp_cli/biomcp": b"GLIBC_2.2.5 GLIBC_2.3 GLIBC_2.3.4 GLIBC_2.4 GLIBC_2.9 GLIBC_2.17"
        },
    )
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.28"]) == 0
    assert "highest glibc reference 2.17 within floor 2.28" in capsys.readouterr().out


def test_floor_argument_compares_as_integers_not_decimals(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"biomcp_cli/biomcp": b"GLIBC_2.9"})
    # A floor of 2.10 accepts a 2.9 reference: 9 < 10 as a minor,
    # where a decimal comparison would read 2.9 as larger.
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.10"]) == 0
    # A floor of 2.30 parses as thirty, so a 2.9 reference passes.
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2.30"]) == 0
    wheel30 = build_wheel(tmp_path / "w30.whl", {"biomcp_cli/biomcp": b"GLIBC_2.30"})
    assert main(["check-wheel-glibc-floor.py", str(wheel30), "2.30"]) == 0
    assert main(["check-wheel-glibc-floor.py", str(wheel30), "2.28"]) == 1
    assert "highest glibc reference 2.30 exceeds floor 2.28" in capsys.readouterr().out


def test_malformed_floor_argument_prints_usage_and_exits_two(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    wheel = build_wheel(tmp_path / "w.whl", {"biomcp_cli/biomcp": b"GLIBC_2.28"})
    assert main(["check-wheel-glibc-floor.py", str(wheel), "2,28"]) == 2
    captured = capsys.readouterr()
    assert "MAJOR.MINOR" in captured.err
    assert "Usage" in captured.err


def test_elf_mode_scans_a_raw_binary(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    binary = tmp_path / "biomcp"
    binary.write_bytes(b"\x7fELF" + b"GLIBC_2.2.5 GLIBC_2.3 GLIBC_2.9" + b"\x00")
    assert elf_floor(binary) == (2, 9)
    assert main(["check-wheel-glibc-floor.py", "--elf", str(binary), "2.28"]) == 0
    assert (
        "OK biomcp: highest glibc reference 2.9 within floor 2.28"
        in capsys.readouterr().out
    )

    newer = tmp_path / "newer"
    newer.write_bytes(b"\x7fELF" + b"GLIBC_2.34" + b"\x00")
    assert elf_floor(newer) == (2, 34)
    assert main(["check-wheel-glibc-floor.py", "--elf", str(newer), "2.28"]) == 1
    assert (
        "FAIL newer: highest glibc reference 2.34 exceeds floor 2.28"
        in capsys.readouterr().out
    )


def test_elf_mode_reads_stdin_and_fails_closed_without_markers(
    tmp_path: Path, capsys: pytest.CaptureFixture
) -> None:
    import subprocess
    import sys

    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts" / "check-wheel-glibc-floor.py"),
            "--elf",
            "-",
            "2.28",
        ],
        input=b"GLIBC_2.28",
        capture_output=True,
    )
    assert result.returncode == 0, result.stderr
    assert b"OK stdin: highest glibc reference 2.28 within floor 2.28" in result.stdout

    markerless = tmp_path / "clean"
    markerless.write_bytes(b"\x7fELF no markers here")
    with pytest.raises(SystemExit, match="no GLIBC marker"):
        elf_floor(markerless)


def test_the_console_launcher_member_is_scanned(tmp_path: Path) -> None:
    """The wheel's second program, biomcp-cli, carries the same floor.

    A launcher binary referencing only GLIBC_2.34 must fail the 2.28
    floor; before the member filter knew its name, it shipped unseen.
    """
    launcher = build_wheel(
        tmp_path / "w-launcher.whl",
        {
            "biomcp_cli/biomcp": b"GLIBC_2.17",
            "biomcp-cli": b"GLIBC_2.34",
        },
    )
    assert wheel_floor(launcher) == (2, 34)


def test_a_dir_qualified_launcher_is_scanned(tmp_path: Path) -> None:
    qualified = build_wheel(
        tmp_path / "w-qualified.whl",
        {"bin/biomcp-cli": b"GLIBC_2.29"},
    )
    assert wheel_floor(qualified) == (2, 29)
