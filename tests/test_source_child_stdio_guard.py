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
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]

# The wheel launcher is a pass-through: it must hand the child the
# parent's streams verbatim. It is the only allowed inheritor.
ALLOW_INHERIT = {"src/main_biomcp_cli.rs"}

# Any path-qualified Command::new (std::process, tokio::process,
# process:: after `use std::process;`) and the bare imported form.
COMMAND_NEW = re.compile(
    r"(?<!\w)(?:\w+::)*Command::new\s*\("
    r"|<(?:\w+::)*Command>\s*::\s*new\s*\("
)
# `use ... Command as X;` makes X::new a spawn of the same type, and
# so does every braced-group spelling:
# `use std::process::{Command as Cmd, ...}`, `use std::process::{self,
# Command as Cmd}`, `pub use std::process::Command as Cmd`, and a
# local `type Cmd = std::process::Command;`. The resolver reads the
# whole use statement (which may span lines), so an alias cannot hide
# behind brace or line wrapping.
COMMAND_ALIAS_USE = re.compile(
    r"use\s+[\w:]*Command\s+as\s+(\w+)\s*;"
    r"|use\s+[\w:]*\{[^}]*?\bCommand\s+as\s+(\w+)\b[^}]*\}"
    r"|type\s+(\w+)\s*=\s*[\w:]*Command\s*;",
)
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


def alias_names(source: str) -> set[str]:
    """Every local name bound to Command by a use or type alias.

    The three capture groups of COMMAND_ALIAS_USE are alternative
    spellings of the same binding; an empty-string capture means the
    alternative did not match.
    """
    names: set[str] = set()
    for groups in COMMAND_ALIAS_USE.findall(source):
        names.update(name for name in groups if name)
    return names


COMMAND_NEW_BINDING = re.compile(
    r"let\s+(?:mut\s+)?(\w+)\s*=\s*<?[\w:]*Command>?\s*::\s*new\s*;"
)


def spawn_pattern(source: str) -> re.Pattern[str]:
    """The Command::new pattern for this file, aliased imports included.

    A function binding (`let mk = Command::new;`) spawns through the
    bare name, so its call sites join the pattern too.
    """
    parts = [COMMAND_NEW.pattern]
    for alias in sorted(alias_names(source)):
        parts.append(r"(?<!\w)" + re.escape(alias) + r"::new\s*\(")
    for binding in COMMAND_NEW_BINDING.findall(source):
        parts.append(r"(?<!\w)" + re.escape(binding) + r"\s*\(")
    return re.compile("|".join(parts))


def inherit_bindings(source: str) -> set[str]:
    """Names bound to an inheriting Stdio anywhere in the file.

    `.stdout(io)` where `io` was bound from `Stdio::inherit()` is an
    inherit set through a reborrow; the violation names it as one.
    """
    return set(
        re.findall(
            r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:&\s*)?Stdio::inherit\s*\(",
            source,
        )
    )


