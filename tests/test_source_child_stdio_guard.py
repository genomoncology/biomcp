"""Non-test code under src/ must not spawn children that inherit stdout.

A child spawned by ``biomcp`` in stdio mode inherits the server's
stdout, and anything it prints lands between JSON-RPC frames. GitHub
#283 is the live case: ``icacls.exe`` writes a localized success line
on every managed write and strict clients drop the session. This
guard fails when non-test code under ``src/`` builds a
``Command`` without an explicit captured or discarded stdout, so the
corruption cannot come back quietly.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# The wheel launcher is a pass-through: it must hand the child the
# parent's streams verbatim. It is the only allowed inheritor.
ALLOW_INHERIT = {"src/main_biomcp_cli.rs"}

COMMAND_NEW = re.compile(
    r"(?:std::process|tokio::process)::Command::new\s*\(|(?<![:\w])Command::new\s*\("
)


def production_source(path: Path) -> str:
    """Return the code before the first ``#[cfg(test)]`` in a file."""
    return path.read_text(encoding="utf-8").split("#[cfg(test)]", maxsplit=1)[0]


def is_test_path(relative: str) -> bool:
    parts = Path(relative).parts
    # Separate test modules: src/**/tests/** and files named *tests.rs.
    if len(parts) > 2 and "tests" in parts[1:-1]:
        return True
    return parts[-1].endswith("tests.rs")


def statement_windows(source: str) -> list[str]:
    """Return each ``Command::new`` statement plus chained continuations.

    A builder chain may span statements when the binding is reused
    (``let mut command = Command::new(x);`` followed by
    ``command.args(..).stdout(..);``), so the window extends across
    following statements that begin with the same binding.
    """
    windows: list[str] = []
    for match in COMMAND_NEW.finditer(source):
        # Find the binding this statement creates, if any, by scanning
        # the segment between the previous statement boundary and the
        # Command::new occurrence.
        boundary = max(
            source.rfind(";", 0, match.start()),
            source.rfind("{", 0, match.start()),
            source.rfind("}", 0, match.start()),
            0,
        )
        let_match = re.search(
            r"let\s+(?:mut\s+)?(\w+)\s*=\s*[\w:]*\s*$",
            source[boundary : match.start()],
            re.MULTILINE,
        )
        binding = let_match.group(1) if let_match else None
        end = source.find(";", match.start())
        if end == -1:
            windows.append(source[match.start() :])
            continue
        end += 1
        while binding:
            rest = source[end:]
            continuation = re.match(r"\s*" + re.escape(binding) + r"\s*\.", rest)
            if not continuation:
                break
            next_end = rest.find(";")
            if next_end == -1:
                end = len(source)
                break
            end += next_end + 1
        windows.append(source[match.start() : end])
    return windows


def child_stdio_violations(relative: str, source: str) -> list[str]:
    if relative.replace("\\", "/") in ALLOW_INHERIT:
        return []
    violations: list[str] = []
    for window in statement_windows(source):
        if re.search(r"\.output\s*\(", window):
            # .output() captures stdin, stdout, and stderr.
            continue
        stdout = re.search(r"\.stdout\s*\(\s*([^)]*)", window)
        if stdout is None:
            violations.append(
                f"{relative}: child stdout unset: {window.splitlines()[0].strip()}"
            )
        elif "Stdio::inherit" in stdout.group(1):
            violations.append(
                f"{relative}: child stdout inherits: {window.splitlines()[0].strip()}"
            )
    return violations


def test_source_children_never_inherit_stdout() -> None:
    violations: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.rs")):
        relative = str(path.relative_to(ROOT)).replace("\\", "/")
        if is_test_path(relative):
            continue
        violations.extend(child_stdio_violations(relative, production_source(path)))
    assert not violations, "children inheriting stdout:\n" + "\n".join(violations)


def test_guard_catches_the_icacls_regression_shape() -> None:
    # The exact pre-1246 shape: a spawned icacls.exe whose stdout is
    # unset, so the localized success line lands in the MCP stream.
    regression = (
        'let status = std::process::Command::new("icacls.exe")\n'
        "    .arg(path)\n"
        '    .args(["/inheritance:r", "/grant:r"])\n'
        "    .arg(&grant)\n"
        "    .status()?;\n"
    )
    violations = child_stdio_violations("src/cache/private.rs", regression)
    assert violations, "guard must catch the unset-stdout spawn"
    assert "stdout unset" in violations[0]


def test_guard_flags_inherit_and_passes_safe_shapes() -> None:
    inherit = (
        'let out = Command::new("tool").arg("x")\n'
        "    .stdout(Stdio::inherit())\n"
        "    .status()?;\n"
    )
    assert "stdout inherits" in child_stdio_violations("src/a.rs", inherit)[0]

    for safe in (
        # Discarded streams are the 1246 fix.
        'Command::new("icacls.exe").arg(path)\n'
        "    .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())\n"
        "    .status()?;\n",
        # Piped output is captured by the parent.
        "let mut cmd = tokio::process::Command::new(exe);\n"
        "cmd.kill_on_drop(true).stdin(Stdio::null())\n"
        "    .stdout(Stdio::piped()).stderr(Stdio::piped());\n",
        # .output() captures all three streams.
        "let smoke = std::process::Command::new(&stage_path)\n"
        '    .arg("version")\n'
        "    .output()?;\n",
        # Setters chained on the binding in following statements count.
        "let mut command = Command::new(sibling);\n"
        "command\n"
        "    .args(env::args_os().skip(1))\n"
        "    .stdin(Stdio::inherit())\n"
        "    .stdout(Stdio::piped())\n"
        "    .stderr(Stdio::inherit());\n",
    ):
        assert not child_stdio_violations("src/a.rs", safe), safe


def test_the_launcher_stays_the_only_allowed_inheritor() -> None:
    launcher = (ROOT / "src" / "main_biomcp_cli.rs").read_text(encoding="utf-8")
    assert "Stdio::inherit()" in launcher, "launcher contract moved; update the guard"
