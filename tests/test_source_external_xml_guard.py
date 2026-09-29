"""External XML must only be parsed through the capped scanner.

``crate::xml::parse_external_xml`` rejects entity declarations and
nesting bombs in a pre-parse byte scan because roxmltree's parser
recurses per open tag and a deep document overflows the thread stack
before any post-parse check can run (tickets 1243 and 1255). This
guard fails when any code under ``src/`` calls ``Document::parse``
or ``Document::parse_with_options`` directly — outside
``src/xml.rs``, the one owner of the wrapped call — so a new source
or a renamed import cannot quietly reopen the recursive-parse path.
Comments are stripped before scanning, so prose that mentions the
call does not trip the guard.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# The wrapped parse lives only here; every other XML consumer must
# route through crate::xml::parse_external_xml.
ALLOWED_DIRECT_PARSE = {"src/xml.rs"}

# `Document::parse(` / `Document::parse_with_options(`, path-qualified
# or imported bare, under any local alias of roxmltree::Document. A
# template, compiled per resolved name in direct_parse_violations.
PARSE_CALL_TEMPLATE = r"(?<!\w)(?:\w+::)*{name}::parse(?:_with_options)?\s*\("


def document_parse_names(source: str) -> set[str]:
    """Local names a direct parse call can arrive under.

    ``Document`` itself always counts: a bare ``Document::parse(`` in
    this tree is roxmltree's, and a shadowing local type of that name
    would deserve the allowlist, not silence. ``use roxmltree::{...Document
    as X...}`` adds X.
    """
    names = {"Document"}
    # use roxmltree::{Document as Doc, Node};
    for use_match in re.finditer(r"use\s+roxmltree::\{([^}]*)\}", source):
        for item in use_match.group(1).split(","):
            aliased = re.fullmatch(r"\s*Document\s+as\s+(\w+)\s*", item)
            if aliased:
                names.add(aliased.group(1))
    # use roxmltree::Document as Doc;
    for aliased in re.finditer(r"use\s+roxmltree::Document\s+as\s+(\w+)\s*;", source):
        names.add(aliased.group(1))
    return names


def strip_rust_comments(source: str) -> str:
    """Blank out // and /* */ comments, keeping strings intact.

    Nested block comments (legal Rust), string literals, and char
    literals are honored so comment text that merely mentions a call
    is removed while a call inside a string literal survives as text
    we can still see. Newlines are preserved so line numbers in
    violation messages stay honest.
    """
    out: list[str] = []
    index = 0
    length = len(source)
    normal, line_comment, block_comment, string, char = range(5)
    state = normal
    depth = 0
    while index < length:
        char_here = source[index]
        nxt = source[index + 1] if index + 1 < length else ""
        if state == normal:
            if char_here == "/" and nxt == "/":
                state = line_comment
                out.append("  ")
                index += 2
                continue
            if char_here == "/" and nxt == "*":
                state = block_comment
                depth = 1
                out.append("  ")
                index += 2
                continue
            if char_here == '"':
                state = string
                out.append(char_here)
                index += 1
                continue
            if char_here == "'":
                # A char literal, not a lifetime or loop label: it has
                # a closing quote right after one plain or escaped
                # character.
                if nxt == "\\":
                    close = source.find("'", index + 2)
                    if close != -1 and close <= index + 3:
                        state = char
                        out.append(char_here)
                        index += 1
                        continue
                elif nxt and source[index + 2 : index + 3] == "'":
                    state = char
                    out.append(char_here)
                    index += 1
                    continue
            out.append(char_here)
            index += 1
        elif state == line_comment:
            if char_here == "\n":
                state = normal
                out.append("\n")
            else:
                out.append(" ")
            index += 1
        elif state == block_comment:
            if char_here == "/" and nxt == "*":
                depth += 1
                out.append("  ")
                index += 2
            elif char_here == "*" and nxt == "/":
                depth -= 1
                out.append("  ")
                index += 2
                if depth == 0:
                    state = normal
            else:
                out.append("\n" if char_here == "\n" else " ")
                index += 1
        elif state == string:
            out.append(char_here)
            if char_here == "\\" and nxt:
                out.append(nxt)
                index += 2
                continue
            if char_here == '"':
                state = normal
            index += 1
        else:  # char literal
            out.append(char_here)
            if char_here == "\\" and nxt:
                out.append(nxt)
                index += 2
                continue
            if char_here == "'":
                state = normal
            index += 1
    return "".join(out)


def direct_parse_violations(relative: str, source: str) -> list[str]:
    """Direct Document::parse calls in one file, comments excluded."""
    violations: list[str] = []
    if relative.replace("\\", "/") in ALLOWED_DIRECT_PARSE:
        return violations
    stripped = strip_rust_comments(source)
    for name in sorted(document_parse_names(source)):
        pattern = re.compile(PARSE_CALL_TEMPLATE.format(name=re.escape(name)))
        for match in re.finditer(pattern, stripped):
            line = stripped.count("\n", 0, match.start()) + 1
            head = stripped[match.start() : match.start() + 60].splitlines()[0]
            violations.append(f"{relative}:{line}: {head}")
    return violations


def tracked_rust_files(root: Path) -> list[Path]:
    listed = subprocess.run(
        ["git", "ls-files", "--", "src"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return [root / line for line in listed.splitlines() if line.endswith(".rs")]


def collect_violations(root: Path) -> list[str]:
    violations: list[str] = []
    for path in tracked_rust_files(root):
        relative = str(path.relative_to(root)).replace("\\", "/")
        violations.extend(
            direct_parse_violations(relative, path.read_text(encoding="utf-8"))
        )
    return violations


def test_external_xml_is_only_parsed_through_the_capped_scanner() -> None:
    violations = collect_violations(ROOT)
    assert not violations, (
        "direct Document::parse outside src/xml.rs "
        "(route through crate::xml::parse_external_xml):\n" + "\n".join(violations)
    )


def test_guard_catches_a_planted_direct_call_in_a_scratch_tree(tmp_path: Path) -> None:
    # A scratch git tree with files the checker has never seen: both
    # the bare imported form and a path-qualified call must fail the
    # scan, and only tracked files are scanned.
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "xml.rs").write_text(
        "pub(crate) fn parse_external_xml() {}\n", encoding="utf-8"
    )
    planted = tmp_path / "src" / "sources" / "new_source.rs"
    planted.parent.mkdir(parents=True)
    planted.write_text(
        "use roxmltree::Document;\n"
        "fn parse_a(xml: &str) {\n"
        "    let doc = Document::parse(xml).unwrap();\n"
        "}\n"
        "fn parse_b(xml: &str) {\n"
        "    let doc = roxmltree::Document::parse(xml).unwrap();\n"
        "}\n",
        encoding="utf-8",
    )
    untracked = tmp_path / "src" / "sources" / "scratch.rs"
    untracked.write_text(
        "fn parse(xml: &str) {\n"
        "    let doc = roxmltree::Document::parse(xml).unwrap();\n"
        "}\n",
        encoding="utf-8",
    )
    subprocess.run(
        ["git", "add", "src/xml.rs", "src/sources/new_source.rs"],
        cwd=tmp_path,
        check=True,
    )
    violations = [
        violation
        for path in tracked_rust_files(tmp_path)
        for violation in direct_parse_violations(
            str(path.relative_to(tmp_path)).replace("\\", "/"),
            path.read_text(encoding="utf-8"),
        )
    ]
    assert violations == [
        "src/sources/new_source.rs:3: Document::parse(xml).unwrap();",
        "src/sources/new_source.rs:6: roxmltree::Document::parse(xml).unwrap();",
    ], violations


def test_guard_ignores_a_comment_that_mentions_the_call() -> None:
    source = (
        "// Document::parse would overflow on deep input; use the\n"
        "// capped scanner instead (roxmltree::Document::parse recurses).\n"
        "/* block: Document::parse_with_options(x) mentioned here */\n"
        "fn parse(xml: &str) {\n"
        "    let _ = xml;\n"
        "}\n"
    )
    assert direct_parse_violations("src/sources/a.rs", source) == []


def test_guard_keeps_a_call_inside_a_string_literal_visible() -> None:
    # A call smuggled into a string is still source we scan; comment
    # stripping must not eat string contents.
    source = 'let code = "Document::parse(x)";\n'
    assert direct_parse_violations("src/a.rs", source), "string content must survive"


def test_guard_catches_an_aliased_import() -> None:
    source = (
        "use roxmltree::{Document as Doc, Node};\n"
        "fn parse(xml: &str) {\n"
        "    let doc = Doc::parse(xml).unwrap();\n"
        "}\n"
    )
    violations = direct_parse_violations("src/sources/a.rs", source)
    assert violations == ["src/sources/a.rs:3: Doc::parse(xml).unwrap();"], violations


def test_guard_allows_the_capped_scanner_itself() -> None:
    xml_rs = (ROOT / "src" / "xml.rs").read_text(encoding="utf-8")
    assert direct_parse_violations("src/xml.rs", xml_rs) == []


def test_guard_allows_the_wrapped_call_shape_in_consumers() -> None:
    source = (
        "use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};\n"
        "fn parse(xml: &str) {\n"
        "    let doc = parse_external_xml(xml, ARTICLE_XML_NODE_LIMIT).unwrap();\n"
        "}\n"
    )
    assert direct_parse_violations("src/sources/a.rs", source) == []