def strip_comments_and_strings(text: str) -> str:
    """Blank comments and string/char literals, preserving offsets.

    Setter-looking text inside prose comments or string literals is
    not a setter call; scanning must not false-positive on it.
    """
    out = list(text)
    i, n = 0, len(text)
    mode = None
    start = 0
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if mode is None:
            if c == "/" and nxt == "/":
                mode, start = "line", i
                i += 2
            elif c == "/" and nxt == "*":
                mode, start = "block", i
                i += 2
            elif c == '"':
                mode, start = "str", i
                i += 1
            # Apostrophes are lifetimes here: a char literal is at
            # most a few characters and cannot hold a setter call, so
            # scanning never opens char mode.
            else:
                i += 1
            continue
        if mode == "line":
            if c == "\n":
                for k in range(start, i):
                    out[k] = " "
                mode = None
            i += 1
        elif mode == "block":
            if c == "*" and nxt == "/":
                for k in range(start, min(i + 2, n)):
                    out[k] = " "
                mode = None
                i += 2
            else:
                i += 1
        else:
            if c == "\\":
                i += 2
                continue
            if mode == "str" and c == '"':
                for k in range(start, min(i + 1, n)):
                    out[k] = " "
                mode = None
            i += 1
    if mode is not None:
        for k in range(start, n):
            out[k] = " "
    return "".join(out)


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
        # Only a BARE test predicate marks test code. A quoted
        # feature name containing "test" (`feature = "test-utils"`)
        # is production code behind a feature flag and stays scanned.
        predicates = [
            piece.strip()
            for piece in re.split(r"[(),]+", attribute.group(1))
            if piece.strip()
        ]
        if not any(
            predicate == "test" and '"' not in predicate for predicate in predicates
        ):
            continue
        if attribute.start() < cursor:
            continue
        out.append(source[cursor : attribute.start()])
        rest_at = attribute.end()
        while rest_at < len(source) and source[rest_at] in " \t\r\n":
            rest_at += 1
        item = re.match(r"(?:pub(?:\([^)]*\))?\s+)?(\w+)", source[rest_at:])
        keyword = item.group(1) if item else None
        if keyword in ITEM_KEYWORDS:
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
    # Fail closed on unresolved Command bindings: a use statement that
    # mentions Command in a shape the resolver did not consume is a
    # guard gap, not a pass.
    for statement in re.findall(r"use\s[^;]*;", source):
        if "Command" not in statement:
            continue
        # Only process-module bindings can be spawn types; a local
        # `use super::{DiseaseCommand, ...}` enum is not.
        if not re.search(r"\b(?:std|tokio)::process::", statement):
            continue
        if COMMAND_ALIAS_USE.search(statement):
            continue
        if re.search(r"\bCommand\s+as\s+\w+", statement):
            # A rename the resolver could not consume (nested braces,
            # an unseen grouping) is a guard gap.
            violations.append(
                f"{relative}: unresolved Command binding the alias resolver "
                f"did not consume: {statement.splitlines()[0][:60]}"
            )
        # A bare `Command` in the braces keeps its own name, and the
        # unqualified COMMAND_NEW pattern already covers its spawns.
    inherits = inherit_bindings(source)
    for window in statement_windows(source, spawn_pattern(source)):
        head = window.splitlines()[0].strip()
        # Setter-looking text inside comments or string literals is
        # prose, not a call.
        scanned = strip_comments_and_strings(window)
        for stream in STREAMS:
            setters = re.findall(r"\." + stream + r"\s*\(\s*([^)]*)", scanned)
            if not setters:
                violations.append(f"{relative}: child {stream} unset: {head}")
                continue
            for argument in setters:
                if "Stdio::inherit" in argument or "inherit()" in argument:
                    violations.append(
                        f"{relative}: child {stream} inherits: {head}"
                    )
                    continue
                if re.search(r"\bstd::io::(stdout|stderr|stdin)\b", argument):
                    violations.append(
                        f"{relative}: child {stream} is the parent's live stream: {head}"
                    )
                    continue
                if any(
                    re.search(r"\b" + re.escape(name) + r"\b", argument)
                    for name in inherits
                ):
                    violations.append(
                        f"{relative}: child {stream} inherits through a "
                        f"reborrowed binding: {head}"
                    )
                    continue
                if not re.search(r"Stdio::(?:null|piped)\b", argument):
                    # A variable or expression the guard cannot see
                    # through (pre-bound elsewhere) fails closed.
                    violations.append(
                        f"{relative}: child {stream} setting not provably "
                        f"null/piped ({argument.strip()[:40]}): {head}"
                    )
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


def test_guard_catches_the_braced_alias_with_a_missing_stream() -> None:
    # The 2026-09-27 leak: `use std::process::{Command as Cmd}` hid
    # from the unbraced alias pattern, so stdin could go missing.
    braced = (
        "use std::process::{Command as Cmd};\n"
        "fn f() {\n"
        '    let out = Cmd::new("icacls.exe")\n'
        "        .stdout(Stdio::null())\n"
        "        .stderr(Stdio::null())\n"
        "        .status()?;\n"
        "}\n"
    )
    violations = child_stdio_violations("src/a.rs", braced)
    assert violations == ['src/a.rs: child stdin unset: Cmd::new("icacls.exe")'], (
        violations
    )


def test_guard_catches_a_module_renamed_process_import() -> None:
    # `use std::process as proc; proc::Command::new` is caught by the
    # path-qualified pattern itself — no alias table needed.
    renamed = (
        "use std::process as proc;\n"
        "fn f() {\n"
        '    let out = proc::Command::new("tool")\n'
        "        .status()?;\n"
        "}\n"
    )
    violations = child_stdio_violations("src/a.rs", renamed)
    assert any("stdin unset" in v for v in violations), violations


