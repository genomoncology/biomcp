"""Non-test code under src/ must not spawn children that inherit stdio.

A child spawned by ``biomcp`` in stdio mode inherits the server's
stdin, stdout, and stderr, and anything it prints lands between
JSON-RPC frames (GitHub #283: ``icacls.exe`` wrote a localized success
line on every managed write and strict clients dropped the session).
This guard fails when non-test code under ``src/`` builds a
``Command`` without an explicit, non-inheriting setting for all three
streams — stdin, stdout, and stderr — so the corruption cannot come
back quietly through any stream.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# The wheel launcher is a pass-through: it must hand the child the
# parent's streams verbatim. It is the only allowed inheritor.
ALLOW_INHERIT = {"src/main_biomcp_cli.rs"}

# Any path-qualified Command::new (std::process, tokio::process,
# process:: after `use std::process;`) and the bare imported form.
COMMAND_NEW = re.compile(r"(?<!\w)(?:\w+::)*Command::new\s*\(")
# `use ... Command as X;` makes X::new a spawn of the same type.
COMMAND_ALIAS_USE = re.compile(r"use\s+[\w:]+Command\s+as\s+(\w+)\s*;")
# The three streams a child must not inherit.
STREAMS = ("stdin", "stdout", "stderr")
ITEM_KEYWORDS = (
    "mod",
    "static",
    "const",
    "fn",
    "struct",
    "enum",
    "impl",
    "use",
    "type",
    "trait",
)


def spawn_pattern(source: str) -> re.Pattern[str]:
    """The Command::new pattern for this file, aliased imports included."""
    names = [COMMAND_ALIAS_USE.findall(source)]
    parts = [COMMAND_NEW.pattern]
    for alias in sorted(set(sum(names, []))):
        parts.append(r"(?<!\w)" + re.escape(alias) + r"::new\s*\(")
    return re.compile("|".join(parts))


def strip_test_regions(source: str) -> str:
    """Remove cfg(test) items so a mid-file test module hides nothing.

    A file's production code continues after an in-file test module
    (``src/sources/ca_bundle.rs`` keeps a cfg(test) static near the
    top and real loader code after it), so cutting at the first
    ``#[cfg(test)]`` would leave most of such a file unscanned.
    Instead: cfg(test) module blocks are removed by brace matching,
    cfg(test) single items by their own extent, and a cfg(test)
    statement (an attribute inside a production function) leaves the
    statement itself in place. Test code is skipped conservatively;
    nothing else is.
    """
    out: list[str] = []
    cursor = 0
    for attribute in re.finditer(r"#\[cfg\(([^)]*)\)\]", source):
        if "test" not in attribute.group(1):
            continue
        if attribute.start() < cursor:
            continue
        out.append(source[cursor : attribute.start()])
        rest_at = attribute.end()
        while rest_at < len(source) and source[rest_at] in " \t\r\n":
            rest_at += 1
        item = re.match(r"(?:pub(?:\([^)]*\))?\s+)?(\w+)", source[rest_at:])
        keyword = item.group(1) if item else None
        if keyword == "mod" and item:
            opened = source.find("{", rest_at)
            if opened == -1:
                cursor = len(source)
                break
            cursor = _matching_brace(source, opened) + 1
        elif keyword in ITEM_KEYWORDS:
            semicolon = source.find(";", rest_at)
            opened = source.find("{", rest_at)
            if opened != -1 and (semicolon == -1 or opened < semicolon):
                cursor = _matching_brace(source, opened) + 1
            elif semicolon != -1:
                cursor = semicolon + 1
            else:
                cursor = len(source)
        else:
            # A cfg(test) statement inside a production function: drop
            # only the attribute text; the statement stays scannable.
            cursor = attribute.end()
    out.append(source[cursor:])
    return "".join(out)


def _matching_brace(source: str, opened: int) -> int:
    depth = 0
    for index in range(opened, len(source)):
        char = source[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    return len(source) - 1


def is_test_path(relative: str) -> bool:
    parts = Path(relative).parts
    # Separate test modules: src/**/tests/** and files named *tests.rs.
    if len(parts) > 2 and "tests" in parts[1:-1]:
        return True
    return parts[-1].endswith("tests.rs")


def statement_windows(source: str, pattern: re.Pattern[str]) -> list[str]:
    """Return each spawn statement plus chained continuations.

    A builder chain may span statements when the binding is reused
    (``let mut command = Command::new(x);`` followed by
    ``command.args(..).stdout(..);``), so the window extends across
    following statements that begin with the same binding.
    """
    windows: list[str] = []
    seen: set[int] = set()
    for match in pattern.finditer(source):
        if match.start() in seen:
            continue
        seen.add(match.start())
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
    for window in statement_windows(source, spawn_pattern(source)):
        if re.search(r"\.output\s*\(", window):
            # .output() captures stdin, stdout, and stderr.
            continue
        head = window.splitlines()[0].strip()
        for stream in STREAMS:
            setter = re.search(r"\." + stream + r"\s*\(\s*([^)]*)", window)
            if setter is None:
                violations.append(f"{relative}: child {stream} unset: {head}")
            elif "Stdio::inherit" in setter.group(1):
                violations.append(f"{relative}: child {stream} inherits: {head}")
    return violations


def test_source_children_never_inherit_stdio() -> None:
    violations: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.rs")):
        relative = str(path.relative_to(ROOT)).replace("\\", "/")
        if is_test_path(relative):
            continue
        violations.extend(
            child_stdio_violations(
                relative, strip_test_regions(path.read_text(encoding="utf-8"))
            )
        )
    assert not violations, "children inheriting stdio:\n" + "\n".join(violations)


def test_guard_catches_the_icacls_regression_shape() -> None:
    # The exact pre-1246 shape: a spawned icacls.exe whose streams are
    # all unset, so the localized success line lands in the MCP stream.
    regression = (
        'let status = std::process::Command::new("icacls.exe")\n'
        "    .arg(path)\n"
        '    .args(["/inheritance:r", "/grant:r"])\n'
        "    .arg(&grant)\n"
        "    .status()?;\n"
    )
    violations = child_stdio_violations("src/cache/private.rs", regression)
    assert len(violations) == 3, violations
    for stream in STREAMS:
        assert any(f"{stream} unset" in violation for violation in violations), (
            stream,
            violations,
        )


def test_guard_catches_each_missing_stream_alone() -> None:
    # Removing any one of the three null streams from the fix must
    # trip the guard for exactly that stream.
    full = (
        'Command::new("icacls.exe").arg(path)\n'
        "    .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())\n"
        "    .status()?;\n"
    )
    assert not child_stdio_violations("src/cache/private.rs", full)
    for stream in STREAMS:
        broken = full.replace(f".{stream}(Stdio::null())", "")
        violations = child_stdio_violations("src/cache/private.rs", broken)
        assert [v for v in violations if "unset" in v] == [
            f'src/cache/private.rs: child {stream} unset: Command::new("icacls.exe").arg(path)'
        ], (stream, violations)


def test_guard_flags_each_inheriting_stream_alone() -> None:
    full = (
        'Command::new("tool").arg("x")\n'
        "    .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())\n"
        "    .status()?;\n"
    )
    for stream in STREAMS:
        inherit = full.replace(
            f".{stream}(Stdio::null())", f".{stream}(Stdio::inherit())"
        )
        violations = child_stdio_violations("src/a.rs", inherit)
        assert violations == [
            f'src/a.rs: child {stream} inherits: Command::new("tool").arg("x")'
        ], (
            stream,
            violations,
        )


def test_guard_catches_path_qualified_and_aliased_spawns() -> None:
    qualified = (
        'let status = process::Command::new("icacls.exe")\n'
        "    .arg(path)\n    .status()?;\n"
    )
    violations = child_stdio_violations("src/a.rs", qualified)
    assert any("stdin unset" in v for v in violations), violations

    aliased = (
        "use std::process::Command as Cmd;\n"
        "fn f() {\n"
        '    let out = Cmd::new("icacls.exe")\n'
        "        .stdin(Stdio::null())\n"
        "        .stdout(Stdio::null())\n"
        "        .status()?;\n"
        "}\n"
    )
    violations = child_stdio_violations("src/a.rs", aliased)
    assert violations == ['src/a.rs: child stderr unset: Cmd::new("icacls.exe")'], (
        violations
    )


def test_guard_scans_production_code_after_a_midfile_test_module() -> None:
    # src/sources/ca_bundle.rs keeps a cfg(test) static near the top
    # and production loader code after it; cutting at the first
    # #[cfg(test)] would leave that loader unscanned.
    source = (
        "static RESOLVED: OnceLock<u8> = OnceLock::new();\n"
        "#[cfg(test)]\n"
        "static PARSE_COUNT: AtomicUsize = AtomicUsize::new(0);\n"
        "fn parse_certificates() {\n"
        "    #[cfg(test)]\n"
        "    PARSE_COUNT.fetch_add(1, Ordering::SeqCst);\n"
        "}\n"
        "fn late_spawn() {\n"
        '    let out = Command::new("late")\n'
        "        .status()?;\n"
        "}\n"
    )
    violations = child_stdio_violations(
        "src/sources/ca_bundle.rs", strip_test_regions(source)
    )
    assert violations, "production code after a mid-file test item must be scanned"
    assert all("late_spawn" not in v or "unset" in v for v in violations)
    assert any("late" in v for v in violations), violations


def test_guard_passes_safe_shapes() -> None:
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
        "    .stdin(Stdio::null())\n"
        "    .stdout(Stdio::piped())\n"
        "    .stderr(Stdio::null());\n",
    ):
        assert not child_stdio_violations("src/a.rs", safe), safe


def test_the_launcher_stays_the_only_allowed_inheritor() -> None:
    launcher = (ROOT / "src" / "main_biomcp_cli.rs").read_text(encoding="utf-8")
    assert "Stdio::inherit()" in launcher, "launcher contract moved; update the guard"