def test_guard_fails_closed_on_an_unresolved_command_binding(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A rename the resolver does not know fails the guard, not passes.

    Simulates the stale-resolver future: a future Rust spelling binds
    `Command as Cmd` in a shape COMMAND_ALIAS_USE has not learned. The
    fail-closed rule must flag the statement instead of letting the
    spawn through unguarded.
    """
    resolved = "use std::process::{Command as Cmd};\n"
    assert not [
        v for v in child_stdio_violations("src/a.rs", resolved) if "unresolved" in v
    ]
    monkeypatch.setattr(
        sys.modules[__name__],
        "COMMAND_ALIAS_USE",
        re.compile(r"(?!x)x"),  # matches nothing: the stale resolver
    )
    violations = child_stdio_violations("src/a.rs", resolved)
    assert any("unresolved Command binding" in v for v in violations), violations


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


def test_guard_scans_production_code_after_a_semicolon_test_module() -> None:
    # The house shape: a `#[cfg(test)] mod tests;` declaration near the
    # top (render/markdown files) with production functions after it.
    # The stripper must cut at the semicolon, not brace-hunt into the
    # first production item.
    source = (
        "use std::fmt;\n"
        "#[cfg(test)]\n"
        "mod tests;\n"
        "pub fn drug_markdown_with_region() -> String {\n"
        '    let out = Command::new("render-helper")\n'
        "        .status()?;\n"
        "    String::new()\n"
        "}\n"
    )
    violations = child_stdio_violations(
        "src/render/markdown/drug.rs", strip_test_regions(source)
    )
    assert any("render-helper" in v for v in violations), violations


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
        # .output() with explicit non-inheriting setters: the
        # exemption is closed (1264) — a bare .output() with an
        # .stderr(Stdio::inherit()) before it still inherits.
        "let smoke = std::process::Command::new(&stage_path)\n"
        '    .arg("version")\n'
        "    .stdin(std::process::Stdio::null())\n"
        "    .stdout(std::process::Stdio::piped())\n"
        "    .stderr(std::process::Stdio::piped())\n"
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


def test_output_exemption_is_closed() -> None:
    """The shapes the 2026-09-28 review listed must fail the guard."""
    bad_shapes = (
        # .output() no longer exempts an inheriting stderr.
        "let out = std::process::Command::new(x)\n"
        "    .stderr(Stdio::inherit())\n"
        "    .output()?;\n",
        # The parent's live stream object is an inherit in disguise.
        "let out = std::process::Command::new(x)\n"
        "    .stdin(Stdio::null())\n"
        "    .stdout(std::io::stdout())\n"
        "    .stderr(Stdio::null())\n"
        "    .status()?;\n",
        # A pre-bound inheriting variable fails closed.
        "let s = Stdio::inherit();\n"
        "std::process::Command::new(x)\n"
        "    .stdin(Stdio::null())\n"
        "    .stdout(s)\n"
        "    .stderr(Stdio::null())\n"
        "    .status()?;\n",
    )
    for bad in bad_shapes:
        violations = child_stdio_violations("src/a.rs", bad)
        assert violations, f"guard must fail this shape:\n{bad}"


def test_the_launcher_stays_the_only_allowed_inheritor() -> None:
    launcher = (ROOT / "src" / "main_biomcp_cli.rs").read_text(encoding="utf-8")
    assert "Stdio::inherit()" in launcher, "launcher contract moved; update the guard"

def test_guard_catches_round_three_shapes() -> None:
    """Ticket 1269: the neighboring spellings the prior guard missed."""
    full = (
        'Command::new("tool").arg("x")\n'
        "    .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())\n"
        "    .status()?;\n"
    )
    # A second inherit setter after the null one: the last setter
    # wins at spawn time, so any inherit anywhere in the chain fails.
    sneak = full.replace(
        ".stderr(Stdio::null())",
        ".stderr(Stdio::null()).stderr(Stdio::inherit())",
    )
    assert child_stdio_violations("src/a.rs", sneak) == [
        'src/a.rs: child stderr inherits: Command::new("tool").arg("x")'
    ], sneak

    # Setter-looking text in comments and strings is prose.
    prose = (
        full
        + 'let note = ".stdin(Stdio::inherit()) looks dangerous";\n'
        + "// .stdout(Stdio::inherit()) discussed in review\n"
    )
    assert not child_stdio_violations("src/a.rs", prose), prose

    # Turbofish generics and function bindings are spawns.
    turbo = full.replace('Command::new("tool")', '<Command>::new("tool")')
    assert child_stdio_violations(
        "src/a.rs", "use std::process::Command;\n" + turbo.replace(
            ".stdin(Stdio::null())", ""
        ).replace(".stdout(Stdio::null())", "").replace(".stderr(Stdio::null())", "")
    ), turbo

    binding = (
        "use std::process::Command;\n"
        "fn f() {\n"
        "    let mk = Command::new;\n"
        '    mk("tool").stdin(Stdio::null())\n'
        "        .stdout(Stdio::null()).stderr(Stdio::null()).status()?;\n"
        "}\n"
    )
    # The binding itself is a legal full-null spawn shape; removing a
    # stream from its chain must fail.
    broken_binding = binding.replace(".stderr(Stdio::null())", "")
    assert child_stdio_violations("src/a.rs", broken_binding), broken_binding

    # An inherit through a reborrowed binding names the reborrow.
    reborrow = (
        "fn f() {\n"
        "    let io = Stdio::inherit();\n"
        '    Command::new("tool").arg("x")\n'
        "        .stdin(Stdio::null()).stdout(&io).stderr(Stdio::null())\n"
        "        .status()?;\n"
        "}\n"
    )
    assert child_stdio_violations("src/a.rs", reborrow) == [
        "src/a.rs: child stdout inherits through a reborrowed binding: "
        'Command::new("tool").arg("x")'
    ], reborrow

    # A cfg whose FEATURE name contains "test" is production code.
    featured = (
        "#[cfg(feature = \"test-utils\")]\n"
        "fn f() {\n"
        '    Command::new("tool").arg("x")\n'
        "        .stdin(Stdio::inherit())\n"
        "        .stdout(Stdio::inherit()).stderr(Stdio::inherit())\n"
        "        .status()?;\n"
        "}\n"
    )
    stripped = strip_test_regions(featured)
    assert "Command::new" in stripped, "feature cfg must not strip production code"
    assert any(
        "inherits" in v for v in child_stdio_violations("src/a.rs", stripped)
    ), featured
